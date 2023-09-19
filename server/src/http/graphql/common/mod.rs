pub mod taxonomy;
pub mod species;
pub mod filters;

pub use taxonomy::Taxonomy;
pub use species::{SpeciesCard, SpeciesPhoto};
pub use filters::{FilterItem, FilterType, FilterAction, convert_filters};

use async_graphql::{SimpleObject, OutputType};

use super::{species::SpecimenSummary, markers::SpeciesMarker, species::WholeGenome};


#[derive(SimpleObject)]
#[graphql(concrete(name = "SpeciesCardPage", params(SpeciesCard)))]
#[graphql(concrete(name = "SpecimenSummaryPage", params(SpecimenSummary)))]
#[graphql(concrete(name = "WholeGenomePage", params(WholeGenome)))]
#[graphql(concrete(name = "SpeciesMarkerPage", params(SpeciesMarker)))]
pub struct Page<T: OutputType> {
    pub records: Vec<T>,
    pub total: i64
}
