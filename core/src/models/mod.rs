pub mod operation_logs;

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use diesel::{AsChangeset, Associations, Identifiable, Insertable, Queryable, Selectable};
pub use operation_logs::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{schema, schema_gnl};

pub const ACCEPTED_NAMES: [TaxonomicStatus; 6] = [
    TaxonomicStatus::Accepted,
    TaxonomicStatus::Undescribed,
    TaxonomicStatus::SpeciesInquirenda,
    TaxonomicStatus::ManuscriptName,
    TaxonomicStatus::Hybrid,
    TaxonomicStatus::Informal,
];

pub const SPECIES_RANKS: [TaxonomicRank; 7] = [
    TaxonomicRank::Species,
    TaxonomicRank::Subspecies,
    TaxonomicRank::AggregateSpecies,
    TaxonomicRank::Infraspecies,
    TaxonomicRank::Varietas,
    TaxonomicRank::Variety,
    TaxonomicRank::Subvariety,
];

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "schema::sql_types::AccessRightsStatus"]
pub enum AccessRightsStatus {
    #[default]
    Open,
    Restricted,
    Conditional,
    Variable,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "schema::sql_types::DataReuseStatus"]
pub enum DataReuseStatus {
    #[default]
    Limited,
    Unlimited,
    None,
    Variable,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "schema::sql_types::SourceContentType"]
pub enum SourceContentType {
    #[default]
    TaxonomicBackbone,
    EcologicalTraits,
    GenomicData,
    Specimens,
    NongenomicData,
    MorphologicalTraits,
    BiochemicalTraits,
    MixedDatatypes,
    FunctionalTraits,
    Ethnobiology,
}

#[derive(Queryable, Insertable, Debug, Clone, Default, Serialize, Deserialize)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = schema::sources)]
pub struct Source {
    pub id: Uuid,
    pub name: String,
    pub author: String,
    pub rights_holder: String,
    pub access_rights: String,
    pub license: String,
    pub reuse_pill: Option<DataReuseStatus>,
    pub access_pill: Option<AccessRightsStatus>,
    pub content_type: Option<SourceContentType>,
    pub lists_id: Option<String>,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Default, Serialize, Deserialize)]
#[diesel(table_name = schema::datasets)]
pub struct Dataset {
    pub id: Uuid,
    pub source_id: Uuid,
    pub global_id: String,
    pub name: String,
    pub short_name: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub citation: Option<String>,
    pub license: Option<String>,
    pub rights_holder: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub reuse_pill: Option<DataReuseStatus>,
    pub access_pill: Option<AccessRightsStatus>,
    pub publication_year: Option<i16>,
    pub content_type: Option<SourceContentType>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Insertable, Associations, Deserialize)]
#[diesel(belongs_to(Dataset))]
#[diesel(table_name = schema::dataset_versions)]
pub struct DatasetVersion {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub imported_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "schema::sql_types::TaxonomicStatus"]
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

impl std::fmt::Display for TaxonomicStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TaxonomicStatus::Accepted => "Accepted",
            TaxonomicStatus::Undescribed => "Undescribed",
            TaxonomicStatus::SpeciesInquirenda => "SpeciesInquirenda",
            TaxonomicStatus::TaxonInquirendum => "TaxonInquirendum",
            TaxonomicStatus::ManuscriptName => "ManuscriptName",
            TaxonomicStatus::Hybrid => "Hybrid",
            TaxonomicStatus::Synonym => "Synonym",
            TaxonomicStatus::Homonym => "Homonym",
            TaxonomicStatus::Unaccepted => "Unaccepted",
            TaxonomicStatus::Informal => "Informal",
            TaxonomicStatus::Placeholder => "Placeholder",
            TaxonomicStatus::Basionym => "Basionym",
            TaxonomicStatus::NomenclaturalSynonym => "NomenclaturalSynonym",
            TaxonomicStatus::TaxonomicSynonym => "TaxonomicSynonym",
            TaxonomicStatus::ReplacedSynonym => "ReplacedSynonym",
            TaxonomicStatus::Misspelled => "Misspelled",
            TaxonomicStatus::OrthographicVariant => "OrthographicVariant",
            TaxonomicStatus::Misapplied => "Misapplied",
            TaxonomicStatus::Excluded => "Excluded",
            TaxonomicStatus::AlternativeName => "AlternativeName",
            TaxonomicStatus::ProParteMisapplied => "ProParteMisapplied",
            TaxonomicStatus::ProParteTaxonomicSynonym => "ProParteTaxonomicSynonym",
            TaxonomicStatus::DoubtfulMisapplied => "DoubtfulMisapplied",
            TaxonomicStatus::DoubtfulTaxonomicSynonym => "DoubtfulTaxonomicSynonym",
            TaxonomicStatus::DoubtfulProParteMisapplied => "DoubtfulProParteMisapplied",
            TaxonomicStatus::DoubtfulProParteTaxonomicSynonym => "DoubtfulProParteTaxonomicSynonym",
            TaxonomicStatus::Unassessed => "Unassessed",
            TaxonomicStatus::Unavailable => "Unavailable",
            TaxonomicStatus::Uncertain => "Uncertain",
            TaxonomicStatus::UnjustifiedEmendation => "UnjustifiedEmendation",
            TaxonomicStatus::NomenDubium => "NomenDubium",
            TaxonomicStatus::NomenNudum => "NomenNudum",
            TaxonomicStatus::NomenOblitum => "NomenOblitum",
            TaxonomicStatus::InterimUnpublished => "InterimUnpublished",
            TaxonomicStatus::IncorrectGrammaticalAgreementOfSpecificEpithet => {
                "IncorrectGrammaticalAgreementOfSpecificEpithet"
            }
            TaxonomicStatus::SupersededCombination => "SupersededCombination",
            TaxonomicStatus::SupersededRank => "SupersededRank",
        };

        write!(f, "{}", s)
    }
}


