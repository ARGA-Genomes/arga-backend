use arga_core::models;
use async_graphql::*;
use bigdecimal::ToPrimitive;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::Database;
use crate::http::Error;

use super::common::taxonomy::TaxonomicStatus;
use super::dataset::DatasetDetails;

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "models::Action")]
pub enum Action {
    Create,
    Update,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum AtomTextType {
    Empty,
    ScientificName,
    ActedOn,
    NomenclaturalAct,
    SourceUrl,
    Publication,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum AtomTaxonomicStatusType {
    TaxonomicStatus,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum AtomDateTimeType {
    CreatedAt,
    UpdatedAt,
}

#[derive(Union)]
pub enum Atom {
    Text(AtomText),
    TaxonomicStatus(AtomTaxonomicStatus),
    DateTime(AtomDateTime),
}

impl Atom {
    pub fn text(r#type: AtomTextType, value: String) -> Atom {
        Atom::Text(AtomText { r#type, value })
    }

    pub fn taxonomic_status(r#type: AtomTaxonomicStatusType, value: TaxonomicStatus) -> Atom {
        Atom::TaxonomicStatus(AtomTaxonomicStatus { r#type, value })
    }

    pub fn datetime(r#type: AtomDateTimeType, value: DateTime<Utc>) -> Atom {
        Atom::DateTime(AtomDateTime { r#type, value })
    }
}

#[derive(SimpleObject)]
pub struct AtomText {
    r#type: AtomTextType,
    value: String,
}

#[derive(SimpleObject)]
pub struct AtomTaxonomicStatus {
    r#type: AtomTaxonomicStatusType,
    value: TaxonomicStatus,
}

#[derive(SimpleObject)]
pub struct AtomDateTime {
    r#type: AtomDateTimeType,
    value: DateTime<Utc>,
}

#[derive(OneofObject)]
pub enum OperationBy {
    EntityId(String),
}

#[derive(SimpleObject)]
pub struct Operation {
    pub operation_id: u64,
    pub reference_id: u64,
    pub dataset: DatasetDetails,
    pub entity_id: String,
    pub action: Action,
    pub atom: Atom,
}

impl Operation {
    pub async fn new(db: &Database, by: OperationBy) -> Result<Vec<Operation>, Error> {
        let records = match by {
            OperationBy::EntityId(id) => db.provenance.find_by_entity_id_with_dataset(&id).await?,
        };

        let mut operations = Vec::with_capacity(records.len());
        for (record, dataset) in records {
            let mut op = Operation::try_from(record)?;
            op.dataset = dataset.into();
            operations.push(op);
        }

        Ok(operations)
    }
}

impl TryFrom<models::Operation> for Operation {
    type Error = Error;

    fn try_from(value: models::Operation) -> Result<Self, Self::Error> {
        Ok(Self {
            operation_id: value.operation_id.to_u64().ok_or(Error::InvalidData(
                "operation_id".to_string(),
                "Operation".to_string(),
                value.operation_id.to_string(),
            ))?,
            reference_id: value.reference_id.to_u64().ok_or(Error::InvalidData(
                "reference_id".to_string(),
                "Operation".to_string(),
                value.operation_id.to_string(),
            ))?,
            entity_id: value.object_id,
            action: value.action.into(),
            atom: value.atom.into(),
            dataset: DatasetDetails::default(),
        })
    }
}

impl From<models::Atom> for Atom {
    fn from(value: models::Atom) -> Self {
        match value {
            models::Atom::Empty => Atom::text(AtomTextType::Empty, "".to_string()),
            models::Atom::ScientificName { value } => {
                Atom::text(AtomTextType::ScientificName, value)
            }
            models::Atom::ActedOn { value } => Atom::text(AtomTextType::ActedOn, value),
            models::Atom::TaxonomicStatus(value) => {
                Atom::taxonomic_status(AtomTaxonomicStatusType::TaxonomicStatus, value.into())
            }
            models::Atom::NomenclaturalAct { value } => {
                Atom::text(AtomTextType::NomenclaturalAct, value)
            }
            models::Atom::SourceUrl { value } => Atom::text(AtomTextType::SourceUrl, value),
            models::Atom::Publication { value } => Atom::text(AtomTextType::Publication, value),
            models::Atom::CreatedAt(value) => Atom::datetime(AtomDateTimeType::CreatedAt, value),
            models::Atom::UpdatedAt(value) => Atom::datetime(AtomDateTimeType::UpdatedAt, value),
        }
    }
}
