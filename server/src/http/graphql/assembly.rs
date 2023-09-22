use async_graphql::*;
use chrono::NaiveDate;
use chrono::NaiveTime;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;
use crate::database::Database;
use crate::http::Error;
use crate::http::Context as State;

use crate::database::models;
use crate::index::names::GetNames;


#[derive(MergedObject)]
pub struct Assembly(AssemblyDetails, AssemblyQuery);

impl Assembly {
    pub async fn new(db: &Database, record_id: &str) -> Result<Assembly, Error> {
        let assembly = db.assembly.find_by_record_id(record_id).await?;
        let details = assembly.clone().into();
        let query = AssemblyQuery { assembly };
        Ok(Assembly(details, query))
    }
}


struct AssemblyQuery {
    assembly: models::AssemblyEvent,
}

#[Object]
impl AssemblyQuery {
    async fn canonical_name(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let state = ctx.data::<State>().unwrap();
        let name = state.database.find_by_assembly_id(&self.assembly.id).await?;
        Ok(name.canonical_name)
    }

    /// Get the assembly statistics associated with the provided assembly
    ///
    /// These stats are different to the stats used elsewhere throughout the backend,
    /// specifically they are imported data and reflect statistics about the assembly
    /// itself rather than stats about the arga index
    async fn stats(&self, ctx: &Context<'_>) -> Result<AssemblyDetailsStats, Error> {
        let state = ctx.data::<State>().unwrap();
        let stats = state.database.assembly.stats(&self.assembly.id).await.unwrap_or_default();
        Ok(stats.into())
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct AssemblyDetails {
    pub id: Uuid,
    pub name: Option<String>,
    pub event_date: Option<NaiveDate>,
    pub event_time: Option<NaiveTime>,
    pub version_status: Option<String>,
    pub quality: Option<String>,
    pub assembly_type: Option<String>,
    pub assembled_by: Option<String>,
    pub genome_size: Option<i64>,
}

impl From<models::AssemblyEvent> for AssemblyDetails {
    fn from(value: models::AssemblyEvent) -> Self {
        Self {
            id: value.id,
            name: value.name,
            event_date: value.event_date,
            event_time: value.event_time,
            version_status: value.version_status,
            quality: value.quality,
            assembly_type: value.assembly_type,
            assembled_by: value.assembled_by,
            genome_size: value.genome_size,
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct AssemblyDetailsStats {
    pub id: String,
    pub total_length: Option<i32>,
    pub spanned_gaps: Option<i32>,
    pub unspanned_gaps: Option<i32>,
    pub region_count: Option<i32>,
    pub scaffold_count: Option<i32>,
    pub scaffold_n50: Option<i32>,
    pub scaffold_l50: Option<i32>,
    pub scaffold_n75: Option<i32>,
    pub scaffold_n90: Option<i32>,
    pub contig_count: Option<i32>,
    pub contig_n50: Option<i32>,
    pub contig_l50: Option<i32>,
    pub total_gap_length: Option<i32>,
    pub molecule_count: Option<i32>,
    pub top_level_count: Option<i32>,
    pub component_count: Option<i32>,
    pub gc_perc: Option<i32>,
}

impl From<models::AssemblyStats> for AssemblyDetailsStats {
    fn from(value: models::AssemblyStats) -> Self {
        Self {
            id: value.id.to_string(),
            total_length: value.total_length,
            spanned_gaps: value.spanned_gaps,
            unspanned_gaps: value.unspanned_gaps,
            region_count: value.region_count,
            scaffold_count: value.scaffold_count,
            scaffold_n50: value.scaffold_n50,
            scaffold_l50: value.scaffold_l50,
            scaffold_n75: value.scaffold_n75,
            scaffold_n90: value.scaffold_n90,
            contig_count: value.contig_count,
            contig_n50: value.contig_n50,
            contig_l50: value.contig_l50,
            total_gap_length: value.total_gap_length,
            molecule_count: value.molecule_count,
            top_level_count: value.top_level_count,
            component_count: value.component_count,
            gc_perc: value.gc_perc,
        }
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct BioSample {
    pub id: String,
    pub accession: String,

    pub sra: Option<String>,
    pub submission_date: Option<String>,
    pub publication_date: Option<String>,
    pub last_update: Option<String>,
    pub title: Option<String>,
    pub owner: Option<String>,
    pub attributes: Option<serde_json::Value>,
}

impl From<models::BioSample> for BioSample {
    fn from(value: models::BioSample) -> Self {
        Self {
            id: value.id.to_string(),
            accession: value.accession,
            sra: value.sra,
            submission_date: value.submission_date.map(|d| d.to_string()),
            publication_date: value.publication_date.map(|d| d.to_string()),
            last_update: value.last_update.map(|d| d.to_string()),
            title: value.title,
            owner: value.owner,
            attributes: value.attributes,
        }
    }
}
