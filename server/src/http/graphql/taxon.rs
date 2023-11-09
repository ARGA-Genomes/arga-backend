use async_graphql::*;
use serde::Deserialize;
use serde::Serialize;

use crate::database::Database;
use crate::http::Error;
use crate::http::Context as State;

use crate::database::taxa;
use super::common::taxonomy::TaxonDetails;


#[derive(Clone, Debug, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum TaxonRank {
    Domain,
    Kingdom,
    Phylum,
    Class,
    Order,
    Family,
    Genus,
    Species,
}


#[derive(MergedObject)]
pub struct Taxon(TaxonDetails, TaxonQuery);

impl Taxon {
    pub async fn new(db: &Database, rank: TaxonRank, canonical_name: String) -> Result<Taxon, Error> {
        let rank = into_taxon_rank(rank, canonical_name);
        let taxon = db.taxa.find_by_taxon_rank(&rank).await?;
        let details = taxon.clone().into();
        let query = TaxonQuery { rank };
        Ok(Taxon(details, query))
    }
}


pub struct TaxonQuery {
    rank: taxa::TaxonRank,
}

#[Object]
impl TaxonQuery {
    async fn summary(&self, ctx: &Context<'_>) -> Result<TaxonSummary, Error> {
        let state = ctx.data::<State>().unwrap();
        let rank = self.rank.clone().into();
        let summary = state.database.taxa.taxon_summary(&rank).await?;
        Ok(summary.into())
    }

    async fn data_summary(&self, ctx: &Context<'_>) -> Result<Vec<DataBreakdown>, Error> {
        let state = ctx.data::<State>().unwrap();
        let summaries = state.database.taxa.data_summary(&self.rank).await?;
        let summaries = summaries.into_iter().map(|r| r.into()).collect();
        Ok(summaries)
    }

    async fn species_summary(&self, ctx: &Context<'_>) -> Result<Vec<DataBreakdown>, Error> {
        let state = ctx.data::<State>().unwrap();
        let rank = self.rank.clone().into();
        let summaries = state.database.taxa.species_summary(&rank).await?;
        let summaries = summaries.into_iter().map(|r| r.into()).collect();
        Ok(summaries)
    }
}


#[derive(SimpleObject)]
pub struct TaxonSummary {
    pub children: i64,
    pub species: i64,
}

impl From<taxa::TaxonSummary> for TaxonSummary {
    fn from(value: taxa::TaxonSummary) -> Self {
        Self {
            children: value.children,
            species: value.species,
        }
    }
}


#[derive(SimpleObject)]
pub struct DataBreakdown {
    pub name: Option<String>,
    pub markers: i64,
    pub genomes: i64,
    pub specimens: i64,
    pub other: i64,
}

impl From<taxa::DataSummary> for DataBreakdown {
    fn from(value: taxa::DataSummary) -> Self {
        Self {
            name: value.rank,
            markers: value.markers.unwrap_or(0),
            genomes: value.genomes.unwrap_or(0),
            specimens: value.specimens.unwrap_or(0),
            other: value.other.unwrap_or(0),
        }
    }
}

impl From<taxa::SpeciesSummary> for DataBreakdown {
    fn from(value: taxa::SpeciesSummary) -> Self {
        Self {
            name: Some(value.name),
            markers: value.markers.unwrap_or(0) as i64,
            genomes: value.genomes.unwrap_or(0) as i64,
            specimens: value.specimens.unwrap_or(0) as i64,
            other: value.other.unwrap_or(0) as i64,
        }
    }
}


fn into_taxon_rank(rank: TaxonRank, value: String) -> taxa::TaxonRank {
    match rank {
        TaxonRank::Domain => taxa::TaxonRank::Domain(value),
        TaxonRank::Kingdom => taxa::TaxonRank::Kingdom(value),
        TaxonRank::Phylum => taxa::TaxonRank::Phylum(value),
        TaxonRank::Class => taxa::TaxonRank::Class(value),
        TaxonRank::Order => taxa::TaxonRank::Order(value),
        TaxonRank::Family => taxa::TaxonRank::Family(value),
        TaxonRank::Genus => taxa::TaxonRank::Genus(value),
        TaxonRank::Species => taxa::TaxonRank::Species(value),
    }
}
