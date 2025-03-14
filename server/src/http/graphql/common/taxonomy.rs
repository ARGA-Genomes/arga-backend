use async_graphql::{Enum, SimpleObject};
use serde::{Deserialize, Serialize};

use crate::database::extensions::classification_filters::Classification;
use crate::database::models;


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, Default)]
pub struct TaxonDetails {
    pub scientific_name: String,
    pub canonical_name: String,
    pub authorship: Option<String>,
    pub status: TaxonomicStatus,
    pub rank: TaxonomicRank,
    pub nomenclatural_code: String,
    pub citation: Option<String>,
    pub source: Option<String>,
    pub source_url: Option<String>,
    pub dataset_id: uuid::Uuid,
    pub entity_id: Option<String>,
}

impl From<models::Taxon> for TaxonDetails {
    fn from(value: models::Taxon) -> Self {
        Self {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
            authorship: value.authorship,
            status: value.status.into(),
            rank: value.rank.into(),
            nomenclatural_code: value.nomenclatural_code,
            citation: value.citation,
            source: None,
            source_url: None,
            dataset_id: value.dataset_id,
            entity_id: value.entity_id,
        }
    }
}

impl From<models::TaxonWithDataset> for TaxonDetails {
    fn from(value: models::TaxonWithDataset) -> Self {
        Self {
            scientific_name: value.taxon.scientific_name,
            canonical_name: value.taxon.canonical_name,
            authorship: value.taxon.authorship,
            status: value.taxon.status.into(),
            rank: value.taxon.rank.into(),
            nomenclatural_code: value.taxon.nomenclatural_code,
            citation: value.taxon.citation,
            source: Some(value.dataset.name),
            source_url: value.dataset.url,
            dataset_id: value.taxon.dataset_id,
            entity_id: value.taxon.entity_id,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject, Default)]
pub struct NameDetails {
    pub scientific_name: String,
    pub canonical_name: String,
    pub authorship: Option<String>,
}

impl From<models::Name> for NameDetails {
    fn from(value: models::Name) -> Self {
        Self {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
            authorship: value.authorship,
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

    pub traits: Option<Vec<String>>,
    pub attributes: Option<serde_json::Value>,
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
            traits: None,
            attributes: None,
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
            traits: value.traits,
            attributes: value.attributes,
        }
    }
}

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "models::TaxonomicStatus")]
pub enum TaxonomicStatus {
    Accepted,
    Undescribed,
    SpeciesInquirenda,
    TaxonInquirendum,
    ManuscriptName,
    Hybrid,
    Synonym,
    Homonym,
    Unaccepted,
    Informal,
    Placeholder,

    Basionym,
    NomenclaturalSynonym,
    TaxonomicSynonym,
    ReplacedSynonym,

    Misspelled,
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

    Unassessed,
    Unavailable,
    Uncertain,
    UnjustifiedEmendation,

    NomenDubium,
    NomenNudum,
    NomenOblitum,

    InterimUnpublished,
    IncorrectGrammaticalAgreementOfSpecificEpithet,
    SupersededCombination,
    SupersededRank,
}

impl Default for TaxonomicStatus {
    fn default() -> Self {
        TaxonomicStatus::Unaccepted
    }
}


#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "models::NomenclaturalActType")]
pub enum NomenclaturalActType {
    SpeciesNova,
    CombinatioNova,
    RevivedStatus,
    GenusSpeciesNova,
    SubspeciesNova,
    NameUsage,
    SubgenusPlacement,

    OriginalDescription,
    Redescription,
    Demotion,
    Promotion,
    Synonymisation,
    HeterotypicSynonymy,
    HomotypicSynonymy,
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
    Spiders,
    Reptiles,
    Sponges,

    Bacteria,
    ProtistsAndOtherUnicellularOrganisms,
    FrogsAndOtherAmphibians,
    Birds,
    Mammals,
    HigherPlants,
    Mosses,
    Liverworts,
    Hornworts,
    Diatoms,
    Chromists,
    ConifersAndCycads,
    Ferns,
}


#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "models::TaxonomicRank")]
pub enum TaxonomicRank {
    Domain,

    Superkingdom,
    Kingdom,
    Subkingdom,
    Infrakingdom,

    Superphylum,
    Phylum,
    Subphylum,
    Infraphylum,
    Parvphylum,

    Gigaclass,
    Megaclass,
    Superclass,
    Class,
    Subclass,
    Infraclass,
    Subterclass,

    Superorder,
    Order,
    Hyporder,
    Minorder,
    Suborder,
    Infraorder,
    Parvorder,

    Epifamily,
    Superfamily,
    Family,
    Subfamily,

    Supertribe,
    Tribe,
    Subtribe,
    Genus,
    Subgenus,
    Infragenus,
    Species,
    Subspecies,

    Variety,
    Subvariety,

    Natio,
    Mutatio,

    Unranked,
    HigherTaxon,

    AggregateGenera,
    AggregateSpecies,
    Supercohort,
    Cohort,
    Subcohort,
    Division,
    IncertaeSedis,
    Section,
    Subsection,
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

    Pathovar,
    Serovar,
    Biovar,
}

impl Default for TaxonomicRank {
    fn default() -> Self {
        TaxonomicRank::Unranked
    }
}