// TODO: this would be better as a derive macro so that we dont miss string
// to enum conversions, or use serde enum deserialization
impl From<String> for TaxonomicStatus {
    fn from(value: String) -> Self {
        use TaxonomicStatus::*;

        match value.as_str() {
            "Accepted" => Accepted,
            "Undescribed" => Undescribed,
            "SpeciesInquirenda" => SpeciesInquirenda,
            "TaxonInquirendum" => TaxonInquirendum,
            "ManuscriptName" => ManuscriptName,
            "Hybrid" => Hybrid,
            "Synonym" => Synonym,
            "Homonym" => Homonym,
            "Unaccepted" => Unaccepted,
            "Informal" => Informal,
            "Placeholder" => Placeholder,
            "Basionym" => Basionym,
            "NomenclaturalSynonym" => NomenclaturalSynonym,
            "TaxonomicSynonym" => TaxonomicSynonym,
            "ReplacedSynonym" => ReplacedSynonym,
            "Misspelled" => Misspelled,
            "OrthographicVariant" => OrthographicVariant,
            "Misapplied" => Misapplied,
            "Excluded" => Excluded,
            "AlternativeName" => AlternativeName,
            "ProParteMisapplied" => ProParteMisapplied,
            "ProParteTaxonomicSynonym" => ProParteTaxonomicSynonym,
            "DoubtfulMisapplied" => DoubtfulMisapplied,
            "DoubtfulTaxonomicSynonym" => DoubtfulTaxonomicSynonym,
            "DoubtfulProParteMisapplied" => DoubtfulProParteMisapplied,
            "DoubtfulProParteTaxonomicSynonym" => DoubtfulProParteTaxonomicSynonym,
            "Unassessed" => Unassessed,
            "Unavailable" => Unavailable,
            "Uncertain" => Uncertain,
            "UnjustifiedEmendation" => UnjustifiedEmendation,
            "NomenDubium" => NomenDubium,
            "NomenNudum" => NomenNudum,
            "NomenOblitum" => NomenOblitum,
            "InterimUnpublished" => InterimUnpublished,
            "IncorrectGrammaticalAgreementOfSpecificEpithet" => IncorrectGrammaticalAgreementOfSpecificEpithet,
            "SupersededCombination" => SupersededCombination,
            "SupersededRank" => SupersededRank,
            _ => Unaccepted,
        }
    }
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "schema::sql_types::TaxonomicRank"]
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
        Self::Unranked
    }
}

impl std::fmt::Display for TaxonomicRank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TaxonomicRank::Domain => "Domain",
            TaxonomicRank::Superkingdom => "Superkingdom",
            TaxonomicRank::Kingdom => "Kingdom",
            TaxonomicRank::Subkingdom => "Subkingdom",
            TaxonomicRank::Infrakingdom => "Infrakingdom",
            TaxonomicRank::Superphylum => "Superphylum",
            TaxonomicRank::Phylum => "Phylum",
            TaxonomicRank::Subphylum => "Subphylum",
            TaxonomicRank::Infraphylum => "Infraphylum",
            TaxonomicRank::Parvphylum => "Parvphylum",
            TaxonomicRank::Gigaclass => "Gigaclass",
            TaxonomicRank::Megaclass => "Megaclass",
            TaxonomicRank::Superclass => "Superclass",
            TaxonomicRank::Class => "Class",
            TaxonomicRank::Subclass => "Subclass",
            TaxonomicRank::Infraclass => "Infraclass",
            TaxonomicRank::Subterclass => "Subterclass",
            TaxonomicRank::Superorder => "Superorder",
            TaxonomicRank::Order => "Order",
            TaxonomicRank::Hyporder => "Hyporder",
            TaxonomicRank::Minorder => "Minorder",
            TaxonomicRank::Suborder => "Suborder",
            TaxonomicRank::Infraorder => "Infraorder",
            TaxonomicRank::Parvorder => "Parvorder",
            TaxonomicRank::Epifamily => "Epifamily",
            TaxonomicRank::Superfamily => "Superfamily",
            TaxonomicRank::Family => "Family",
            TaxonomicRank::Subfamily => "Subfamily",
            TaxonomicRank::Supertribe => "Supertribe",
            TaxonomicRank::Tribe => "Tribe",
            TaxonomicRank::Subtribe => "Subtribe",
            TaxonomicRank::Genus => "Genus",
            TaxonomicRank::Subgenus => "Subgenus",
            TaxonomicRank::Species => "Species",
            TaxonomicRank::Subspecies => "Subspecies",
            TaxonomicRank::Variety => "Variety",
            TaxonomicRank::Subvariety => "Subvariety",
            TaxonomicRank::Natio => "Natio",
            TaxonomicRank::Mutatio => "Mutatio",
            TaxonomicRank::Unranked => "Unranked",
            TaxonomicRank::HigherTaxon => "Higher Taxon",
            TaxonomicRank::AggregateGenera => "Aggregate Genera",
            TaxonomicRank::AggregateSpecies => "Aggregate Species",
            TaxonomicRank::Supercohort => "Supercohort",
            TaxonomicRank::Cohort => "Cohort",
            TaxonomicRank::Subcohort => "Subcohort",
            TaxonomicRank::Division => "Division",
            TaxonomicRank::IncertaeSedis => "Incertae Sedis",
            TaxonomicRank::Infragenus => "Infragenus",
            TaxonomicRank::Section => "Section",
            TaxonomicRank::Subsection => "Subsection",
            TaxonomicRank::Subdivision => "Subdivision",
            TaxonomicRank::Regnum => "Regnum",
            TaxonomicRank::Familia => "Familia",
            TaxonomicRank::Classis => "Classis",
            TaxonomicRank::Ordo => "Ordo",
            TaxonomicRank::Varietas => "Varietas",
            TaxonomicRank::Forma => "Forma",
            TaxonomicRank::Subforma => "Subforma",
            TaxonomicRank::Subclassis => "Subclassis",
            TaxonomicRank::Superordo => "Superordo",
            TaxonomicRank::Sectio => "Sectio",
            TaxonomicRank::Subsectio => "Subsectio",
            TaxonomicRank::Nothovarietas => "Nothovarietas",
            TaxonomicRank::Subvarietas => "Subvarietas",
            TaxonomicRank::Series => "Series",
            TaxonomicRank::Subseries => "Subseries",
            TaxonomicRank::Superspecies => "Superspecies",
            TaxonomicRank::Infraspecies => "Infraspecies",
            TaxonomicRank::Subfamilia => "Subfamilia",
            TaxonomicRank::Subordo => "Subordo",
            TaxonomicRank::Regio => "Regio",
            TaxonomicRank::SpecialForm => "Special Form",
            TaxonomicRank::Pathovar => "Pathovar",
            TaxonomicRank::Serovar => "Serovar",
            TaxonomicRank::Biovar => "Biovar",
        };

        write!(f, "{}", s)
    }
}

