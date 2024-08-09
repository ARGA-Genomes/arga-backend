use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Jsonb;
use diesel::{AsExpression, Associations, FromSqlRow, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{schema, Dataset, DatasetVersion, TaxonomicActType, TaxonomicRank, TaxonomicStatus};
use crate::crdt::DataFrameOperation;
use crate::models::NomenclaturalActType;

#[derive(Clone, Debug, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "schema::sql_types::OperationAction"]
pub enum Action {
    Create,
    Update,
}


#[derive(Debug, Clone, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum TaxonAtom {
    Empty,
    EntityId(String),
    TaxonId(String),
    AcceptedNameUsageId(String),
    ParentNameUsageId(String),
    ParentTaxon(String),

    ScientificName(String),
    Authorship(String),
    CanonicalName(String),
    AcceptedNameUsage(String),
    ParentNameUsage(String),

    TaxonomicRank(TaxonomicRank),
    TaxonomicStatus(TaxonomicStatus),
    NomenclaturalCode(String),
    NomenclaturalStatus(String),

    NamePublishedIn(String),
    NamePublishedInYear(String),
    NamePublishedInUrl(String),

    Citation(String),
    References(String),
    LastUpdated(String),
}

impl FromSql<Jsonb, Pg> for TaxonAtom {
    fn from_sql(value: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        serde_json::from_value(FromSql::<Jsonb, Pg>::from_sql(value)?).map_err(|e| e.into())
    }
}

impl ToSql<Jsonb, Pg> for TaxonAtom {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let json = serde_json::to_value(self)?;
        <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&json, &mut out.reborrow())
    }
}

impl Default for TaxonAtom {
    fn default() -> Self {
        Self::Empty
    }
}

impl ToString for TaxonAtom {
    fn to_string(&self) -> String {
        use TaxonAtom::*;

        match self {
            Empty => "Empty",
            EntityId(_) => "EntityId",
            TaxonId(_) => "TaxonId",
            AcceptedNameUsageId(_) => "AcceptedNameUsageId",
            ParentNameUsageId(_) => "ParentNameUsageId",
            ParentTaxon(_) => "ParentTaxon",
            ScientificName(_) => "ScientificName",
            CanonicalName(_) => "CanonicalName",
            AcceptedNameUsage(_) => "AcceptedNameUsage",
            ParentNameUsage(_) => "ParentNameUsage",
            Authorship(_) => "Authorship",
            TaxonomicRank(_) => "TaxonomicRank",
            TaxonomicStatus(_) => "TaxonomicStatus",
            NomenclaturalCode(_) => "NomenclaturalCode",
            NomenclaturalStatus(_) => "NomenclaturalStatus",
            NamePublishedIn(_) => "NamePublishedIn",
            NamePublishedInYear(_) => "NamePublishedInYear",
            NamePublishedInUrl(_) => "NamePublishedInUrl",
            Citation(_) => "Citation",
            References(_) => "References",
            LastUpdated(_) => "LastUpdated",
        }
        .to_string()
    }
}


#[derive(Debug)]
pub enum NomenclaturalActTypeError {
    InvalidNomenclaturalActType(String),
}

impl TryFrom<&str> for NomenclaturalActType {
    type Error = NomenclaturalActTypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use NomenclaturalActType::*;

        match value {
            "sp. nov." => Ok(SpeciesNova),
            "spec. nov." => Ok(SpeciesNova),
            "new species" => Ok(SpeciesNova),
            "comb. nov." => Ok(CombinatioNova),
            "stat. rev." => Ok(RevivedStatus),
            "gen. et sp. nov." => Ok(GenusSpeciesNova),
            "subsp. nov." => Ok(SubspeciesNova),
            val => Err(NomenclaturalActTypeError::InvalidNomenclaturalActType(val.to_string())),
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum NomenclaturalActAtom {
    Empty,
    Publication(String),
    PublicationDate(String),
    ActedOn(String),
    Act(NomenclaturalActType),
    SourceUrl(String),

    ScientificName(String),
    CanonicalName(String),
    AuthorityName(String),
    AuthorityYear(String),
    BasionymAuthorityName(String),
    BasionymAuthorityYear(String),
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
            Publication(_) => "Publication",
            PublicationDate(_) => "PublicationDate",
            ActedOn(_) => "ActedOn",
            Act(_) => "Act",
            SourceUrl(_) => "SourceUrl",

            ScientificName(_) => "ScientificName",
            CanonicalName(_) => "CanonicalName",
            AuthorityName(_) => "AuthorityName",
            AuthorityYear(_) => "AuthorityYear",
            BasionymAuthorityName(_) => "BasionymAuthorityName",
            BasionymAuthorityYear(_) => "BasionymAuthorityYear",
        }
        .to_string()
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq)]
#[diesel(sql_type = diesel::sql_types::Jsonb)]
pub enum TaxonomicActAtom {
    Empty,
    EntityId(String),
    Publication(String),
    PublicationDate(String),
    Taxon(String),
    AcceptedTaxon(String),
    Act(TaxonomicActType),
    SourceUrl(String),
    CreatedAt(DateTime<Utc>),
    UpdatedAt(DateTime<Utc>),
}

