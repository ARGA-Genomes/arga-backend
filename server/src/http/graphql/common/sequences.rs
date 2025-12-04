use async_graphql::*;
use chrono::{NaiveDate, NaiveTime};

use crate::database::models;


#[derive(Clone, Debug, SimpleObject)]
pub struct LibraryDetails {
    pub entity_id: String,
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

impl From<models::Library> for LibraryDetails {
    fn from(value: models::Library) -> Self {
        LibraryDetails {
            entity_id: value.entity_id,
            library_id: value.library_id,
            event_date: value.event_date,
            event_time: value.event_time,
            prepared_by: value.prepared_by,
            concentration: value.concentration,
            concentration_unit: value.concentration_unit,
            pcr_cycles: value.pcr_cycles,
            layout: value.layout,
            selection: value.selection,
            bait_set_name: value.bait_set_name,
            bait_set_reference: value.bait_set_reference,
            construction_protocol: value.construction_protocol,
            source: value.source,
            insert_size: value.insert_size,
            design_description: value.design_description,
            strategy: value.strategy,
            index_tag: value.index_tag,
            index_dual_tag: value.index_dual_tag,
            index_oligo: value.index_oligo,
            index_dual_oligo: value.index_dual_oligo,
            location: value.location,
            remarks: value.remarks,
            dna_treatment: value.dna_treatment,
            number_of_libraries_pooled: value.number_of_libraries_pooled,
            pcr_replicates: value.pcr_replicates,
        }
    }
}


#[derive(Clone, Debug, SimpleObject)]
pub struct AssemblyDetails {
    pub entity_id: String,
    pub assembly_id: String,
    pub event_date: Option<NaiveDate>,
    pub event_time: Option<NaiveTime>,
    pub assembly_name: Option<String>,
    pub r#type: Option<String>,
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

impl From<models::Assembly> for AssemblyDetails {
    fn from(value: models::Assembly) -> Self {
        AssemblyDetails {
            entity_id: value.entity_id,
            assembly_id: value.assembly_id,
            event_date: value.event_date,
            event_time: value.event_time,
            assembly_name: value.name,
            r#type: value.type_,
            level: value.level,
            method: value.method,
            method_version: value.method_version,
            method_link: value.method_link,
            size: value.size,
            size_ungapped: value.size,
            minimum_gap_length: value.minimum_gap_length,
            guanine_cytosine_percent: value.guanine_cytosine_percent,
            completeness: value.completeness,
            completeness_method: value.completeness_method,
            coverage: value.coverage,
            representation: value.representation,
            source_molecule: value.source_molecule,
            reference_genome_used: value.reference_genome_used,
            reference_genome_link: value.reference_genome_link,
            number_of_scaffolds: value.number_of_scaffolds,
            number_of_contigs: value.number_of_contigs,
            number_of_replicons: value.number_of_replicons,
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


#[derive(Clone, Debug, SimpleObject)]
pub struct AnnotationDetails {
    pub entity_id: String,
    pub assembly_id: String,
    pub name: Option<String>,
    pub provider: Option<String>,
    pub event_date: Option<NaiveDate>,
    pub number_of_genes: Option<i32>,
    pub number_of_proteins: Option<i32>,
}

impl From<models::Annotation> for AnnotationDetails {
    fn from(value: models::Annotation) -> Self {
        AnnotationDetails {
            entity_id: value.entity_id,
            assembly_id: value.assembly_id,
            name: value.name,
            provider: value.provider,
            event_date: value.event_date,
            number_of_genes: value.number_of_genes,
            number_of_proteins: value.number_of_proteins,
        }
    }
}


#[derive(Clone, Debug, SimpleObject)]
pub struct DepositionDetails {
    pub entity_id: String,
    pub assembly_id: String,
    pub event_date: Option<NaiveDate>,
    pub url: Option<String>,
    pub institution: Option<String>,
}

impl From<models::Deposition> for DepositionDetails {
    fn from(value: models::Deposition) -> Self {
        DepositionDetails {
            entity_id: value.entity_id,
            assembly_id: value.assembly_id,
            event_date: value.event_date,
            url: value.url,
            institution: value.institution,
        }
    }
}