#[derive(Clone, Queryable, Insertable, Associations, Debug, Serialize, Deserialize)]
#[diesel(belongs_to(Taxon, foreign_key = parent_id))]
#[diesel(table_name = schema_gnl::taxa_dag)]
pub struct TaxonTreeNode {
    pub taxon_id: Uuid,
    pub taxon_scientific_name: String,
    pub taxon_canonical_name: String,
    pub id: Uuid,
    pub parent_id: Option<Uuid>,

    pub rank: TaxonomicRank,
    pub scientific_name: String,
    pub canonical_name: String,
    pub depth: i32,
}

#[derive(Identifiable, Queryable, Selectable, Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = schema::taxa)]
pub struct Taxon {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub parent_id: Option<Uuid>,

    pub status: TaxonomicStatus,
    pub rank: TaxonomicRank,

    pub scientific_name: String,
    pub canonical_name: String,
    pub authorship: Option<String>,

    pub nomenclatural_code: String,
    pub citation: Option<String>,
    pub vernacular_names: Option<Vec<Option<String>>>,
    pub description: Option<String>,
    pub remarks: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub entity_id: Option<String>,
}

#[derive(Queryable, Selectable, Debug, Deserialize, Clone)]
#[diesel(table_name = schema::taxa)]
pub struct TaxonWithDataset {
    #[diesel(embed)]
    pub taxon: Taxon,
    #[diesel(embed)]
    pub dataset: Dataset,
}

#[derive(Identifiable, Insertable, Selectable, Queryable, Associations, Debug, Clone)]
#[diesel(belongs_to(Taxon))]
#[diesel(belongs_to(Name))]
#[diesel(table_name = schema::taxon_names)]
#[diesel(primary_key(taxon_id, name_id))]
pub struct TaxonName {
    pub taxon_id: Uuid,
    pub name_id: Uuid,
}

#[derive(Queryable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema_gnl::species)]
pub struct Species {
    pub id: Uuid,
    pub scientific_name: String,
    pub canonical_name: String,
    pub authorship: Option<String>,
    pub dataset_id: Uuid,
    pub status: TaxonomicStatus,
    pub rank: TaxonomicRank,
    pub classification: serde_json::Value,

    pub genomes: i64,
    pub loci: i64,
    pub specimens: i64,
    pub other: i64,
    pub total_genomic: i64,

