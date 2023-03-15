pub mod providers;
pub mod filters;
pub mod search;
pub mod overview;
pub mod genus;
pub mod family;
pub mod species;
pub mod stats;


use async_graphql::SimpleObject;
use serde::{Serialize, Deserialize};


/// Taxonomic information of a species.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct Taxonomy {
    /// The species name without authors.
    pub canonical_name: Option<String>,
    /// The species name author.
    pub authorship: Option<String>,

    pub kingdom: Option<String>,
    pub phylum: Option<String>,
    pub class: Option<String>,
    pub order: Option<String>,
    pub family: Option<String>,
    pub genus: Option<String>,
}
