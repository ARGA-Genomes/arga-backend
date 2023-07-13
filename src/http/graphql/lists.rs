use std::collections::HashMap;

use async_graphql::*;

use serde::Deserialize;
use serde::Serialize;
use tracing::instrument;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::http::Error;
use crate::http::Context as State;
use crate::index::Taxonomy;
use crate::index::lists;
use crate::index::lists::{Filters, GetListNames, GetListPhotos, GetListStats, GetListTaxa, ListDataSummary, ListStats, Pagination};
use crate::index::stats::GetSpeciesStats;
use crate::database::{schema, Database};
use crate::database::models::{NameList, TaxonPhoto, Name as ArgaName};


#[derive(Debug, Enum, Eq, PartialEq, Copy, Clone)]
pub enum FilterType {
    Kingdom,
    Phylum,
}

#[derive(Debug, Enum, Eq, PartialEq, Copy, Clone)]
pub enum FilterAction {
    Include,
    Exclude,
}

#[derive(Debug, InputObject)]
pub struct FilterItem {
    filter: FilterType,
    action: FilterAction,
    value: String,
}

impl From<FilterItem> for lists::FilterItem {
    fn from(item: FilterItem) -> Self {
        let filter = match item.filter {
            FilterType::Kingdom => lists::Filter::Kingdom(item.value),
            FilterType::Phylum => lists::Filter::Phylum(item.value),
        };

        match item.action {
            FilterAction::Include => lists::FilterItem::Include(filter),
            FilterAction::Exclude => lists::FilterItem::Exclude(filter),
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct ListSpecies {
    pub taxonomy: Taxonomy,
    pub photo: Option<SpeciesPhoto>,
    pub data_summary: ListDataSummary,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct SpeciesPhoto {
    url: String,
    source: Option<String>,
    publisher: Option<String>,
    license: Option<String>,
    rights_holder: Option<String>,
}

impl From<TaxonPhoto> for SpeciesPhoto {
    fn from(value: TaxonPhoto) -> Self {
        Self {
            url: value.url,
            source: value.source,
            publisher: value.publisher,
            license: value.license,
            rights_holder: value.rights_holder,
        }
    }
}


pub struct Lists {
    pub list: NameList,
    pub names: Vec<ArgaName>,
    pub filters: Filters,
}

#[Object]
impl Lists {
    #[graphql(skip)]
    pub async fn new(
        db: &Database,
        name: String,
        filters: Filters,
        pagination: Pagination
    ) -> Result<Lists, Error>
    {
        use schema::name_lists as lists;
        let mut conn = db.pool.get().await?;

        let list = lists::table
            .filter(lists::name.eq(&name))
            .get_result::<NameList>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = list {
            return Err(Error::NotFound(name));
        }

        let list = list?;
        let names = db.list_names(&list, &filters, &pagination).await?;

        Ok(Lists { list, names, filters })
    }

    #[instrument(skip(self, ctx))]
    async fn species(&self, ctx: &Context<'_>) -> Result<Vec<ListSpecies>, Error> {
        let state = ctx.data::<State>().unwrap();

        let mut species: HashMap<Uuid, ListSpecies> = HashMap::new();

        // get the taxonomic information for all the names associated with the list
        // we also stub out the ListSpecies struct to make the rest of the data
        // association easier by mapping the name uuid to a final struct output
        let taxa = state.database.list_taxa(&self.names).await?;
        for taxon in taxa {
            let taxonomy = Taxonomy {
                scientific_name: taxon.scientific_name.unwrap(),
                canonical_name: taxon.canonical_name,
                authorship: taxon.scientific_name_authorship,
                kingdom: taxon.kingdom,
                phylum: taxon.phylum,
                class: taxon.class,
                order: taxon.order,
                family: taxon.family,
                genus: taxon.genus,
                vernacular_group: None,
            };

            species.insert(taxon.name_id, ListSpecies {
                taxonomy,
                photo: None,
                data_summary: ListDataSummary::default(),
            });
        }

        // assign the photo associated with the name
        let photos = state.database.list_photos(&self.names).await?;
        for photo in photos.into_iter() {
            if let Some(item) = species.get_mut(&photo.name_id) {
                item.photo = Some(photo.into());
            }
        }

        // assign the data summary associated with the name
        let stats = state.solr.species_stats(&self.names).await?;
        for stat in stats.into_iter() {
            if let Some(item) = species.get_mut(&stat.name.id) {
                item.data_summary = ListDataSummary {
                    whole_genomes: stat.whole_genomes,
                    partial_genomes: stat.partial_genomes,
                    organelles: stat.organelles,
                    barcodes: stat.barcodes,
                    other: stat.total - stat.whole_genomes - stat.organelles - stat.barcodes - stat.partial_genomes,
                }
            }
        }

        // sort by name and output the combined species data
        let mut species: Vec<ListSpecies> = species.into_values().collect();
        species.sort_by(|a, b| a.taxonomy.scientific_name.cmp(&b.taxonomy.scientific_name));
        Ok(species)
    }

    #[instrument(skip(self, ctx))]
    async fn stats(&self, ctx: &Context<'_>) -> Result<ListStats, Error> {
        let state = ctx.data::<State>().unwrap();
        let stats = state.database.list_stats(&self.list, &self.filters).await?;
        Ok(stats)
    }
}
