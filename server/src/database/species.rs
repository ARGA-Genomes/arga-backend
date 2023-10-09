use async_trait::async_trait;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel::Queryable;
use serde::{Serialize, Deserialize};
use tracing::instrument;
use uuid::Uuid;

use crate::database::extensions::whole_genome_filters;
use crate::http::graphql::common::Taxonomy;
use crate::index::species::{self, GetSpecies, GetRegions, GetMedia, GetConservationStatus, GetTraceFiles};

use super::models::{Taxon, Name, NameAttribute, RegionType, TaxonPhoto, TraceFile, ConservationStatus, IndigenousKnowledge, WholeGenome, Marker};
use super::extensions::{sum_if, Paginate};
use super::{schema, schema_gnl, Database, Error, PgPool, PageResult};


const NCBI_REFSEQ_DATASET_ID: &str = "ARGA:TL:0002002";


#[derive(Debug, Clone, Default, Queryable, Serialize, Deserialize)]
pub struct AssemblySummary {
    pub name_id: Uuid,
    pub reference_genomes: i64,
    pub whole_genomes: i64,
    pub partial_genomes: i64,
}

#[derive(Debug, Clone, Default, Queryable, Serialize, Deserialize)]
pub struct MarkerSummary {
    pub name_id: Uuid,
    pub barcodes: i64,
}

#[derive(Debug, Queryable)]
pub struct VernacularName {
    pub name: String,
    pub language: Option<String>,
}

#[derive(Debug, Queryable)]
pub struct SpecimenSummary {
    pub id: Uuid,
    pub dataset_name: String,
    pub record_id: String,
    pub accession: Option<String>,
    pub type_status: Option<String>,
    pub locality: Option<String>,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,

    pub sequences: i64,
    pub whole_genomes: i64,
    pub markers: i64,
}


#[derive(Clone)]
pub struct SpeciesProvider {
    pub pool: PgPool,
}

impl SpeciesProvider {
    /// Get taxonomic information for a specific species.
    pub async fn taxonomy(&self, name_id: &Uuid) -> Result<Taxon, Error> {
        use schema::taxa;
        let mut conn = self.pool.get().await?;

        let taxon = taxa::table
            .filter(taxa::name_id.eq(name_id))
            .first::<Taxon>(&mut conn)
            .await?;

        Ok(taxon)
    }

    pub async fn vernacular_names(&self, name_id: &Uuid) -> Result<Vec<VernacularName>, Error> {
        use schema::{vernacular_names, name_vernacular_names};
        let mut conn = self.pool.get().await?;

        let names = name_vernacular_names::table
            .inner_join(vernacular_names::table)
            .select((vernacular_names::vernacular_name, vernacular_names::language))
            .filter(name_vernacular_names::name_id.eq(name_id))
            .load::<VernacularName>(&mut conn)
            .await?;

        Ok(names)
    }

    pub async fn synonyms(&self, name_id: &Uuid) -> Result<Vec<Taxon>, Error> {
        use schema::{taxon_history, taxa};
        let mut conn = self.pool.get().await?;

        let (old_taxa, new_taxa) = diesel::alias!(taxa as old_taxa, taxa as new_taxa);

        let synonyms = taxon_history::table
            .inner_join(old_taxa.on(taxon_history::old_taxon_id.eq(old_taxa.field(taxa::id))))
            .inner_join(new_taxa.on(taxon_history::new_taxon_id.eq(new_taxa.field(taxa::id))))
            .filter(new_taxa.field(taxa::name_id).eq(name_id))
            .select(old_taxa.fields(taxa::all_columns))
            .load::<Taxon>(&mut conn)
            .await?;

        Ok(synonyms)
    }

