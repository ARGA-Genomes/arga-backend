use std::collections::HashMap;

use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use tracing::instrument;
use uuid::Uuid;

use crate::database::models::TaxonPhoto;
use crate::http::Error;
use crate::http::Context as State;

use crate::index::genus::{GetGenus, Taxonomy};
use crate::index::lists::ListDataSummary;

use super::lists::ListSpecies;


pub struct Genus {
    pub genus: String,
}

#[Object]
impl Genus {
    #[instrument(skip(self, ctx))]
    async fn taxonomy(&self, ctx: &Context<'_>) -> Result<Taxonomy, Error> {
        let state = ctx.data::<State>().unwrap();
        let taxonomy = state.database.taxonomy(&self.genus).await.unwrap();

        Ok(taxonomy)
    }

    async fn species(&self, ctx: &Context<'_>) -> Result<Vec<ListSpecies>, Error> {
        use crate::database::schema::taxon_photos;

        let state = ctx.data::<State>().unwrap();
        let mut conn = state.database.pool.get().await?;

        let mut species: HashMap<Uuid, ListSpecies> = HashMap::new();

        let ranked = state.database.genus.species(&self.genus).await?;
        for taxon in ranked {
            let taxonomy = Taxonomy {
                scientific_name: taxon.scientific_name.unwrap_or_default(),
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
        };

        // assign the photo associated with the name
        let name_ids: Vec<Uuid> = species.keys().map(|k| k.clone()).collect();
        let photos = taxon_photos::table
            .filter(taxon_photos::name_id.eq_any(&name_ids))
            .load::<TaxonPhoto>(&mut conn)
            .await?;

        for photo in photos.into_iter() {
            if let Some(item) = species.get_mut(&photo.name_id) {
                item.photo = Some(photo.into());
            }
        }

        // assign the data summary associated with the name
        let stats = state.database.genus.species_summary(&name_ids).await?;
        for stat in stats.into_iter() {
            if let Some(item) = species.get_mut(&stat.name_id) {
                item.data_summary = ListDataSummary {
                    whole_genomes: stat.whole_genomes,
                    partial_genomes: stat.partial_genomes,
                    organelles: stat.organelles,
                    barcodes: stat.barcodes,
                    other: stat.other,
                }
            }
        }

        // sort by name and output the combined species data
        let mut species: Vec<ListSpecies> = species.into_values().collect();
        species.sort_by(|a, b| a.taxonomy.scientific_name.cmp(&b.taxonomy.scientific_name));
        Ok(species)
    }
}
