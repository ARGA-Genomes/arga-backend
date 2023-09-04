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
    pub canonical_name: Option<String>,
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

    pub vernacular_group: TaxonomicVernacularGroup,

    /// Renamed taxonomy for the same species
    pub synonyms: Vec<Taxonomy>,
    pub vernacular_names: Vec<VernacularName>,
}

impl From<models::Taxon> for Taxonomy {
    fn from(value: models::Taxon) -> Self {
        Self {
            vernacular_group: value.vernacular_group().into(),
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
    Valid,
    Undescribed,
    SpeciesInquirenda,
    Hybrid,
    Synonym,
    Invalid,
}

impl Default for TaxonomicStatus {
    fn default() -> Self { TaxonomicStatus::Invalid }
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

    None,
}

impl Default for TaxonomicVernacularGroup {
    fn default() -> Self { TaxonomicVernacularGroup::None }
}
