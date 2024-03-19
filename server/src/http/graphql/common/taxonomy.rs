use async_graphql::{SimpleObject, Enum};
use serde::{Serialize, Deserialize};

use crate::database::models;


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, Default)]
pub struct TaxonDetails {
    pub scientific_name: String,
    pub canonical_name: String,
    pub authorship: Option<String>,
    pub status: TaxonomicStatus,
    pub nomenclatural_code: String,
    pub citation: Option<String>,
    pub source: Option<String>,
    pub source_url: Option<String>,
}

impl From<models::Taxon> for TaxonDetails {
    fn from(value: models::Taxon) -> Self {
        Self {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
            authorship: value.authorship,
            status: value.status.into(),
            nomenclatural_code: value.nomenclatural_code,
            citation: value.citation,
            source: None,
            source_url: None,
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
    /// The authors of the scientific name
    pub authorship: Option<String>,
    /// The taxonomic status of the species
    pub status: TaxonomicStatus,

    pub rank: TaxonomicRank,
    pub nomenclatural_code: String,
    pub citation: Option<String>,
    pub source: Option<String>,
    pub source_url: Option<String>,

    pub vernacular_group: Option<TaxonomicVernacularGroup>,

    /// Renamed taxonomy for the same species
    pub synonyms: Vec<Taxonomy>,
}

impl From<models::Taxon> for Taxonomy {
    fn from(value: models::Taxon) -> Self {
        Self {
            // vernacular_group: value.vernacular_group().map(|v| v.into()),
            vernacular_group: None,
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
            authorship: value.authorship,
            status: value.status.into(),
            synonyms: vec![],
            rank: value.rank.into(),
            nomenclatural_code: value.nomenclatural_code,
            citation: value.citation,
            source: None,
            source_url: None,
        }
    }
}

impl From<models::Species> for Taxonomy {
    fn from(value: models::Species) -> Self {
        Self {
            vernacular_group: value.vernacular_group().map(|v| v.into()),
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
            authorship: value.authorship,
            status: value.status.into(),
            synonyms: vec![],
            rank: value.rank.into(),
            nomenclatural_code: "".to_string(),
            citation: None,
            source: None,
            source_url: None,
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
    Informal,
    Placeholder,

    Basionym,
    NomenclaturalSynonym,
    TaxonomicSynonym,
    ReplacedSynonym,

    OrthographicVariant,
    Misapplied,
    Excluded,
    AlternativeName,

    ProParteMisapplied,
    ProParteTaxonomicSynonym,

    DoubtfulMisapplied,
    DoubtfulTaxonomicSynonym,
    DoubtfulProParteMisapplied,
    DoubtfulProParteTaxonomicSynonym,
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


#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "models::TaxonomicRank")]
pub enum TaxonomicRank {
    Domain,
    Superkingdom,
    Kingdom,
    Subkingdom,
    Phylum,
    Subphylum,
    Superclass,
    Class,
    Subclass,
    Superorder,
    Order,
    Suborder,
    Hyporder,
    Minorder,
    Superfamily,
    Family,
    Subfamily,
    Supertribe,
    Tribe,
    Subtribe,
    Genus,
    Subgenus,
    Species,
    Subspecies,

    Unranked,
    HigherTaxon,

    AggregateGenera,
    AggregateSpecies,
    Cohort,
    Subcohort,
    Division,
    IncertaeSedis,
    Infraclass,
    Infraorder,
    Infragenus,
    Section,
    Subdivision,

    Regnum,
    Familia,
    Classis,
    Ordo,
    Varietas,
    Forma,
    Subforma,
    Subclassis,
    Superordo,
    Sectio,
    Subsectio,
    Nothovarietas,
    Subvarietas,
    Series,
    Subseries,
    Superspecies,
    Infraspecies,
    Subfamilia,
    Subordo,
    Regio,
    SpecialForm,
}

impl Default for TaxonomicRank {
    fn default() -> Self { TaxonomicRank::Unranked }
}
