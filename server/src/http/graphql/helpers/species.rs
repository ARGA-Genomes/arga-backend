use std::collections::HashMap;

use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use uuid::Uuid;

use crate::database::{Database, schema};
use crate::database::models::{Taxon, TaxonPhoto};
use crate::http::Error;
use crate::http::graphql::common::SpeciesCard;


pub struct SpeciesHelper {
    database: Database,
}

impl SpeciesHelper {
    pub fn new(database: &Database) -> SpeciesHelper {
        SpeciesHelper { database: database.clone() }
    }

    pub async fn taxonomy(&self, name_ids: &Vec<Uuid>) -> Result<Vec<Taxon>, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.database.pool.get().await?;

        let records = taxa
            .filter(name_id.eq_any(name_ids))
            .load::<Taxon>(&mut conn)
            .await?;

        Ok(records)
    }

    /// Create a list of species cards from a list of name ids
    ///
    /// This will enrich the provided names with additional data such as
    /// taxonomy, species photos, and data summaries.
    pub async fn cards(&self, taxa: Vec<Taxon>) -> Result<Vec<SpeciesCard>, Error> {
        use schema::taxon_photos;
        let mut conn = self.database.pool.get().await?;

        let name_ids = taxa.iter().map(|taxon| taxon.name_id).collect();

        let mut cards: HashMap<Uuid, SpeciesCard> = HashMap::new();

        // create the card with the taxa and some defaults
        for taxon in taxa {
            cards.insert(taxon.name_id, SpeciesCard {
                taxonomy: taxon.into(),
                ..Default::default()
            });
        }

        // assign the photo associated with the name
        let photos = taxon_photos::table
            .filter(taxon_photos::name_id.eq_any(&name_ids))
            .load::<TaxonPhoto>(&mut conn)
            .await?;

        for photo in photos.into_iter() {
            cards.entry(photo.name_id).and_modify(|card| card.photo = Some(photo.into()));
        }

        // assign the data summary associated with the name
        let assembly_stats = self.database.species.assembly_summary(&name_ids).await?;

        for stat in assembly_stats.into_iter() {
            cards.entry(stat.name_id).and_modify(|card| {
                card.data_summary.whole_genomes = stat.whole_genomes;
                card.data_summary.partial_genomes = stat.partial_genomes;
            });
        }

        let marker_stats = self.database.species.marker_summary(&name_ids).await?;

        for stat in marker_stats.into_iter() {
            cards.entry(stat.name_id).and_modify(|card| {
                card.data_summary.barcodes = stat.barcodes
            });
        }

        // sort by name and output the combined species data
        let mut cards: Vec<SpeciesCard> = cards.into_values().collect();
        cards.sort_by(|a, b| a.taxonomy.scientific_name.cmp(&b.taxonomy.scientific_name));
        Ok(cards)
    }
}
