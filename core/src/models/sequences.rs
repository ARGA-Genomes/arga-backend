use chrono::{NaiveDate, NaiveTime};
use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use super::schema;


#[derive(Clone, Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::libraries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Library {
    pub entity_id: String,
    pub extract_id: String,
    pub species_name_id: i64,
    pub publication_id: Option<String>,
    pub library_id: String,

    pub event_date: Option<NaiveDate>,
    pub event_time: Option<NaiveTime>,
    pub prepared_by: Option<String>,
    pub concentration: Option<f64>,
    pub concentration_unit: Option<String>,
    pub pcr_cycles: Option<i32>,
    pub layout: Option<String>,
    pub selection: Option<String>,
    pub bait_set_name: Option<String>,
    pub bait_set_reference: Option<String>,
    pub construction_protocol: Option<String>,
    pub source: Option<String>,
    pub insert_size: Option<String>,
    pub design_description: Option<String>,
    pub strategy: Option<String>,
    pub index_tag: Option<String>,
    pub index_dual_tag: Option<String>,
    pub index_oligo: Option<String>,
    pub index_dual_oligo: Option<String>,
    pub location: Option<String>,
    pub remarks: Option<String>,
    pub dna_treatment: Option<String>,
    pub number_of_libraries_pooled: Option<i32>,
    pub pcr_replicates: Option<i32>,
}


#[derive(Clone, Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::sequence_runs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SequenceRun {
    pub entity_id: String,
    pub library_id: String,
    pub species_name_id: i64,
    pub publication_id: Option<String>,
    pub sequence_run_id: String,

    pub event_date: Option<NaiveDate>,
    pub event_time: Option<NaiveTime>,
    pub facility: Option<String>,
    pub instrument_or_method: Option<String>,
    pub platform: Option<String>,
    pub kit_chemistry: Option<String>,
    pub flowcell_type: Option<String>,
    pub cell_movie_length: Option<String>,
    pub base_caller_model: Option<String>,
    pub fast5_compression: Option<String>,
    pub analysis_software: Option<String>,
    pub analysis_software_version: Option<String>,
    pub target_gene: Option<String>,
    pub sra_run_accession: Option<String>,
}


#[derive(Clone, Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::assemblies)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Assembly {
    pub entity_id: String,
    pub species_name_id: i64,
    pub publication_id: Option<String>,
    pub assembly_id: String,

    pub event_date: Option<NaiveDate>,
    pub event_time: Option<NaiveTime>,
    pub name: Option<String>,
    pub type_: Option<String>,
    pub level: Option<String>,
    pub method: Option<String>,
    pub method_version: Option<String>,
    pub method_link: Option<String>,
    pub size: Option<i64>,
    pub size_ungapped: Option<i64>,
    pub minimum_gap_length: Option<i64>,
    pub guanine_cytosine_percent: Option<f64>,
    pub completeness: Option<String>,
    pub completeness_method: Option<String>,
    pub coverage: Option<String>,
    pub representation: Option<String>,
    pub source_molecule: Option<String>,
    pub reference_genome_used: Option<String>,
    pub reference_genome_link: Option<String>,
    pub number_of_scaffolds: Option<i32>,
    pub number_of_contigs: Option<i32>,
    pub number_of_replicons: Option<i32>,
    pub hybrid: Option<String>,
    pub hybrid_information: Option<String>,
    pub polishing_or_scaffolding_method: Option<String>,
    pub polishing_or_scaffolding_data: Option<String>,
    pub computational_infrastructure: Option<String>,
    pub system_used: Option<String>,
    pub assembly_n50: Option<String>,
}