impl TaxonomicRank {
    pub fn to_classification(&self, name: String) -> Classification {
        match self {
            TaxonomicRank::Domain => Classification::Domain(name),
            TaxonomicRank::Superkingdom => Classification::Superkingdom(name),
            TaxonomicRank::Kingdom => Classification::Kingdom(name),
            TaxonomicRank::Subkingdom => Classification::Subkingdom(name),
            TaxonomicRank::Infrakingdom => Classification::Infrakingdom(name),
            TaxonomicRank::Superphylum => Classification::Superphylum(name),
            TaxonomicRank::Phylum => Classification::Phylum(name),
            TaxonomicRank::Subphylum => Classification::Subphylum(name),
            TaxonomicRank::Infraphylum => Classification::Infraphylum(name),
            TaxonomicRank::Parvphylum => Classification::Parvphylum(name),
            TaxonomicRank::Gigaclass => Classification::Gigaclass(name),
            TaxonomicRank::Megaclass => Classification::Megaclass(name),
            TaxonomicRank::Superclass => Classification::Superclass(name),
            TaxonomicRank::Class => Classification::Class(name),
            TaxonomicRank::Subclass => Classification::Subclass(name),
            TaxonomicRank::Infraclass => Classification::Infraclass(name),
            TaxonomicRank::Subterclass => Classification::Subterclass(name),
            TaxonomicRank::Superorder => Classification::Superorder(name),
            TaxonomicRank::Order => Classification::Order(name),
            TaxonomicRank::Hyporder => Classification::Hyporder(name),
            TaxonomicRank::Minorder => Classification::Minorder(name),
            TaxonomicRank::Suborder => Classification::Suborder(name),
            TaxonomicRank::Infraorder => Classification::Infraorder(name),
            TaxonomicRank::Parvorder => Classification::Parvorder(name),
            TaxonomicRank::Epifamily => Classification::Epifamily(name),
            TaxonomicRank::Superfamily => Classification::Superfamily(name),
            TaxonomicRank::Family => Classification::Family(name),
            TaxonomicRank::Subfamily => Classification::Subfamily(name),
            TaxonomicRank::Supertribe => Classification::Supertribe(name),
            TaxonomicRank::Tribe => Classification::Tribe(name),
            TaxonomicRank::Subtribe => Classification::Subtribe(name),
            TaxonomicRank::Genus => Classification::Genus(name),
            TaxonomicRank::Subgenus => Classification::Subgenus(name),
            TaxonomicRank::Infragenus => Classification::Infragenus(name),
            TaxonomicRank::Species => Classification::Species(name),
            TaxonomicRank::Subspecies => Classification::Subspecies(name),
            TaxonomicRank::Variety => Classification::Variety(name),
            TaxonomicRank::Subvariety => Classification::Subvariety(name),
            TaxonomicRank::Natio => Classification::Natio(name),
            TaxonomicRank::Mutatio => Classification::Mutatio(name),
            TaxonomicRank::Unranked => Classification::Unranked(name),
            TaxonomicRank::HigherTaxon => Classification::HigherTaxon(name),
            TaxonomicRank::AggregateGenera => Classification::AggregateGenera(name),
            TaxonomicRank::AggregateSpecies => Classification::AggregateSpecies(name),
            TaxonomicRank::Supercohort => Classification::Supercohort(name),
            TaxonomicRank::Cohort => Classification::Cohort(name),
            TaxonomicRank::Subcohort => Classification::Subcohort(name),
            TaxonomicRank::Division => Classification::Division(name),
            TaxonomicRank::IncertaeSedis => Classification::IncertaeSedis(name),
            TaxonomicRank::Section => Classification::Section(name),
            TaxonomicRank::Subsection => Classification::Subsection(name),
            TaxonomicRank::Subdivision => Classification::Subdivision(name),
            TaxonomicRank::Regnum => Classification::Regnum(name),
            TaxonomicRank::Familia => Classification::Familia(name),
            TaxonomicRank::Classis => Classification::Classis(name),
            TaxonomicRank::Ordo => Classification::Ordo(name),
            TaxonomicRank::Varietas => Classification::Varietas(name),
            TaxonomicRank::Forma => Classification::Forma(name),
            TaxonomicRank::Subforma => Classification::Subforma(name),
            TaxonomicRank::Subclassis => Classification::Subclassis(name),
            TaxonomicRank::Superordo => Classification::Superordo(name),
            TaxonomicRank::Sectio => Classification::Sectio(name),
            TaxonomicRank::Subsectio => Classification::Subsectio(name),
            TaxonomicRank::Nothovarietas => Classification::Nothovarietas(name),
            TaxonomicRank::Subvarietas => Classification::Subvarietas(name),
            TaxonomicRank::Series => Classification::Series(name),
            TaxonomicRank::Subseries => Classification::Subseries(name),
            TaxonomicRank::Superspecies => Classification::Superspecies(name),
            TaxonomicRank::Infraspecies => Classification::Infraspecies(name),
            TaxonomicRank::Subfamilia => Classification::Subfamilia(name),
            TaxonomicRank::Subordo => Classification::Subordo(name),
            TaxonomicRank::Regio => Classification::Regio(name),
            TaxonomicRank::SpecialForm => Classification::SpecialForm(name),
            TaxonomicRank::Pathovar => Classification::Pathovar(name),
            TaxonomicRank::Serovar => Classification::Serovar(name),
            TaxonomicRank::Biovar => Classification::Biovar(name),
        }
    }
}


pub fn sort_taxa_priority(taxa: &mut Vec<models::TaxonWithDataset>) {
    use std::cmp::Ordering;

    taxa.sort_by(|a, b| match (a.dataset.name.as_str(), b.dataset.name.as_str()) {
        ("Atlas of Living Australia", _) => Ordering::Less,
        (_, "Atlas of Living Australia") => Ordering::Greater,
        _ => a.dataset.name.cmp(&b.dataset.name),
    });
}
