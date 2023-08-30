use diesel::pg::Pg;
use diesel::prelude::*;

use arga_core::schema::taxa;
use diesel::sql_types::{Bool, Nullable};


pub enum FilterKind {
    Kingdom(String),
    Phylum(String),
    Genus(String),
}

pub enum Filter {
    Include(FilterKind),
}


type BoxedTaxaExpression<'a> = Box<dyn BoxableExpression<taxa::table, Pg, SqlType = Nullable<Bool>> + 'a>;


/// Filter the taxa table with a global filter enum
pub fn with_filter(filter: &Filter) -> BoxedTaxaExpression {
    match filter {
        Filter::Include(kind) => match kind {
            FilterKind::Kingdom(value) => Box::new(taxa::kingdom.eq(value)),
            FilterKind::Phylum(value) => Box::new(taxa::phylum.eq(value)),
            FilterKind::Genus(value) => Box::new(taxa::genus.eq(value)),
        }
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
