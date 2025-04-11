use async_graphql::{Enum, SimpleObject};
use serde::{Deserialize, Serialize};

use super::Taxonomy;
use crate::database::models::TaxonPhoto;


#[derive(Clone, Debug, Default, Serialize, Deserialize, SimpleObject)]
pub struct SpeciesCard {
    pub taxonomy: Taxonomy,
    pub photo: Option<SpeciesPhoto>,
    pub data_summary: SpeciesDataSummary,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, SimpleObject)]
pub struct SpeciesDataSummary {
    pub genomes: i64,
    pub loci: i64,
    pub specimens: i64,
    pub other: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, SimpleObject)]
pub struct SpeciesPhoto {
    url: String,
    source: Option<String>,
    publisher: Option<String>,
    license: Option<String>,
    rights_holder: Option<String>,
    priority: i32,
}

impl From<TaxonPhoto> for SpeciesPhoto {
    fn from(value: TaxonPhoto) -> Self {
        Self {
            url: value.url,
            source: value.source,
            publisher: value.publisher,
            license: value.license,
            rights_holder: value.rights_holder,
            priority: value.priority,
        }
    }
}


#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "crate::database::extensions::filters::DataType")]
pub enum DataType {
    Genome,
    Locus,
    Specimen,
    Other,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "crate::database::extensions::species_filters::SpeciesSort")]
pub enum SpeciesSort {
    ScientificName,
    CanonicalName,
    ClassificationHierarchy,
    Genomes,
    Loci,
    Specimens,
    Other,
    TotalGenomic,
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "crate::database::extensions::species_filters::SortDirection")]
pub enum SortDirection {
    Asc,
    Desc,
}
