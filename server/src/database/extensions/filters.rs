use diesel::pg::Pg;
use diesel::prelude::*;

use arga_core::schema::taxa;
use arga_core::models::TaxonomicVernacularGroup;
use diesel::sql_types::{Bool, Nullable};


pub enum FilterKind {
    Kingdom(String),
    Phylum(String),
    Class(String),
    Order(String),
    Family(String),
    Tribe(String),
    Genus(String),

    VernacularGroup(TaxonomicVernacularGroup)
}

pub enum Filter {
    Include(FilterKind),
    Exclude(FilterKind),
}


type BoxedTaxaExpression<'a> = Box<dyn BoxableExpression<taxa::table, Pg, SqlType = Nullable<Bool>> + 'a>;


/// Filter the taxa table with a global filter enum
pub fn with_filter(filter: &Filter) -> BoxedTaxaExpression {
    match filter {
        Filter::Include(kind) => match kind {
            FilterKind::Kingdom(value) => Box::new(taxa::kingdom.eq(value)),
            FilterKind::Phylum(value) => Box::new(taxa::phylum.eq(value)),
            FilterKind::Class(value) => Box::new(taxa::class.eq(value)),
            FilterKind::Order(value) => Box::new(taxa::order.eq(value)),
            FilterKind::Family(value) => Box::new(taxa::family.eq(value)),
            FilterKind::Tribe(value) => Box::new(taxa::tribe.eq(value)),
            FilterKind::Genus(value) => Box::new(taxa::genus.eq(value)),

            FilterKind::VernacularGroup(group) => with_vernacular_group(group),
        }
        Filter::Exclude(kind) => match kind {
            FilterKind::Kingdom(value) => Box::new(taxa::kingdom.ne(value)),
            FilterKind::Phylum(value) => Box::new(taxa::phylum.ne(value)),
            FilterKind::Class(value) => Box::new(taxa::class.ne(value)),
            FilterKind::Order(value) => Box::new(taxa::order.ne(value)),
            FilterKind::Family(value) => Box::new(taxa::family.ne(value)),
            FilterKind::Tribe(value) => Box::new(taxa::tribe.ne(value)),
            FilterKind::Genus(value) => Box::new(taxa::genus.ne(value)),

            FilterKind::VernacularGroup(group) => without_vernacular_group(group),
        }
    }
}


/// Filter the taxa table with a vernacular group value
pub fn with_vernacular_group(group: &TaxonomicVernacularGroup) -> BoxedTaxaExpression {
    match group {
        TaxonomicVernacularGroup::FloweringPlants => Box::new(taxa::kingdom.eq("Plantae")),
        TaxonomicVernacularGroup::Animals => Box::new(taxa::kingdom.eq("Animalia")),
        TaxonomicVernacularGroup::BrownAlgae => Box::new(taxa::phylum.eq("Phaeophyceae")),
        TaxonomicVernacularGroup::RedAlgae => Box::new(taxa::phylum.eq("Rhodophyta")),
        TaxonomicVernacularGroup::GreenAlgae => Box::new(taxa::phylum.eq("Chlorophyta")),
        TaxonomicVernacularGroup::Crustaceans => Box::new(taxa::subphylum.eq("Crustacea")),
        TaxonomicVernacularGroup::Echinoderms => Box::new(taxa::phylum.eq("Echinodermata")),
        TaxonomicVernacularGroup::FinFishes => Box::new(taxa::class.eq("Actinopterygii")),
        TaxonomicVernacularGroup::CoralsAndJellyfishes => Box::new(taxa::phylum.eq("Cnidaria")),
        TaxonomicVernacularGroup::Cyanobacteria => Box::new(taxa::phylum.eq("Cyanobacteria")),
        TaxonomicVernacularGroup::Molluscs => Box::new(taxa::phylum.eq("Mollusca")),
        TaxonomicVernacularGroup::SharksAndRays => Box::new(taxa::subclass.eq("Elasmobranchii")),
        TaxonomicVernacularGroup::Insects => Box::new(taxa::class.eq("Insecta")),
        TaxonomicVernacularGroup::Fungi => Box::new(taxa::kingdom.eq("Fungi")),
        TaxonomicVernacularGroup::Bacteria => Box::new(taxa::kingdom.eq("Bacteria")),
        TaxonomicVernacularGroup::ProtistsAndOtherUnicellularOrganisms => Box::new(taxa::kingdom.eq("Protozoa")),
        TaxonomicVernacularGroup::FrogsAndOtherAmphibians => Box::new(taxa::class.eq("Amphibia")),
        TaxonomicVernacularGroup::Birds => Box::new(taxa::class.eq("Aves")),
        TaxonomicVernacularGroup::Mammals => Box::new(taxa::class.eq("Mammalia")),
        TaxonomicVernacularGroup::Seaweeds => Box::new(taxa::kingdom.eq("Chromista")),
        TaxonomicVernacularGroup::HigherPlants => Box::new(taxa::kingdom.eq("Plantae")),
    }
}

