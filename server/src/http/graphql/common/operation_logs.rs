use arga_core::crdt::hlc::HybridTimestamp;
use async_graphql::*;
use bigdecimal::ToPrimitive;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::datasets::{DatasetDetails, DatasetVersion};
use super::taxonomy::{NomenclaturalActType, TaxonomicRank, TaxonomicStatus};
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
#[graphql(concrete(name = "TaxonAtomText", params(TaxonAtomTextType, String)))]
#[graphql(concrete(name = "TaxonAtomRank", params(TaxonAtomRankType, TaxonomicRank)))]
#[graphql(concrete(name = "TaxonAtomStatus", params(TaxonAtomStatusType, TaxonomicStatus)))]
pub struct Atom<A: OutputType, T: OutputType> {
    pub r#type: A,
    pub value: T,
}


type NomenclaturalActAtomText = Atom<NomenclaturalActAtomTextType, String>;
type NomenclaturalActAtomDateTime = Atom<NomenclaturalActAtomDateTimeType, UtcDateTime>;
type NomenclaturalActAtomAct = Atom<NomenclaturalActAtomActType, NomenclaturalActType>;
type SpecimenAtomText = Atom<SpecimenAtomTextType, String>;
type TaxonAtomText = Atom<TaxonAtomTextType, String>;
type TaxonAtomRank = Atom<TaxonAtomRankType, TaxonomicRank>;
type TaxonAtomStatus = Atom<TaxonAtomStatusType, TaxonomicStatus>;


