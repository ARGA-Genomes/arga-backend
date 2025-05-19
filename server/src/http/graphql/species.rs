use arga_core::models;
use arga_core::models::IndigenousKnowledge;
use async_graphql::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use super::common::attributes::AttributeValueType;
use super::common::taxonomy::sort_taxa_priority;
use super::common::{
    convert_whole_genome_filters,
    DatasetDetails,
    Page,
    SpeciesPhoto,
    Taxonomy,
    WholeGenomeFilterItem,
};
use super::markers::SpeciesMarker;
use super::taxon::{into_classification, TaxonNode, TaxonRank};
use crate::database::models::{Name as ArgaName, Name};
use crate::database::sources::ALA_DATASET_ID;
use crate::database::{schema, species, Database};
use crate::http::{Context as State, Error};


pub struct Species {
    pub canonical_name: String,
    pub name: ArgaName,
    pub names: Vec<Name>,
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
        use schema::{datasets, names, taxa, taxon_names};
        let mut conn = db.pool.get().await?;

        // get all names that match the canonical name in the request
        let mut name_ids = names::table
            .filter(names::canonical_name.eq(&canonical_name))
            .select(names::id)
            .load::<Uuid>(&mut conn)
            .await?;

        // get names that are identified as being the same species
        // via a taxonomic system
        let linked_taxa_query = taxon_names::table
            .select(taxon_names::taxon_id)
            .filter(taxon_names::name_id.eq_any(&name_ids))
            .into_boxed();

        let linked_name_ids = taxon_names::table
            .inner_join(taxa::table.on(taxa::id.eq(taxon_names::taxon_id)))
            .inner_join(datasets::table.on(datasets::id.eq(taxa::dataset_id)))
            .filter(taxon_names::taxon_id.eq_any(linked_taxa_query))
            .filter(datasets::global_id.eq(ALA_DATASET_ID))
            .select(taxon_names::name_id)
            .load::<Uuid>(&mut conn)
            .await?;

        name_ids.extend(linked_name_ids);

        let names = names::table
            .filter(names::id.eq_any(name_ids))
            .load::<Name>(&mut conn)
            .await?;

        if names.len() == 0 {
            return Err(Error::NotFound(canonical_name));
        }

        Ok(Species {
            canonical_name,
            name: names[0].clone(),
            names,
        })
    }

    #[instrument(skip(self, ctx))]
    async fn taxonomy(&self, ctx: &Context<'_>) -> Result<Vec<Taxonomy>, Error> {
        let state = ctx.data::<State>()?;
        let taxa = state.database.species.taxonomy(&self.names).await?;

        let mut details = Vec::new();
        for taxon in taxa {
            let dataset = state.database.datasets.find_by_id(&taxon.dataset_id).await?;
            let mut taxon: Taxonomy = taxon.into();
            taxon.source = Some(dataset.name);
            taxon.source_url = dataset.url;
            details.push(taxon);
        }

        Ok(details)
    }

    async fn vernacular_names(&self, ctx: &Context<'_>) -> Result<Vec<VernacularName>, Error> {
        let state = ctx.data::<State>()?;
        let name_ids = self.names.iter().map(|n| n.id.clone()).collect();
        let vernacular_names = state.database.species.vernacular_names(&name_ids).await?;
        let vernacular_names = vernacular_names.into_iter().map(|n| n.into()).collect();
        Ok(vernacular_names)
    }

    async fn synonyms(&self, _ctx: &Context<'_>) -> Result<Vec<Synonym>, Error> {
        let mut synonyms = Vec::new();
        for name in &self.names {
            if name.canonical_name != self.canonical_name {
                synonyms.push(name.clone().into())
            }
        }
        Ok(synonyms)
    }

    #[instrument(skip(self, _ctx))]
    async fn regions(&self, _ctx: &Context<'_>) -> Regions {
        Regions {
            names: self.names.clone(),
        }
    }

    #[instrument(skip(self, ctx))]
    async fn photos(&self, ctx: &Context<'_>) -> Result<Vec<SpeciesPhoto>, Error> {
        let state = ctx.data::<State>()?;
        let photos = state.database.species.photos(&self.names).await?;
        let photos = photos.into_iter().map(|p| p.into()).collect();
        Ok(photos)
    }

    #[instrument(skip(self, ctx))]
    async fn specimens(&self, ctx: &Context<'_>, page: i64, page_size: i64) -> Result<Page<SpecimenSummary>, Error> {
        let state = ctx.data::<State>()?;
        let page = state.database.species.specimens(&self.names, page, page_size).await?;
        let specimens = page.records.into_iter().map(|r| r.into()).collect();
        Ok(Page {
            records: specimens,
            total: page.total,
        })
    }

    #[instrument(skip(self, ctx))]
    async fn whole_genomes(
        &self,
        ctx: &Context<'_>,
        page: i64,
        page_size: i64,
        filters: Option<Vec<WholeGenomeFilterItem>>,
    ) -> Result<Page<WholeGenome>, Error> {
        let state = ctx.data::<State>()?;
        let filters = convert_whole_genome_filters(filters.unwrap_or_default())?;
        let page = state
            .database
            .species
            .whole_genomes(&self.names, &filters, page, page_size)
            .await?;
        let sequences = page.records.into_iter().map(|r| r.into()).collect();
        Ok(Page {
            records: sequences,
            total: page.total,
        })
    }

    async fn markers(&self, ctx: &Context<'_>, page: i64, page_size: i64) -> Result<Page<SpeciesMarker>, Error> {
        let state = ctx.data::<State>()?;
        let page = state.database.species.loci(&self.names, page, page_size).await?;
        let markers = page.records.into_iter().map(|m| m.into()).collect();
        Ok(Page {
            records: markers,
            total: page.total,
        })
    }

    async fn genomic_components(
        &self,
        ctx: &Context<'_>,
        page: i64,
        page_size: i64,
    ) -> Result<Page<GenomicComponent>, Error> {
        let state = ctx.data::<State>()?;
        let page = state
            .database
            .species
            .genomic_components(&self.names, page, page_size)
            .await?;
        let components = page.records.into_iter().map(|m| m.into()).collect();
        Ok(Page {
            records: components,
            total: page.total,
        })
    }

    async fn reference_genome(&self, ctx: &Context<'_>) -> Result<Option<WholeGenome>, Error> {
        let state = ctx.data::<State>()?;
        let genome = state.database.species.reference_genome(&self.names).await?;
        let genome = genome.map(|g| g.into());
        Ok(genome)
    }

    async fn indigenous_ecological_knowledge(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<IndigenousEcologicalTrait>, Error> {
        let state = ctx.data::<State>()?;
        let name_ids: Vec<Uuid> = self.names.iter().map(|name| name.id.clone()).collect();
        let records = state.database.species.indigenous_knowledge(&name_ids).await?;
        let traits = records.into_iter().map(|r| r.into()).collect();
        Ok(traits)
    }

    async fn attributes(&self, ctx: &Context<'_>) -> Result<Vec<NameAttribute>, Error> {
        let state = ctx.data::<State>()?;
        let records = state.database.species.attributes(&self.names).await?;
        let attributes = records.into_iter().map(|r| r.into()).collect();
        Ok(attributes)
    }

    async fn data_summary(&self, ctx: &Context<'_>) -> Result<SpeciesGenomicDataSummary, Error> {
        let state = ctx.data::<State>()?;
        let name_ids: Vec<Uuid> = self.names.iter().map(|name| name.id.clone()).collect();
        let summary = state.database.species.data_summary(&name_ids).await?;
        Ok(summary.into())
    }
}


