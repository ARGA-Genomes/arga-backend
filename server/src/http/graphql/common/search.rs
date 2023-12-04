use async_graphql::Enum;
use serde::{Serialize, Deserialize};


#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "arga_core::search::DataType")]
pub enum SearchDataType {
    Taxon,
    Genome,
    Locus,
    Specimen,
}
