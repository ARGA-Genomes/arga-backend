use diesel::pg::Pg;
use diesel::prelude::*;

// use arga_core::schema::{taxa, ecology, names};
use arga_core::{schema_gnl::taxa_filter as taxa, models::TaxonomicStatus};
use arga_core::models::{TaxonomicVernacularGroup, BushfireRecoveryTrait};
use diesel::sql_types::{Bool, Nullable};


#[derive(Clone)]
pub enum FilterKind {
    Classification(Classification),
    VernacularGroup(TaxonomicVernacularGroup),
    HasData(DataType),
    Ecology(String),
    Ibra(String),
    Imcra(String),
    State(String),
    DrainageBasin(String),

    // attribute filters
    BushfireRecovery(BushfireRecoveryTrait),
}

#[derive(Clone)]
pub enum Classification {
    Kingdom(String),
    Phylum(String),
    Class(String),
    Order(String),
    Family(String),
    Tribe(String),
    Genus(String),
}

#[derive(Clone)]
pub enum DataType {
    Genome,
    Locus,
    Specimen,
    Other,
}


#[derive(Clone)]
pub enum Filter {
    Include(FilterKind),
    Exclude(FilterKind),
}


pub fn filter_taxa(filters: &Vec<Filter>) -> taxa::BoxedQuery<Pg> {
    taxa::table
        .select(taxa::all_columns)
        .filter(taxa::status.eq_any(&[TaxonomicStatus::Accepted, TaxonomicStatus::Undescribed, TaxonomicStatus::Hybrid]))
        .filter(with_filters(&filters).unwrap())
        .into_boxed()
}


// type BoxedTaxaExpression<'a> = Box<dyn BoxableExpression<taxa::table, Pg, SqlType = Nullable<Bool>> + 'a>;
// type BoxedEcologyExpression<'a> = Box<dyn BoxableExpression<ecology::table, Pg, SqlType = Bool> + 'a>;

type BoxedTaxaExpression<'a> = Box<dyn BoxableExpression<taxa::table, Pg, SqlType = Nullable<Bool>> + 'a>;


/// Filter the taxa table with a global filter enum
pub fn with_filter(filter: &Filter) -> BoxedTaxaExpression {
    match filter {
        Filter::Include(kind) => match kind {
            FilterKind::Classification(classification) => with_classification(classification),
            FilterKind::VernacularGroup(group) => with_vernacular_group(group),
            FilterKind::HasData(data_type) => with_data(data_type),
            FilterKind::Ecology(ecology) => with_ecology(ecology),
            FilterKind::Ibra(ibra) => with_ibra(ibra),
            FilterKind::Imcra(imcra) => with_imcra(imcra),
            FilterKind::State(state) => with_state(state),
            FilterKind::DrainageBasin(drainage_basin) => with_drainage_basin(drainage_basin),
            FilterKind::BushfireRecovery(traits) => with_bushfire_recovery_trait(traits),
        }
        Filter::Exclude(kind) => match kind {
            FilterKind::Classification(classification) => without_classification(classification),
            FilterKind::VernacularGroup(group) => without_vernacular_group(group),
            FilterKind::HasData(data_type) => without_data(data_type),
            FilterKind::Ecology(ecology) => without_ecology(ecology),
            FilterKind::Ibra(ibra) => without_ibra(ibra),
            FilterKind::Imcra(imcra) => without_imcra(imcra),
            FilterKind::State(state) => without_state(state),
            FilterKind::DrainageBasin(drainage_basin) => without_drainage_basin(drainage_basin),
            FilterKind::BushfireRecovery(traits) => without_bushfire_recovery_trait(traits),
        }
    }
}

/// Narrow down the results from the taxa table with multiple filters
pub fn with_filters(filters: &Vec<Filter>) -> Option<BoxedTaxaExpression> {
    let mut predicates: Option<BoxedTaxaExpression> = None;
    for filter in filters {
        let predicate = with_filter(filter);

        predicates = match predicates {
            None => Some(predicate),
            Some(others) => Some(Box::new(others.and(predicate))),
        }
    }

    predicates
}


