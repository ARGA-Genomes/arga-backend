use async_trait::async_trait;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel::Queryable;
use serde::{Serialize, Deserialize};
use tracing::instrument;
use uuid::Uuid;

use crate::database::sum_if;
use crate::http::graphql::common::Taxonomy;
use crate::index::specimen;
use crate::index::species::{self, GetSpecies, GetRegions, GetMedia, GetSpecimens, GetConservationStatus, GetTraceFiles};
use super::{schema, Database, Error, PgPool};
use super::models::{Taxon, Name, RegionType, TaxonPhoto, Specimen, TraceFile, ConservationStatus};


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


#[derive(Clone)]
pub struct SpeciesProvider {
    pub pool: PgPool,
}

impl SpeciesProvider {
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
        use schema::markers::dsl::*;
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
impl GetSpecimens for Database {
    type Error = Error;

    async fn specimens(&self, name: &Name) -> Result<Vec<specimen::SpecimenDetails>, Error> {
        use schema::specimens::dsl::*;
        let mut conn = self.pool.get().await?;

        let records = specimens
            .filter(name_id.eq(name.id))
            .limit(20)
            .order((type_status, institution_name, institution_code))
            .load::<Specimen>(&mut conn)
            .await?;

        let results = records.into_iter().map(|r| r.into()).collect();
        Ok(results)
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
