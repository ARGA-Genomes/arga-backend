use arga_core::models;
use async_graphql::*;
use bigdecimal::ToPrimitive;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::common::species::{SortDirection, SpeciesSort};
use super::common::taxonomy::{NomenclaturalActType, TaxonDetails, TaxonomicRank};
use super::common::{convert_filters, AccessionEvent, CollectionEvent, FilterItem, NameDetails, Page, SpeciesCard};
use super::helpers::{self, csv, SpeciesHelper};
use crate::database::extensions::classification_filters::Classification;
use crate::database::extensions::filters::{Filter, FilterKind};
use crate::database::extensions::species_filters::{self};
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
    Supercohort,
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

    Pathovar,
    Serovar,
    Biovar,
}

#[derive(InputObject)]
pub struct TaxonByClassification {
    pub canonical_name: String,
    pub rank: TaxonRank,
    pub dataset_id: Uuid,
}

#[derive(OneofObject)]
pub enum TaxonBy {
    Id(Uuid),
    Classification(TaxonByClassification),
}

#[derive(MergedObject)]
pub struct Taxon(TaxonDetails, TaxonQuery);

impl Taxon {
    pub async fn new(db: &Database, by: TaxonBy, filters: Option<Vec<FilterItem>>) -> Result<Taxon, Error> {
        let taxon = match by {
            TaxonBy::Id(id) => db.taxa.find_by_id(&id).await?,
            TaxonBy::Classification(name) => {
                let classification = into_classification(name.rank, name.canonical_name);
                db.taxa
                    .find_one_by_classification(&classification, &name.dataset_id)
                    .await?
            }
        };

        Ok(Taxon::init(
            taxon,
            match filters {
                Some(filter) => convert_filters(filter)?,
                _ => vec![],
            },
        ))
    }

    pub fn init(taxon: models::Taxon, filters: Vec<Filter>) -> Taxon {
        let details = taxon.clone().into();
        let query = TaxonQuery { taxon, filters };
        Taxon(details, query)
    }
}

pub struct TaxonQuery {
    taxon: models::Taxon,
    filters: Vec<Filter>,
}

#[Object]
impl TaxonQuery {
    async fn hierarchy(&self, ctx: &Context<'_>) -> Result<Vec<TaxonNode>, Error> {
        let state = ctx.data::<State>()?;
        let hierarchy = state.database.taxa.hierarchy(&self.taxon.id).await?;
        let hierarchy = hierarchy.into_iter().map(TaxonNode::from).collect();
        Ok(hierarchy)
    }

    async fn summary(&self, ctx: &Context<'_>, rank: TaxonomicRank) -> Result<RankSummary, Error> {
        let state = ctx.data::<State>()?;
        let summary = state.database.taxa.rank_summary(&self.taxon.id, &rank.into()).await?;
        Ok(summary.into())
    }

    async fn summary_csv(&self, ctx: &Context<'_>, rank: TaxonomicRank) -> Result<String, async_graphql::Error> {
        let state = ctx.data::<State>()?;

        let rank_summary = state.database.taxa.rank_summary(&self.taxon.id, &rank.into()).await?;
        let species_summary = state
            .database
            .taxa
            .rank_summary(&self.taxon.id, &TaxonomicRank::Species.into())
            .await?;

        csv::rank_summaries(rank_summary.into(), species_summary.into()).await
    }

    async fn species_genomic_data_summary(&self, ctx: &Context<'_>) -> Result<Vec<DataBreakdown>, Error> {
        let state = ctx.data::<State>()?;
        let summaries = state.database.taxa.species_genomic_data_summary(&self.taxon.id).await?;
        let summaries = summaries.into_iter().map(|r| r.into()).collect();
        Ok(summaries)
    }

    async fn species_genomic_data_summary_csv(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let state = ctx.data::<State>()?;
        let summaries = state.database.taxa.species_genomic_data_summary(&self.taxon.id).await?;
        let summaries: Vec<DataBreakdown> = summaries.into_iter().map(|r| r.into()).collect();

        let csv = csv::generic(summaries).await?;
        Ok(csv)
    }

    async fn species_genomes_summary(&self, ctx: &Context<'_>) -> Result<Vec<DataBreakdown>, Error> {
        let state = ctx.data::<State>()?;
        let summaries = state.database.taxa.species_genomes_summary(&self.taxon.id).await?;
        let summaries = summaries.into_iter().map(|r| r.into()).collect();
        Ok(summaries)
    }

