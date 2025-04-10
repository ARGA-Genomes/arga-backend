use arga_core::models::{ACCEPTED_NAMES, SPECIES_RANKS};
use arga_core::schema_gnl::species;
use diesel::dsl::not;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Varchar};
use serde_json::Value;

use super::classification_filters::{Classification, decompose_classification};

type BoxedExpression<'a> = Box<dyn BoxableExpression<species::table, Pg, SqlType = Bool> + 'a>;

#[derive(Clone, Debug)]
pub enum SpeciesFilter {
    Classification(Classification),
}

#[derive(Clone, Debug)]
pub enum SpeciesSort {
    ScientificName,
    CanonicalName,
    ClassificationHierarchy,
    Genomes,
    Loci,
    Specimens,
    Other,
    TotalGenomic,
}

#[derive(Clone, Debug)]
pub enum SortDirection {
    Asc,
    Desc,
}

pub fn with_sorting(
    query: species::BoxedQuery<'_, diesel::pg::Pg>,
    sort: SpeciesSort,
    direction: SortDirection,
) -> species::BoxedQuery<'_, diesel::pg::Pg> {
    use species;
    match direction {
        SortDirection::Asc => match sort {
            SpeciesSort::ScientificName => query.order_by(species::scientific_name.asc()),
            SpeciesSort::CanonicalName => query.order_by(species::canonical_name.asc()),
            SpeciesSort::ClassificationHierarchy => query.order_by(species::scientific_name.asc()),
            SpeciesSort::Genomes => query.order_by(species::genomes.asc()),
            SpeciesSort::Loci => query.order_by(species::loci.asc()),
            SpeciesSort::Specimens => query.order_by(species::specimens.asc()),
            SpeciesSort::Other => query.order_by(species::other.asc()),
            SpeciesSort::TotalGenomic => query.order_by(species::total_genomic.asc()),
        },
        SortDirection::Desc => match sort {
            SpeciesSort::ScientificName => query.order_by(species::scientific_name.desc()),
            SpeciesSort::CanonicalName => query.order_by(species::canonical_name.desc()),
            SpeciesSort::ClassificationHierarchy => query.order_by(species::scientific_name.desc()),
            SpeciesSort::Genomes => query.order_by(species::genomes.desc()),
            SpeciesSort::Loci => query.order_by(species::loci.desc()),
            SpeciesSort::Specimens => query.order_by(species::specimens.desc()),
            SpeciesSort::Other => query.order_by(species::other.desc()),
            SpeciesSort::TotalGenomic => query.order_by(species::total_genomic.desc()),
        },
    }
}


/// Filter the classifications table that belong to the provided classification
pub fn with_classification(classification: &Classification) -> BoxedExpression {
    use diesel::dsl::sql;

    // we do string interpolation here since we don't have a jsonb infix operator yet
    // but its safe from injection as it is converting an enum to a string which has
    // hardcoded values. in other words, its not user input
    let (taxon_rank, value) = decompose_classification(classification);
    let filter = format!("classification->>'{}'", taxon_rank.to_string().to_lowercase());
    Box::new(sql::<Varchar>(&filter).eq(value))
}

/// Filter the classifications table that belong to the provided classification, but only with accepted names and ranks
pub fn with_accepted_classification(classification: &Classification) -> BoxedExpression {
    Box::new(
        species::status
            .eq_any(ACCEPTED_NAMES)
            .and(species::rank.eq_any(SPECIES_RANKS))
            .and(with_classification(classification)),
    )
}

/// Filter the classification species view with a global filter enum
pub fn with_species_filter(filter: &SpeciesFilter) -> BoxedExpression {
    match filter {
        SpeciesFilter::Classification(value) => with_classification(value),
    }
}

/// Narrow down the results from the classification species view with multiple filters
pub fn with_species_filters(filters: &Vec<SpeciesFilter>) -> Option<BoxedExpression> {
    let mut predicates: Option<BoxedExpression> = None;

    for filter in filters {
        let predicate = with_species_filter(filter);

        predicates = match predicates {
            None => Some(predicate),
            Some(others) => Some(Box::new(others.and(predicate))),
        }
    }

    predicates
}

type AttrsBoxedExpression<'a> = Box<dyn BoxableExpression<species::table, Pg, SqlType = Bool> + 'a>;


/// Filter species based on their associated name attributes JSON
pub fn with_attribute(attibute: &Value) -> AttrsBoxedExpression {
    Box::new(species::attributes.contains(attibute).assume_not_null())
}

/// Filter species based on their associated name attributes JSON
pub fn without_attribute(attibute: &Value) -> AttrsBoxedExpression {
    Box::new(not(species::attributes.contains(attibute).assume_not_null()))
}
