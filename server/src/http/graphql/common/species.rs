use async_graphql::SimpleObject;
use serde::{Serialize, Deserialize};

use crate::database::models::TaxonPhoto;

use super::Taxonomy;


#[derive(Clone, Debug, Default, Serialize, Deserialize, SimpleObject)]
pub struct SpeciesCard {
    pub taxonomy: Taxonomy,
    pub photo: Option<SpeciesPhoto>,
    pub data_summary: SpeciesDataSummary,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, SimpleObject)]
pub struct SpeciesDataSummary {
    pub whole_genomes: i64,
    pub partial_genomes: i64,
    pub organelles: i64,
    pub barcodes: i64,
    pub other: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, SimpleObject)]
pub struct SpeciesPhoto {
    url: String,
    source: Option<String>,
    publisher: Option<String>,
    license: Option<String>,
    rights_holder: Option<String>,
}

impl From<TaxonPhoto> for SpeciesPhoto {
    fn from(value: TaxonPhoto) -> Self {
        Self {
            url: value.url,
            source: value.source,
            publisher: value.publisher,
            license: value.license,
            rights_holder: value.rights_holder,
        }
    }
}
