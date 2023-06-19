use async_graphql::SimpleObject;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use uuid::Uuid;


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct AssemblyDetails {
    pub id: String,
    pub accession: String,
    pub nuccore: Option<String>,
    pub refseq_category: Option<String>,
    pub specific_host: Option<String>,
    pub clone_strain: Option<String>,
    pub version_status: Option<String>,
    pub contam_screen_input: Option<String>,
    pub release_type: Option<String>,
    pub genome_rep: Option<String>,
    pub gbrs_paired_asm: Option<String>,
    pub paired_asm_comp: Option<String>,
    pub excluded_from_refseq: Option<String>,
    pub relation_to_type_material: Option<String>,
    pub asm_not_live_date: Option<String>,
    pub other_catalog_numbers: Option<String>,
    pub recorded_by: Option<String>,
    pub genetic_accession_uri: Option<String>,
    pub event_date: Option<String>,
}

#[async_trait]
pub trait GetAssembly {
    type Error;
    async fn get_assembly(&self, accession: &str) -> Result<AssemblyDetails, Self::Error>;
}


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct AssemblyStats {
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

#[async_trait]
pub trait GetAssemblyStats {
    type Error;

    async fn get_assembly_stats(&self, assembly_id: &Uuid) -> Result<AssemblyStats, Self::Error>;
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

#[async_trait]
pub trait GetBioSamples {
    type Error;

    async fn get_biosamples(&self, accession: &str) -> Result<Vec<BioSample>, Self::Error>;
}
