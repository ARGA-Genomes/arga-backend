use arga_core::crdt::hlc::HybridTimestamp;
use async_graphql::*;
use bigdecimal::ToPrimitive;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::datasets::{DatasetDetails, DatasetVersion};
use super::taxonomy::NomenclaturalActType;
use crate::database::{models, Database};
use crate::http::Error;


type UtcDateTime = DateTime<Utc>;


#[derive(SimpleObject)]
#[graphql(concrete(name = "NomenclaturalActAtomText", params(NomenclaturalActAtomTextType, String)))]
#[graphql(concrete(
    name = "NomenclaturalActAtomDateTime",
    params(NomenclaturalActAtomDateTimeType, UtcDateTime)
))]
#[graphql(concrete(
    name = "NomenclaturalActAtomType",
    params(NomenclaturalActAtomActType, NomenclaturalActType)
))]
#[graphql(concrete(name = "SpecimenAtomText", params(SpecimenAtomTextType, String)))]
#[graphql(concrete(name = "SpecimenAtomNumber", params(SpecimenAtomNumberType, f64)))]
pub struct Atom<A: OutputType, T: OutputType> {
    pub r#type: A,
    pub value: T,
}


type NomenclaturalActAtomText = Atom<NomenclaturalActAtomTextType, String>;
type NomenclaturalActAtomDateTime = Atom<NomenclaturalActAtomDateTimeType, UtcDateTime>;
type NomenclaturalActAtomAct = Atom<NomenclaturalActAtomActType, NomenclaturalActType>;
type SpecimenAtomText = Atom<SpecimenAtomTextType, String>;
type SpecimenAtomNumber = Atom<SpecimenAtomNumberType, f64>;


#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "models::Action")]
pub enum Action {
    Create,
    Update,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TaxonAtomTextType {
    Empty,
    TaxonId,
    AcceptedNameUsageId,
    ParentNameUsageId,

    CanonicalName,
    AcceptedNameUsage,
    ParentNameUsage,
    ScientificNameAuthorship,

    TaxonRank,
    TaxonomicStatus,
    NomenclaturalCode,
    NomenclaturalStatus,

    NamePublishedIn,
    NamePublishedInYear,
    NamePublishedInUrl,

    Citation,
    References,
    LastUpdated,
}


#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum NomenclaturalActAtomTextType {
    Empty,
    ActedOn,
    Act,
    SourceUrl,
    Publication,
    PublicationDate,

    ScientificName,
    CanonicalName,
    AuthorityName,
    AuthorityYear,
    BasionymAuthorityName,
    BasionymAuthorityYear,

    Genus,
    SpecificEpithet,
    Rank,
}


#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum NomenclaturalActAtomDateTimeType {
    CreatedAt,
    UpdatedAt,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum NomenclaturalActAtomActType {
    NomenclaturalActType,
}

#[derive(Union)]
pub enum NomenclaturalActAtom {
    Text(NomenclaturalActAtomText),
    Act(NomenclaturalActAtomAct),
    DateTime(NomenclaturalActAtomDateTime),
}

