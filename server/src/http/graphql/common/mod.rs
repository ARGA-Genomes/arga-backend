pub mod attributes;
pub mod datasets;
pub mod filters;
pub mod operation_logs;
pub mod search;
pub mod species;
pub mod taxonomy;
pub mod whole_genomes;

use async_graphql::{OutputType, SimpleObject};
pub use datasets::{DatasetDetails, DatasetVersion};
pub use filters::{
    FilterAction,
    FilterItem,
    FilterType,
    WholeGenomeFilterItem,
    WholeGenomeFilterType,
    convert_filters,
    convert_whole_genome_filters,
};
pub use species::{SpeciesCard, SpeciesDataSummary, SpeciesPhoto};
pub use taxonomy::{NameDetails, Taxonomy};

use super::markers::SpeciesMarker;
use super::species::{GenomicComponent, SpecimenSummary, WholeGenome};


#[derive(SimpleObject)]
#[graphql(concrete(name = "SpeciesCardPage", params(SpeciesCard)))]
#[graphql(concrete(name = "SpecimenSummaryPage", params(SpecimenSummary)))]
#[graphql(concrete(name = "WholeGenomePage", params(WholeGenome)))]
#[graphql(concrete(name = "SpeciesMarkerPage", params(SpeciesMarker)))]
#[graphql(concrete(name = "GenomicComponentPage", params(GenomicComponent)))]
pub struct Page<T: OutputType> {
    pub records: Vec<T>,
    pub total: i64,
}
