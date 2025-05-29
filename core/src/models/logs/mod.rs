pub mod specimens;

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
pub use specimens::*;
use strum::Display;

use super::{Dataset, DatasetVersion};
use crate::schema;


#[derive(Clone, Debug, Display, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "schema::sql_types::OperationAction"]
pub enum Action {
    Create,
    Update,
}

pub trait LogOperation<T> {
    /// The hash of the entity id
    fn id(&self) -> &BigDecimal;
    fn entity_id(&self) -> &String;
    fn action(&self) -> &Action;
    fn atom(&self) -> &T;
}

pub trait LogOperationDataset {
    fn dataset_version(&self) -> &DatasetVersion;
    fn dataset(&self) -> &Dataset;
}
