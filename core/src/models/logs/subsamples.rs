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
pub enum SubsampleAtom {
    #[default]
    Empty,

    /// Used to link the subsample to a name
    ScientificName(String),
    /// The global identifier for the specimen that the subsample is from
    SpecimenId(String),
    /// The global publication id of the collected subsample.
    PublicationId(String),
    /// The global subsample id of the collected subsample.
    SubsampleId(String),

    EventDate(chrono::NaiveDate),
    EventTime(chrono::NaiveTime),
    InstitutionName(String),
    InstitutionCode(String),
    SampleType(String),
    Name(String),
    Custodian(String),
    Description(String),
    Notes(String),
    CultureMethod(String),
    CultureMedia(String),
    WeightOrVolume(String),
    PreservationMethod(String),
    PreservationTemperature(String),
    PreservationDuration(String),
    Quality(String),
    CellType(String),
    CellLine(String),
    LabHost(String),
    CloneName(String),
    SampleProcessing(String),
    SamplePooling(String),
}

#[derive(OperationLog, Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::subsample_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SubsampleOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: SubsampleAtom,
}