    async fn species_genomes_summary_csv(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let state = ctx.data::<State>()?;
        let summaries = state.database.taxa.species_genomes_summary(&self.taxon.id).await?;
        let summaries: Vec<DataBreakdown> = summaries.into_iter().map(|r| r.into()).collect();

        let csv = csv::generic(summaries).await?;
        Ok(csv)
    }

    async fn nomenclatural_acts(&self, ctx: &Context<'_>) -> Result<Vec<NomenclaturalAct>, Error> {
        let state = ctx.data::<State>()?;
        let acts = state.database.taxa.nomenclatural_acts(&self.taxon.id).await?;
        let acts = acts.into_iter().map(|r| r.into()).collect();
        Ok(acts)
    }

    async fn taxonomic_acts(&self, ctx: &Context<'_>) -> Result<Vec<TaxonomicAct>, Error> {
        let state = ctx.data::<State>()?;
        let acts = state.database.taxa.taxonomic_acts(&self.taxon.id).await?;
        let acts = acts.into_iter().map(|r| r.into()).collect();
        Ok(acts)
    }

    async fn type_specimens(&self, ctx: &Context<'_>) -> Result<Vec<TypeSpecimen>, Error> {
        let state = ctx.data::<State>()?;
        let specimens = state.database.taxa.type_specimens(&self.taxon.id).await?;
        let specimens = specimens.into_iter().map(|r| r.into()).collect();
        Ok(specimens)
    }

    async fn species(
        &self,
        ctx: &Context<'_>,
        page: i64,
        page_size: i64,
        sort: Option<SpeciesSort>,
        sort_direction: Option<SortDirection>,
    ) -> Result<Page<SpeciesCard>, Error> {
        let state = ctx.data::<State>()?;
        let helper = SpeciesHelper::new(&state.database);

        let classification =
            into_classification(TaxonRank::from(self.taxon.rank.clone()), self.taxon.canonical_name.clone());

        let mut filters = self.filters.clone();
        filters.push(Filter::Include(FilterKind::Classification(classification)));

        let page = state
            .database
            .taxa
            .species(
                &filters,
                &self.taxon.dataset_id,
                page,
                page_size,
                match sort {
                    Some(srt) => srt.into(),
                    _ => species_filters::SpeciesSort::ScientificName,
                },
                match sort_direction {
                    Some(dir) => dir.into(),
                    _ => species_filters::SortDirection::Asc,
                },
            )
            .await?;

        let cards = helper.filtered_cards(page.records).await?;

        Ok(Page {
            records: cards,
            total: page.total,
        })
    }

    async fn species_csv(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let state = ctx.data::<State>()?;

        let classification =
            into_classification(TaxonRank::from(self.taxon.rank.clone()), self.taxon.canonical_name.clone());

        let mut filters = self.filters.clone();
        filters.push(Filter::Include(FilterKind::Classification(classification)));

        let page = state
            .database
            .taxa
            .species(
                &filters,
                &self.taxon.dataset_id,
                1,       // hard coded page size
                1000000, // some arbitrary number of records that hopefully is enough for all of them (1 million)
                species_filters::SpeciesSort::ScientificName,
                species_filters::SortDirection::Asc,
            )
            .await?;

        let csv = helpers::csv::species(page.records).await?;

        Ok(csv)
    }
}


#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "models::PublicationType")]
pub enum PublicationType {
    Book,
    BookChapter,
    JournalArticle,
    JournalVolume,
    ProceedingsPaper,
    Url,
}

#[derive(SimpleObject)]
pub struct Publication {
    pub entity_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub published_year: i32,
    pub published_date: Option<DateTime<Utc>>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub doi: Option<String>,
    pub source_urls: Vec<String>,
    pub publication_type: Option<PublicationType>,
    pub citation: Option<String>,
}

impl From<models::Publication> for Publication {
    fn from(value: models::Publication) -> Self {
        Self {
            entity_id: value.entity_id,
            title: value.title,
            authors: value.authors.into_iter().filter_map(|v| v).collect(),
            published_year: value.published_year,
            published_date: value.published_date,
            language: value.language,
            publisher: value.publisher,
            doi: value.doi,
            source_urls: value
                .source_urls
                .map(|i| i.into_iter().filter_map(|v| v).collect())
                .unwrap_or_default(),
            publication_type: value.publication_type.map(|t| t.into()),
            citation: value.citation,
        }
    }
}


