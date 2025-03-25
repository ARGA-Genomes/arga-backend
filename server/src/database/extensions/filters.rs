use arga_core::models::{TaxonomicStatus, TaxonomicVernacularGroup};
// use arga_core::schema::{taxa, ecology, names};
use arga_core::schema_gnl::species;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;

use super::classification_filters::{Classification, decompose_classification};

#[derive(Clone, Debug)]
pub enum FilterKind {
    Classification(Classification),
    VernacularGroup(TaxonomicVernacularGroup),
    HasData(DataType),
    // Ecology(String),
    // Ibra(String),
    // Imcra(String),
    // State(String),
    // DrainageBasin(String),

    // attribute filters
    // BushfireRecovery(BushfireRecoveryTrait),
}


#[derive(Clone, Debug)]
pub enum DataType {
    Genome,
    Locus,
    Specimen,
    Other,
}


#[derive(Clone, Debug)]
pub enum Filter {
    Include(FilterKind),
    Exclude(FilterKind),
}


pub fn filter_taxa(filters: &Vec<Filter>) -> species::BoxedQuery<Pg> {
    species::table
        .select(species::all_columns)
        .filter(species::status.eq_any(&[
            TaxonomicStatus::Accepted,
            TaxonomicStatus::Undescribed,
            TaxonomicStatus::Hybrid,
        ]))
        .filter(with_filters(&filters).unwrap())
        .into_boxed()
}

// type BoxedTaxaExpression<'a> = Box<dyn BoxableExpression<taxa::table, Pg, SqlType = Nullable<Bool>> + 'a>;
// type BoxedEcologyExpression<'a> = Box<dyn BoxableExpression<ecology::table, Pg, SqlType = Bool> + 'a>;

type BoxedExpression<'a> = Box<dyn BoxableExpression<species::table, Pg, SqlType = Bool> + 'a>;


/// Filter the species table with a global filter enum
pub fn with_filter(filter: &Filter) -> BoxedExpression {
    match filter {
        Filter::Include(kind) => match kind {
            FilterKind::Classification(classification) => with_classification(classification),
            FilterKind::VernacularGroup(group) => with_vernacular_group(group),
            FilterKind::HasData(data_type) => with_data(data_type),
            // FilterKind::Ecology(ecology) => with_ecology(ecology),
            // FilterKind::Ibra(ibra) => with_ibra(ibra),
            // FilterKind::Imcra(imcra) => with_imcra(imcra),
            // FilterKind::State(state) => with_state(state),
            // FilterKind::DrainageBasin(drainage_basin) => with_drainage_basin(drainage_basin),
            // FilterKind::BushfireRecovery(traits) => with_bushfire_recovery_trait(traits),
        },
        Filter::Exclude(kind) => match kind {
            FilterKind::Classification(classification) => without_classification(classification),
            FilterKind::VernacularGroup(group) => without_vernacular_group(group),
            FilterKind::HasData(data_type) => without_data(data_type),
            // FilterKind::Ecology(ecology) => without_ecology(ecology),
            // FilterKind::Ibra(ibra) => without_ibra(ibra),
            // FilterKind::Imcra(imcra) => without_imcra(imcra),
            // FilterKind::State(state) => without_state(state),
            // FilterKind::DrainageBasin(drainage_basin) => without_drainage_basin(drainage_basin),
            // FilterKind::BushfireRecovery(traits) => without_bushfire_recovery_trait(traits),
        },
    }
}


/// Narrow down the results from the taxa table with multiple filters
pub fn with_filters(filters: &Vec<Filter>) -> Option<BoxedExpression> {
    let mut predicates: Option<BoxedExpression> = None;
    for filter in filters {
        let predicate = with_filter(filter);

        predicates = match predicates {
            None => Some(predicate),
            Some(others) => Some(Box::new(others.and(predicate))),
        }
    }

    predicates
}


// /// Filter the taxa table with a vernacular group value
// pub fn with_drainage_basin(value: &str) -> BoxedTaxaExpression {
//     Box::new(taxa::drainage_basin.contains(vec![value]))
// }

// pub fn without_drainage_basin(value: &str) -> BoxedTaxaExpression {
//     Box::new(taxa::drainage_basin.contains(vec![value]))
// }

// /// Filter the taxa table with a vernacular group value
// pub fn with_state(value: &str) -> BoxedTaxaExpression {
//     Box::new(taxa::state.contains(vec![value]))
// }

// pub fn without_state(value: &str) -> BoxedTaxaExpression {
//     Box::new(taxa::state.contains(vec![value]))
// }