/// Filter the taxa table with a vernacular group value
pub fn with_drainage_basin(value: &str) -> BoxedTaxaExpression {
    Box::new(taxa::drainage_basin.contains(vec![value]))
}

pub fn without_drainage_basin(value: &str) -> BoxedTaxaExpression {
    Box::new(taxa::drainage_basin.contains(vec![value]))
}

/// Filter the taxa table with a vernacular group value
pub fn with_state(value: &str) -> BoxedTaxaExpression {
    Box::new(taxa::state.contains(vec![value]))
}

pub fn without_state(value: &str) -> BoxedTaxaExpression {
    Box::new(taxa::state.contains(vec![value]))
}

/// Filter the taxa table with a vernacular group value
pub fn with_imcra(value: &str) -> BoxedTaxaExpression {
    Box::new(taxa::imcra.contains(vec![value]))
}

pub fn without_imcra(value: &str) -> BoxedTaxaExpression {
    Box::new(taxa::imcra.contains(vec![value]))
}

/// Filter the taxa table with a vernacular group value
pub fn with_ibra(value: &str) -> BoxedTaxaExpression {
    Box::new(taxa::ibra.contains(vec![value]))
}

pub fn without_ibra(value: &str) -> BoxedTaxaExpression {
    Box::new(taxa::ibra.contains(vec![value]))
}

/// Filter the taxa table with a vernacular group value
pub fn with_ecology(value: &str) -> BoxedTaxaExpression {
    Box::new(taxa::ecology.contains(vec![value]))
}

pub fn without_ecology(value: &str) -> BoxedTaxaExpression {
    Box::new(taxa::ecology.contains(vec![value]))
}

/// Filter the taxa table with a vernacular group value
pub fn with_vernacular_group(group: &TaxonomicVernacularGroup) -> BoxedTaxaExpression {
    use TaxonomicVernacularGroup as Group;
    match group {
        Group::FloweringPlants => Box::new(taxa::kingdom.eq("Plantae")),
        Group::Animals => Box::new(taxa::kingdom.eq("Animalia")),
        Group::BrownAlgae => Box::new(taxa::phylum.eq("Phaeophyceae")),
        Group::RedAlgae => Box::new(taxa::phylum.eq("Rhodophyta")),
        Group::GreenAlgae => Box::new(taxa::phylum.eq("Chlorophyta")),
        Group::Crustaceans => Box::new(taxa::subphylum.eq("Crustacea")),
        Group::Echinoderms => Box::new(taxa::phylum.eq("Echinodermata")),
        Group::FinFishes => Box::new(taxa::class.eq("Actinopterygii")),
        Group::CoralsAndJellyfishes => Box::new(taxa::phylum.eq("Cnidaria")),
        Group::Cyanobacteria => Box::new(taxa::phylum.eq("Cyanobacteria")),
        Group::Molluscs => Box::new(taxa::phylum.eq("Mollusca")),
        Group::SharksAndRays => Box::new(taxa::subclass.eq("Elasmobranchii")),
        Group::Insects => Box::new(taxa::class.eq("Insecta")),
        Group::Fungi => Box::new(taxa::kingdom.eq("Fungi")),
        Group::Bacteria => Box::new(taxa::kingdom.eq("Bacteria")),
        Group::ProtistsAndOtherUnicellularOrganisms => Box::new(taxa::kingdom.eq("Protozoa")),
        Group::FrogsAndOtherAmphibians => Box::new(taxa::class.eq("Amphibia")),
        Group::Birds => Box::new(taxa::class.eq("Aves")),
        Group::Mammals => Box::new(taxa::class.eq("Mammalia")),
        Group::Seaweeds => Box::new(taxa::kingdom.eq("Chromista")),
        Group::HigherPlants => Box::new(taxa::kingdom.eq("Plantae")),
    }
}

