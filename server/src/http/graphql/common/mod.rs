pub mod agents;
pub mod attributes;
pub mod data_products;
pub mod datasets;
pub mod extractions;
pub mod filters;
pub mod operation_logs;
pub mod publications;
pub mod search;
pub mod sequences;
pub mod species;
pub mod specimens;
pub mod subsamples;
pub mod taxonomy;
pub mod whole_genomes;

pub use agents::Agent;
use async_graphql::{OutputType, SimpleObject};
pub use data_products::DataProductDetails;
pub use datasets::{DatasetDetails, DatasetVersion};
pub use extractions::DnaExtractDetails;
pub use filters::{
    FilterAction,
    FilterItem,
    FilterType,
    WholeGenomeFilterItem,
    WholeGenomeFilterType,
    convert_filters,
    convert_whole_genome_filters,
};
pub use publications::{Publication, PublicationType};
pub use sequences::{AssemblyDetails, LibraryDetails};
pub use species::{SpeciesCard, SpeciesDataSummary, SpeciesPhoto};
pub use specimens::{
    AccessionEvent,
    CollectionDetails,
    CollectionEvent,
    OrganismDetails,
    RegistrationDetails,
    TissueDetails,
};
pub use subsamples::SubsampleDetails;
pub use taxonomy::{NameDetails, Taxonomy};

use super::markers::SpeciesMarker;
use super::species::{GenomicComponent, SpecimenOptions, SpecimenSummary, WholeGenome};


#[derive(SimpleObject)]
#[graphql(concrete(name = "SpeciesCardPage", params(SpeciesCard)))]
#[graphql(concrete(name = "WholeGenomePage", params(WholeGenome)))]
#[graphql(concrete(name = "SpeciesMarkerPage", params(SpeciesMarker)))]
#[graphql(concrete(name = "GenomicComponentPage", params(GenomicComponent)))]
#[graphql(concrete(name = "AssemblyPage", params(AssemblyDetails)))]
pub struct Page<T: OutputType> {
    pub records: Vec<T>,
    pub total: i64,
}


#[derive(SimpleObject)]
#[graphql(concrete(name = "SpecimenSummaryPage", params(SpecimenSummary, SpecimenOptions)))]
pub struct FilteredPage<T: OutputType, Options: OutputType> {
    pub records: Vec<T>,
    pub total: i64,
    pub options: Options,
}
