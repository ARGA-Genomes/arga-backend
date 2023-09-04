use diesel::pg::Pg;
use diesel::prelude::*;

use arga_core::schema::taxa;
use diesel::sql_types::{Bool, Nullable};


pub enum FilterKind {
    Kingdom(String),
    Phylum(String),
    Class(String),
    Order(String),
    Family(String),
    Tribe(String),
    Genus(String),
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
        }
        Filter::Exclude(kind) => match kind {
            FilterKind::Kingdom(value) => Box::new(taxa::kingdom.ne(value)),
            FilterKind::Phylum(value) => Box::new(taxa::phylum.ne(value)),
            FilterKind::Class(value) => Box::new(taxa::class.ne(value)),
            FilterKind::Order(value) => Box::new(taxa::order.ne(value)),
            FilterKind::Family(value) => Box::new(taxa::family.ne(value)),
            FilterKind::Tribe(value) => Box::new(taxa::tribe.ne(value)),
            FilterKind::Genus(value) => Box::new(taxa::genus.ne(value)),
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
