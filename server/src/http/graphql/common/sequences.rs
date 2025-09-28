use async_graphql::*;
use chrono::{NaiveDate, NaiveTime};

use crate::database::models;


#[derive(Clone, Debug, SimpleObject)]
pub struct AssemblyDetails {
    pub entity_id: String,
    pub assembly_id: String,
    pub event_date: Option<NaiveDate>,
    pub event_time: Option<NaiveTime>,
    pub name: Option<String>,
    pub type_: Option<String>,
    pub method: Option<String>,
    pub method_version: Option<String>,
    pub method_link: Option<String>,
    pub size: Option<String>,
    pub minimum_gap_length: Option<String>,
    pub completeness: Option<String>,
    pub completeness_method: Option<String>,
    pub source_molecule: Option<String>,
    pub reference_genome_used: Option<String>,
    pub reference_genome_link: Option<String>,
    pub number_of_scaffolds: Option<String>,
    pub genome_coverage: Option<String>,
    pub hybrid: Option<String>,
    pub hybrid_information: Option<String>,
    pub polishing_or_scaffolding_method: Option<String>,
    pub polishing_or_scaffolding_data: Option<String>,
    pub computational_infrastructure: Option<String>,
    pub system_used: Option<String>,
    pub assembly_n50: Option<String>,
}

impl From<models::Assembly> for AssemblyDetails {
    fn from(value: models::Assembly) -> Self {
        AssemblyDetails {
            entity_id: value.entity_id,
            assembly_id: value.assembly_id,
            event_date: value.event_date,
            event_time: value.event_time,
            name: value.name,
            type_: value.type_,
            method: value.method,
            method_version: value.method_version,
            method_link: value.method_link,
            size: value.size,
            minimum_gap_length: value.minimum_gap_length,
            completeness: value.completeness,
            completeness_method: value.completeness_method,
            source_molecule: value.source_molecule,
            reference_genome_used: value.reference_genome_used,
            reference_genome_link: value.reference_genome_link,
            number_of_scaffolds: value.number_of_scaffolds,
            genome_coverage: value.genome_coverage,
            hybrid: value.hybrid,
            hybrid_information: value.hybrid_information,
            polishing_or_scaffolding_method: value.polishing_or_scaffolding_method,
            polishing_or_scaffolding_data: value.polishing_or_scaffolding_data,
            computational_infrastructure: value.computational_infrastructure,
            system_used: value.system_used,
            assembly_n50: value.assembly_n50,
        }
    }
}