#[derive(Clone, Debug, SimpleObject)]
pub struct SpeciesGenomicDataSummary {
    /// The total amount of whole genomes available
    pub genomes: i64,
    /// The total amount of loci available
    pub loci: i64,
    /// The total amount of specimens available
    pub specimens: i64,
    /// The total amount of other genomic data
    pub other: i64,
    /// The total amount of genomic data
    pub total_genomic: i64,
}

impl From<species::DataSummary> for SpeciesGenomicDataSummary {
    fn from(value: species::DataSummary) -> Self {
        Self {
            genomes: value.genomes.unwrap_or_default(),
            loci: value.loci.unwrap_or_default(),
            specimens: value.specimens.unwrap_or_default(),
            other: value.other.unwrap_or_default(),
            total_genomic: value.total_genomic.unwrap_or_default(),
        }
    }
}


pub struct Regions {
    names: Vec<Name>,
}

#[Object]
impl Regions {
    async fn ibra(&self, ctx: &Context<'_>) -> Result<Vec<RegionDistribution>, Error> {
        let state = ctx.data::<State>()?;
        let regions = state.database.species.regions_ibra(&self.names).await?;
        let regions = regions.into_iter().map(RegionDistribution::new).collect();
        Ok(regions)
    }

    async fn imcra(&self, ctx: &Context<'_>) -> Result<Vec<RegionDistribution>, Error> {
        let state = ctx.data::<State>()?;
        let regions = state.database.species.regions_imcra(&self.names).await?;
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
        let state = ctx.data::<State>()?;
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
    pub accession: Option<String>,
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


#[derive(Clone, Debug, SimpleObject)]
pub struct GenomicComponent {
    pub sequence_id: Uuid,
    pub dna_extract_id: Uuid,
    pub dataset_name: String,

    pub record_id: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub accession: Option<String>,
    pub sequenced_by: Option<String>,
    pub material_sample_id: Option<String>,
    pub estimated_size: Option<String>,

    pub release_date: Option<String>,
    pub deposited_by: Option<String>,
    pub data_type: Option<String>,

    pub title: Option<String>,
    pub url: Option<String>,
    pub source_uri: Option<String>,
    pub funding_attribution: Option<String>,
    pub rights_holder: Option<String>,
    pub access_rights: Option<String>,
}

impl From<models::GenomicComponent> for GenomicComponent {
    fn from(value: models::GenomicComponent) -> Self {
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
            release_date: value.release_date,
            deposited_by: value.deposited_by,
            data_type: value.data_type,
            title: value.title,
            url: value.url,
            source_uri: value.source_uri,
            funding_attribution: value.funding_attribution,
            rights_holder: value.rights_holder,
            access_rights: value.access_rights,
        }
    }
}


/// A specimen from a specific species.
#[derive(Clone, Debug, SimpleObject)]
pub struct SpecimenSummary {
    pub id: Uuid,
    pub dataset_name: String,
    pub record_id: String,
    pub entity_id: Option<String>,
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
            entity_id: value.entity_id,
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


/// Common vernacular names for a specific species
#[derive(Clone, Debug, SimpleObject)]
pub struct VernacularName {
    pub dataset_id: Uuid,
    pub vernacular_name: String,
    pub citation: Option<String>,
    pub source_url: Option<String>,
}

impl From<models::VernacularName> for VernacularName {
    fn from(value: models::VernacularName) -> Self {
        Self {
            dataset_id: value.dataset_id,
            vernacular_name: value.vernacular_name,
            citation: value.citation,
            source_url: value.source_url,
        }
    }
}


#[derive(Clone, Debug, SimpleObject)]
pub struct Synonym {
    pub scientific_name: String,
    pub canonical_name: String,
    pub authorship: Option<String>,
}

impl From<models::Name> for Synonym {
    fn from(value: models::Name) -> Self {
        Self {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
            authorship: value.authorship,
        }
    }
}
