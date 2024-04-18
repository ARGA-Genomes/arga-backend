use arga_core::models::Species;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::Queryable;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::extensions::Paginate;
use super::models::{
    GenomicComponent,
    IndigenousKnowledge,
    Marker,
    Name,
    NameAttribute,
    RegionType,
    Regions,
    Taxon,
    TaxonName,
    TaxonPhoto,
    TraceFile,
    VernacularName,
    WholeGenome,
};
use super::{schema, schema_gnl, Database, Error, PageResult, PgPool};
use crate::database::extensions::whole_genome_filters;
use crate::index::species::{self, GetConservationStatus, GetRegions, GetTraceFiles};


const NCBI_REFSEQ_DATASET_ID: &str = "ARGA:TL:0002002";


#[derive(Debug, Clone, Default, Queryable, Serialize, Deserialize)]
pub struct Summary {
    pub id: Uuid,
    pub genomes: i64,
    pub loci: i64,
    pub specimens: i64,
    pub other: i64,
    pub total_genomic: i64,
}

#[derive(Debug, Clone, Default, Queryable, Serialize, Deserialize)]
pub struct MarkerSummary {
    pub name_id: Uuid,
    pub barcodes: i64,
}


#[derive(Debug, Queryable)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSummary {
    pub genomes: Option<i64>,
    pub loci: Option<i64>,
    pub specimens: Option<i64>,
    pub other: Option<i64>,
    pub total_genomic: Option<i64>,
}


#[derive(Clone)]
pub struct SpeciesProvider {
    pub pool: PgPool,
}

impl SpeciesProvider {
    /// Get taxonomic information for a specific species.
    pub async fn taxonomy(&self, names: &Vec<Name>) -> Result<Vec<Species>, Error> {
        use schema_gnl::species;
        let mut conn = self.pool.get().await?;

        let taxa = TaxonName::belonging_to(names)
            .inner_join(species::table)
            .select(species::all_columns)
            .load(&mut conn)
            .await?;

        Ok(taxa)
    }

    pub async fn vernacular_names(&self, name_ids: &Vec<Uuid>) -> Result<Vec<VernacularName>, Error> {
        use schema::vernacular_names::dsl::*;
        let mut conn = self.pool.get().await?;

        let names = vernacular_names
            .filter(name_id.eq_any(name_ids))
            .load::<VernacularName>(&mut conn)
            .await?;

        Ok(names)
    }

    pub async fn synonyms(&self, name_id: &Uuid) -> Result<Vec<Taxon>, Error> {
        use schema::{taxa, taxon_history};
        let mut conn = self.pool.get().await?;

        let (old_taxa, new_taxa) = diesel::alias!(taxa as old_taxa, taxa as new_taxa);

        // FIXME: determine synonyms based on a taxonomic system and taxon_names
        let synonyms = vec![];
        // let synonyms = taxon_history::table
        //     .inner_join(old_taxa.on(taxon_history::old_taxon_id.eq(old_taxa.field(taxa::id))))
        //     .inner_join(new_taxa.on(taxon_history::new_taxon_id.eq(new_taxa.field(taxa::id))))
        //     .filter(new_taxa.field(taxa::name_id).eq(name_id))
        //     .select(old_taxa.fields(taxa::all_columns))
        //     .load::<Taxon>(&mut conn)
        //     .await?;

        Ok(synonyms)
    }

