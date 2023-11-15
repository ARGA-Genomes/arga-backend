use diesel::pg::Pg;
use diesel::prelude::*;

use arga_core::schema_gnl::classification_species;
use arga_core::models::TaxonomicRank;
use diesel::sql_types::Bool;

use super::classification_filters::Classification;


type BoxedExpression<'a> = Box<dyn BoxableExpression<classification_species::table, Pg, SqlType = Bool> + 'a>;


#[derive(Clone)]
pub enum FilterKind {
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
            FilterKind::ParentClassification(value) => with_parent_classification(value),
        }
        Filter::Exclude(kind) => match kind {
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
pub fn with_parent_classification(classification: &Classification) -> BoxedExpression {
    use classification_species::parent_rank as rank;
    use classification_species::parent_canonical_name as name;

    match classification {
        Classification::Domain(value) => Box::new(rank.eq(TaxonomicRank::Domain).and(name.eq(value))),
        Classification::Superkingdom(value) => Box::new(rank.eq(TaxonomicRank::Superkingdom).and(name.eq(value))),
        Classification::Kingdom(value) => Box::new(rank.eq(TaxonomicRank::Kingdom).and(name.eq(value))),
        Classification::Subkingdom(value) => Box::new(rank.eq(TaxonomicRank::Subkingdom).and(name.eq(value))),
        Classification::Phylum(value) => Box::new(rank.eq(TaxonomicRank::Phylum).and(name.eq(value))),
        Classification::Subphylum(value) => Box::new(rank.eq(TaxonomicRank::Subphylum).and(name.eq(value))),
        Classification::Superclass(value) => Box::new(rank.eq(TaxonomicRank::Superclass).and(name.eq(value))),
        Classification::Class(value) => Box::new(rank.eq(TaxonomicRank::Class).and(name.eq(value))),
        Classification::Subclass(value) => Box::new(rank.eq(TaxonomicRank::Subclass).and(name.eq(value))),
        Classification::Superorder(value) => Box::new(rank.eq(TaxonomicRank::Superorder).and(name.eq(value))),
        Classification::Order(value) => Box::new(rank.eq(TaxonomicRank::Order).and(name.eq(value))),
        Classification::Suborder(value) => Box::new(rank.eq(TaxonomicRank::Suborder).and(name.eq(value))),
        Classification::Superfamily(value) => Box::new(rank.eq(TaxonomicRank::Superfamily).and(name.eq(value))),
        Classification::Family(value) => Box::new(rank.eq(TaxonomicRank::Family).and(name.eq(value))),
        Classification::Subfamily(value) => Box::new(rank.eq(TaxonomicRank::Subfamily).and(name.eq(value))),
        Classification::Supertribe(value) => Box::new(rank.eq(TaxonomicRank::Supertribe).and(name.eq(value))),
        Classification::Tribe(value) => Box::new(rank.eq(TaxonomicRank::Tribe).and(name.eq(value))),
        Classification::Subtribe(value) => Box::new(rank.eq(TaxonomicRank::Subtribe).and(name.eq(value))),
        Classification::Genus(value) => Box::new(rank.eq(TaxonomicRank::Genus).and(name.eq(value))),
        Classification::Subgenus(value) => Box::new(rank.eq(TaxonomicRank::Subgenus).and(name.eq(value))),
        Classification::Species(value) => Box::new(rank.eq(TaxonomicRank::Species).and(name.eq(value))),
        Classification::Subspecies(value) => Box::new(rank.eq(TaxonomicRank::Subspecies).and(name.eq(value))),
        Classification::Unranked(value) => Box::new(rank.eq(TaxonomicRank::Unranked).and(name.eq(value))),
        Classification::HigherTaxon(value) => Box::new(rank.eq(TaxonomicRank::HigherTaxon).and(name.eq(value))),
    }
}

/// Filter the classifications table that do not belong to the provided classification
pub fn without_parent_classification(classification: &Classification) -> BoxedExpression {
    use classification_species::parent_rank as rank;
    use classification_species::parent_canonical_name as name;

    match classification {
        Classification::Domain(value) => Box::new(rank.eq(TaxonomicRank::Domain).and(name.ne(value))),
        Classification::Superkingdom(value) => Box::new(rank.eq(TaxonomicRank::Superkingdom).and(name.ne(value))),
        Classification::Kingdom(value) => Box::new(rank.eq(TaxonomicRank::Kingdom).and(name.ne(value))),
        Classification::Subkingdom(value) => Box::new(rank.eq(TaxonomicRank::Subkingdom).and(name.ne(value))),
        Classification::Phylum(value) => Box::new(rank.eq(TaxonomicRank::Phylum).and(name.ne(value))),
        Classification::Subphylum(value) => Box::new(rank.eq(TaxonomicRank::Subphylum).and(name.ne(value))),
        Classification::Superclass(value) => Box::new(rank.eq(TaxonomicRank::Superclass).and(name.ne(value))),
        Classification::Class(value) => Box::new(rank.eq(TaxonomicRank::Class).and(name.ne(value))),
        Classification::Subclass(value) => Box::new(rank.eq(TaxonomicRank::Subclass).and(name.ne(value))),
        Classification::Superorder(value) => Box::new(rank.eq(TaxonomicRank::Superorder).and(name.ne(value))),
        Classification::Order(value) => Box::new(rank.eq(TaxonomicRank::Order).and(name.ne(value))),
        Classification::Suborder(value) => Box::new(rank.eq(TaxonomicRank::Suborder).and(name.ne(value))),
        Classification::Superfamily(value) => Box::new(rank.eq(TaxonomicRank::Superfamily).and(name.ne(value))),
        Classification::Family(value) => Box::new(rank.eq(TaxonomicRank::Family).and(name.ne(value))),
        Classification::Subfamily(value) => Box::new(rank.eq(TaxonomicRank::Subfamily).and(name.ne(value))),
        Classification::Supertribe(value) => Box::new(rank.eq(TaxonomicRank::Supertribe).and(name.ne(value))),
        Classification::Tribe(value) => Box::new(rank.eq(TaxonomicRank::Tribe).and(name.ne(value))),
        Classification::Subtribe(value) => Box::new(rank.eq(TaxonomicRank::Subtribe).and(name.ne(value))),
        Classification::Genus(value) => Box::new(rank.eq(TaxonomicRank::Genus).and(name.ne(value))),
        Classification::Subgenus(value) => Box::new(rank.eq(TaxonomicRank::Subgenus).and(name.ne(value))),
        Classification::Species(value) => Box::new(rank.eq(TaxonomicRank::Species).and(name.ne(value))),
        Classification::Subspecies(value) => Box::new(rank.eq(TaxonomicRank::Subspecies).and(name.ne(value))),
        Classification::Unranked(value) => Box::new(rank.eq(TaxonomicRank::Unranked).and(name.ne(value))),
        Classification::HigherTaxon(value) => Box::new(rank.eq(TaxonomicRank::HigherTaxon).and(name.ne(value))),
    }
}