impl Default for TaxonomicActAtom {
    fn default() -> Self {
        Self::Empty
    }
}

impl FromSql<Jsonb, Pg> for TaxonomicActAtom {
    fn from_sql(value: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        serde_json::from_value(FromSql::<Jsonb, Pg>::from_sql(value)?).map_err(|e| e.into())
    }
}

impl ToSql<Jsonb, Pg> for TaxonomicActAtom {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let json = serde_json::to_value(self)?;
        <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&json, &mut out.reborrow())
    }
}

impl ToString for TaxonomicActAtom {
    fn to_string(&self) -> String {
        use TaxonomicActAtom::*;

        match self {
            Empty => "Empty",
            EntityId(_) => "EntityId",
            Publication(_) => "Publication",
            PublicationDate(_) => "PublicationDate",
            Taxon(_) => "Taxon",
            AcceptedTaxon(_) => "AcceptedTaxon",
            Act(_) => "Act",
            SourceUrl(_) => "SourceUrl",
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
    SpecimenId(String),
    EventDate(String),
    EventTime(String),
    CollectedBy(String),
    FieldNumber(String),
    CatalogNumber(String),
    RecordNumber(String),
    IndividualCount(String),
    OrganismQuantity(String),
    OrganismQuantityType(String),
    Sex(String),
    GenotypicSex(String),
    PhenotypicSex(String),
    LifeStage(String),
    ReproductiveCondition(String),
    Behavior(String),
    EstablishmentMeans(String),
    DegreeOfEstablishment(String),
    Pathway(String),
    OccurrenceStatus(String),
    Preparation(String),
    OtherCatalogNumbers(String),
    EnvBroadScale(String),
    EnvLocalScale(String),
    EnvMedium(String),
    Habitat(String),
    RefBiomaterial(String),
    SourceMatId(String),
    SpecificHost(String),
    Strain(String),
    Isolate(String),
    FieldNotes(String),
    Remarks(String),
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
            SpecimenId(_) => "SpecimenId",
            EventDate(_) => "EventDate",
            EventTime(_) => "EventTime",
            CollectedBy(_) => "CollectedBy",
            FieldNumber(_) => "FieldNumber",
            CatalogNumber(_) => "CatalogNumber",
            RecordNumber(_) => "RecordNumber",
            IndividualCount(_) => "IndividualCount",
            OrganismQuantity(_) => "OrganismQuantity",
            OrganismQuantityType(_) => "OrganismQuantityType",
            Sex(_) => "Sex",
            GenotypicSex(_) => "GenotypicSex",
            PhenotypicSex(_) => "PhenotypicSex",
            LifeStage(_) => "LifeStage",
            ReproductiveCondition(_) => "ReproductiveCondition",
            Behavior(_) => "Behavior",
            EstablishmentMeans(_) => "EstablishmentMeans",
            DegreeOfEstablishment(_) => "DegreeOfEstablishment",
            Pathway(_) => "Pathway",
            OccurrenceStatus(_) => "OccurrenceStatus",
            Preparation(_) => "Preparation",
            OtherCatalogNumbers(_) => "OtherCatalogNumbers",
            EnvBroadScale(_) => "EnvBroadScale",
            EnvLocalScale(_) => "EnvLocalScale",
            EnvMedium(_) => "EnvMedium",
            Habitat(_) => "Habitat",
            RefBiomaterial(_) => "RefBiomaterial",
            SourceMatId(_) => "SourceMatId",
            SpecificHost(_) => "SpecificHost",
            Strain(_) => "Strain",
            Isolate(_) => "Isolate",
            FieldNotes(_) => "FieldNotes",
            Remarks(_) => "Remarks",
        }
        .to_string()
    }
}

