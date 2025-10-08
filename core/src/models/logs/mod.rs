pub mod agents;
pub mod data_products;
pub mod extractions;
pub mod sequences;
pub mod specimens;
pub mod subsamples;

pub use agents::*;
use bigdecimal::BigDecimal;
pub use data_products::*;
pub use extractions::*;
pub use sequences::*;
use serde::{Deserialize, Serialize};
pub use specimens::*;
use strum::Display;
pub use subsamples::*;

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


/// Generate an entity ID hash compatible with the operation log tables.
///
/// Every entity_id in a log table is a content derived hash that can be matched on
/// based on some value. It actually generates a 64bit number but we convert it to a
/// string as the entity ids on the log tables are varchar.
///
/// This means we can search for a specific record by querying for a specific id
/// derived from a value such as `entity_id("MyRecord-1234")`, including foreign keys
/// linking to a specific entity like specimen_id.
pub fn entity_hash(value: &str) -> String {
    xxhash_rust::xxh3::xxh3_64(value.as_bytes()).to_string()
}
