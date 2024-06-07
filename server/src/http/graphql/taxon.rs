use arga_core::models;
use async_graphql::*;
use bigdecimal::ToPrimitive;
use serde::{Deserialize, Serialize};

use super::common::datasets::DatasetDetails;
use super::common::taxonomy::{TaxonDetails, TaxonomicRank, TaxonomicStatus};
use crate::database::extensions::classification_filters::Classification;
use crate::database::{taxa, Database};
use crate::http::{Context as State, Error};

#[derive(Clone, Debug, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
#[graphql(remote = "models::TaxonomicRank")]
pub enum TaxonRank {
    Domain,

    Superkingdom,
    Kingdom,
    Subkingdom,
    Infrakingdom,

    Superphylum,
    Phylum,
    Subphylum,
    Infraphylum,
    Parvphylum,

    Gigaclass,
    Megaclass,
    Superclass,
    Class,
    Subclass,
    Infraclass,
    Subterclass,

    Superorder,
    Order,
    Hyporder,
    Minorder,
    Suborder,
    Infraorder,
    Parvorder,

    Epifamily,
    Superfamily,
    Family,
    Subfamily,

    Supertribe,
    Tribe,
    Subtribe,
    Genus,
    Subgenus,
    Infragenus,
    Species,
    Subspecies,

    Variety,
    Subvariety,

    Natio,
    Mutatio,

    Unranked,
    HigherTaxon,

    AggregateGenera,
    AggregateSpecies,
    Cohort,
    Subcohort,
    Division,
    IncertaeSedis,
    Section,
    Subsection,
    Subdivision,

    Regnum,
    Familia,
    Classis,
    Ordo,
    Varietas,
    Forma,
    Subforma,
    Subclassis,
    Superordo,
    Sectio,
    Subsectio,
    Nothovarietas,
    Subvarietas,
    Series,
    Subseries,
    Superspecies,
    Infraspecies,
    Subfamilia,
    Subordo,
    Regio,
    SpecialForm,
}

#[derive(MergedObject)]
pub struct Taxon(TaxonDetails, TaxonQuery);

impl Taxon {
    pub async fn new(db: &Database, rank: TaxonRank, canonical_name: String) -> Result<Taxon, Error> {
        let classification = into_classification(rank, canonical_name);
        let taxon = db.taxa.find_by_classification(&classification).await?;
        let mut details: TaxonDetails = taxon.clone().into();

        // get source info
        let dataset = db.datasets.find_by_id(&taxon.dataset_id).await?;
        details.source = Some(dataset.name);
        details.source_url = dataset.url;

        let query = TaxonQuery { classification, taxon };
        Ok(Taxon(details, query))
    }
}

pub struct TaxonQuery {
    classification: Classification,
    taxon: models::Taxon,
}

#[Object]
impl TaxonQuery {
    async fn hierarchy(&self, ctx: &Context<'_>) -> Result<Vec<TaxonNode>, Error> {
        let state = ctx.data::<State>().unwrap();
        let hierarchy = state.database.taxa.hierarchy(&self.classification).await?;
        let hierarchy = hierarchy.into_iter().map(TaxonNode::from).collect();
        Ok(hierarchy)
    }

    async fn summary(&self, ctx: &Context<'_>) -> Result<TaxonSummary, Error> {
        let state = ctx.data::<State>().unwrap();
        let summary = state.database.taxa.taxon_summary(&self.classification).await?;
        Ok(summary.into())
    }

    async fn descendants(&self, ctx: &Context<'_>, rank: TaxonomicRank) -> Result<Vec<TaxonSummary>> {
        let state = ctx.data::<State>().unwrap();
        let summaries = state
            .database
            .taxa
            .descendant_summary(&self.classification, rank.into())
            .await?;
        let summaries = summaries.into_iter().map(|r| r.into()).collect();
        Ok(summaries)
    }

    async fn species_summary(&self, ctx: &Context<'_>) -> Result<Vec<DataBreakdown>, Error> {
        let state = ctx.data::<State>().unwrap();
        let summaries = state.database.taxa.species_summary(&self.classification).await?;
        let summaries = summaries.into_iter().map(|r| r.into()).collect();
        Ok(summaries)
    }

    async fn species_genome_summary(&self, ctx: &Context<'_>) -> Result<Vec<DataBreakdown>, Error> {
        let state = ctx.data::<State>().unwrap();
        let summaries = state.database.taxa.species_genome_summary(&self.classification).await?;
        let summaries = summaries.into_iter().map(|r| r.into()).collect();
        Ok(summaries)
    }