// /// Filter the taxa table with a vernacular group value
// pub fn with_imcra(value: &str) -> BoxedTaxaExpression {
//     Box::new(taxa::imcra.contains(vec![value]))
// }

// pub fn without_imcra(value: &str) -> BoxedTaxaExpression {
//     Box::new(taxa::imcra.contains(vec![value]))
// }

// /// Filter the taxa table with a vernacular group value
// pub fn with_ibra(value: &str) -> BoxedTaxaExpression {
//     Box::new(taxa::ibra.contains(vec![value]))
// }

// pub fn without_ibra(value: &str) -> BoxedTaxaExpression {
//     Box::new(taxa::ibra.contains(vec![value]))
// }

// /// Filter the taxa table with a vernacular group value
// pub fn with_ecology(value: &str) -> BoxedTaxaExpression {
//     Box::new(taxa::ecology.contains(vec![value]))
// }

// pub fn without_ecology(value: &str) -> BoxedTaxaExpression {
//     Box::new(taxa::ecology.contains(vec![value]))
// }


/// Filter the species table with a vernacular group value
pub fn with_vernacular_group(group: &TaxonomicVernacularGroup) -> BoxedExpression {
    use TaxonomicVernacularGroup as Group;
    match group {
        Group::FloweringPlants => Box::new(species::classification.retrieve_as_text("subclassis").eq("Magnoliidae")),
        Group::Animals => Box::new(species::classification.retrieve_as_text("kingdom").eq("Animalia")),
        Group::BrownAlgae => Box::new(species::classification.retrieve_as_text("classis").eq("Phaeophyceae")),
        Group::RedAlgae => Box::new(species::classification.retrieve_as_text("division").eq("Rhodophyta")),
        Group::GreenAlgae => Box::new(species::classification.retrieve_as_text("division").eq("Chlorophyta")),
        Group::Crustaceans => Box::new(species::classification.retrieve_as_text("subphylum").eq("Crustacea")),
        Group::Echinoderms => Box::new(species::classification.retrieve_as_text("phylum").eq("Echinodermata")),
        Group::FinFishes => Box::new(species::classification.retrieve_as_text("class").eq("Actinopterygii")),
        Group::CoralsAndJellyfishes => Box::new(species::classification.retrieve_as_text("phylum").eq("Cnidaria")),
        Group::Cyanobacteria => Box::new(species::classification.retrieve_as_text("division").eq("Cyanobacteria")),
        Group::Molluscs => Box::new(species::classification.retrieve_as_text("phylum").eq("Mollusca")),
        Group::SharksAndRays => Box::new(
            species::classification
                .retrieve_as_text("subclass")
                .eq("Elasmobranchii"),
        ),
        Group::Insects => Box::new(species::classification.retrieve_as_text("class").eq("Insecta")),
        Group::Fungi => Box::new(species::classification.retrieve_as_text("regnum").eq("Fungi")),
        Group::Bacteria => Box::new(species::classification.retrieve_as_text("kingdom").eq("Bacteria")),
        Group::ProtistsAndOtherUnicellularOrganisms => {
            Box::new(species::classification.retrieve_as_text("superkingdom").eq("Protista"))
        }
        Group::FrogsAndOtherAmphibians => Box::new(species::classification.retrieve_as_text("class").eq("Amphibia")),
        Group::Birds => Box::new(species::classification.retrieve_as_text("class").eq("Aves")),
        Group::Mammals => Box::new(species::classification.retrieve_as_text("class").eq("Mammalia")),
        Group::HigherPlants => Box::new(species::classification.retrieve_as_text("regnum").eq("Plantae")),
        Group::Spiders => Box::new(species::classification.retrieve_as_text("order").eq("Araneae")),
        Group::Reptiles => Box::new(species::classification.retrieve_as_text("class").eq("Reptilia")),
        Group::Mosses => Box::new(species::classification.retrieve_as_text("classis").eq("Bryopsida")),
        Group::Sponges => Box::new(species::classification.retrieve_as_text("phylum").eq("Porifera")),
        Group::Liverworts => Box::new(
            species::classification
                .retrieve_as_text("division")
                .eq("Marchantiophyta"),
        ),
        Group::Hornworts => Box::new(
            species::classification
                .retrieve_as_text("division")
                .eq("Anthocerotophyta"),
        ),
        Group::Diatoms => Box::new(
            species::classification
                .retrieve_as_text("division")
                .eq("Bacillariophyta"),
        ),
        Group::Chromists => Box::new(species::classification.retrieve_as_text("regnum").eq("Chromista")),
        Group::ConifersAndCycads => Box::new(
            species::classification
                .retrieve_as_text("ordo")
                .eq_any(vec!["Pinales", "Cycadales"]),
        ),
        Group::Ferns => Box::new(
            species::classification
                .retrieve_as_text("subclassis")
                .eq("Polypodiidae"),
        ),
    }
}