#[derive(SimpleObject)]
pub struct NomenclaturalAct {
    pub entity_id: String,
    pub act: NomenclaturalActType,
    pub source_url: String,
    pub publication: Publication,
    pub name: super::names::Name,
    pub acted_on: NameDetails,
}

impl From<taxa::NomenclaturalAct> for NomenclaturalAct {
    fn from(value: taxa::NomenclaturalAct) -> Self {
        Self {
            entity_id: value.entity_id,
            act: value.act.into(),
            source_url: value.source_url,
            publication: value.publication.into(),
            name: super::names::Name::new(value.name),
            acted_on: value.acted_on.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct TaxonomicAct {
    pub entity_id: String,
    pub source_url: Option<String>,
    pub taxon: TaxonDetails,
    pub accepted_taxon: Option<TaxonDetails>,
    pub data_created_at: Option<DateTime<Utc>>,
    pub data_updated_at: Option<DateTime<Utc>>,
}

impl From<taxa::TaxonomicAct> for TaxonomicAct {
    fn from(value: taxa::TaxonomicAct) -> Self {
        Self {
            entity_id: value.entity_id,
            source_url: value.source_url,
            taxon: value.taxon.into(),
            accepted_taxon: value.accepted_taxon.map(|t| t.into()),
            data_created_at: value.data_created_at,
            data_updated_at: value.data_updated_at,
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
pub struct RankSummary {
    /// Total amount of taxa in the rank
    pub total: i64,
    /// Total amount of taxa in the rank with genomes
    pub genomes: i64,
    /// Total amount of taxa in the rank with any genomic data
    pub genomic_data: i64,
}

impl From<taxa::RankSummary> for RankSummary {
    fn from(value: taxa::RankSummary) -> Self {
        Self {
            total: value.total,
            genomes: value.genomes,
            genomic_data: value.genomic_data,
        }
    }
}


#[derive(Clone, Debug, SimpleObject)]
pub struct TypeSpecimen {
    pub accession: AccessionEvent,
    pub collection: CollectionEvent,
    pub name: NameDetails,
}

impl From<taxa::TypeSpecimen> for TypeSpecimen {
    fn from(value: taxa::TypeSpecimen) -> Self {
        TypeSpecimen {
            accession: value.accession.into(),
            collection: value.collection.into(),
            name: value.name.into(),
        }
    }
}


#[derive(SimpleObject, Serialize)]
pub struct DataBreakdown {
    pub scientific_name: String,
    pub canonical_name: String,
    pub loci: i64,
    pub genomes: i64,
    pub specimens: i64,
    pub other: i64,
    pub total_genomic: i64,
}

impl From<taxa::DataSummary> for DataBreakdown {
    fn from(value: taxa::DataSummary) -> Self {
        Self {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
            loci: value.loci.map(|v| v.to_i64().unwrap_or(0)).unwrap_or(0),
            genomes: value.genomes.map(|v| v.to_i64().unwrap_or(0)).unwrap_or(0),
            specimens: value.specimens.map(|v| v.to_i64().unwrap_or(0)).unwrap_or(0),
            other: value.other.map(|v| v.to_i64().unwrap_or(0)).unwrap_or(0),
            total_genomic: value.total_genomic.map(|v| v.to_i64().unwrap_or(0)).unwrap_or(0),
        }
    }
}

impl From<taxa::Summary> for DataBreakdown {
    fn from(value: taxa::Summary) -> Self {
        Self {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
            loci: value.loci.map(|v| v.to_i64().unwrap_or(0)).unwrap_or(0),
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
            scientific_name: "".to_string(),
            canonical_name: value.canonical_name,
            loci: value.loci,
            genomes: value.genomes,
            specimens: value.specimens,
            other: value.other,
            total_genomic: value.total_genomic,
        }
    }
}


pub fn into_classification(rank: TaxonRank, value: String) -> Classification {
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
        TaxonRank::Supercohort => Classification::Supercohort(value),
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
        TaxonRank::Pathovar => Classification::Pathovar(value),
        TaxonRank::Serovar => Classification::Serovar(value),
        TaxonRank::Biovar => Classification::Biovar(value),
    }
}
