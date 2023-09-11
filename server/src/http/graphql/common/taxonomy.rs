use async_graphql::{SimpleObject, Enum};
use serde::{Serialize, Deserialize};

use crate::database::models;
use crate::database::species;


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct VernacularName {
    name: String,
    language: Option<String>,
}

impl From<species::VernacularName> for VernacularName {
    fn from(value: species::VernacularName) -> Self {
        Self {
            name: value.name,
            language: value.language,
        }
    }
}


/// Taxonomic information of a species.
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, Default)]
pub struct Taxonomy {
    /// The species scientific name
    pub scientific_name: String,
    /// The species name without authors
    pub canonical_name: String,
    /// The species name authority
    pub authority: Option<String>,
    /// The taxonomic status of the species
    pub status: TaxonomicStatus,

    pub kingdom: Option<String>,
    pub phylum: Option<String>,
    pub class: Option<String>,
    pub order: Option<String>,
    pub family: Option<String>,
    pub genus: Option<String>,

    pub vernacular_group: Option<TaxonomicVernacularGroup>,

    /// Renamed taxonomy for the same species
    pub synonyms: Vec<Taxonomy>,
    pub vernacular_names: Vec<VernacularName>,
}

impl From<models::Taxon> for Taxonomy {
    fn from(value: models::Taxon) -> Self {
        Self {
            vernacular_group: value.vernacular_group().map(|v| v.into()),
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
            authority: value.species_authority,
            status: value.status.into(),
            kingdom: value.kingdom,
            phylum: value.phylum,
            class: value.class,
            order: value.order,
            family: value.family,
            genus: value.genus,
            synonyms: vec![],
            vernacular_names: vec![],
        }
    }
}

impl From<models::FilteredTaxon> for Taxonomy {
    fn from(value: models::FilteredTaxon) -> Self {
        Self {
            vernacular_group: None,
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
            authority: value.species_authority,
            status: value.status.into(),
            kingdom: value.kingdom,
            phylum: value.phylum,
            class: value.class,
            order: value.order,
            family: value.family,
            genus: value.genus,
            synonyms: vec![],
            vernacular_names: vec![],
        }
    }
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "models::TaxonomicStatus")]
pub enum TaxonomicStatus {
    Accepted,
    Undescribed,
    SpeciesInquirenda,
    ManuscriptName,
    Hybrid,
    Synonym,
    Unaccepted,
}

impl Default for TaxonomicStatus {
    fn default() -> Self { TaxonomicStatus::Unaccepted }
}


#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "models::TaxonomicVernacularGroup")]
pub enum TaxonomicVernacularGroup {
    FloweringPlants,
    Animals,
    BrownAlgae,
    RedAlgae,
    GreenAlgae,
    Crustaceans,
    Echinoderms,
    FinFishes,
    CoralsAndJellyfishes,
    Cyanobacteria,
    Molluscs,
    SharksAndRays,
    Insects,
    Fungi,

    Bacteria,
    ProtistsAndOtherUnicellularOrganisms,
    FrogsAndOtherAmphibians,
    Birds,
    Mammals,
    Seaweeds,
    HigherPlants,
}