/// Filter the species table excluding a vernacular group value
pub fn without_vernacular_group(group: &TaxonomicVernacularGroup) -> BoxedExpression {
    use TaxonomicVernacularGroup as Group;
    match group {
        Group::FloweringPlants => Box::new(species::classification.retrieve_as_text("subclassis").ne("Magnoliidae")),
        Group::Animals => Box::new(species::classification.retrieve_as_text("kingdom").ne("Animalia")),
        Group::BrownAlgae => Box::new(species::classification.retrieve_as_text("classis").ne("Phaeophyceae")),
        Group::RedAlgae => Box::new(species::classification.retrieve_as_text("division").ne("Rhodophyta")),
        Group::GreenAlgae => Box::new(species::classification.retrieve_as_text("division").ne("Chlorophyta")),
        Group::Crustaceans => Box::new(species::classification.retrieve_as_text("subphylum").ne("Crustacea")),
        Group::Echinoderms => Box::new(species::classification.retrieve_as_text("phylum").ne("Echinodermata")),
        Group::FinFishes => Box::new(species::classification.retrieve_as_text("class").ne("Actinopterygii")),
        Group::CoralsAndJellyfishes => Box::new(species::classification.retrieve_as_text("phylum").ne("Cnidaria")),
        Group::Cyanobacteria => Box::new(species::classification.retrieve_as_text("division").ne("Cyanobacteria")),
        Group::Molluscs => Box::new(species::classification.retrieve_as_text("phylum").ne("Mollusca")),
        Group::SharksAndRays => Box::new(
            species::classification
                .retrieve_as_text("subclass")
                .ne("Elasmobranchii"),
        ),
        Group::Insects => Box::new(species::classification.retrieve_as_text("class").ne("Insecta")),
        Group::Fungi => Box::new(species::classification.retrieve_as_text("regnum").ne("Fungi")),
        Group::Bacteria => Box::new(species::classification.retrieve_as_text("kingdom").ne("Bacteria")),
        Group::ProtistsAndOtherUnicellularOrganisms => {
            Box::new(species::classification.retrieve_as_text("superkingdom").ne("Protista"))
        }
        Group::FrogsAndOtherAmphibians => Box::new(species::classification.retrieve_as_text("class").ne("Amphibia")),
        Group::Birds => Box::new(species::classification.retrieve_as_text("class").ne("Aves")),
        Group::Mammals => Box::new(species::classification.retrieve_as_text("class").ne("Mammalia")),
        Group::HigherPlants => Box::new(species::classification.retrieve_as_text("regnum").ne("Plantae")),
        Group::Spiders => Box::new(species::classification.retrieve_as_text("order").ne("Araneae")),
        Group::Reptiles => Box::new(species::classification.retrieve_as_text("class").ne("Reptilia")),
        Group::Mosses => Box::new(species::classification.retrieve_as_text("classis").ne("Bryopsida")),
        Group::Sponges => Box::new(species::classification.retrieve_as_text("phylum").ne("Porifera")),
        Group::Liverworts => Box::new(
            species::classification
                .retrieve_as_text("division")
                .ne("Marchantiophyta"),
        ),
        Group::Hornworts => Box::new(
            species::classification
                .retrieve_as_text("division")
                .ne("Anthocerotophyta"),
        ),
        Group::Diatoms => Box::new(
            species::classification
                .retrieve_as_text("division")
                .ne("Bacillariophyta"),
        ),
        Group::Chromists => Box::new(species::classification.retrieve_as_text("regnum").ne("Chromista")),
        Group::ConifersAndCycads => Box::new(
            species::classification
                .retrieve_as_text("ordo")
                .ne("Pinales")
                .and(species::classification.retrieve_as_text("ordo").ne("Cycadales")),
        ),
        Group::Ferns => Box::new(
            species::classification
                .retrieve_as_text("subclassis")
                .ne("Polypodiidae"),
        ),
    }
}

/// Filter the taxa table that belong to the provided classification
pub fn with_classification(classification: &Classification) -> BoxedExpression {
    let (taxon_rank, value) = decompose_classification(classification);
    let taxon_rank = taxon_rank.to_string().to_lowercase();
    Box::new(species::classification.retrieve_as_text(taxon_rank).eq(value))
}

