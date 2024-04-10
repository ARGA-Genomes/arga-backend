use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
    sql_types::Jsonb,
    AsExpression, Associations, FromSqlRow, Insertable, Queryable, Selectable,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{schema, DatasetVersion, TaxonomicStatus};

#[derive(Clone, Debug, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "schema::sql_types::OperationAction"]
pub enum Action {
    Create,
    Update,
}

#[derive(Debug, Clone, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum NomenclaturalActAtom {
    Empty,
    ScientificName(String),
    ActedOn(String),
    TaxonomicStatus(TaxonomicStatus),
    NomenclaturalAct(String),
    SourceUrl(String),
    Publication(String),
    CreatedAt(DateTime<Utc>),
    UpdatedAt(DateTime<Utc>),
}

impl FromSql<Jsonb, Pg> for NomenclaturalActAtom {
    fn from_sql(value: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        serde_json::from_value(FromSql::<Jsonb, Pg>::from_sql(value)?).map_err(|e| e.into())
    }
}

impl ToSql<Jsonb, Pg> for NomenclaturalActAtom {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let json = serde_json::to_value(self)?;
        <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&json, &mut out.reborrow())
    }
}

impl ToString for NomenclaturalActAtom {
    fn to_string(&self) -> String {
        use NomenclaturalActAtom::*;

        match self {
            Empty => "Empty",
            ScientificName(_) => "ScientificName",
            ActedOn(_) => "ActedOn",
            TaxonomicStatus(_) => "TaxonomicStatus",
            NomenclaturalAct(_) => "NomenclaturalAct",
            SourceUrl(_) => "SourceUrl",
            Publication(_) => "Publication",
            CreatedAt(_) => "CreatedAt",
            UpdatedAt(_) => "UpdatedAt",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum SpecimenAtom {
    Empty,
    RecordId(String),
    MaterialSampleId(String),
    OrganismId(String),
    ScientificName(String),

    InstitutionName(String),
    InstitutionCode(String),
    CollectionCode(String),
    RecordedBy(String),
    IdentifiedBy(String),
    IdentifiedDate(String),

    TypeStatus(String),
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

    Details(String),
    Remarks(String),
    IdentificationRemarks(String),
}

impl FromSql<Jsonb, Pg> for SpecimenAtom {
    fn from_sql(value: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        serde_json::from_value(FromSql::<Jsonb, Pg>::from_sql(value)?).map_err(|e| e.into())
    }
}

impl ToSql<Jsonb, Pg> for SpecimenAtom {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let json = serde_json::to_value(self)?;
        <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&json, &mut out.reborrow())
    }
}

impl ToString for SpecimenAtom {
    fn to_string(&self) -> String {
        use SpecimenAtom::*;

        match self {
            Empty => "Empty",
            RecordId(_) => "RecordId",
            MaterialSampleId(_) => "MaterialSampleId",
            OrganismId(_) => "OrganismId",
            ScientificName(_) => "ScientificName",
            InstitutionName(_) => "InstitutionName",
            InstitutionCode(_) => "InstitutionCode",
            CollectionCode(_) => "CollectionCode",
            RecordedBy(_) => "RecordedBy",
            IdentifiedBy(_) => "IdentifiedBy",
            IdentifiedDate(_) => "IdentifiedDate",
            TypeStatus(_) => "TypeStatus",
            Locality(_) => "Locality",
            Country(_) => "Country",
            CountryCode(_) => "CountryCode",
            StateProvince(_) => "StateProvince",
            County(_) => "County",
            Municipality(_) => "Municipality",
            Latitude(_) => "Latitude",
            Longitude(_) => "Longitude",
            Elevation(_) => "Elevation",
            Depth(_) => "Depth",
            ElevationAccuracy(_) => "ElevationAccuracy",
            DepthAccuracy(_) => "DepthAccuracy",
            LocationSource(_) => "LocationSource",
            Details(_) => "Details",
            Remarks(_) => "Remarks",
            IdentificationRemarks(_) => "IdentificationRemarks",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum CollectionEventAtom {
    Empty,
}

impl FromSql<Jsonb, Pg> for CollectionEventAtom {
    fn from_sql(value: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        serde_json::from_value(FromSql::<Jsonb, Pg>::from_sql(value)?).map_err(|e| e.into())
    }
}

impl ToSql<Jsonb, Pg> for CollectionEventAtom {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let json = serde_json::to_value(self)?;
        <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&json, &mut out.reborrow())
    }
}

impl ToString for CollectionEventAtom {
    fn to_string(&self) -> String {
        use CollectionEventAtom::*;

        match self {
            Empty => "Empty",
        }
        .to_string()
    }
}

pub trait LogOperation<T> {
    fn action(&self) -> &Action;
    fn atom(&self) -> &T;
}

#[derive(Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::nomenclatural_act_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NomenclaturalActOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: NomenclaturalActAtom,
}

impl LogOperation<NomenclaturalActAtom> for NomenclaturalActOperation {
    fn action(&self) -> &Action {
        &self.action
    }

    fn atom(&self) -> &NomenclaturalActAtom {
        &self.atom
    }
}

#[derive(Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
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

impl LogOperation<SpecimenAtom> for SpecimenOperation {
    fn action(&self) -> &Action {
        &self.action
    }

    fn atom(&self) -> &SpecimenAtom {
        &self.atom
    }
}

#[derive(Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::collection_event_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CollectionEventOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub dataset_version_id: Uuid,
    pub entity_id: String,
    pub action: Action,
    pub atom: CollectionEventAtom,
}

impl LogOperation<CollectionEventAtom> for CollectionEventOperation {
    fn action(&self) -> &Action {
        &self.action
    }

    fn atom(&self) -> &CollectionEventAtom {
        &self.atom
    }
}