/// Filter the taxa table excluding a vernacular group value
pub fn without_vernacular_group(group: &TaxonomicVernacularGroup) -> BoxedTaxaExpression {
    use TaxonomicVernacularGroup as Group;
    match group {
        Group::FloweringPlants => Box::new(taxa::kingdom.ne("Plantae")),
        Group::Animals => Box::new(taxa::kingdom.ne("Animalia")),
        Group::BrownAlgae => Box::new(taxa::phylum.ne("Phaeophyceae")),
        Group::RedAlgae => Box::new(taxa::phylum.ne("Rhodophyta")),
        Group::GreenAlgae => Box::new(taxa::phylum.ne("Chlorophyta")),
        Group::Crustaceans => Box::new(taxa::subphylum.ne("Crustacea")),
        Group::Echinoderms => Box::new(taxa::phylum.ne("Echinodermata")),
        Group::FinFishes => Box::new(taxa::class.ne("Actinopterygii")),
        Group::CoralsAndJellyfishes => Box::new(taxa::phylum.ne("Cnidaria")),
        Group::Cyanobacteria => Box::new(taxa::phylum.ne("Cyanobacteria")),
        Group::Molluscs => Box::new(taxa::phylum.ne("Mollusca")),
        Group::SharksAndRays => Box::new(taxa::subclass.ne("Elasmobranchii")),
        Group::Insects => Box::new(taxa::class.ne("Insecta")),
        Group::Fungi => Box::new(taxa::kingdom.ne("Fungi")),
        Group::Bacteria => Box::new(taxa::kingdom.ne("Bacteria")),
        Group::ProtistsAndOtherUnicellularOrganisms => Box::new(taxa::kingdom.ne("Protozoa")),
        Group::FrogsAndOtherAmphibians => Box::new(taxa::class.ne("Amphibia")),
        Group::Birds => Box::new(taxa::class.ne("Aves")),
        Group::Mammals => Box::new(taxa::class.ne("Mammalia")),
        Group::Seaweeds => Box::new(taxa::kingdom.ne("Chromista")),
        Group::HigherPlants => Box::new(taxa::kingdom.ne("Plantae")),
    }
}

/// Filter the taxa table that belong to the provided classification
pub fn with_classification(classification: &Classification) -> BoxedTaxaExpression {
    match classification {
        Classification::Kingdom(value) => Box::new(taxa::hierarchy.contains(vec![value])),
        Classification::Phylum(value) => Box::new(taxa::hierarchy.contains(vec![value])),
        Classification::Class(value) => Box::new(taxa::hierarchy.contains(vec![value])),
        Classification::Order(value) => Box::new(taxa::hierarchy.contains(vec![value])),
        Classification::Family(value) => Box::new(taxa::hierarchy.contains(vec![value])),
        Classification::Tribe(value) => Box::new(taxa::hierarchy.contains(vec![value])),
        Classification::Genus(value) => Box::new(taxa::hierarchy.contains(vec![value])),

        // Classification::Kingdom(value) => Box::new(taxa::kingdom.eq(value)),
        // Classification::Phylum(value) => Box::new(taxa::phylum.eq(value)),
        // Classification::Class(value) => Box::new(taxa::class.eq(value)),
        // Classification::Order(value) => Box::new(taxa::order.eq(value)),
        // Classification::Family(value) => Box::new(taxa::family.eq(value)),
        // Classification::Tribe(value) => Box::new(taxa::tribe.eq(value)),
        // Classification::Genus(value) => Box::new(taxa::genus.eq(value)),
    }
}

/// Filter the taxa table that belong to the provided classification
pub fn without_classification(classification: &Classification) -> BoxedTaxaExpression {
    match classification {
        Classification::Kingdom(value) => Box::new(taxa::kingdom.ne(value)),
        Classification::Phylum(value) => Box::new(taxa::phylum.ne(value)),
        Classification::Class(value) => Box::new(taxa::class.ne(value)),
        Classification::Order(value) => Box::new(taxa::order.ne(value)),
        Classification::Family(value) => Box::new(taxa::family.ne(value)),
        Classification::Tribe(value) => Box::new(taxa::tribe.ne(value)),
        Classification::Genus(value) => Box::new(taxa::genus.ne(value)),
    }
}

