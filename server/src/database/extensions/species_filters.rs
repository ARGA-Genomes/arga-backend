use arga_core::schema_gnl::species;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Varchar};

use super::classification_filters::{decompose_classification, Classification};


type BoxedExpression<'a> = Box<dyn BoxableExpression<species::table, Pg, SqlType = Bool> + 'a>;


#[derive(Clone, Debug)]
pub enum SpeciesFilter {
    Classification(Classification),
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