    pub traits: Option<Vec<String>>,
    pub attributes: Option<serde_json::Value>,
    pub vernacular_names: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ClassificationJson {
    pub domain: Option<String>,
    pub superkingdom: Option<String>,
    pub kingdom: Option<String>,
    pub subkingdom: Option<String>,
    pub phylum: Option<String>,
    pub subphylum: Option<String>,
    pub superclass: Option<String>,
    pub class: Option<String>,
    pub subclass: Option<String>,
    pub superorder: Option<String>,
    pub order: Option<String>,
    pub suborder: Option<String>,
    pub hyporder: Option<String>,
    pub superfamily: Option<String>,
    pub family: Option<String>,
    pub subfamily: Option<String>,
    pub supertribe: Option<String>,
    pub tribe: Option<String>,
    pub subtribe: Option<String>,
    pub genus: Option<String>,
    pub subgenus: Option<String>,
    pub species: Option<String>,
    pub subspecies: Option<String>,
    pub unranked: Option<String>,
    pub higher_taxon: Option<String>,
    pub aggregate_genera: Option<String>,
    pub aggregate_species: Option<String>,
    pub cohort: Option<String>,
    pub subcohort: Option<String>,
    pub division: Option<String>,
    pub incertae_sedis: Option<String>,
    pub infraclass: Option<String>,
    pub infraorder: Option<String>,
    pub section: Option<String>,
    pub subdivision: Option<String>,
    pub regnum: Option<String>,
    pub familia: Option<String>,
    pub classis: Option<String>,
    pub ordo: Option<String>,
    pub varietas: Option<String>,
    pub forma: Option<String>,
    pub subclassis: Option<String>,
    pub superordo: Option<String>,
    pub sectio: Option<String>,
    pub nothovarietas: Option<String>,
    pub subvarietas: Option<String>,
    pub series: Option<String>,
    pub infraspecies: Option<String>,
    pub subfamilia: Option<String>,
    pub subordo: Option<String>,
    pub regio: Option<String>,
    pub special_form: Option<String>,
}

impl Species {
    pub fn vernacular_group(&self) -> Option<TaxonomicVernacularGroup> {
        use TaxonomicVernacularGroup as Group;

        let classification =
            serde_json::from_value::<ClassificationJson>(self.classification.clone()).unwrap_or_default();

        let kingdom = classification.kingdom.as_ref().map(String::as_str);
        let superkingdom = classification.superkingdom.as_ref().map(String::as_str);
        let phylum = classification.phylum.as_ref().map(String::as_str);
        let subphylum = classification.subphylum.as_ref().map(String::as_str);
        let class = classification.class.as_ref().map(String::as_str);
        let subclass = classification.subclass.as_ref().map(String::as_str);
        let order = classification.order.as_ref().map(String::as_str);

        let regnum = classification.regnum.as_ref().map(String::as_str);
        let division = classification.division.as_ref().map(String::as_str);
        let classis = classification.classis.as_ref().map(String::as_str);
        let ordo = classification.ordo.as_ref().map(String::as_str);
        let subclassis = classification.subclassis.as_ref().map(String::as_str);

        // animals
        Some(match kingdom {
            Some("Archaea") => Group::Bacteria,
            Some("Bacteria") => match division {
                //FIXME: links straight to eukaryota
                Some("Cyanobacteria") => Group::Cyanobacteria,
                _ => Group::Bacteria,
            },
            Some("Animalia") => match phylum {
                Some("Echinodermata") => Group::Echinoderms,
                Some("Cnidaria") => Group::CoralsAndJellyfishes,
                Some("Mollusca") => Group::Molluscs,
                Some("Porifera") => Group::Sponges,
                Some("Arthropoda") => match (subphylum, class) {
                    (Some("Crustacea"), _) => Group::Crustaceans,
                    (_, Some("Insecta")) => Group::Insects,
                    (_, Some("Arachnida")) => match order {
                        Some("Araneae") => Group::Spiders, // new icon: spiders taxon = Araneae
                        _ => Group::Animals,
                    },
                    _ => Group::Animals,
                },
                Some("Chordata") => match class {
                    Some("Amphibia") => Group::FrogsAndOtherAmphibians,
                    Some("Aves") => Group::Birds,
                    Some("Reptilia") => Group::Reptiles, // new icon: reptiles taxon = Reptilia
                    Some("Mammalia") => Group::Mammals,
                    Some("Actinopterygii") => Group::FinFishes,
                    Some("Chondrichthyes") => match subclass {
                        Some("Elasmobranchii") => Group::SharksAndRays,
                        _ => Group::Animals,
                    },
                    _ => Group::Animals,
                },
                _ => Group::Animals,
            },

            // ALA system: plants
            Some("Plantae") => match phylum {
                Some("Bryophyta") => match class {
                    Some("Bryopsida") => Group::Mosses,
                    _ => Group::HigherPlants,
                },
                Some("Marchantiophyta") => Group::Liverworts,
                Some("Anthocerotophyta") => Group::Hornworts,
                Some("Charophyta") => match (order, subclass) {
                    (Some("Pinales"), _) => Group::ConifersAndCycads,
                    (Some("Cycadales"), _) => Group::ConifersAndCycads,
                    (_, Some("Polypodiidae")) => Group::Ferns,
                    (_, Some("Magnoliidae")) => Group::FloweringPlants,
                    _ => Group::HigherPlants,
                },
                _ => Group::HigherPlants,
            },

            // ALA system: chromists
            Some("Chromista") => Group::Chromists,

            // ALA system: fungi
            Some("Fungi") => Group::Fungi,

            // plants
            None => match regnum {
                Some("Plantae") => match division {
                    Some("Bryophyta") => match classis {
                        Some("Bryopsida") => Group::Mosses,
                        _ => Group::HigherPlants,
                    },
                    Some("Marchantiophyta") => Group::Liverworts,
                    Some("Anthocerotophyta") => Group::Hornworts,
                    Some("Charophyta") => match (ordo, subclassis) {
                        (Some("Pinales"), _) => Group::ConifersAndCycads,
                        (Some("Cycadales"), _) => Group::ConifersAndCycads,
                        (_, Some("Polypodiidae")) => Group::Ferns,
                        (_, Some("Magnoliidae")) => Group::FloweringPlants,
                        _ => Group::HigherPlants,
                    },
                    _ => Group::HigherPlants,
                },
                Some("Chromista") => Group::Chromists,
                Some("Fungi") => Group::Fungi,

                // protists
                None => match superkingdom {
                    Some("Protista") => Group::ProtistsAndOtherUnicellularOrganisms,
                    // algae
                    None => match division {
                        Some("Phaeophyta") => Group::BrownAlgae,
                        Some("Rhodophyta") => Group::RedAlgae,
                        Some("Chlorophyta") => Group::GreenAlgae,
                        Some("Bacillariophyta") => Group::Diatoms,
                        _ => return None,
                    },
                    _ => return None,
                },
                _ => return None,
            },
            _ => return None,
        })
    }
}

#[derive(Identifiable, Insertable, Selectable, Queryable, Associations, Debug, Clone)]
#[diesel(belongs_to(Dataset))]
#[diesel(belongs_to(Taxon))]
#[diesel(belongs_to(NamePublication, foreign_key = publication_id))]
#[diesel(table_name = schema::taxon_history)]
pub struct TaxonHistory {
    pub id: Uuid,
    pub acted_on: Uuid,
    pub taxon_id: Uuid,
    pub dataset_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub publication_id: Option<Uuid>,
    pub source_url: Option<String>,
    pub entity_id: Option<String>,
}

#[derive(Queryable, Selectable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema::name_publications)]
pub struct NamePublication {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub citation: Option<String>,
    pub published_year: Option<i32>,
    pub source_url: Option<String>,
    pub type_citation: Option<String>,
    pub record_created_at: Option<DateTime<Utc>>,
    pub record_updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "schema::sql_types::NomenclaturalActType"]
pub enum NomenclaturalActType {
    SpeciesNova,
    SubspeciesNova,
    GenusSpeciesNova,
    CombinatioNova,
    RevivedStatus,
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

#[derive(Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::nomenclatural_acts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NomenclaturalAct {
    pub id: Uuid,
    pub entity_id: String,
    pub name_id: Uuid,
    pub acted_on_id: Uuid,

