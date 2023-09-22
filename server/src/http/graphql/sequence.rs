use async_graphql::*;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use uuid::Uuid;

use crate::database::Database;
use crate::http::Error;
use crate::http::Context as State;

use crate::database::models;


#[derive(OneofObject)]
pub enum SequenceBy {
    Id(Uuid),
    RecordId(String),
    SpecimenRecordId(String),
}

#[derive(MergedObject)]
pub struct Sequence(SequenceDetails, SequenceQuery);

impl Sequence {
    pub async fn new(db: &Database, by: &SequenceBy) -> Result<Option<Sequence>, Error> {
        let sequence = match by {
            SequenceBy::Id(id) => db.sequences.find_by_id(&id).await?,
            SequenceBy::RecordId(id) => db.sequences.find_by_record_id(&id).await?,
            SequenceBy::SpecimenRecordId(id) => db.sequences.find_by_specimen_record_id(&id).await?,
        };

        match sequence {
            None => Ok(None),
            Some(sequence) => {
                let details = sequence.clone().into();
                let query = SequenceQuery { sequence };
                Ok(Some(Sequence(details, query)))
            }
        }
    }
}


struct SequenceQuery {
    sequence: models::Sequence,
}

#[Object]
impl SequenceQuery {
    async fn dataset_name(&self, ctx: &Context<'_>) -> Result<String, Error> {
        let state = ctx.data::<State>().unwrap();
        let dataset = state.database.datasets.find_by_id(&self.sequence.dataset_id).await?;
        Ok(dataset.name)
    }

    async fn events(&self, ctx: &Context<'_>) -> Result<SequenceEvents, Error> {
        let state = ctx.data::<State>().unwrap();
        let sequencing = state.database.sequences.sequencing_events(&self.sequence.id).await?;
        let sequencing_runs = state.database.sequences.sequencing_run_events(&self.sequence.id).await?;
        let assemblies = state.database.sequences.assembly_events(&self.sequence.id).await?;
        let annotations = state.database.sequences.annotation_events(&self.sequence.id).await?;
        let depositions = state.database.sequences.data_deposition_events(&self.sequence.id).await?;

        Ok(SequenceEvents {
            sequencing: sequencing.into_iter().map(|r| r.into()).collect(),
            sequencing_runs: sequencing_runs.into_iter().map(|r| r.into()).collect(),
            assemblies: assemblies.into_iter().map(|r| r.into()).collect(),
            annotations: annotations.into_iter().map(|r| r.into()).collect(),
            data_depositions: depositions.into_iter().map(|r| r.into()).collect(),
        })
    }
}


/// A specimen from a specific species.
#[derive(Clone, Debug, SimpleObject)]
pub struct SequenceDetails {
    pub id: Uuid,
    pub dna_extract_id: Uuid,
    pub record_id: String,
}

impl From<models::Sequence> for SequenceDetails {
    fn from(value: models::Sequence) -> Self {
        Self {
            id: value.id,
            dna_extract_id: value.dna_extract_id,
            record_id: value.record_id,
        }
    }
}


#[derive(SimpleObject)]
pub struct SequenceEvents {
    sequencing: Vec<SequencingEvent>,
    sequencing_runs: Vec<SequencingRunEvent>,
    assemblies: Vec<AssemblyEvent>,
    annotations: Vec<AnnotationEvent>,
    data_depositions: Vec<DataDepositionEvent>,
}


#[derive(Clone, Debug, SimpleObject)]
pub struct SequencingEvent {
    pub id: Uuid,

    pub event_date: Option<NaiveDate>,
    pub event_time: Option<NaiveTime>,
    pub sequenced_by: Option<String>,
    pub material_sample_id: Option<String>,

    pub concentration: Option<f64>,
    pub amplicon_size: Option<i64>,
    pub estimated_size: Option<i64>,
    pub bait_set_name: Option<String>,
    pub bait_set_reference: Option<String>,

    pub target_gene: Option<String>,
    pub dna_sequence: Option<String>,
}

impl From<models::SequencingEvent> for SequencingEvent {
    fn from(value: models::SequencingEvent) -> Self {
        Self {
            id: value.id,
            event_date: value.event_date,
            event_time: value.event_time,
            sequenced_by: value.sequenced_by,
            material_sample_id: value.material_sample_id,
            concentration: value.concentration,
            amplicon_size: value.amplicon_size,
            estimated_size: value.estimated_size,
            bait_set_name: value.bait_set_name,
            bait_set_reference: value.bait_set_reference,
            target_gene: value.target_gene,
            dna_sequence: value.dna_sequence,
        }
    }
}

#[derive(Clone, Debug, SimpleObject)]
pub struct SequencingRunEvent {
    pub id: Uuid,
    pub sequencing_event_id: Uuid,