/// Filter the taxa table with a particular trait attribute
pub fn with_bushfire_recovery_trait(attr: &BushfireRecoveryTrait) -> BoxedTaxaExpression {
    use BushfireRecoveryTrait as Trait;
    match attr {
        Trait::VulnerableToWildfire => Box::new(taxa::traits.contains(vec!["vulnerable_wildfire"])),
        Trait::FireDroughtInteractions => Box::new(taxa::traits.contains(vec!["Interactive effects of fire and drought"])),
        Trait::FireDiseaseInteractions => Box::new(taxa::traits.contains(vec!["Fire-disease interactions"])),
        Trait::HighFireSeverity => Box::new(taxa::traits.contains(vec!["High fire severity"])),
        Trait::WeedInvasion => Box::new(taxa::traits.contains(vec!["Weed invasion"])),
        Trait::ChangedTemperatureRegimes => Box::new(taxa::traits.contains(vec!["Elevated winter temperatures or changed temperature regimes"])),
        Trait::FireSensitivity => Box::new(taxa::traits.contains(vec!["Fire sensitivity"])),
        Trait::PostFireErosion => Box::new(taxa::traits.contains(vec!["Post-fire erosion"])),
        Trait::PostFireHerbivoreImpact => Box::new(taxa::traits.contains(vec!["Post-fire herbivore impacts"])),
        Trait::CumulativeHighRiskExposure => Box::new(taxa::traits.contains(vec!["Cumulative exposure to high risks"])),
        Trait::OtherThreats => Box::new(taxa::traits.contains(vec!["Other plausible threats or expert-driven nominations"])),
    }
}

/// Filter the taxa table excluding a particular trait attribute
pub fn without_bushfire_recovery_trait(attr: &BushfireRecoveryTrait) -> BoxedTaxaExpression {
    use BushfireRecoveryTrait as Trait;
    match attr {
        Trait::VulnerableToWildfire => Box::new(taxa::traits.contains(vec!["vulnerable_wildfire"])),
        Trait::FireDroughtInteractions => Box::new(taxa::traits.contains(vec!["Interactive effects of fire and drought"])),
        Trait::FireDiseaseInteractions => Box::new(taxa::traits.contains(vec!["Fire-disease interactions"])),
        Trait::HighFireSeverity => Box::new(taxa::traits.contains(vec!["High fire severity"])),
        Trait::WeedInvasion => Box::new(taxa::traits.contains(vec!["Weed invasion"])),
        Trait::ChangedTemperatureRegimes => Box::new(taxa::traits.contains(vec!["Elevated winter temperatures or changed temperature regimes"])),
        Trait::FireSensitivity => Box::new(taxa::traits.contains(vec!["Fire sensitivity"])),
        Trait::PostFireErosion => Box::new(taxa::traits.contains(vec!["Post-fire erosion"])),
        Trait::PostFireHerbivoreImpact => Box::new(taxa::traits.contains(vec!["Post-fire herbivore impacts"])),
        Trait::CumulativeHighRiskExposure => Box::new(taxa::traits.contains(vec!["Cumulative exposure to high risks"])),
        Trait::OtherThreats => Box::new(taxa::traits.contains(vec!["Other plausible threats or expert-driven nominations"])),
    }
}


/// Filter the taxa table to records that have a specific type of associated data
pub fn with_data(data_type: &DataType) -> BoxedTaxaExpression {
    match data_type {
        DataType::Genome => Box::new(taxa::genomes.gt(0)),
        DataType::Locus => Box::new(taxa::markers.gt(0)),
        DataType::Specimen => Box::new(taxa::specimens.gt(0)),
        DataType::Other => Box::new(taxa::other.gt(0)),
    }
}

/// Filter the taxa table to records that dont have a specific type of associated data
pub fn without_data(data_type: &DataType) -> BoxedTaxaExpression {
    match data_type {
        DataType::Genome => Box::new(taxa::genomes.eq(0)),
        DataType::Locus => Box::new(taxa::markers.eq(0)),
        DataType::Specimen => Box::new(taxa::specimens.eq(0)),
        DataType::Other => Box::new(taxa::other.eq(0)),
    }
}