    pub act: NomenclaturalActType,
    pub source_url: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub publication_id: Uuid,
}

#[derive(Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::taxonomic_acts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TaxonomicAct {
    pub id: Uuid,
    pub entity_id: String,
    pub taxon_id: Uuid,
    pub accepted_taxon_id: Option<Uuid>,
    pub source_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub data_created_at: Option<DateTime<Utc>>,
    pub data_updated_at: Option<DateTime<Utc>>,
}

#[derive(Queryable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema_gnl::undescribed_species)]
pub struct UndescribedSpecies {
    pub genus: String,
    pub genus_authority: Option<String>,
    pub names: Vec<String>,
}

#[derive(Clone, Queryable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::users)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "schema::sql_types::JobStatus"]
pub enum JobStatus {
    Pending,
    Initialized,
    Running,
    Completed,
    Failed,
    Dead,
}

#[derive(Queryable, Debug, Deserialize)]
#[diesel(table_name = schema::jobs)]
pub struct Job {
    pub id: Uuid,
    pub status: JobStatus,
    pub worker: String,
    pub payload: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Identifiable, Queryable, Insertable, Selectable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema::names)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Name {
    pub id: Uuid,
    pub scientific_name: String,
    pub canonical_name: String,
    pub authorship: Option<String>,
}

impl From<Taxon> for Name {
    fn from(value: Taxon) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
            authorship: value.authorship,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "schema::sql_types::RegionType"]
pub enum RegionType {
    Ibra,
    Imcra,
    State,
    DrainageBasin,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::regions)]
pub struct Regions {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub region_type: RegionType,
    pub values: Vec<Option<String>>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::ecology)]
pub struct Ecology {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub values: Vec<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::taxon_photos)]
pub struct TaxonPhoto {
    pub id: Uuid,
    pub taxon_id: Uuid,
    pub url: String,
    pub source: Option<String>,
    pub publisher: Option<String>,
    pub license: Option<String>,
    pub rights_holder: Option<String>,
    pub priority: i32,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::indigenous_knowledge)]
pub struct IndigenousKnowledge {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub name: String,
    pub food_use: bool,
    pub medicinal_use: bool,
    pub cultural_connection: bool,
    pub last_updated: DateTime<Utc>,
    pub source_url: Option<String>,
}

#[derive(Clone, Queryable, Selectable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::specimens)]
pub struct Specimen {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,

    pub record_id: String,
    pub material_sample_id: Option<String>,
    pub organism_id: Option<String>,

    pub institution_name: Option<String>,
    pub institution_code: Option<String>,
    pub collection_code: Option<String>,
    pub recorded_by: Option<String>,
    pub identified_by: Option<String>,
    pub identified_date: Option<String>,

    pub type_status: Option<String>,
    pub locality: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub state_province: Option<String>,
    pub county: Option<String>,
    pub municipality: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub elevation: Option<f64>,
    pub depth: Option<f64>,
    pub elevation_accuracy: Option<f64>,
    pub depth_accuracy: Option<f64>,
    pub location_source: Option<String>,

    pub details: Option<String>,
    pub remarks: Option<String>,
    pub identification_remarks: Option<String>,
    pub entity_id: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::subsamples)]
pub struct Subsample {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub specimen_id: Uuid,

