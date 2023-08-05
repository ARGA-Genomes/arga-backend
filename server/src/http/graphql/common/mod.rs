pub mod taxonomy;
pub mod species;

pub use taxonomy::Taxonomy;
pub use species::{SpeciesCard, SpeciesPhoto};

use async_graphql::{SimpleObject, OutputType};


#[derive(SimpleObject)]
#[graphql(concrete(name = "SpeciesCardPage", params(SpeciesCard)))]
pub struct Page<T: OutputType> {
    pub records: Vec<T>,
    pub total: i64
}