/// Filter the taxa table that belong to the provided classification
pub fn without_classification(classification: &Classification) -> BoxedExpression {
    let (taxon_rank, value) = decompose_classification(classification);
    let taxon_rank = taxon_rank.to_string().to_lowercase();
    Box::new(species::classification.retrieve_as_text(taxon_rank).ne(value))
}

// TODO: re-enable curated dataset filtering
// /// Filter the taxa table with a particular trait attribute
// pub fn with_bushfire_recovery_trait(attr: &BushfireRecoveryTrait) -> BoxedTaxaExpression {
//     use BushfireRecoveryTrait as Trait;
//     match attr {
//         Trait::VulnerableToWildfire => Box::new(taxa::traits.contains(vec!["vulnerable_wildfire"])),
//         Trait::FireDroughtInteractions => Box::new(taxa::traits.contains(vec!["Interactive effects of fire and drought"])),
//         Trait::FireDiseaseInteractions => Box::new(taxa::traits.contains(vec!["Fire-disease interactions"])),
//         Trait::HighFireSeverity => Box::new(taxa::traits.contains(vec!["High fire severity"])),
//         Trait::WeedInvasion => Box::new(taxa::traits.contains(vec!["Weed invasion"])),
//         Trait::ChangedTemperatureRegimes => Box::new(taxa::traits.contains(vec!["Elevated winter temperatures or changed temperature regimes"])),
//         Trait::FireSensitivity => Box::new(taxa::traits.contains(vec!["Fire sensitivity"])),
//         Trait::PostFireErosion => Box::new(taxa::traits.contains(vec!["Post-fire erosion"])),
//         Trait::PostFireHerbivoreImpact => Box::new(taxa::traits.contains(vec!["Post-fire herbivore impacts"])),
//         Trait::CumulativeHighRiskExposure => Box::new(taxa::traits.contains(vec!["Cumulative exposure to high risks"])),
//         Trait::OtherThreats => Box::new(taxa::traits.contains(vec!["Other plausible threats or expert-driven nominations"])),
//     }
// }

// /// Filter the taxa table excluding a particular trait attribute
// pub fn without_bushfire_recovery_trait(attr: &BushfireRecoveryTrait) -> BoxedTaxaExpression {
//     use BushfireRecoveryTrait as Trait;
//     match attr {
//         Trait::VulnerableToWildfire => Box::new(taxa::traits.contains(vec!["vulnerable_wildfire"])),
//         Trait::FireDroughtInteractions => Box::new(taxa::traits.contains(vec!["Interactive effects of fire and drought"])),
//         Trait::FireDiseaseInteractions => Box::new(taxa::traits.contains(vec!["Fire-disease interactions"])),
//         Trait::HighFireSeverity => Box::new(taxa::traits.contains(vec!["High fire severity"])),
//         Trait::WeedInvasion => Box::new(taxa::traits.contains(vec!["Weed invasion"])),
//         Trait::ChangedTemperatureRegimes => Box::new(taxa::traits.contains(vec!["Elevated winter temperatures or changed temperature regimes"])),
//         Trait::FireSensitivity => Box::new(taxa::traits.contains(vec!["Fire sensitivity"])),
//         Trait::PostFireErosion => Box::new(taxa::traits.contains(vec!["Post-fire erosion"])),
//         Trait::PostFireHerbivoreImpact => Box::new(taxa::traits.contains(vec!["Post-fire herbivore impacts"])),
//         Trait::CumulativeHighRiskExposure => Box::new(taxa::traits.contains(vec!["Cumulative exposure to high risks"])),
//         Trait::OtherThreats => Box::new(taxa::traits.contains(vec!["Other plausible threats or expert-driven nominations"])),
//     }
// }


/// Filter the species table to records that have a specific type of associated data
pub fn with_data(data_type: &DataType) -> BoxedExpression {
    match data_type {
        DataType::Genome => Box::new(species::genomes.gt(0)),
        DataType::Locus => Box::new(species::loci.gt(0)),
        DataType::Specimen => Box::new(species::specimens.gt(0)),
        DataType::Other => Box::new(species::other.gt(0)),
    }
}


/// Filter the species table to records that dont have a specific type of associated data
pub fn without_data(data_type: &DataType) -> BoxedExpression {
    match data_type {
        DataType::Genome => Box::new(species::genomes.eq(0)),
        DataType::Locus => Box::new(species::loci.eq(0)),
        DataType::Specimen => Box::new(species::specimens.eq(0)),
        DataType::Other => Box::new(species::other.eq(0)),
    }
}
