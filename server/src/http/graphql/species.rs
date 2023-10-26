use arga_core::models;
use arga_core::models::IndigenousKnowledge;
use async_graphql::*;
use chrono::NaiveDateTime;
use serde::Deserialize;
use serde::Serialize;
use tracing::instrument;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::http::Error;
use crate::http::Context as State;

use crate::index::species::{
    ConservationStatus,
    GetConservationStatus,
    GetTraceFiles,
    TraceFile,
    GenomicData,
    Photo,
    GetSpecies,
    GetGenomicData,
    GetMedia,
};
use crate::database::{schema, Database};
use crate::database::models::Name as ArgaName;
use crate::database::species;
use super::common::{
    Page,
    Taxonomy,
    WholeGenomeFilterItem,
    convert_whole_genome_filters,
};
use super::dataset::DatasetDetails;
use super::markers::SpeciesMarker;


pub struct Species {
    pub canonical_name: String,
    pub name: ArgaName,
    pub all_names: Vec<ArgaName>,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct IndigenousEcologicalTrait {
    pub id: String,
    pub name: String,
    pub dataset_name: String,
    pub food_use: bool,
    pub medicinal_use: bool,
    pub cultural_connection: bool,
    pub source_url: Option<String>,
}

impl From<(IndigenousKnowledge, String)> for IndigenousEcologicalTrait {
    fn from(source: (IndigenousKnowledge, String)) -> Self {
        let (value, dataset_name) = source;
        Self {
            id: value.id.to_string(),
            name: value.name,
            dataset_name,
            food_use: value.food_use,
            medicinal_use: value.medicinal_use,
            cultural_connection: value.cultural_connection,
            source_url: value.source_url,
        }
    }
}


#[Object]
impl Species {
    #[graphql(skip)]
    pub async fn new(db: &Database, canonical_name: String) -> Result<Species, Error> {
        use schema::names;
        let mut conn = db.pool.get().await?;

        let names = names::table
            .filter(names::canonical_name.eq(&canonical_name))
            .load::<ArgaName>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = names {
            return Err(Error::NotFound(canonical_name));
        }
        let names = names?;

        Ok(Species { canonical_name, name: names[0].clone(), all_names: names })
    }

    #[instrument(skip(self, ctx))]
    async fn taxonomy(&self, ctx: &Context<'_>) -> Result<Taxonomy, Error> {
        let state = ctx.data::<State>().unwrap();
        let synonyms = state.database.species.synonyms(&self.name.id).await?;
        let vernacular_names = state.database.species.vernacular_names(&self.name.id).await?;

        let mut taxonomy: Taxonomy = state.database.species.taxonomy(&self.name.id).await?.into();
        taxonomy.synonyms = synonyms.into_iter().map(|s| s.into()).collect();
        taxonomy.vernacular_names = vernacular_names.into_iter().map(|s| s.into()).collect();

        Ok(taxonomy)
    }

    #[instrument(skip(self, _ctx))]
    async fn regions(&self, _ctx: &Context<'_>) -> Regions {
        Regions { name: self.name.clone() }
    }

    #[instrument(skip(self, ctx))]
    async fn data(&self, ctx: &Context<'_>) -> Result<Vec<GenomicData>, Error> {
        let state = ctx.data::<State>().unwrap();
        let taxonomy = state.database.taxonomy(&self.name).await?;
        let data = state.solr.genomic_data(&taxonomy.canonical_name).await?;

        Ok(data)
    }

    #[instrument(skip(self, ctx))]
    async fn photos(&self, ctx: &Context<'_>) -> Result<Vec<Photo>, Error> {
        let state = ctx.data::<State>().unwrap();
        let photos = state.database.photos(&self.name).await?;
        Ok(photos)
    }

    #[instrument(skip(self, ctx))]
    async fn specimens(&self, ctx: &Context<'_>, page: i64, page_size: i64) -> Result<Page<SpecimenSummary>, Error> {
        let state = ctx.data::<State>().unwrap();
        let page = state.database.species.specimens(&self.name, page, page_size).await?;
        let specimens = page.records.into_iter().map(|r| r.into()).collect();
        Ok(Page {
            records: specimens,
            total: page.total,
        })
    }