impl NomenclaturalActAtom {
    pub fn text(r#type: NomenclaturalActAtomTextType, value: String) -> NomenclaturalActAtom {
        NomenclaturalActAtom::Text(NomenclaturalActAtomText { r#type, value })
    }

    pub fn datetime(r#type: NomenclaturalActAtomDateTimeType, value: DateTime<Utc>) -> NomenclaturalActAtom {
        NomenclaturalActAtom::DateTime(NomenclaturalActAtomDateTime { r#type, value })
    }

    pub fn act(r#type: NomenclaturalActAtomActType, value: NomenclaturalActType) -> NomenclaturalActAtom {
        NomenclaturalActAtom::Act(NomenclaturalActAtomAct { r#type, value })
    }
}


#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum SpecimenAtomTextType {
    Empty,
    RecordId,
    MaterialSampleId,
    OrganismId,
    ScientificName,

    InstitutionName,
    InstitutionCode,
    CollectionCode,
    RecordedBy,
    IdentifiedBy,
    IdentifiedDate,

    TypeStatus,
    Locality,
    Country,
    CountryCode,
    StateProvince,
    County,
    Municipality,
    LocationSource,

    Details,
    Remarks,
    IdentificationRemarks,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum SpecimenAtomNumberType {
    Latitude,
    Longitude,
    Elevation,
    Depth,
    ElevationAccuracy,
    DepthAccuracy,
}

#[derive(Union)]
pub enum SpecimenAtom {
    Text(SpecimenAtomText),
    Number(SpecimenAtomNumber),
}

impl SpecimenAtom {
    pub fn text(r#type: SpecimenAtomTextType, value: String) -> SpecimenAtom {
        SpecimenAtom::Text(SpecimenAtomText { r#type, value })
    }

    pub fn number(r#type: SpecimenAtomNumberType, value: f64) -> SpecimenAtom {
        SpecimenAtom::Number(SpecimenAtomNumber { r#type, value })
    }
}


#[derive(OneofObject)]
pub enum OperationBy {
    EntityId(String),
}


#[derive(SimpleObject)]
pub struct NomenclaturalActOperation {
    pub operation_id: u64,
    pub parent_id: u64,
    pub entity_id: String,
    pub dataset: DatasetDetails,
    pub dataset_version: DatasetVersion,
    pub action: Action,
    pub atom: NomenclaturalActAtom,
    pub logged_at: DateTime<Utc>,
}

impl NomenclaturalActOperation {
    pub async fn new(db: &Database, by: OperationBy) -> Result<Vec<NomenclaturalActOperation>, Error> {
        let records = match by {
            OperationBy::EntityId(id) => db.provenance.find_by_entity_id_with_dataset(&id).await?,
        };

        let mut operations = Vec::with_capacity(records.len());
        for (record, dataset_version, dataset) in records {
            let mut op = NomenclaturalActOperation::try_from(record)?;
            op.dataset = dataset.into();
            op.dataset_version = dataset_version.into();
            operations.push(op);
        }

        Ok(operations)
    }
}

impl TryFrom<models::NomenclaturalActOperation> for NomenclaturalActOperation {
    type Error = Error;

    fn try_from(value: models::NomenclaturalActOperation) -> Result<Self, Self::Error> {
        let ts = HybridTimestamp::new(value.operation_id.to_u64().unwrap());

        Ok(Self {
            operation_id: value.operation_id.to_u64().ok_or(Error::InvalidData(
                "operation_id".to_string(),
                "Operation".to_string(),
                value.operation_id.to_string(),
            ))?,
            parent_id: value.parent_id.to_u64().ok_or(Error::InvalidData(
                "parent_id".to_string(),
                "Operation".to_string(),
                value.operation_id.to_string(),
            ))?,
            entity_id: value.entity_id,
            action: value.action.into(),
            atom: value.atom.into(),
            dataset: DatasetDetails::default(),
            dataset_version: DatasetVersion::default(),
            logged_at: ts.into(),
        })
    }
}

impl From<models::NomenclaturalActAtom> for NomenclaturalActAtom {
    fn from(value: models::NomenclaturalActAtom) -> Self {
        use models::NomenclaturalActAtom::*;
        use {NomenclaturalActAtom as Atom, NomenclaturalActAtomTextType as Text};

        match value {
            Empty => Atom::text(Text::Empty, "".to_string()),
            Publication(value) => Atom::text(Text::Publication, value),
            PublicationDate(value) => Atom::text(Text::PublicationDate, value),
            ActedOn(value) => Atom::text(Text::ActedOn, value),
            Act(value) => Atom::act(NomenclaturalActAtomActType::NomenclaturalActType, value.into()),
            SourceUrl(value) => Atom::text(Text::SourceUrl, value),
            ScientificName(value) => Atom::text(Text::ScientificName, value),
            CanonicalName(value) => Atom::text(Text::CanonicalName, value),
            AuthorityName(value) => Atom::text(Text::AuthorityName, value),
            AuthorityYear(value) => Atom::text(Text::AuthorityYear, value),
            BasionymAuthorityName(value) => Atom::text(Text::BasionymAuthorityName, value),
            BasionymAuthorityYear(value) => Atom::text(Text::BasionymAuthorityYear, value),
        }
    }
}


#[derive(SimpleObject)]
pub struct SpecimenOperation {
    pub operation_id: u64,
    pub parent_id: u64,
    pub entity_id: String,
    pub dataset_version: DatasetVersion,
    pub dataset: DatasetDetails,
    pub action: Action,
    pub atom: SpecimenAtom,
    pub logged_at: DateTime<Utc>,
}

impl SpecimenOperation {
    pub async fn new(db: &Database, by: OperationBy) -> Result<Vec<SpecimenOperation>, Error> {
        let records = match by {
            OperationBy::EntityId(id) => db.provenance.find_specimen_logs_by_entity_id_with_dataset(&id).await?,
        };

        let mut operations = Vec::with_capacity(records.len());
        for (record, version, dataset) in records {
            let mut op = SpecimenOperation::try_from(record)?;
            op.dataset = dataset.into();
            op.dataset_version = version.into();
            operations.push(op);
        }

        Ok(operations)
    }
}

impl TryFrom<models::SpecimenOperation> for SpecimenOperation {
    type Error = Error;

    fn try_from(value: models::SpecimenOperation) -> Result<Self, Self::Error> {
        let ts = HybridTimestamp::new(value.operation_id.to_u64().unwrap());

        Ok(Self {
            operation_id: value.operation_id.to_u64().ok_or(Error::InvalidData(
                "operation_id".to_string(),
                "Operation".to_string(),
                value.operation_id.to_string(),
            ))?,
            parent_id: value.parent_id.to_u64().ok_or(Error::InvalidData(
                "parent_id".to_string(),
                "Operation".to_string(),
                value.operation_id.to_string(),
            ))?,
            entity_id: value.entity_id,
            action: value.action.into(),
            atom: value.atom.into(),
            dataset: DatasetDetails::default(),
            dataset_version: DatasetVersion::default(),
            logged_at: ts.into(),
        })
    }
}

impl From<models::SpecimenAtom> for SpecimenAtom {
    fn from(value: models::SpecimenAtom) -> Self {
        use models::SpecimenAtom::*;
        use {SpecimenAtom as Atom, SpecimenAtomNumberType as Number, SpecimenAtomTextType as Text};

        match value {
            Empty => Atom::text(Text::Empty, "".to_string()),
            RecordId(value) => Atom::text(Text::RecordId, value),
            MaterialSampleId(value) => Atom::text(Text::MaterialSampleId, value),
            OrganismId(value) => Atom::text(Text::OrganismId, value),
            ScientificName(value) => Atom::text(Text::ScientificName, value),
            InstitutionName(value) => Atom::text(Text::InstitutionName, value),
            InstitutionCode(value) => Atom::text(Text::InstitutionCode, value),
            CollectionCode(value) => Atom::text(Text::CollectionCode, value),
            RecordedBy(value) => Atom::text(Text::RecordedBy, value),
            IdentifiedBy(value) => Atom::text(Text::IdentifiedBy, value),
            IdentifiedDate(value) => Atom::text(Text::IdentifiedDate, value),
            TypeStatus(value) => Atom::text(Text::TypeStatus, value),
            Locality(value) => Atom::text(Text::Locality, value),
            Country(value) => Atom::text(Text::Country, value),
            CountryCode(value) => Atom::text(Text::CountryCode, value),
            StateProvince(value) => Atom::text(Text::StateProvince, value),
            County(value) => Atom::text(Text::County, value),
            Municipality(value) => Atom::text(Text::Municipality, value),
            Latitude(value) => Atom::number(Number::Latitude, value),
            Longitude(value) => Atom::number(Number::Longitude, value),
            Elevation(value) => Atom::number(Number::Elevation, value),
            Depth(value) => Atom::number(Number::Depth, value),
            ElevationAccuracy(value) => Atom::number(Number::ElevationAccuracy, value),
            DepthAccuracy(value) => Atom::number(Number::DepthAccuracy, value),
            LocationSource(value) => Atom::text(Text::LocationSource, value),
            Details(value) => Atom::text(Text::Details, value),
            Remarks(value) => Atom::text(Text::Remarks, value),
            IdentificationRemarks(value) => Atom::text(Text::IdentificationRemarks, value),
        }
    }
}
