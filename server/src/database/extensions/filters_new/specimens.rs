use arga_core::schema::{accession_events, collection_events, specimens};
use arga_core::schema_gnl::specimen_stats;
use diesel::dsl::{InnerJoinQuerySource, LeftJoinQuerySource};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Nullable};


type FilterableQuerySource = LeftJoinQuerySource<
    LeftJoinQuerySource<InnerJoinQuerySource<specimens::table, specimen_stats::table>, collection_events::table>,
    accession_events::table,
>;

type FilterExpression = Box<dyn BoxableExpression<FilterableQuerySource, Pg, SqlType = Nullable<Bool>>>;


pub enum Filter {
    Institution(Vec<String>),
    Country(Vec<String>),
    Data(Vec<HasData>),
}

pub enum HasData {
    Genomes,
    Loci,
    GenomicData,
}


#[diesel::dsl::auto_type]
pub fn with_filter_tables() -> _ {
    specimens::table
        .inner_join(specimen_stats::table)
        .left_join(collection_events::table)
        .left_join(accession_events::table)
}


#[diesel::dsl::auto_type(no_type_alias)]
pub fn with_any_institution(names: Vec<String>) -> _ {
    accession_events::institution_name.eq_any(names)
}

#[diesel::dsl::auto_type(no_type_alias)]
pub fn with_any_country(names: Vec<String>) -> _ {
    collection_events::country.eq_any(names)
}

pub fn with_data(data_type: HasData) -> FilterExpression {
    match data_type {
        HasData::Genomes => Box::new(specimen_stats::full_genomes.nullable().gt(0)),
        HasData::Loci => Box::new(specimen_stats::loci.nullable().gt(0)),
        HasData::GenomicData => Box::new(specimen_stats::other_genomic.nullable().gt(0)),
    }
}

pub fn with_filter(filter: Filter) -> FilterExpression {
    match filter {
        Filter::Institution(values) => Box::new(with_any_institution(values).nullable()),
        Filter::Country(values) => Box::new(with_any_country(values).nullable()),
        Filter::Data(values) => {
            let mut predicates = None;

            for value in values {
                let predicate = with_data(value);

                predicates = match predicates {
                    None => Some(predicate),
                    Some(others) => Some(Box::new(others.or(predicate))),
                }
            }

            Box::new(predicates.unwrap_or_else(|| Box::new(diesel::dsl::sql::<Nullable<Bool>>("1=1"))))
        }
    }
}

pub fn with_filters(filters: Vec<Filter>) -> Option<FilterExpression> {
    let mut predicates = None;

    // builds a 'where .. and ..' expression
    for filter in filters {
        let predicate = with_filter(filter);

        predicates = match predicates {
            None => Some(predicate),
            Some(others) => Some(Box::new(others.and(predicate))),
        }
    }

    predicates
}


pub trait DynamicFilters: Sized {
    fn dynamic_filters(self, filters: Vec<Filter>) -> diesel::helper_types::Filter<Self, FilterExpression>
    where
        Self: diesel::query_dsl::methods::FilterDsl<FilterExpression>,
    {
        match with_filters(filters) {
            Some(predicates) => diesel::query_dsl::methods::FilterDsl::<FilterExpression>::filter(self, predicates),
            None => diesel::query_dsl::methods::FilterDsl::<FilterExpression>::filter(
                self,
                Box::new(diesel::dsl::sql::<Nullable<Bool>>("1=1")),
            ),
        }
    }
}

impl<T> DynamicFilters for T {}