    pub record_id: String,
    pub material_sample_id: Option<String>,
    pub institution_name: Option<String>,
    pub institution_code: Option<String>,
    pub type_status: Option<String>,
    pub entity_id: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::dna_extracts)]
pub struct DnaExtract {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub subsample_id: Uuid,
    pub record_id: String,
    pub entity_id: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::sequences)]
pub struct Sequence {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub dna_extract_id: Uuid,
    pub record_id: String,
    pub entity_id: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::collection_events)]
pub struct CollectionEvent {
    pub id: Uuid,
    pub entity_id: String,
    pub field_collecting_id: String,
    pub name_id: Uuid,
    pub organism_id: Uuid,
    pub specimen_id: Option<Uuid>,

    pub event_date: Option<chrono::NaiveDate>,
    pub event_time: Option<chrono::NaiveTime>,
    pub collected_by: Option<String>,
    pub collection_remarks: Option<String>,
    pub identified_by: Option<String>,
    pub identified_date: Option<chrono::NaiveDate>,
    pub identification_remarks: Option<String>,

    pub locality: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub state_province: Option<String>,
    pub county: Option<String>,
    pub municipality: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub elevation: Option<f64>,
    pub depth: Option<f64>,
    pub elevation_accuracy: Option<f64>,
    pub depth_accuracy: Option<f64>,
    pub location_source: Option<String>,

    pub preparation: Option<String>,
    pub environment_broad_scale: Option<String>,
    pub environment_local_scale: Option<String>,
    pub environment_medium: Option<String>,
    pub habitat: Option<String>,
    pub specific_host: Option<String>,
    pub individual_count: Option<String>,
    pub organism_quantity: Option<String>,
    pub organism_quantity_type: Option<String>,

    pub strain: Option<String>,
    pub isolate: Option<String>,
    pub field_notes: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::accession_events)]
pub struct AccessionEvent {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub specimen_id: Uuid,

    pub event_date: Option<String>,
    pub event_time: Option<String>,
    pub accession: String,
    pub accessioned_by: Option<String>,
    pub material_sample_id: Option<String>,

    pub institution_name: Option<String>,
    pub institution_code: Option<String>,
    pub type_status: Option<String>,
    pub entity_id: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::subsample_events)]
pub struct SubsampleEvent {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub subsample_id: Uuid,
    pub event_date: Option<String>,
    pub event_time: Option<String>,
    pub subsampled_by: Option<String>,
    pub preparation_type: Option<String>,
    pub entity_id: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::dna_extraction_events)]
pub struct DnaExtractionEvent {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub dna_extract_id: Uuid,

    pub event_date: Option<String>,
    pub event_time: Option<String>,
    pub extracted_by: Option<String>,

    pub preservation_type: Option<String>,
    pub preparation_type: Option<String>,
    pub extraction_method: Option<String>,
    pub measurement_method: Option<String>,
    pub concentration_method: Option<String>,
    pub quality: Option<String>,

    pub concentration: Option<f64>,
    pub absorbance_260_230: Option<f64>,
    pub absorbance_260_280: Option<f64>,
    pub entity_id: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::sequencing_events)]
pub struct SequencingEvent {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub sequence_id: Uuid,

    pub event_date: Option<String>,
    pub event_time: Option<String>,
    pub sequenced_by: Option<String>,
    pub material_sample_id: Option<String>,

    pub concentration: Option<f64>,
    pub amplicon_size: Option<i64>,
    pub estimated_size: Option<String>,
    pub bait_set_name: Option<String>,
    pub bait_set_reference: Option<String>,

    pub target_gene: Option<String>,
    pub dna_sequence: Option<String>,
    pub entity_id: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::sequencing_run_events)]
pub struct SequencingRunEvent {
    pub id: Uuid,
    pub sequencing_event_id: Uuid,

    pub trace_id: Option<String>,
    pub trace_name: Option<String>,
    pub trace_link: Option<String>,
    pub sequencing_date: Option<NaiveDateTime>,
    pub sequencing_center: Option<String>,
    pub sequencing_center_code: Option<String>,
    pub sequencing_method: Option<String>,
    pub target_gene: Option<String>,
    pub direction: Option<String>,
    pub pcr_primer_name_forward: Option<String>,
    pub pcr_primer_name_reverse: Option<String>,
    pub sequence_primer_forward_name: Option<String>,
    pub sequence_primer_reverse_name: Option<String>,

    pub library_protocol: Option<String>,
    pub analysis_description: Option<String>,
    pub analysis_software: Option<String>,
    pub entity_id: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::assembly_events)]
pub struct AssemblyEvent {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub sequence_id: Uuid,

    pub event_date: Option<String>,
    pub event_time: Option<String>,
    pub assembled_by: Option<String>,

    pub name: Option<String>,
    pub version_status: Option<String>,
    pub quality: Option<String>,
    pub assembly_type: Option<String>,
    pub genome_size: Option<i64>,
    pub entity_id: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::annotation_events)]
pub struct AnnotationEvent {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub sequence_id: Uuid,

    pub event_date: Option<String>,
    pub event_time: Option<String>,
    pub annotated_by: Option<String>,

    pub representation: Option<String>,
    pub release_type: Option<String>,
    pub coverage: Option<String>,
    pub replicons: Option<i64>,
    pub standard_operating_procedures: Option<String>,
    pub entity_id: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::deposition_events)]