/// Filter the taxa table excluding a vernacular group value
pub fn without_vernacular_group(group: &TaxonomicVernacularGroup) -> BoxedTaxaExpression {
    match group {
        TaxonomicVernacularGroup::FloweringPlants => Box::new(taxa::kingdom.ne("Plantae")),
        TaxonomicVernacularGroup::Animals => Box::new(taxa::kingdom.ne("Animalia")),
        TaxonomicVernacularGroup::BrownAlgae => Box::new(taxa::phylum.ne("Phaeophyceae")),
        TaxonomicVernacularGroup::RedAlgae => Box::new(taxa::phylum.ne("Rhodophyta")),
        TaxonomicVernacularGroup::GreenAlgae => Box::new(taxa::phylum.ne("Chlorophyta")),
        TaxonomicVernacularGroup::Crustaceans => Box::new(taxa::subphylum.ne("Crustacea")),
        TaxonomicVernacularGroup::Echinoderms => Box::new(taxa::phylum.ne("Echinodermata")),
        TaxonomicVernacularGroup::FinFishes => Box::new(taxa::class.ne("Actinopterygii")),
        TaxonomicVernacularGroup::CoralsAndJellyfishes => Box::new(taxa::phylum.ne("Cnidaria")),
        TaxonomicVernacularGroup::Cyanobacteria => Box::new(taxa::phylum.ne("Cyanobacteria")),
        TaxonomicVernacularGroup::Molluscs => Box::new(taxa::phylum.ne("Mollusca")),
        TaxonomicVernacularGroup::SharksAndRays => Box::new(taxa::subclass.ne("Elasmobranchii")),
        TaxonomicVernacularGroup::Insects => Box::new(taxa::class.ne("Insecta")),
        TaxonomicVernacularGroup::Fungi => Box::new(taxa::kingdom.ne("Fungi")),
        TaxonomicVernacularGroup::Bacteria => Box::new(taxa::kingdom.ne("Bacteria")),
        TaxonomicVernacularGroup::ProtistsAndOtherUnicellularOrganisms => Box::new(taxa::kingdom.ne("Protozoa")),
        TaxonomicVernacularGroup::FrogsAndOtherAmphibians => Box::new(taxa::class.ne("Amphibia")),
        TaxonomicVernacularGroup::Birds => Box::new(taxa::class.ne("Aves")),
        TaxonomicVernacularGroup::Mammals => Box::new(taxa::class.ne("Mammalia")),
        TaxonomicVernacularGroup::Seaweeds => Box::new(taxa::kingdom.ne("Chromista")),
        TaxonomicVernacularGroup::HigherPlants => Box::new(taxa::kingdom.ne("Plantae")),
    }
}


/// Narrow down the results from the taxa table with multiple filters
pub fn with_taxonomy(filters: &Vec<Filter>) -> Option<BoxedTaxaExpression> {
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
