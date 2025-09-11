use bigdecimal::BigDecimal;
use core_derive::{Atom, OperationLog};
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Jsonb;
use diesel::{AsExpression, Associations, FromSqlRow, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use strum::Display;
use uuid::Uuid;

use super::{Action, LogOperation};
use crate::crdt::DataFrameOperation;
use crate::models::{DatasetVersion, schema};


#[derive(Atom, Debug, Clone, Default, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Display)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum ExtractionAtom {
    #[default]
    Empty,

    /// Used to link the subsample to a name
    ScientificName(String),
    /// The global identifier for the subsample that the extraction is from
    SubsampleId(String),
    /// The global publication id of the collected extraction.
    PublicationId(String),
    /// The global extraction id of the collected DNA.
    ExtractId(String),

    EventDate(chrono::NaiveDate),
    EventTime(chrono::NaiveTime),
    ExtractedBy(String),
    MaterialExtractedBy(String),
    NucleicAcidType(String),
    PreparationType(String),
    PreservationType(String),
    PreservationMethod(String),
    ExtractionMethod(String),
    ConcentrationMethod(String),
    Conformation(String),
    Concentration(f64),
    ConcentrationUnit(String),
    Quantification(String),
    Absorbance260230Ratio(f64),
    Absorbance260280Ratio(f64),
    CellLysisMethod(String),
    ActionExtracted(String),
    NumberOfExtractsPolled(String),
}

#[derive(OperationLog, Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::extraction_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ExtractionOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: ExtractionAtom,
}