    async fn history(&self, ctx: &Context<'_>) -> Result<Vec<HistoryItem>, Error> {
        let state = ctx.data::<State>().unwrap();
        let history = state.database.taxa.history(&self.taxon.id).await?;
        let history = history.into_iter().map(|r| r.into()).collect();
        Ok(history)
    }
}

#[derive(SimpleObject)]
pub struct NamePublication {
    pub citation: Option<String>,
    pub published_year: Option<i32>,
    pub source_url: Option<String>,
    pub type_citation: Option<String>,
}

impl From<models::NamePublication> for NamePublication {
    fn from(value: models::NamePublication) -> Self {
        Self {
            citation: value.citation,
            published_year: value.published_year,
            source_url: value.source_url,
            type_citation: value.type_citation,
        }
    }
}

#[derive(SimpleObject)]
pub struct HistoryItem {
    pub dataset: DatasetDetails,
    pub nomenclatural_act: String,
    pub status: TaxonomicStatus,
    pub rank: TaxonomicRank,
    pub scientific_name: String,
    pub canonical_name: String,
    pub authorship: Option<String>,
    pub citation: Option<String>,
    pub source_url: Option<String>,
    pub publication: Option<NamePublication>,
    pub entity_id: Option<String>,
}

impl From<taxa::HistoryItem> for HistoryItem {
    fn from(value: taxa::HistoryItem) -> Self {
        Self {
            dataset: value.dataset.into(),
            nomenclatural_act: value.act.name,
            status: value.taxon.status.into(),
            rank: value.taxon.rank.into(),
            scientific_name: value.taxon.scientific_name,
            canonical_name: value.taxon.canonical_name,
            authorship: value.taxon.authorship,
            citation: value.taxon.citation,
            source_url: value.source_url,
            publication: value.publication.map(|publication| publication.into()),
            entity_id: value.entity_id,
        }
    }
}

#[derive(SimpleObject)]
pub struct TaxonNode {
    pub rank: TaxonomicRank,
    pub scientific_name: String,
    pub canonical_name: String,
    pub depth: i32,
}

impl From<models::TaxonTreeNode> for TaxonNode {
    fn from(value: models::TaxonTreeNode) -> Self {
        Self {
            rank: value.rank.into(),
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
            depth: value.depth,
        }
    }
}

#[derive(SimpleObject)]
pub struct TaxonSummary {
    /// The name of the taxon this summary pertains to
    pub canonical_name: String,
    /// Total amount of descendant species
    pub species: i64,
    /// Total amount of descendant species with genomes
    pub species_genomes: i64,
    /// Total amount of descendant species with any genomic data
    pub species_data: i64,
}

impl From<taxa::TaxonSummary> for TaxonSummary {
    fn from(value: taxa::TaxonSummary) -> Self {
        Self {
            canonical_name: value.canonical_name,
            species: value.species,
            species_genomes: value.species_genomes,
            species_data: value.species_data,
        }
    }
}

#[derive(SimpleObject)]
pub struct DataBreakdown {
    pub name: String,
    pub markers: i64,
    pub genomes: i64,
    pub specimens: i64,
    pub other: i64,
    pub total_genomic: i64,
}

impl From<taxa::DataSummary> for DataBreakdown {
    fn from(value: taxa::DataSummary) -> Self {
        Self {
            name: value.canonical_name,
            markers: value.markers.map(|v| v.to_i64().unwrap_or(0)).unwrap_or(0),
            genomes: value.genomes.map(|v| v.to_i64().unwrap_or(0)).unwrap_or(0),
            specimens: value.specimens.map(|v| v.to_i64().unwrap_or(0)).unwrap_or(0),
            other: value.other.map(|v| v.to_i64().unwrap_or(0)).unwrap_or(0),
            total_genomic: value.total_genomic.map(|v| v.to_i64().unwrap_or(0)).unwrap_or(0),
        }
    }
}

impl From<taxa::SpeciesSummary> for DataBreakdown {
    fn from(value: taxa::SpeciesSummary) -> Self {
        Self {
            name: value.name,
            markers: value.markers,
            genomes: value.genomes,
            specimens: value.specimens,
            other: value.other,
            total_genomic: value.total_genomic,
        }
    }
}

