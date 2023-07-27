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

    pub vernacular_group: Option<String>,

    /// Renamed taxonomy for the same species
    pub synonyms: Vec<Taxonomy>,
    pub vernacular_names: Vec<VernacularName>,
}

impl From<models::Taxon> for Taxonomy {
    fn from(value: models::Taxon) -> Self {
        Self {
            vernacular_group: derive_vernacular_group(&value),
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

#[derive(Clone, Debug, Copy, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum TaxonomicStatus {
    Valid,
    Undescribed,
    SpeciesInquirenda,
    Hybrid,
    Synonym,
    Invalid,
}

impl From<models::TaxonomicStatus> for TaxonomicStatus {
    fn from(value: models::TaxonomicStatus) -> Self {
        match value {
            models::TaxonomicStatus::Valid => TaxonomicStatus::Valid,
            models::TaxonomicStatus::Undescribed => TaxonomicStatus::Undescribed,
            models::TaxonomicStatus::SpeciesInquirenda => TaxonomicStatus::SpeciesInquirenda,
            models::TaxonomicStatus::Hybrid => TaxonomicStatus::Hybrid,
            models::TaxonomicStatus::Synonym => TaxonomicStatus::Synonym,
            models::TaxonomicStatus::Invalid => TaxonomicStatus::Invalid,
        }
    }
}

impl Default for TaxonomicStatus {
    fn default() -> Self {
        TaxonomicStatus::Invalid
    }
}


fn derive_vernacular_group(taxon: &models::Taxon) -> Option<String> {
    match taxon.kingdom.as_ref().map(|k| k.as_str()) {
        Some("Archaea") => Some("bacteria".into()),
        Some("Bacteria") => Some("bacteria".into()),
        Some("Protozoa") => Some("protists and other unicellular organisms".into()),
        Some("Fungi") => Some("mushrooms and other fungi".into()),
        Some("Animalia") => match taxon.phylum.as_ref().map(|k| k.as_str()) {
            Some("Mollusca") => Some("molluscs".into()),
            Some("Arthropoda") => match taxon.class.as_ref().map(|k| k.as_str()) {
                Some("Insecta") => Some("insects".into()),
                _ => None,
            }
            Some("Chordata") => match taxon.class.as_ref().map(|k| k.as_str()) {
                Some("Amphibia") => Some("frogs and other amphibians".into()),
                Some("Aves") => Some("birds".into()),
                Some("Mammalia") => Some("mammals".into()),
                _ => None,
            }
            _ => None,
        }
        Some("Chromista") => Some("seaweeds and other algae".into()),
        Some("Plantae") => match taxon.phylum.as_ref().map(|k| k.as_str()) {
            Some("Rhodophyta") => Some("seaweeds and other algae".into()),
            _ => Some("higher plants".into()),
        }
        _ => None,
    }
}