pub struct DepositionEvent {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub sequence_id: Uuid,

    pub event_date: Option<String>,
    pub event_time: Option<String>,
    pub accession: Option<String>,
    pub submitted_by: Option<String>,
    pub material_sample_id: Option<String>,

    pub collection_name: Option<String>,
    pub collection_code: Option<String>,
    pub institution_name: Option<String>,

    pub data_type: Option<String>,
    pub excluded_from_refseq: Option<String>,
    pub asm_not_live_date: Option<String>,
    pub source_uri: Option<String>,

    pub title: Option<String>,
    pub url: Option<String>,
    pub funding_attribution: Option<String>,
    pub rights_holder: Option<String>,
    pub access_rights: Option<String>,
    pub reference: Option<String>,
    pub last_updated: Option<NaiveDate>,
    pub entity_id: Option<String>,
}

// postgres arrays allows nulls to be entered into an array
// so diesel will treat it as an array of optional numbers.
// we shorten the type here for readability
pub type IntArray = Vec<Option<i32>>;

#[derive(Clone, Queryable, Debug, Serialize, Deserialize)]
pub struct TraceFile {
    pub id: Uuid,
    pub name_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    pub metadata: serde_json::Value,

    pub peak_locations_user: Option<IntArray>,
    pub peak_locations_basecaller: Option<IntArray>,
    pub quality_values_user: Option<IntArray>,
    pub quality_values_basecaller: Option<IntArray>,
    pub sequences_user: Option<IntArray>,
    pub sequences_basecaller: Option<IntArray>,

    pub measurements_voltage: Option<IntArray>,
    pub measurements_current: Option<IntArray>,
    pub measurements_power: Option<IntArray>,
    pub measurements_temperature: Option<IntArray>,

    pub analyzed_g: Option<IntArray>,
    pub analyzed_a: Option<IntArray>,
    pub analyzed_t: Option<IntArray>,
    pub analyzed_c: Option<IntArray>,

    pub raw_g: Option<IntArray>,
    pub raw_a: Option<IntArray>,
    pub raw_t: Option<IntArray>,
    pub raw_c: Option<IntArray>,
}

#[derive(Debug, Queryable, Insertable, Default, Clone)]
#[diesel(table_name = schema::assemblies)]
pub struct Assembly {
    pub id: Uuid,
    pub name_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    pub accession: String,
    pub bioproject_id: Option<String>,
    pub biosample_id: Option<String>,
    pub material_sample_id: Option<String>,
    pub nuccore: Option<String>,
    pub refseq_category: Option<String>,
    pub specific_host: Option<String>,
    pub clone_strain: Option<String>,
    pub version_status: Option<String>,
    pub contam_screen_input: Option<String>,
    pub release_type: Option<String>,
    pub genome_rep: Option<String>,
    pub gbrs_paired_asm: Option<String>,
    pub paired_asm_comp: Option<String>,
    pub excluded_from_refseq: Option<String>,
    pub relation_to_type_material: Option<String>,
    pub asm_not_live_date: Option<String>,
    pub other_catalog_numbers: Option<String>,
    pub recorded_by: Option<String>,
    pub genetic_accession_uri: Option<String>,
    pub event_date: Option<String>,
}

#[derive(Debug, Queryable, Insertable, Default)]
#[diesel(table_name = schema::assembly_stats)]
pub struct AssemblyStats {
    pub id: Uuid,
    pub assembly_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    pub total_length: Option<i32>,
    pub spanned_gaps: Option<i32>,
    pub unspanned_gaps: Option<i32>,
    pub region_count: Option<i32>,
    pub scaffold_count: Option<i32>,
    pub scaffold_n50: Option<i32>,
    pub scaffold_l50: Option<i32>,
    pub scaffold_n75: Option<i32>,
    pub scaffold_n90: Option<i32>,
    pub contig_count: Option<i32>,
    pub contig_n50: Option<i32>,
    pub contig_l50: Option<i32>,
    pub total_gap_length: Option<i32>,
    pub molecule_count: Option<i32>,
    pub top_level_count: Option<i32>,
    pub component_count: Option<i32>,
    pub gc_perc: Option<i32>,
}

#[derive(Debug, Queryable, Insertable, Default, Clone)]
#[diesel(table_name = schema::biosamples)]
pub struct BioSample {
    pub id: Uuid,
    pub name_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    pub accession: String,
    pub sra: Option<String>,
    pub submission_date: Option<String>,
    pub publication_date: Option<String>,
    pub last_update: Option<String>,
    pub title: Option<String>,
    pub owner: Option<String>,
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Queryable, Insertable, Default)]
#[diesel(table_name = schema_gnl::markers)]
pub struct Marker {
    pub sequence_id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub dna_extract_id: Uuid,

    pub dataset_name: String,
    pub record_id: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub accession: Option<String>,
    pub sequenced_by: Option<String>,
    pub material_sample_id: Option<String>,
    pub target_gene: String,
    pub release_date: Option<String>,
}

/// Whole genomes are chromosome assemblies. For our model this requires
/// at least an annotation event so that we can determine whether it is
/// a full or partial genome based on the genome representation field.
#[derive(Debug, Queryable, Default, Clone)]
#[diesel(table_name = schema_gnl::whole_genomes)]
pub struct WholeGenome {
    pub sequence_id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub dna_extract_id: Uuid,

