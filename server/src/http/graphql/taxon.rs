use async_graphql::*;
use serde::Deserialize;
use serde::Serialize;

use crate::database::Database;
use crate::http::Error;

use crate::database::models;
use crate::database::taxa;

use super::common::Taxonomy;


#[derive(Clone, Debug, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum TaxonRank {
    Kingdom,
    Phylum,
    Class,
    Order,
    Family,
    Genus,
    Species,
}


#[derive(MergedObject)]
pub struct Taxon(Taxonomy, TaxonQuery);

impl Taxon {
    pub async fn new(db: &Database, rank: TaxonRank, canonical_name: String) -> Result<Taxon, Error> {
        let rank = match rank {
            TaxonRank::Kingdom => taxa::TaxonRank::Kingdom(canonical_name),
            TaxonRank::Phylum => taxa::TaxonRank::Phylum(canonical_name),
            TaxonRank::Class => taxa::TaxonRank::Class(canonical_name),
            TaxonRank::Order => taxa::TaxonRank::Order(canonical_name),
            TaxonRank::Family => taxa::TaxonRank::Family(canonical_name),
            TaxonRank::Genus => taxa::TaxonRank::Genus(canonical_name),
            TaxonRank::Species => taxa::TaxonRank::Species(canonical_name),
        };

        let taxon = db.taxa.find_by_taxon_rank(&rank).await?;
        let details = taxon.clone().into();
        let query = TaxonQuery { taxon };
        Ok(Taxon(details, query))
    }
}


pub struct TaxonQuery {
    taxon: models::Taxon,
}

#[Object]
impl TaxonQuery {
    async fn tmp(&self, ctx: &Context<'_>) -> Result<String, Error> {
        Ok(self.taxon.canonical_name.clone())
    }
}