    #[instrument(skip(self, ctx))]
    async fn conservation(&self, ctx: &Context<'_>) -> Result<Vec<ConservationStatus>> {
        let state = ctx.data::<State>().unwrap();

        let mut statuses = Vec::new();
        for name in &self.all_names {
            let records = state.database.conservation_status(name).await?;
            statuses.extend(records);
        }

        Ok(statuses)
    }

    #[instrument(skip(self, ctx))]
    async fn whole_genomes(
        &self,
        ctx: &Context<'_>,
        page: i64,
        page_size: i64,
        filters: Option<Vec<WholeGenomeFilterItem>>,
    ) -> Result<Page<WholeGenome>, Error>
    {
        let state = ctx.data::<State>().unwrap();
        let filters = convert_whole_genome_filters(filters.unwrap_or_default())?;
        let page = state.database.species.whole_genomes(&self.name, &filters, page, page_size).await?;
        let sequences = page.records.into_iter().map(|r| r.into()).collect();
        Ok(Page {
            records: sequences,
            total: page.total,
        })
    }

    #[instrument(skip(self, ctx))]
    async fn trace_files(&self, ctx: &Context<'_>) -> Result<Vec<TraceFile>, Error> {
        let state = ctx.data::<State>().unwrap();
        let records = state.database.trace_files(&self.all_names).await?;
        Ok(records)
    }

    async fn markers(&self, ctx: &Context<'_>, page: i64, page_size: i64) -> Result<Page<SpeciesMarker>, Error> {
        let state = ctx.data::<State>().unwrap();
        let page = state.database.species.markers(&self.name, page, page_size).await?;
        let markers = page.records.into_iter().map(|m| m.into()).collect();
        Ok(Page {
            records: markers,
            total: page.total,
        })
    }

    async fn reference_genome(&self, ctx: &Context<'_>) -> Result<Option<WholeGenome>, Error> {
        let state = ctx.data::<State>().unwrap();
        let genome = state.database.species.reference_genome(&self.name).await?;
        let genome = genome.map(|g| g.into());
        Ok(genome)
    }

    async fn indigenous_ecological_knowledge(&self, ctx: &Context<'_>) -> Result<Vec<IndigenousEcologicalTrait>, Error> {
        let state = ctx.data::<State>().unwrap();
        let name_ids: Vec<Uuid> = self.all_names.iter().map(|name| name.id.clone()).collect();
        let records = state.database.species.indigenous_knowledge(&name_ids).await?;
        let traits = records.into_iter().map(|r| r.into()).collect();
        Ok(traits)
    }

    async fn attributes(&self, ctx: &Context<'_>) -> Result<Vec<NameAttribute>, Error> {
        let state = ctx.data::<State>().unwrap();
        let records = state.database.species.attributes(&self.name).await?;
        let attributes = records.into_iter().map(|r| r.into()).collect();
        Ok(attributes)
    }
}


pub struct Regions {
    name: ArgaName,
}

#[Object]
impl Regions {
    async fn ibra(&self, ctx: &Context<'_>) -> Result<Vec<RegionDistribution>, Error> {
        let state = ctx.data::<State>().unwrap();
        let regions = state.database.species.regions_ibra(&self.name).await?;
        let regions = regions.into_iter().map(RegionDistribution::new).collect();
        Ok(regions)
    }

    async fn imcra(&self, ctx: &Context<'_>) -> Result<Vec<RegionDistribution>, Error> {
        let state = ctx.data::<State>().unwrap();
        let regions = state.database.species.regions_imcra(&self.name).await?;
        let regions = regions.into_iter().map(RegionDistribution::new).collect();
        Ok(regions)
    }
}


#[derive(MergedObject)]
pub struct RegionDistribution(RegionDetails, RegionQuery);

impl RegionDistribution {
    pub fn new(regions: models::Regions) -> RegionDistribution {
        let details = regions.clone().into();
        let query = RegionQuery { regions };
        RegionDistribution(details, query)
    }
}

/// Regions that a species inhabit.
///
/// Regions are less granular than a distribution and serves to more
/// clearly identify geographic locations inhabited by a particular species.
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, SimpleObject)]
pub struct RegionDetails {
    pub names: Vec<String>,
}


pub struct RegionQuery {
    regions: models::Regions,
}

#[Object]
impl RegionQuery {
    pub async fn dataset(&self, ctx: &Context<'_>) -> Result<DatasetDetails, Error> {
        let state = ctx.data::<State>().unwrap();
        let dataset = state.database.datasets.find_by_id(&self.regions.dataset_id).await?;
        Ok(dataset.into())
    }
}

