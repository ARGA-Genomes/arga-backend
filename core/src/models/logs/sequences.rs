use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveTime};
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
pub enum LibraryAtom {
    #[default]
    Empty,
    ExtractId(String),
    LibraryId(String),
    PublicationId(String),
    ScientificName(String),

    EventDate(NaiveDate),
    EventTime(NaiveTime),
    PreparedBy(String),
    Concentration(f64),
    ConcentrationUnit(String),
    PcrCycles(i32),
    Layout(String),
    Selection(String),
    BaitSetName(String),
    BaitSetReference(String),
    ConstructionProtocol(String),
    Source(String),
    InsertSize(String),
    DesignDescription(String),
    Strategy(String),
    IndexTag(String),
    IndexDualTag(String),
    IndexOligo(String),
    IndexDualOligo(String),
    Location(String),
    Remarks(String),
    DnaTreatment(String),
    NumberOfLibrariesPooled(i32),
    PcrReplicates(i32),
}

#[derive(OperationLog, Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::library_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LibraryOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: LibraryAtom,
}


#[derive(Atom, Debug, Clone, Default, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Display)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum SequenceRunAtom {
    #[default]
    Empty,
    LibraryId(String),
    SequenceRunId(String),
    PublicationId(String),
    ScientificName(String),

    EventDate(NaiveDate),
    EventTime(NaiveTime),
    Facility(String),
    InstrumentOrMethod(String),
    Platform(String),
    KitChemistry(String),
    FlowcellType(String),
    CellMovieLength(String),
    BaseCallerModel(String),
    Fast5Compression(String),
    AnalysisSoftware(String),
    AnalysisSoftwareVersion(String),
    TargetGene(String),
    SraRunAccession(String),
}

#[derive(OperationLog, Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::sequence_run_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SequenceRunOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: SequenceRunAtom,
}


#[derive(Atom, Debug, Clone, Default, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Display)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum AssemblyAtom {
    #[default]
    Empty,
    // libraries and assemblies actually have a many to many relationship
    // but for now we treat it as a one to many
    LibraryId(String),
    AssemblyId(String),
    PublicationId(String),
    ScientificName(String),

    EventDate(NaiveDate),
    EventTime(NaiveTime),
    Name(String),
    Type(String),
    Method(String),
    MethodVersion(String),
    MethodLink(String),
    Size(String),
    MinimumGapLength(String),
    Completeness(String),
    CompletenessMethod(String),
    SourceMolecule(String),
    ReferenceGenomeUsed(String),
    ReferenceGenomeLink(String),
    NumberOfScaffolds(String),
    GenomeCoverage(String),
    Hybrid(String),
    HybridInformation(String),
    PolishingOrScaffoldingMethod(String),
    PolishingOrScaffoldingData(String),
    ComputationalInfrastructure(String),
    SystemUsed(String),
    AssemblyN50(String),
}

#[derive(OperationLog, Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::assembly_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssemblyOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: AssemblyAtom,
}