    pub trace_id: Option<String>,
    pub trace_name: Option<String>,
    pub trace_link: Option<String>,
    pub sequencing_date: Option<NaiveDateTime>,
    pub sequencing_center: Option<String>,
    pub sequencing_center_code: Option<String>,
    pub sequencing_method: Option<String>,
    pub target_gene: Option<String>,
    pub direction: Option<String>,
    pub pcr_primer_name_forward: Option<String>,
    pub pcr_primer_name_reverse: Option<String>,
    pub sequence_primer_forward_name: Option<String>,
    pub sequence_primer_reverse_name: Option<String>,

    pub library_protocol: Option<String>,
    pub analysis_description: Option<String>,
    pub analysis_software: Option<String>,

}

impl From<models::SequencingRunEvent> for SequencingRunEvent {
    fn from(value: models::SequencingRunEvent) -> Self {
        Self {
            id: value.id,
            sequencing_event_id: value.sequencing_event_id,
            trace_id: value.trace_id,
            trace_name: value.trace_name,
            trace_link: value.trace_link,
            sequencing_date: value.sequencing_date,
            sequencing_center: value.sequencing_center,
            sequencing_center_code: value.sequencing_center_code,
            sequencing_method: value.sequencing_method,
            target_gene: value.target_gene,
            direction: value.direction,
            pcr_primer_name_forward: value.pcr_primer_name_forward,
            pcr_primer_name_reverse: value.pcr_primer_name_reverse,
            sequence_primer_forward_name: value.sequence_primer_forward_name,
            sequence_primer_reverse_name: value.sequence_primer_reverse_name,
            library_protocol: value.library_protocol,
            analysis_description: value.analysis_description,
            analysis_software: value.analysis_software,
        }
    }
}

#[derive(Clone, Debug, SimpleObject)]
pub struct AssemblyEvent {
    pub id: Uuid,
    pub name: Option<String>,
    pub event_date: Option<NaiveDate>,
    pub event_time: Option<NaiveTime>,
    pub assembled_by: Option<String>,
    pub version_status: Option<String>,
    pub quality: Option<String>,
    pub assembly_type: Option<String>,
    pub genome_size: Option<i64>,
}

impl From<models::AssemblyEvent> for AssemblyEvent {
    fn from(value: models::AssemblyEvent) -> Self {
        Self {
            id: value.id,
            event_date: value.event_date,
            event_time: value.event_time,
            assembled_by: value.assembled_by,
            name: value.name,
            version_status: value.version_status,
            quality: value.quality,
            assembly_type: value.assembly_type,
            genome_size: value.genome_size,
        }
    }
}

#[derive(Clone, Debug, SimpleObject)]
pub struct AnnotationEvent {
    pub id: Uuid,
    pub event_date: Option<NaiveDate>,
    pub event_time: Option<NaiveTime>,
    pub annotated_by: Option<String>,
    pub representation: Option<String>,
    pub release_type: Option<String>,
    pub coverage: Option<String>,
    pub replicons: Option<i64>,
    pub standard_operating_procedures: Option<String>,
}

impl From<models::AnnotationEvent> for AnnotationEvent {
    fn from(value: models::AnnotationEvent) -> Self {
        Self {
            id: value.id,
            event_date: value.event_date,
            event_time: value.event_time,
            annotated_by: value.annotated_by,
            representation: value.representation,
            release_type: value.release_type,
            coverage: value.coverage,
            replicons: value.replicons,
            standard_operating_procedures: value.standard_operating_procedures,
        }
    }
}

#[derive(Clone, Debug, SimpleObject)]
pub struct DataDepositionEvent {
    pub id: Uuid,

    pub event_date: Option<NaiveDate>,
    pub event_time: Option<NaiveTime>,
    pub submitted_by: Option<String>,

    pub material_sample_id: Option<String>,
    pub collection_name: Option<String>,
    pub collection_code: Option<String>,
    pub institution_name: Option<String>,

    pub data_type: Option<String>,
    pub excluded_from_refseq: Option<String>,
    pub asm_not_live_date: Option<String>,
    pub source_uri: Option<String>,

    pub title: Option<String>,
    pub url: Option<String>,
    pub funding_attribution: Option<String>,
    pub rights_holder: Option<String>,
    pub access_rights: Option<String>,
    pub reference: Option<String>,
    pub last_updated: Option<NaiveDate>,

}

impl From<models::DepositionEvent> for DataDepositionEvent {
    fn from(value: models::DepositionEvent) -> Self {
        Self {
            id: value.id,
            event_date: value.event_date,
            event_time: value.event_time,
            submitted_by: value.submitted_by,
            material_sample_id: value.material_sample_id,
            collection_name: value.collection_name,
            collection_code: value.collection_code,
            institution_name: value.institution_name,
            data_type: value.data_type,
            excluded_from_refseq: value.excluded_from_refseq,
            asm_not_live_date: value.asm_not_live_date,
            source_uri: value.source_uri,
            title: value.title,
            url: value.url,
            funding_attribution: value.funding_attribution,
            rights_holder: value.rights_holder,
            access_rights: value.access_rights,
            reference: value.reference,
            last_updated: value.last_updated,
        }
    }
}
