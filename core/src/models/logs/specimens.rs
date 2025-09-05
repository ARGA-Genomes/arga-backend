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


#[derive(Atom, Debug, Default, Clone, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Display)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum SpecimenAtom {
    #[default]
    Empty,

    /// The globally unique specimen id.
    SpecimenId(String),

    /// Used to link the collection to a name.
    ScientificName(String),

    /// When the specimen registration happened. Strictly YYYY-MM-DD
    EventDate(String),
    /// What time the specimen registration happened. Strictly HH:MM:SS
    EventTime(String),

    /// The name of the institution that owns the specimen.
    InstitutionName(String),
    /// The short code of the institution.
    InstitutionCode(String),
    /// The ID for the specific specimen. Typically the voucher registration number.
    CollectionRepositoryId(String),
    /// The code for the specific collection repository in the institution.
    CollectionRepositoryCode(String),

    TypeStatus(String),

    Preparation(String),

    OtherCatalogNumbers(String),

    Disposition(String),
}

#[derive(OperationLog, Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::specimen_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SpecimenOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: SpecimenAtom,
}


#[derive(Atom, Debug, Clone, Default, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Display)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum CollectionEventAtom {
    #[default]
    Empty,
    /// The globally unique collection id.
    FieldCollectingId(String),
    /// The global specimen id of the collected specimen if any.
    SpecimenId(String),
    /// Used to link the collection to a name
    ScientificName(String),

    /// When the collection happened. Strictly YYYY-MM-DD
    EventDate(chrono::NaiveDate),
    /// What time the collection happened. Strictly HH:MM:SS
    EventTime(chrono::NaiveTime),

    /// The name of the person who did the collection.
    CollectedBy(String),
    /// Free-text notes about the collection event.
    CollectionRemarks(String),

    /// The name of the person who identified the organism at collection.
    IdentifiedBy(String),
    /// The date the organism collection was identified. Strictly YYYY-MM-DD.
    IdentifiedDate(chrono::NaiveDate),
    /// Free-text notes about the identification of the collection event.
    IdentificationRemarks(String),

    /// The global identifier for the organism that was collected
    OrganismId(String),

    Locality(String),
    Country(String),
    CountryCode(String),
    StateProvince(String),
    County(String),
    Municipality(String),
    Latitude(f64),
    Longitude(f64),
    Elevation(f64),
    Depth(f64),
    ElevationAccuracy(f64),
    DepthAccuracy(f64),
    LocationSource(String),

    Preparation(String),

    EnvironmentBroadScale(String),
    EnvironmentLocalScale(String),
    EnvironmentMedium(String),

    /// The habitat this collection was made in.
    Habitat(String),

    /// Scientific name of the host species.
    SpecificHost(String),

    IndividualCount(String),
    OrganismQuantity(String),
    OrganismQuantityType(String),

    Strain(String),
    Isolate(String),
    FieldNotes(String),
}


#[derive(OperationLog, Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::collection_event_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CollectionEventOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: CollectionEventAtom,
}


#[derive(Atom, Debug, Clone, Default, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Display)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum OrganismAtom {
    #[default]
    Empty,
    OrganismId(String),

    /// Used to link the organism to a name
    ScientificName(String),

    Sex(String),
    GenotypicSex(String),
    PhenotypicSex(String),
    LifeStage(String),
    ReproductiveCondition(String),
    Behavior(String),
}

#[derive(OperationLog, Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::organism_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrganismOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: OrganismAtom,
}


#[derive(Atom, Debug, Clone, Default, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Display)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum AccessionEventAtom {
    #[default]
    Empty,

    /// The global specimen id of the collected specimen.
    SpecimenId(String),
    /// Used to link the accession to a name
    ScientificName(String),

    /// When the registration happened. Strictly YYYY-MM-DD
    EventDate(chrono::NaiveDate),
    /// What time the registration happened. Strictly HH:MM:SS
    EventTime(chrono::NaiveTime),

    TypeStatus(String),

    CollectionRepositoryId(String),
    CollectionRepositoryCode(String),
    InstitutionName(String),
    InstitutionCode(String),
    OtherCatalogNumbers(String),

    Disposition(String),
    Preparation(String),

    AccessionedBy(String),
    PreparedBy(String),

    /// The name of the person who identified the organism at collection.
    IdentifiedBy(String),
    /// The date the organism collection was identified. Strictly YYYY-MM-DD.
    IdentifiedDate(chrono::NaiveDate),
    /// Free-text notes about the identification of the collection event.
    IdentificationRemarks(String),
}

#[derive(OperationLog, Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::accession_event_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AccessionEventOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: AccessionEventAtom,
}


#[derive(Atom, Debug, Clone, Default, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Display)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum TissueAtom {
    #[default]
    Empty,

    /// Used to link the specimen to a name
    ScientificName(String),
    /// The global identifier for the organism that the tissue is from
    OrganismId(String),

    /// The global specimen id of the collected tissue.
    TissueId(String),
    /// The global specimen id of the collected material sample.
    MaterialSampleId(String),

    IdentificationVerified(bool),
    ReferenceMaterial(bool),
    Custodian(String),
    Institution(String),
    InstitutionCode(String),
    SamplingProtocol(String),
    TissueType(String),
    Disposition(String),
    Fixation(String),
    Storage(String),
}

#[derive(OperationLog, Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::tissue_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TissueOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: TissueAtom,
}
