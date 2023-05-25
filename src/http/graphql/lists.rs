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
use crate::index::lists::GetListNames;
use crate::index::lists::GetListPhotos;
use crate::index::lists::GetListTaxa;
use crate::index::providers::db::Database;
use crate::index::providers::db::models::TaxonPhoto;
use crate::index::providers::db::models::{UserTaxaList, Name as ArgaName};


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct ListSpecies {
    pub taxonomy: Taxonomy,
    pub photo: Option<SpeciesPhoto>,
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
    pub list: UserTaxaList,
    pub names: Vec<ArgaName>,
}

#[Object]
impl Lists {
    #[graphql(skip)]
    pub async fn new(db: &Database, name: String) -> Result<Lists, Error> {
        use crate::schema::user_taxa_lists as lists;
        let mut conn = db.pool.get().await?;

        let list = lists::table
            .filter(lists::name.eq(&name))
            .get_result::<UserTaxaList>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = list {
            return Err(Error::NotFound(name));
        }

        let list = list?;
        let names = db.list_names(&list).await?;

        Ok(Lists { list, names })
    }

    #[instrument(skip(self, ctx))]
    async fn species(&self, ctx: &Context<'_>) -> Result<Vec<ListSpecies>, Error> {
        let state = ctx.data::<State>().unwrap();

        let mut species: HashMap<Uuid, ListSpecies> = HashMap::new();

        let taxa = state.db_provider.list_taxa(&self.names).await?;
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
            };

            species.insert(taxon.name_id, ListSpecies {
                taxonomy,
                photo: None,
            });
        }

        let photos = state.db_provider.list_photos(&self.names).await?;
        for photo in photos.into_iter() {
            if let Some(item) = species.get_mut(&photo.name_id) {
                item.photo = Some(photo.into());
            }
        }

        let mut species: Vec<ListSpecies> = species.into_values().collect();
        species.sort_by(|a, b| a.taxonomy.scientific_name.cmp(&b.taxonomy.scientific_name));
        Ok(species)
    }
}