    pub async fn assembly_summary(&self, name_ids: &Vec<Uuid>) -> Result<Vec<AssemblySummary>, Error> {
        use schema::assemblies::dsl::*;
        let mut conn = self.pool.get().await?;

        // get the total amounts of assembly records for each name
        let summaries = assemblies
            .group_by(name_id)
            .select((
                name_id,
                sum_if(refseq_category.eq("reference genome")),
                sum_if(genome_rep.eq("Full")),
                sum_if(genome_rep.eq("Partial")),
            ))
            .filter(name_id.eq_any(name_ids))
            .load::<AssemblySummary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    pub async fn marker_summary(&self, name_ids: &Vec<Uuid>) -> Result<Vec<MarkerSummary>, Error> {
        use schema_gnl::markers::dsl::*;
        let mut conn = self.pool.get().await?;

        // get the total amounts of assembly records for each name
        let summaries = markers
            .group_by(name_id)
            .select((
                name_id,
                diesel::dsl::count_star(),
            ))
            .filter(name_id.eq_any(name_ids))
            .load::<MarkerSummary>(&mut conn)
            .await?;

        Ok(summaries)
    }

    pub async fn indigenous_knowledge(&self, name_ids: &Vec<Uuid>) -> Result<Vec<(IndigenousKnowledge, String)>, Error> {
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

    pub async fn specimens(&self, name: &Name, page: i64, page_size: i64) -> PageResult<SpecimenSummary> {
        use schema::{specimens, datasets, accession_events};
        use schema_gnl::specimen_stats;
        let mut conn = self.pool.get().await?;

        let records = specimens::table
            .inner_join(datasets::table)
            .inner_join(specimen_stats::table)
            .left_join(accession_events::table)
            .select((
                specimens::id,
                datasets::name,
                specimens::record_id,
                accession_events::accession.nullable(),
                specimens::type_status,
                specimens::locality,
                specimens::country,
                specimens::latitude,
                specimens::longitude,
                specimen_stats::sequences,
                specimen_stats::whole_genomes,
                specimen_stats::markers,
            ))
            .filter(specimens::name_id.eq(name.id))
            .order((
                specimens::type_status.asc(),
                specimen_stats::sequences.desc(),
            ))
            .paginate(page)
            .per_page(page_size)
            .load::<(SpecimenSummary, i64)>(&mut conn)
            .await?;

        Ok(records.into())
    }

    pub async fn whole_genomes(
        &self,
        name: &Name,
        filters: &Vec<whole_genome_filters::Filter>,
        page: i64,
        page_size: i64
    ) -> PageResult<WholeGenome>
    {
        use schema_gnl::whole_genomes;
        let mut conn = self.pool.get().await?;

        let mut query = whole_genomes::table
            .filter(whole_genomes::name_id.eq(name.id))
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

    pub async fn markers(&self, name: &Name, page: i64, page_size: i64) -> PageResult<Marker> {
        use schema_gnl::markers;
        let mut conn = self.pool.get().await?;

        let records = markers::table
            .filter(markers::name_id.eq(name.id))
            .order(markers::accession)
            .paginate(page)
            .per_page(page_size)
            .load::<(Marker, i64)>(&mut conn)
            .await?;

        Ok(records.into())
    }

    pub async fn reference_genome(&self, name: &Name) -> Result<Option<WholeGenome>, Error> {
        use schema::datasets;
        use schema_gnl::whole_genomes;
        let mut conn = self.pool.get().await?;

        let record = whole_genomes::table
            .inner_join(datasets::table)
            .select(whole_genomes::all_columns)
            .filter(whole_genomes::name_id.eq(name.id))
            .filter(datasets::global_id.eq(NCBI_REFSEQ_DATASET_ID))
            .get_result::<WholeGenome>(&mut conn)
            .await.optional()?;

        Ok(record)
    }

    pub async fn attributes(&self, name: &Name) -> Result<Vec<NameAttribute>, Error> {
        use schema::name_attributes;
        let mut conn = self.pool.get().await?;

        let records = name_attributes::table
            .filter(name_attributes::name_id.eq(name.id))
            .load::<NameAttribute>(&mut conn)
            .await?;

        Ok(records)
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
impl GetSpecies for Database {
    type Error = Error;

    #[instrument(skip(self))]
    async fn taxonomy(&self, name: &Name) -> Result<Taxonomy, Error> {
        use schema::taxa;
        let mut conn = self.pool.get().await?;

        let taxon = taxa::table
            .filter(taxa::name_id.eq(name.id))
            .first::<Taxon>(&mut conn)
            .await?;

        Ok(Taxonomy::from(taxon))
    }

    #[instrument(skip(self))]
    async fn taxa(&self, names: &Vec<Name>) -> Result<Vec<Taxonomy>, Error> {
        use schema::taxa;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let records = taxa::table
            .filter(taxa::name_id.eq_any(name_ids))
            .load::<Taxon>(&mut conn)
            .await?;

        let mut taxa = Vec::with_capacity(records.len());
        for taxon in records {
            taxa.push(Taxonomy::from(taxon))
        }

        Ok(taxa)
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
                filtered.push(species::Region {
                    name,
                });
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
                filtered.push(species::Region {
                    name,
                });
            }
        }

        filtered.sort();
        filtered.dedup();
        Ok(filtered)
    }
}


#[async_trait]
impl GetMedia for Database {
    type Error = Error;

    async fn photos(&self, name: &Name) -> Result<Vec<species::Photo>, Error> {
        use schema::taxon_photos::dsl::*;
        let mut conn = self.pool.get().await?;

        let records = taxon_photos
            .filter(name_id.eq(name.id))
            .load::<TaxonPhoto>(&mut conn)
            .await?;

        let mut photos = Vec::with_capacity(records.len());
        for record in records {
            photos.push(species::Photo {
                url: record.url,
                publisher: record.publisher,
                license: record.license,
                rights_holder: record.rights_holder,
                reference_url: record.source,
            });
        }

        Ok(photos)
    }
}


#[async_trait]
impl GetConservationStatus for Database {
    type Error = Error;

    async fn conservation_status(&self, name: &Name) -> Result<Vec<species::ConservationStatus>, Error> {
        use schema::conservation_statuses::dsl::*;
        let mut conn = self.pool.get().await?;

        let records = conservation_statuses
            .filter(name_id.eq(name.id))
            .load::<ConservationStatus>(&mut conn)
            .await?;

        let records = records.into_iter().map(|r| r.into()).collect();
        Ok(records)
    }
}

impl From<ConservationStatus> for species::ConservationStatus {
    fn from(value: ConservationStatus) -> Self {
        Self {
            status: value.status,
            state: value.state,
            source: value.source,
        }
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