    pub async fn summary(&self, ids: &Vec<Uuid>) -> Result<Vec<Summary>, Error> {
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;

        // get the data summaries for each species record
        let summaries = species
            .select((id, genomes, loci, specimens, other, total_genomic))
            .filter(id.eq_any(ids))
            .load::<Summary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    pub async fn marker_summary(&self, ids: &Vec<Uuid>) -> Result<Vec<MarkerSummary>, Error> {
        use schema_gnl::markers::dsl::*;
        let mut conn = self.pool.get().await?;

        // get the total amounts of assembly records for each name
        let summaries = markers
            .group_by(name_id)
            .select((name_id, diesel::dsl::count_star()))
            .filter(name_id.eq_any(ids))
            .load::<MarkerSummary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    pub async fn indigenous_knowledge(
        &self,
        name_ids: &Vec<Uuid>,
    ) -> Result<Vec<(IndigenousKnowledge, String)>, Error> {
        use schema::datasets;
        use schema::indigenous_knowledge::dsl::*;

        let mut conn = self.pool.get().await?;

        let records = indigenous_knowledge
            .inner_join(datasets::table)
            .select((indigenous_knowledge::all_columns(), datasets::name))
            .filter(name_id.eq_any(name_ids))
            .load::<(IndigenousKnowledge, String)>(&mut conn)
            .await?;

        Ok(records)
    }

    pub async fn specimens(&self, names: &Vec<Name>, page: i64, page_size: i64) -> PageResult<SpecimenSummary> {
        use schema::{accession_events, datasets, specimens};
        use schema_gnl::specimen_stats;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let records = specimens::table
            .inner_join(datasets::table)
            .inner_join(specimen_stats::table)
            .left_join(accession_events::table)
            .select((
                specimens::id,
                datasets::name,
                specimens::record_id,
                specimens::entity_id,
                accession_events::accession.nullable(),
                specimens::institution_code,
                specimens::institution_name,
                specimens::type_status,
                specimens::locality,
                specimens::country,
                specimens::latitude,
                specimens::longitude,
                specimen_stats::sequences,
                specimen_stats::whole_genomes,
                specimen_stats::markers,
            ))
            .filter(specimens::name_id.eq_any(name_ids))
            .order((specimens::type_status.asc(), specimen_stats::sequences.desc()))
            .paginate(page)
            .per_page(page_size)
            .load::<(SpecimenSummary, i64)>(&mut conn)
            .await?;

        Ok(records.into())
    }

    pub async fn whole_genomes(
        &self,
        names: &Vec<Name>,
        filters: &Vec<whole_genome_filters::Filter>,
        page: i64,
        page_size: i64,
    ) -> PageResult<WholeGenome> {
        use schema_gnl::whole_genomes;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let mut query = whole_genomes::table
            .filter(whole_genomes::name_id.eq_any(name_ids))
            .into_boxed();

        if let Some(expr) = whole_genome_filters::with_filters(&filters) {
            query = query.filter(expr);
        }

        let records = query
            .order(whole_genomes::accession)
            .paginate(page)
            .per_page(page_size)
            .load::<(WholeGenome, i64)>(&mut conn)
            .await?;

        Ok(records.into())
    }

    pub async fn loci(&self, names: &Vec<Name>, page: i64, page_size: i64) -> PageResult<Marker> {
        use schema_gnl::markers;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let records = markers::table
            .filter(markers::name_id.eq_any(name_ids))
            .order(markers::accession)
            .paginate(page)
            .per_page(page_size)
            .load::<(Marker, i64)>(&mut conn)
            .await?;

        Ok(records.into())
    }

    pub async fn genomic_components(
        &self,
        names: &Vec<Name>,
        page: i64,
        page_size: i64,
    ) -> PageResult<GenomicComponent> {
        use schema_gnl::genomic_components;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let records = genomic_components::table
            .filter(genomic_components::name_id.eq_any(name_ids))
            .order(genomic_components::accession)
            .paginate(page)
            .per_page(page_size)
            .load::<(GenomicComponent, i64)>(&mut conn)
            .await?;

        Ok(records.into())
    }

    pub async fn reference_genome(&self, names: &Vec<Name>) -> Result<Option<WholeGenome>, Error> {
        use schema::datasets;
        use schema_gnl::whole_genomes;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let record = whole_genomes::table
            .inner_join(datasets::table)
            .select(whole_genomes::all_columns)
            .filter(whole_genomes::name_id.eq_any(name_ids))
            .filter(datasets::global_id.eq(NCBI_REFSEQ_DATASET_ID))
            .get_result::<WholeGenome>(&mut conn)
            .await
            .optional()?;

        Ok(record)
    }

    pub async fn attributes(&self, names: &Vec<Name>) -> Result<Vec<NameAttribute>, Error> {
        use schema::name_attributes;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let records = name_attributes::table
            .filter(name_attributes::name_id.eq_any(name_ids))
            .load::<NameAttribute>(&mut conn)
            .await?;

        Ok(records)
    }

    pub async fn regions_ibra(&self, names: &Vec<Name>) -> Result<Vec<Regions>, Error> {
        use schema::regions;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let regions = regions::table
            .filter(regions::name_id.eq_any(name_ids))
            .filter(regions::region_type.eq(RegionType::Ibra))
            .load::<Regions>(&mut conn)
            .await?;

        Ok(regions)
    }

    pub async fn regions_imcra(&self, names: &Vec<Name>) -> Result<Vec<Regions>, Error> {
        use schema::regions;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let regions = regions::table
            .filter(regions::name_id.eq_any(name_ids))
            .filter(regions::region_type.eq(RegionType::Imcra))
            .load::<Regions>(&mut conn)
            .await?;

        Ok(regions)
    }

    pub async fn photos(&self, names: &Vec<Name>) -> Result<Vec<TaxonPhoto>, Error> {
        use schema::{taxon_names, taxon_photos};
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let photos = taxon_photos::table
            .inner_join(taxon_names::table.on(taxon_names::taxon_id.eq(taxon_photos::taxon_id)))
            .select(taxon_photos::all_columns)
            .filter(taxon_names::name_id.eq_any(name_ids))
            .load::<TaxonPhoto>(&mut conn)
            .await?;

        Ok(photos)
    }

    pub async fn data_summary(&self, name_ids: &Vec<Uuid>) -> Result<DataSummary, Error> {
        use diesel::dsl::sum;
        use schema_gnl::name_data_summaries;

        let mut conn = self.pool.get().await?;

        let (genomes, loci, specimens, other, total_genomic) = name_data_summaries::table
            .select((
                sum(name_data_summaries::genomes),
                sum(name_data_summaries::markers),
                sum(name_data_summaries::specimens),
                sum(name_data_summaries::other),
                sum(name_data_summaries::total_genomic),
            ))
            .filter(name_data_summaries::name_id.eq_any(name_ids))
            .get_result::<(Option<i64>, Option<i64>, Option<i64>, Option<i64>, Option<i64>)>(&mut conn)
            .await?;

        Ok(DataSummary {
            genomes,
            loci,
            specimens,
            other,
            total_genomic,
        })
    }
}


#[derive(Queryable, Debug)]
struct Distribution {
    pub locality: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub threat_status: Option<String>,
    pub source: Option<String>,
}

impl From<Distribution> for species::Distribution {
    fn from(source: Distribution) -> Self {
        Self {
            locality: source.locality,
            country: source.country,
            country_code: source.country_code,
            threat_status: source.threat_status,
            source: source.source,
        }
    }
}


#[async_trait]
impl GetRegions for Database {
    type Error = Error;

    async fn ibra(&self, name: &Name) -> Result<Vec<species::Region>, Error> {
        use schema::regions;
        let mut conn = self.pool.get().await?;

        let regions = regions::table
            .select(regions::values)
            .filter(regions::name_id.eq(name.id))
            .filter(regions::region_type.eq(RegionType::Ibra))
            .load::<Vec<Option<String>>>(&mut conn)
            .await?;

        let mut filtered = Vec::new();
        for region in regions.concat() {
            if let Some(name) = region {
                filtered.push(species::Region { name });
            }
        }

        filtered.sort();
        filtered.dedup();
        Ok(filtered)
    }

    async fn imcra(&self, name: &Name) -> Result<Vec<species::Region>, Error> {
        use schema::regions;
        let mut conn = self.pool.get().await?;

        let regions = regions::table
            .select(regions::values)
            .filter(regions::name_id.eq(name.id))
            .filter(regions::region_type.eq(RegionType::Imcra))
            .load::<Vec<Option<String>>>(&mut conn)
            .await?;

        let mut filtered = Vec::new();
        for region in regions.concat() {
            if let Some(name) = region {
                filtered.push(species::Region { name });
            }
        }

        filtered.sort();
        filtered.dedup();
        Ok(filtered)
    }
}


#[async_trait]
impl GetConservationStatus for Database {
    type Error = Error;

    async fn conservation_status(&self, name: &Name) -> Result<Vec<species::ConservationStatus>, Error> {
        Ok(vec![])
    }
}


#[async_trait]
impl GetTraceFiles for Database {
    type Error = Error;

    async fn trace_files(&self, names: &Vec<Name>) -> Result<Vec<species::TraceFile>, Error> {
        use schema::trace_files::dsl::*;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let records = trace_files
            .filter(name_id.eq_any(name_ids))
            .load::<TraceFile>(&mut conn)
            .await?;

        let records = records.into_iter().map(|r| r.into()).collect();
        Ok(records)
    }
}

impl From<TraceFile> for species::TraceFile {
    fn from(value: TraceFile) -> Self {
        Self {
            id: value.id.to_string(),
            metadata: value.metadata,

            peak_locations_user: value.peak_locations_user.map(from_int_array),
            peak_locations_basecaller: value.peak_locations_basecaller.map(from_int_array),
            quality_values_user: value.quality_values_user.map(from_int_array),
            quality_values_basecaller: value.quality_values_basecaller.map(from_int_array),
            sequences_user: value.sequences_user.map(from_int_array),
            sequences_basecaller: value.sequences_basecaller.map(from_int_array),

            measurements_voltage: value.measurements_voltage.map(from_int_array),
            measurements_current: value.measurements_current.map(from_int_array),
            measurements_power: value.measurements_power.map(from_int_array),
            measurements_temperature: value.measurements_temperature.map(from_int_array),

            analyzed_g: value.analyzed_g.map(from_int_array),
            analyzed_a: value.analyzed_a.map(from_int_array),
            analyzed_t: value.analyzed_t.map(from_int_array),
            analyzed_c: value.analyzed_c.map(from_int_array),

            raw_g: value.raw_g.map(from_int_array),
            raw_a: value.raw_a.map(from_int_array),
            raw_t: value.raw_t.map(from_int_array),
            raw_c: value.raw_c.map(from_int_array),
        }
    }
}

fn from_int_array(values: Vec<Option<i32>>) -> Vec<i32> {
    values.into_iter().map(|v| v.unwrap_or_default()).collect()
}
