use diesel::pg::Pg;
use arga_core::schema_gnl::species;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Varchar};

use super::classification_filters::{decompose_classification, Classification};


type BoxedExpression<'a> = Box<dyn BoxableExpression<species::table, Pg, SqlType = Bool> + 'a>;


#[derive(Clone)]
pub enum FilterKind {
    Classification(Classification),
    // ParentClassification(Classification),
}

#[derive(Clone)]
pub enum Filter {
    Include(FilterKind),
    Exclude(FilterKind),
}


/// Filter the classification species view with a global filter enum
pub fn with_filter(filter: &Filter) -> BoxedExpression {
    match filter {
        Filter::Include(kind) => match kind {
            FilterKind::Classification(value) => with_classification(value),
            // FilterKind::ParentClassification(value) => with_parent_classification(value),
        },
        Filter::Exclude(kind) => match kind {
            FilterKind::Classification(value) => without_classification(value),
            // FilterKind::ParentClassification(value) => without_parent_classification(value),
        },
    }
}

/// Narrow down the results from the classification species view with multiple filters
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

/// Filter the classifications table that do not belong to the provided classification
pub fn without_classification(classification: &Classification) -> BoxedExpression {
    use diesel::dsl::sql;

    let (taxon_rank, value) = decompose_classification(classification);
    let filter = format!("classification->>'{}'", taxon_rank.to_string().to_lowercase());
    Box::new(sql::<Varchar>(&filter).ne(value))
}

// Filter the classifications table that belong to the provided classification
// pub fn with_parent_classification(classification: &Classification) -> BoxedExpression {
//     use classification_species::parent_rank as rank;
//     use classification_species::parent_canonical_name as name;

//     let (taxon_rank, value) = decompose_classification(classification);
//     Box::new(rank.eq(taxon_rank).and(name.eq(value)))
// }

// Filter the classifications table that do not belong to the provided classification
// pub fn without_parent_classification(classification: &Classification) -> BoxedExpression {
//     use classification_species::parent_rank as rank;
//     use classification_species::parent_canonical_name as name;

//     let (taxon_rank, value) = decompose_classification(classification);
//     Box::new(rank.eq(taxon_rank).and(name.ne(value)))
// }
