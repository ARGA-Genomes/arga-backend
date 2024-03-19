pub mod taxonomy;
pub mod species;
pub mod filters;
pub mod attributes;
pub mod whole_genomes;
pub mod search;

pub use taxonomy::Taxonomy;
pub use species::{SpeciesCard, SpeciesPhoto, SpeciesDataSummary};
pub use filters::{
    FilterAction,
    FilterItem,
    FilterType,
    WholeGenomeFilterItem,
    WholeGenomeFilterType,
    SearchFilterItem,
    SearchFilterType,
    convert_filters,
    convert_whole_genome_filters,
    convert_search_filters,
};

use async_graphql::{SimpleObject, OutputType};

use super::{
    species::SpecimenSummary,
    markers::SpeciesMarker,
    species::WholeGenome,
    species::GenomicComponent,
};


#[derive(SimpleObject)]
#[graphql(concrete(name = "SpeciesCardPage", params(SpeciesCard)))]
#[graphql(concrete(name = "SpecimenSummaryPage", params(SpecimenSummary)))]
#[graphql(concrete(name = "WholeGenomePage", params(WholeGenome)))]
#[graphql(concrete(name = "SpeciesMarkerPage", params(SpeciesMarker)))]
#[graphql(concrete(name = "GenomicComponentPage", params(GenomicComponent)))]
pub struct Page<T: OutputType> {
    pub records: Vec<T>,
    pub total: i64
}
