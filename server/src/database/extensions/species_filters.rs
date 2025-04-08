use arga_core::models::{ACCEPTED_NAMES, SPECIES_RANKS};
use arga_core::schema_gnl::species;
use async_graphql::{InputObject, OneofObject};
use chrono::{DateTime, Utc};
use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Nullable, Untyped, Varchar};

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

pub fn with_sorting(sort: SpeciesSort, direction: SortDirection) -> SqlLiteral<Untyped> {
    let clause = match (sort, direction) {
        (SpeciesSort::ScientificName, SortDirection::Asc) => "scientific_name ASC",
        (SpeciesSort::ScientificName, SortDirection::Desc) => "scientific_name DESC",
        (SpeciesSort::CanonicalName, SortDirection::Asc) => "canonical_name ASC",
        (SpeciesSort::CanonicalName, SortDirection::Desc) => "canonical_name DESC",
        // (SpeciesSort::ClassificationHierarchy, SortDirection::Asc) => "classification_hierarchy ASC",
        // (SpeciesSort::ClassificationHierarchy, SortDirection::Desc) => "classification_hierarchy DESC",
        (SpeciesSort::ClassificationHierarchy, SortDirection::Asc) => "scientific_name ASC",
        (SpeciesSort::ClassificationHierarchy, SortDirection::Desc) => "scientific_name DESC",
        (SpeciesSort::Genomes, SortDirection::Asc) => "species.genomes ASC",
        (SpeciesSort::Genomes, SortDirection::Desc) => "genomes DESC",
        (SpeciesSort::Loci, SortDirection::Asc) => "species.loci ASC",
        (SpeciesSort::Loci, SortDirection::Desc) => "species.loci DESC",
        (SpeciesSort::Specimens, SortDirection::Asc) => "specimens ASC",
        (SpeciesSort::Specimens, SortDirection::Desc) => "specimens DESC",
        (SpeciesSort::Other, SortDirection::Asc) => "other ASC",
        (SpeciesSort::Other, SortDirection::Desc) => "other DESC",
        (SpeciesSort::TotalGenomic, SortDirection::Asc) => "total_genomic ASC",
        (SpeciesSort::TotalGenomic, SortDirection::Desc) => "total_genomic DESC",
    };
    sql::<Untyped>(clause)
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

type AttrsBoxedExpression<'a> = Box<dyn BoxableExpression<species::table, Pg, SqlType = Nullable<Bool>> + 'a>;

#[derive(OneofObject)]
pub enum NameAttributeValue {
    Int(i64),
    Bool(bool),
    String(String),
    Timestamp(DateTime<Utc>),
    Decimal(f64),
}

#[derive(InputObject)]
pub struct NameAttributeFilter {
    pub name: String,
    pub value: NameAttributeValue,
}

/// Filter species based on their associated name attributes JSON
pub fn with_attribute(attibute: &NameAttributeFilter) -> AttrsBoxedExpression {
    Box::new(species::attributes.contains(serde_json::json!([{
        "name": attibute.name,
            "value": match &attibute.value {
            NameAttributeValue::Int(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            NameAttributeValue::Bool(b) => serde_json::Value::Bool(*b),
            NameAttributeValue::String(s) => serde_json::Value::String(s.clone()),
            NameAttributeValue::Timestamp(t) => serde_json::Value::String(t.to_rfc3339()),
            NameAttributeValue::Decimal(d) => serde_json::Value::Number(
                serde_json::Number::from_f64(*d).unwrap_or_else(|| serde_json::Number::from(0))
            ),
        }
    }])))
}
