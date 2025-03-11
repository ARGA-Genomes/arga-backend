use arga_core::schema::taxa;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;

use super::filters::DataType;

type BoxedExpression<'a> = Box<dyn BoxableExpression<taxa::table, Pg, SqlType = Bool> + 'a>;

#[derive(Clone, Debug)]
pub enum TaxaFilter {
    Data(DataFilter),
    Taxon(TaxonFilter),
}

#[derive(Clone, Debug)]
pub enum TaxonFilter {
    ScientificName(String),
    CanonicalName(String),
    VernacularGroup(String),
}

#[derive(Clone, Debug)]
pub enum DataFilter {
    HasData(DataType),
}

/// Filter the taxa table that have the provided taxonomy data
pub fn with_taxonomy(taxon: &TaxonFilter) -> BoxedExpression {
    match taxon {
        TaxonFilter::ScientificName(text) => Box::new(taxa::scientific_name.eq(text)),
        TaxonFilter::CanonicalName(text) => Box::new(taxa::canonical_name.eq(text)),
        TaxonFilter::VernacularGroup(text) => todo!(),
    }
}

/// Filter the taxa table with a global filter enum
pub fn with_taxa_filter(filter: &TaxaFilter) -> BoxedExpression {
    match filter {
        TaxaFilter::Taxon(value) => with_taxonomy(value),
        TaxaFilter::Data(value) => todo!(),
    }
}

/// Narrow down the results from the taxa table with multiple filters
pub fn with_taxa_filters(filters: &Vec<TaxaFilter>) -> Option<BoxedExpression> {
    let mut predicates: Option<BoxedExpression> = None;

    for filter in filters {
        let predicate = with_taxa_filter(filter);

        predicates = match predicates {
            None => Some(predicate),
            Some(others) => Some(Box::new(others.and(predicate))),
        }
    }

    predicates
}
