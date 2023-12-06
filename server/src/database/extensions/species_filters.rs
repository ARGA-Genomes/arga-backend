use diesel::pg::Pg;
use diesel::prelude::*;

use arga_core::schema_gnl::classification_species;
use diesel::sql_types::Bool;

use super::classification_filters::{Classification, decompose_classification};


type BoxedExpression<'a> = Box<dyn BoxableExpression<classification_species::table, Pg, SqlType = Bool> + 'a>;


#[derive(Clone)]
pub enum FilterKind {
    Classification(Classification),
    ParentClassification(Classification),
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
            FilterKind::ParentClassification(value) => with_parent_classification(value),
        }
        Filter::Exclude(kind) => match kind {
            FilterKind::Classification(value) => without_classification(value),
            FilterKind::ParentClassification(value) => without_parent_classification(value),
        }
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
    use classification_species::classification_rank as rank;
    use classification_species::classification_canonical_name as name;

    let (taxon_rank, value) = decompose_classification(classification);
    Box::new(rank.eq(taxon_rank).and(name.eq(value)))
}

/// Filter the classifications table that do not belong to the provided classification
pub fn without_classification(classification: &Classification) -> BoxedExpression {
    use classification_species::classification_rank as rank;
    use classification_species::classification_canonical_name as name;

    let (taxon_rank, value) = decompose_classification(classification);
    Box::new(rank.eq(taxon_rank).and(name.ne(value)))
}

/// Filter the classifications table that belong to the provided classification
pub fn with_parent_classification(classification: &Classification) -> BoxedExpression {
    use classification_species::parent_rank as rank;
    use classification_species::parent_canonical_name as name;

    let (taxon_rank, value) = decompose_classification(classification);
    Box::new(rank.eq(taxon_rank).and(name.eq(value)))
}

/// Filter the classifications table that do not belong to the provided classification
pub fn without_parent_classification(classification: &Classification) -> BoxedExpression {
    use classification_species::parent_rank as rank;
    use classification_species::parent_canonical_name as name;

    let (taxon_rank, value) = decompose_classification(classification);
    Box::new(rank.eq(taxon_rank).and(name.ne(value)))
}