    pub dataset_name: String,
    pub record_id: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub accession: Option<String>,
    pub sequenced_by: Option<String>,
    pub material_sample_id: Option<String>,
    pub estimated_size: Option<String>,

    pub assembled_by: Option<String>,
    pub name: Option<String>,
    pub version_status: Option<String>,
    pub quality: Option<String>,
    pub assembly_type: Option<String>,
    pub genome_size: Option<i64>,

    pub annotated_by: Option<String>,
    pub representation: Option<String>,
    pub release_type: Option<String>,

    pub release_date: Option<String>,
    pub deposited_by: Option<String>,
    pub data_type: Option<String>,
    pub excluded_from_refseq: Option<String>,
}

#[derive(Debug, Queryable, Default, Clone)]
#[diesel(table_name = schema_gnl::genomic_components)]
pub struct GenomicComponent {
    pub sequence_id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub dna_extract_id: Uuid,

    pub dataset_name: String,
    pub record_id: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub accession: Option<String>,
    pub sequenced_by: Option<String>,
    pub material_sample_id: Option<String>,
    pub estimated_size: Option<String>,

    pub release_date: Option<String>,
    pub deposited_by: Option<String>,
    pub data_type: Option<String>,

    pub title: Option<String>,
    pub url: Option<String>,
    pub source_uri: Option<String>,
    pub funding_attribution: Option<String>,
    pub rights_holder: Option<String>,
    pub access_rights: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "schema::sql_types::AttributeCategory"]
pub enum AttributeCategory {
    BushfireRecovery,
    VenomousSpecies,
}

#[derive(Clone, Debug, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "schema::sql_types::AttributeValueType"]
pub enum AttributeValueType {
    Boolean,
    Integer,
    Decimal,
    String,
    Timestamp,
}

#[derive(Debug, Queryable, Insertable, Clone, Serialize, Deserialize)]
#[diesel(table_name = schema::name_attributes)]
pub struct NameAttribute {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub name: String,
    pub category: AttributeCategory,
    pub value_type: AttributeValueType,
    pub value_bool: Option<bool>,
    pub value_int: Option<i64>,
    pub value_decimal: Option<BigDecimal>,
    pub value_str: Option<String>,
    pub value_timestamp: Option<NaiveDateTime>,
}

#[derive(Clone)]
pub enum BushfireRecoveryTrait {
    VulnerableToWildfire,
    FireDroughtInteractions,
    FireDiseaseInteractions,
    HighFireSeverity,
    WeedInvasion,
    ChangedTemperatureRegimes,
    FireSensitivity,
    PostFireErosion,
    PostFireHerbivoreImpact,
    CumulativeHighRiskExposure,
    OtherThreats,
}

#[derive(Debug, Queryable, Default, Clone)]
pub struct TraceData {
    pub accession: Option<String>,
    pub trace_id: Option<String>,
    pub trace_name: Option<String>,
    pub trace_link: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::admin_media)]
pub struct AdminMedia {
    pub id: Uuid,
    pub name_id: Uuid,
    pub image_source: String,
    pub url: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub reference_url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub creator: Option<String>,
    pub publisher: Option<String>,
    pub license: Option<String>,
    pub rights_holder: Option<String>,
}

#[derive(Debug, Insertable, Queryable, Deserialize)]
#[diesel(table_name = schema::vernacular_names)]
pub struct VernacularName {
    pub id: Uuid,
    pub dataset_id: Uuid,
    pub name_id: Uuid,
    pub vernacular_name: String,
    pub citation: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "schema::sql_types::PublicationType"]
pub enum PublicationType {
    Book,
    BookChapter,
    JournalArticle,
    JournalVolume,
    ProceedingsPaper,
    Url,
}

#[derive(Queryable, Selectable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema::publications)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Publication {
    pub id: Uuid,
    pub entity_id: String,
    pub title: String,
    pub authors: Vec<Option<String>>,
    pub published_year: i32,
    pub published_date: Option<DateTime<Utc>>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub doi: Option<String>,
    pub source_urls: Option<Vec<Option<String>>>,
    pub publication_type: Option<PublicationType>,
    pub citation: Option<String>,

    pub record_created_at: Option<DateTime<Utc>>,
    pub record_updated_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Queryable, Selectable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema_gnl::taxa_tree_stats)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TaxonTreeStat {
    pub taxon_id: Uuid,
    pub id: Uuid,
    pub tree_depth: i32,
    pub children: i64,
    pub descendants: i64,
    pub loci: Option<BigDecimal>,
    pub genomes: Option<BigDecimal>,
    pub specimens: Option<BigDecimal>,
    pub other: Option<BigDecimal>,
    pub total_genomic: Option<BigDecimal>,
    pub species: Option<i64>,

    pub full_genomes: Option<BigDecimal>,
    pub partial_genomes: Option<BigDecimal>,
    pub complete_genomes: Option<BigDecimal>,
    pub assembly_chromosomes: Option<BigDecimal>,
    pub assembly_scaffolds: Option<BigDecimal>,
    pub assembly_contigs: Option<BigDecimal>,

    pub total_full_genomes_coverage: i64,
    pub total_partial_genomes_coverage: i64,
    pub total_complete_genomes_coverage: i64,
    pub total_assembly_chromosomes_coverage: i64,
    pub total_assembly_scaffolds_coverage: i64,
    pub total_assembly_contigs_coverage: i64,
}