impl From<models::Regions> for RegionDetails {
    fn from(region: models::Regions) -> Self {
        Self {
            names: region.values.into_iter().filter_map(|r| r).collect(),
        }
    }
}


#[derive(Clone, Debug, SimpleObject)]
pub struct WholeGenome {
    pub sequence_id: Uuid,
    pub dna_extract_id: Uuid,
    pub dataset_name: String,

    pub record_id: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub accession: String,
    pub sequenced_by: Option<String>,
    pub material_sample_id: Option<String>,
    pub estimated_size: Option<String>,

    pub assembled_by: Option<String>,
    pub name: Option<String>,
    pub version_status: Option<String>,
    pub quality: Option<String>,
    pub assembly_type: Option<String>,
    pub genome_size: Option<i64>,

    pub annotated_by: Option<String>,
    pub representation: Option<String>,
    pub release_type: Option<String>,

    pub release_date: Option<String>,
    pub deposited_by: Option<String>,
    pub data_type: Option<String>,
    pub excluded_from_refseq: Option<String>,
}

impl From<models::WholeGenome> for WholeGenome {
    fn from(value: models::WholeGenome) -> Self {
        Self {
            sequence_id: value.sequence_id,
            dna_extract_id: value.dna_extract_id,
            dataset_name: value.dataset_name,
            record_id: value.record_id,
            latitude: value.latitude,
            longitude: value.longitude,
            accession: value.accession,
            sequenced_by: value.sequenced_by,
            material_sample_id: value.material_sample_id,
            estimated_size: value.estimated_size,
            assembled_by: value.assembled_by,
            name: value.name,
            version_status: value.version_status,
            quality: value.quality,
            assembly_type: value.assembly_type,
            genome_size: value.genome_size,
            annotated_by: value.annotated_by,
            representation: value.representation,
            release_type: value.release_type,
            release_date: value.release_date,
            deposited_by: value.deposited_by,
            data_type: value.data_type,
            excluded_from_refseq: value.excluded_from_refseq,
        }
    }
}


/// A specimen from a specific species.
#[derive(Clone, Debug, SimpleObject)]
pub struct SpecimenSummary {
    pub id: Uuid,
    pub dataset_name: String,
    pub record_id: String,
    pub accession: Option<String>,
    pub institution_code: Option<String>,
    pub institution_name: Option<String>,
    pub type_status: Option<String>,
    pub locality: Option<String>,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,

    pub sequences: i64,
    pub whole_genomes: i64,
    pub markers: i64,
}

impl From<species::SpecimenSummary> for SpecimenSummary {
    fn from(value: species::SpecimenSummary) -> Self {
        Self {
            id: value.id,
            dataset_name: value.dataset_name,
            record_id: value.record_id,
            accession: value.accession,
            institution_code: value.institution_code,
            institution_name: value.institution_name,
            type_status: value.type_status,
            locality: value.locality,
            country: value.country,
            latitude: value.latitude,
            longitude: value.longitude,
            sequences: value.sequences,
            whole_genomes: value.whole_genomes,
            markers: value.markers,
        }
    }
}


#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "models::AttributeCategory")]
pub enum AttributeCategory {
    BushfireRecovery,
    VenomousSpecies,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "models::AttributeValueType")]
pub enum AttributeValueType {
    Boolean,
    Integer,
    Decimal,
    String,
    Timestamp,
}

/// Attributes for a specific species
#[derive(Clone, Debug, SimpleObject)]
pub struct NameAttribute {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name: String,
    pub category: AttributeCategory,
    pub value_type: AttributeValueType,
    pub value_bool: Option<bool>,
    pub value_int: Option<i64>,
    pub value_decimal: Option<String>,
    pub value_str: Option<String>,
    pub value_timestamp: Option<NaiveDateTime>,
}

impl From<models::NameAttribute> for NameAttribute {
    fn from(value: models::NameAttribute) -> Self {
        Self {
            id: value.id,
            dataset_id: value.dataset_id,
            name: value.name,
            category: value.category.into(),
            value_type: value.value_type.into(),
            value_bool: value.value_bool,
            value_int: value.value_int,
            value_decimal: value.value_decimal.map(|d| d.to_string()),
            value_str: value.value_str,
            value_timestamp: value.value_timestamp,
        }
    }
}