pub trait LogOperation<T> {
    fn id(&self) -> &String;
    fn action(&self) -> &Action;
    fn atom(&self) -> &T;
}

pub trait LogOperationDataset {
    fn dataset_version(&self) -> &DatasetVersion;
    fn dataset(&self) -> &Dataset;
}


#[derive(Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::taxa_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TaxonOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: TaxonAtom,
}

impl From<DataFrameOperation<TaxonAtom>> for TaxonOperation {
    fn from(value: DataFrameOperation<TaxonAtom>) -> Self {
        Self {
            operation_id: value.operation_id,
            parent_id: value.parent_id,
            entity_id: value.entity_id,
            dataset_version_id: value.dataset_version_id,
            action: value.action,
            atom: value.atom,
        }
    }
}

impl LogOperation<TaxonAtom> for TaxonOperation {
    fn id(&self) -> &String {
        &self.entity_id
    }

    fn action(&self) -> &Action {
        &self.action
    }

    fn atom(&self) -> &TaxonAtom {
        &self.atom
    }
}

#[derive(Queryable, Selectable, Debug, Deserialize, Clone)]
#[diesel(table_name = schema::taxa_logs)]
pub struct TaxonOperationWithDataset {
    #[diesel(embed)]
    pub operation: TaxonOperation,
    #[diesel(embed)]
    pub dataset_version: DatasetVersion,
    #[diesel(embed)]
    pub dataset: Dataset,
}

impl LogOperation<TaxonAtom> for TaxonOperationWithDataset {
    fn id(&self) -> &String {
        self.operation.id()
    }

    fn action(&self) -> &Action {
        self.operation.action()
    }

    fn atom(&self) -> &TaxonAtom {
        self.operation.atom()
    }
}

impl LogOperationDataset for TaxonOperationWithDataset {
    fn dataset_version(&self) -> &DatasetVersion {
        &self.dataset_version
    }

    fn dataset(&self) -> &Dataset {
        &self.dataset
    }
}

#[derive(Queryable, Selectable, Insertable, Associations, Debug, Serialize, Deserialize, Clone)]
#[diesel(belongs_to(DatasetVersion))]
#[diesel(table_name = schema::taxonomic_act_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TaxonomicActOperation {
    pub operation_id: BigDecimal,
    pub parent_id: BigDecimal,
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: TaxonomicActAtom,
}

impl From<DataFrameOperation<TaxonomicActAtom>> for TaxonomicActOperation {
    fn from(value: DataFrameOperation<TaxonomicActAtom>) -> Self {
        Self {
            operation_id: value.operation_id,
            parent_id: value.parent_id,
            entity_id: value.entity_id,
            dataset_version_id: value.dataset_version_id,
            action: value.action,
            atom: value.atom,
        }
    }
}

impl LogOperation<TaxonomicActAtom> for TaxonomicActOperation {
    fn id(&self) -> &String {
        &self.entity_id
    }

    fn action(&self) -> &Action {
        &self.action
    }

    fn atom(&self) -> &TaxonomicActAtom {
        &self.atom
    }
}

#[derive(Queryable, Selectable, Debug, Deserialize, Clone)]
#[diesel(table_name = schema::taxonomic_act_logs)]
pub struct TaxonomicActOperationWithDataset {
    #[diesel(embed)]
    pub operation: TaxonomicActOperation,
    #[diesel(embed)]
    pub dataset_version: DatasetVersion,
    #[diesel(embed)]
    pub dataset: Dataset,
}

impl LogOperation<TaxonomicActAtom> for TaxonomicActOperationWithDataset {
    fn id(&self) -> &String {
        self.operation.id()
    }

    fn action(&self) -> &Action {
        self.operation.action()
    }

    fn atom(&self) -> &TaxonomicActAtom {
        self.operation.atom()
    }
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
    fn id(&self) -> &String {
        &self.entity_id
    }

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
    fn id(&self) -> &String {
        &self.entity_id
    }

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
    pub entity_id: String,
    pub dataset_version_id: Uuid,
    pub action: Action,
    pub atom: CollectionEventAtom,
}

impl LogOperation<CollectionEventAtom> for CollectionEventOperation {
    fn id(&self) -> &String {
        &self.entity_id
    }

    fn action(&self) -> &Action {
        &self.action
    }

    fn atom(&self) -> &CollectionEventAtom {
        &self.atom
    }
}