fn into_classification(rank: TaxonRank, value: String) -> Classification {
    match rank {
        TaxonRank::Domain => Classification::Domain(value),
        TaxonRank::Superkingdom => Classification::Superkingdom(value),
        TaxonRank::Kingdom => Classification::Kingdom(value),
        TaxonRank::Subkingdom => Classification::Subkingdom(value),
        TaxonRank::Infrakingdom => Classification::Infrakingdom(value),
        TaxonRank::Superphylum => Classification::Superphylum(value),
        TaxonRank::Phylum => Classification::Phylum(value),
        TaxonRank::Subphylum => Classification::Subphylum(value),
        TaxonRank::Infraphylum => Classification::Infraphylum(value),
        TaxonRank::Parvphylum => Classification::Parvphylum(value),
        TaxonRank::Gigaclass => Classification::Gigaclass(value),
        TaxonRank::Megaclass => Classification::Megaclass(value),
        TaxonRank::Superclass => Classification::Superclass(value),
        TaxonRank::Class => Classification::Class(value),
        TaxonRank::Subclass => Classification::Subclass(value),
        TaxonRank::Infraclass => Classification::Infraclass(value),
        TaxonRank::Subterclass => Classification::Subterclass(value),
        TaxonRank::Superorder => Classification::Superorder(value),
        TaxonRank::Order => Classification::Order(value),
        TaxonRank::Hyporder => Classification::Hyporder(value),
        TaxonRank::Minorder => Classification::Minorder(value),
        TaxonRank::Suborder => Classification::Suborder(value),
        TaxonRank::Infraorder => Classification::Infraorder(value),
        TaxonRank::Parvorder => Classification::Parvorder(value),
        TaxonRank::Epifamily => Classification::Epifamily(value),
        TaxonRank::Superfamily => Classification::Superfamily(value),
        TaxonRank::Family => Classification::Family(value),
        TaxonRank::Subfamily => Classification::Subfamily(value),
        TaxonRank::Supertribe => Classification::Supertribe(value),
        TaxonRank::Tribe => Classification::Tribe(value),
        TaxonRank::Subtribe => Classification::Subtribe(value),
        TaxonRank::Genus => Classification::Genus(value),
        TaxonRank::Subgenus => Classification::Subgenus(value),
        TaxonRank::Species => Classification::Species(value),
        TaxonRank::Subspecies => Classification::Subspecies(value),
        TaxonRank::Variety => Classification::Variety(value),
        TaxonRank::Subvariety => Classification::Subvariety(value),
        TaxonRank::Natio => Classification::Natio(value),
        TaxonRank::Mutatio => Classification::Mutatio(value),
        TaxonRank::Unranked => Classification::Unranked(value),
        TaxonRank::HigherTaxon => Classification::HigherTaxon(value),
        TaxonRank::AggregateGenera => Classification::AggregateGenera(value),
        TaxonRank::AggregateSpecies => Classification::AggregateSpecies(value),
        TaxonRank::Cohort => Classification::Cohort(value),
        TaxonRank::Subcohort => Classification::Subcohort(value),
        TaxonRank::Division => Classification::Division(value),
        TaxonRank::IncertaeSedis => Classification::IncertaeSedis(value),
        TaxonRank::Infragenus => Classification::Infragenus(value),
        TaxonRank::Section => Classification::Section(value),
        TaxonRank::Subsection => Classification::Subsection(value),
        TaxonRank::Subdivision => Classification::Subdivision(value),
        TaxonRank::Regnum => Classification::Regnum(value),
        TaxonRank::Familia => Classification::Familia(value),
        TaxonRank::Classis => Classification::Classis(value),
        TaxonRank::Ordo => Classification::Ordo(value),
        TaxonRank::Varietas => Classification::Varietas(value),
        TaxonRank::Forma => Classification::Forma(value),
        TaxonRank::Subforma => Classification::Subforma(value),
        TaxonRank::Subclassis => Classification::Subclassis(value),
        TaxonRank::Superordo => Classification::Superordo(value),
        TaxonRank::Sectio => Classification::Sectio(value),
        TaxonRank::Subsectio => Classification::Subsectio(value),
        TaxonRank::Nothovarietas => Classification::Nothovarietas(value),
        TaxonRank::Subvarietas => Classification::Subvarietas(value),
        TaxonRank::Series => Classification::Series(value),
        TaxonRank::Subseries => Classification::Subseries(value),
        TaxonRank::Superspecies => Classification::Superspecies(value),
        TaxonRank::Infraspecies => Classification::Infraspecies(value),
        TaxonRank::Subfamilia => Classification::Subfamilia(value),
        TaxonRank::Subordo => Classification::Subordo(value),
        TaxonRank::Regio => Classification::Regio(value),
        TaxonRank::SpecialForm => Classification::SpecialForm(value),
    }
}