#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "models::logs::Action")]
pub enum Action {
    Create,
    Update,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TaxonAtomTextType {
    Empty,
    EntityId,
    DatasetId,
    TaxonId,
    AcceptedNameUsageId,
    ParentNameUsageId,
    ParentTaxon,

    ScientificName,
    Authorship,
    CanonicalName,
    AcceptedNameUsage,
    ParentNameUsage,

    TaxonomicRank,
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
pub enum TaxonAtomRankType {
    TaxonomicRankType,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TaxonAtomStatusType {
    TaxonomicStatusType,
}

#[derive(Union)]
pub enum TaxonAtom {
    Text(TaxonAtomText),
    TaxonomicRank(TaxonAtomRank),
    TaxonomicStatus(TaxonAtomStatus),
}

impl TaxonAtom {
    pub fn text(r#type: TaxonAtomTextType, value: String) -> TaxonAtom {
        TaxonAtom::Text(TaxonAtomText { r#type, value })
    }

    pub fn rank(r#type: TaxonAtomRankType, value: TaxonomicRank) -> TaxonAtom {
        TaxonAtom::TaxonomicRank(TaxonAtomRank { r#type, value })
    }

    pub fn status(r#type: TaxonAtomStatusType, value: TaxonomicStatus) -> TaxonAtom {
        TaxonAtom::TaxonomicStatus(TaxonAtomStatus { r#type, value })
    }
}


#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum NomenclaturalActAtomTextType {
    Empty,
    EntityId,
    ActedOn,
    Act,
    SourceUrl,
    Publication,
    PublicationDate,

    ScientificName,
    Authorship,
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
    SpecimenId,
    ScientificName,

    EventDate,
    EventTime,

    InstitutionName,
    InstitutionCode,
    CollectionRepositoryId,
    CollectionRepositoryCode,

    TypeStatus,
    Preparation,
    OtherCatalogNumbers,
    Disposition,
}


#[derive(Union)]
pub enum SpecimenAtom {
    Text(SpecimenAtomText),
}

impl SpecimenAtom {
    pub fn text(r#type: SpecimenAtomTextType, value: String) -> SpecimenAtom {
        SpecimenAtom::Text(SpecimenAtomText { r#type, value })
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
            EntityId(value) => Atom::text(Text::EntityId, value),
            Publication(value) => Atom::text(Text::Publication, value),
            PublicationDate(value) => Atom::text(Text::PublicationDate, value),
            ActedOn(value) => Atom::text(Text::ActedOn, value),
            Act(value) => Atom::act(NomenclaturalActAtomActType::NomenclaturalActType, value.into()),
            SourceUrl(value) => Atom::text(Text::SourceUrl, value),
            ScientificName(value) => Atom::text(Text::ScientificName, value),
            Authorship(value) => Atom::text(Text::Authorship, value),
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

impl TryFrom<models::logs::SpecimenOperation> for SpecimenOperation {
    type Error = Error;

    fn try_from(value: models::logs::SpecimenOperation) -> Result<Self, Self::Error> {
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
            entity_id: value.entity_id.to_string(),
            action: value.action.into(),
            atom: value.atom.into(),
            dataset: DatasetDetails::default(),
            dataset_version: DatasetVersion::default(),
            logged_at: ts.into(),
        })
    }
}

impl From<models::logs::SpecimenAtom> for SpecimenAtom {
    fn from(value: models::logs::SpecimenAtom) -> Self {
        use models::logs::SpecimenAtom::*;
        use {SpecimenAtom as Atom, SpecimenAtomTextType as Text};

        match value {
            Empty => Atom::text(Text::Empty, "".to_string()),
            SpecimenId(value) => Atom::text(Text::SpecimenId, value),
            ScientificName(value) => Atom::text(Text::ScientificName, value),
            EventDate(value) => Atom::text(Text::EventDate, value),
            EventTime(value) => Atom::text(Text::EventTime, value),
            InstitutionName(value) => Atom::text(Text::InstitutionName, value),
            InstitutionCode(value) => Atom::text(Text::InstitutionCode, value),
            CollectionRepositoryId(value) => Atom::text(Text::CollectionRepositoryId, value),
            CollectionRepositoryCode(value) => Atom::text(Text::CollectionRepositoryCode, value),
            TypeStatus(value) => Atom::text(Text::TypeStatus, value),
            Preparation(value) => Atom::text(Text::Preparation, value),
            OtherCatalogNumbers(value) => Atom::text(Text::OtherCatalogNumbers, value),
            Disposition(value) => Atom::text(Text::Disposition, value),
        }
    }
}


#[derive(SimpleObject)]
pub struct TaxonOperation {
    pub operation_id: u64,
    pub parent_id: u64,
    pub entity_id: String,
    pub dataset_version: DatasetVersion,
    pub dataset: DatasetDetails,
    pub action: Action,
    pub atom: TaxonAtom,
    pub logged_at: DateTime<Utc>,
}

impl TaxonOperation {
    pub async fn new(db: &Database, by: OperationBy) -> Result<Vec<TaxonOperation>, Error> {
        let records = match by {
            OperationBy::EntityId(id) => db.provenance.find_taxon_logs_by_entity_id_with_dataset(&id).await?,
        };

        let mut operations = Vec::with_capacity(records.len());
        for (record, version, dataset) in records {
            let mut op = TaxonOperation::try_from(record)?;
            op.dataset = dataset.into();
            op.dataset_version = version.into();
            operations.push(op);
        }

        Ok(operations)
    }
}

impl TryFrom<models::TaxonOperation> for TaxonOperation {
    type Error = Error;

    fn try_from(value: models::TaxonOperation) -> Result<Self, Self::Error> {
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

impl From<models::TaxonAtom> for TaxonAtom {
    fn from(value: models::TaxonAtom) -> Self {
        use models::TaxonAtom::*;
        use {TaxonAtom as Atom, TaxonAtomTextType as Text};

        match value {
            Empty => Atom::text(Text::Empty, "".to_string()),
            EntityId(value) => Atom::text(Text::EntityId, value),
            DatasetId(value) => Atom::text(Text::DatasetId, value),
            TaxonId(value) => Atom::text(Text::TaxonId, value),
            AcceptedNameUsageId(value) => Atom::text(Text::AcceptedNameUsageId, value),
            ParentNameUsageId(value) => Atom::text(Text::ParentNameUsageId, value),
            ParentTaxon(value) => Atom::text(Text::ParentTaxon, value),
            ScientificName(value) => Atom::text(Text::ScientificName, value),
            Authorship(value) => Atom::text(Text::Authorship, value),
            CanonicalName(value) => Atom::text(Text::CanonicalName, value),
            AcceptedNameUsage(value) => Atom::text(Text::AcceptedNameUsage, value),
            ParentNameUsage(value) => Atom::text(Text::ParentNameUsage, value),
            TaxonomicRank(value) => Atom::rank(TaxonAtomRankType::TaxonomicRankType, value.into()),
            TaxonomicStatus(value) => Atom::status(TaxonAtomStatusType::TaxonomicStatusType, value.into()),
            NomenclaturalCode(value) => Atom::text(Text::NomenclaturalCode, value),
            NomenclaturalStatus(value) => Atom::text(Text::NomenclaturalStatus, value),
            NamePublishedIn(value) => Atom::text(Text::NamePublishedIn, value),
            NamePublishedInYear(value) => Atom::text(Text::NamePublishedInYear, value),
            NamePublishedInUrl(value) => Atom::text(Text::NamePublishedInUrl, value),
            Citation(value) => Atom::text(Text::Citation, value),
            References(value) => Atom::text(Text::References, value),
            LastUpdated(value) => Atom::text(Text::LastUpdated, value),
        }
    }
}
