use arga_core::schema::{accession_events, collection_events, specimens};
use arga_core::schema_gnl::specimen_stats;
use chrono::NaiveDate;
use diesel::dsl::{InnerJoinQuerySource, LeftJoinQuerySource};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Nullable};
use diesel_async::RunQueryDsl;

use super::{Sort, SortOrder};
use crate::database::Error;


type FilterableQuerySource = LeftJoinQuerySource<
    LeftJoinQuerySource<InnerJoinQuerySource<specimens::table, specimen_stats::table>, collection_events::table>,
    accession_events::table,
>;

type FilterExpression<'a> = Box<dyn BoxableExpression<FilterableQuerySource, Pg, SqlType = Nullable<Bool>> + 'a>;


pub enum Filter {
    Names(Vec<uuid::Uuid>),
    Institution(Vec<String>),
    Country(Vec<String>),
    Data(Vec<HasData>),
    CollectedBetween { after: NaiveDate, before: NaiveDate },
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
pub fn with_names(ids: &Vec<uuid::Uuid>) -> _ {
    specimens::name_id.eq_any(ids)
}

#[diesel::dsl::auto_type(no_type_alias)]
pub fn with_any_institution(names: &Vec<String>) -> _ {
    accession_events::institution_name
        .eq_any(names)
        .or(accession_events::institution_code.eq_any(names))
}

#[diesel::dsl::auto_type(no_type_alias)]
pub fn with_any_country(names: &Vec<String>) -> _ {
    collection_events::country.eq_any(names)
}

#[diesel::dsl::auto_type(no_type_alias)]
pub fn with_collection_date_between<'a>(after: &'a NaiveDate, before: &'a NaiveDate) -> _ {
    collection_events::event_date.between(after, before)
}


pub fn with_data(data_type: &HasData) -> FilterExpression {
    match data_type {
        HasData::Genomes => Box::new(specimen_stats::full_genomes.nullable().gt(0)),
        HasData::Loci => Box::new(specimen_stats::loci.nullable().gt(0)),
        HasData::GenomicData => Box::new(specimen_stats::other_genomic.nullable().gt(0)),
    }
}

pub fn with_filter<'a>(filter: &'a Filter) -> FilterExpression<'a> {
    match filter {
        Filter::Names(values) => Box::new(with_names(values).nullable()),
        Filter::Institution(values) => Box::new(with_any_institution(values).nullable()),
        Filter::Country(values) => Box::new(with_any_country(values).nullable()),
        Filter::CollectedBetween { after, before } => Box::new(with_collection_date_between(after, before).nullable()),
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

pub fn with_filters(filters: &Vec<Filter>) -> Option<FilterExpression> {
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


pub trait DynamicFilters<'a>: Sized {
    fn dynamic_filters(self, filters: &'a Vec<Filter>) -> diesel::helper_types::Filter<Self, FilterExpression<'a>>
    where
        Self: diesel::query_dsl::methods::FilterDsl<FilterExpression<'a>>,
    {
        match with_filters(&filters) {
            Some(predicates) => self.filter(predicates),
            None => self.filter(Box::new(diesel::dsl::sql::<Nullable<Bool>>("1=1"))),
        }
    }
}

impl<'a, T> DynamicFilters<'a> for T {}


pub struct Options {
    pub institutions: Vec<String>,
    pub countries: Vec<String>,
}

impl Options {
    pub async fn load(conn: &mut diesel_async::AsyncPgConnection, filters: &Vec<Filter>) -> Result<Options, Error> {
        let institutions = with_filter_tables()
            .group_by(accession_events::institution_code)
            .select(accession_events::institution_code.assume_not_null())
            .filter(accession_events::institution_code.is_not_null())
            .dynamic_filters(filters)
            .load::<String>(conn)
            .await?;

        let countries = with_filter_tables()
            .group_by(collection_events::country)
            .select(collection_events::country.assume_not_null())
            .filter(collection_events::country.is_not_null())
            .dynamic_filters(filters)
            .load::<String>(conn)
            .await?;

        Ok(Options {
            institutions,
            countries,
        })
    }
}


pub mod sorting {
    use arga_core::schema::{accession_events, collection_events};
    use arga_core::schema_gnl::specimen_stats;
    use diesel::expression::expression_types::NotSelectable;
    use diesel::pg::Pg;
    use diesel::prelude::*;

    use super::{FilterableQuerySource, Sort, SortOrder};


    type SortExpression = Box<dyn BoxableExpression<FilterableQuerySource, Pg, SqlType = NotSelectable>>;
    type Sort2Expression =
        Box<dyn BoxableExpression<FilterableQuerySource, Pg, SqlType = (NotSelectable, NotSelectable)>>;
    type Sort3Expression =
        Box<dyn BoxableExpression<FilterableQuerySource, Pg, SqlType = (NotSelectable, NotSelectable, NotSelectable)>>;


    pub enum Sortable {
        Status,
        Voucher,
        Institution,
        Country,
        CollectionDate,
        MetadataScore,
        Genomes,
        Loci,
        GenomicData,
    }


    pub fn by_status(order: SortOrder) -> SortExpression {
        match order {
            SortOrder::Ascending => Box::new(accession_events::type_status.nullable().asc().nulls_last()),
            SortOrder::Descending => Box::new(accession_events::type_status.nullable().desc().nulls_last()),
        }
    }

    pub fn by_voucher(order: SortOrder) -> Sort2Expression {
        match order {
            SortOrder::Ascending => Box::new((
                accession_events::collection_repository_id.nullable().asc().nulls_last(),
                accession_events::collection_repository_code
                    .nullable()
                    .asc()
                    .nulls_last(),
            )),
            SortOrder::Descending => Box::new((
                accession_events::collection_repository_id
                    .nullable()
                    .desc()
                    .nulls_last(),
                accession_events::collection_repository_code
                    .nullable()
                    .desc()
                    .nulls_last(),
            )),
        }
    }

    pub fn by_institution(order: SortOrder) -> Sort2Expression {
        match order {
            SortOrder::Ascending => Box::new((
                accession_events::institution_name.nullable().asc().nulls_last(),
                accession_events::institution_code.nullable().asc().nulls_last(),
            )),
            SortOrder::Descending => Box::new((
                accession_events::institution_name.nullable().desc().nulls_last(),
                accession_events::institution_code.nullable().desc().nulls_last(),
            )),
        }
    }

    pub fn by_country(order: SortOrder) -> SortExpression {
        match order {
            SortOrder::Ascending => Box::new(collection_events::country.nullable().asc().nulls_last()),
            SortOrder::Descending => Box::new(collection_events::country.nullable().desc().nulls_last()),
        }
    }

    pub fn by_collection_date(order: SortOrder) -> SortExpression {
        match order {
            SortOrder::Ascending => Box::new(collection_events::event_date.nullable().asc().nulls_last()),
            SortOrder::Descending => Box::new(collection_events::event_date.nullable().desc().nulls_last()),
        }
    }

    pub fn by_genomes(order: SortOrder) -> SortExpression {
        match order {
            SortOrder::Ascending => Box::new(specimen_stats::full_genomes.nullable().asc()),
            SortOrder::Descending => Box::new(specimen_stats::full_genomes.nullable().desc()),
        }
    }

    pub fn by_loci(order: SortOrder) -> SortExpression {
        match order {
            SortOrder::Ascending => Box::new(specimen_stats::loci.nullable().asc()),
            SortOrder::Descending => Box::new(specimen_stats::loci.nullable().desc()),
        }
    }

    pub fn by_genomic_data(order: SortOrder) -> SortExpression {
        match order {
            SortOrder::Ascending => Box::new(specimen_stats::other_genomic.nullable().asc()),
            SortOrder::Descending => Box::new(specimen_stats::other_genomic.nullable().desc()),
        }
    }

    /// The score of all metadata associated with the specimen
    /// 1. The specimen is registered
    /// 2. The specimen has collection data
    /// 3. The specimen has genomic data
    pub fn by_metadata_score(order: SortOrder) -> Sort3Expression {
        match order {
            SortOrder::Ascending => Box::new((
                accession_events::collection_repository_id
                    .nullable()
                    .asc()
                    .nulls_first(),
                collection_events::event_date.nullable().asc().nulls_first(),
                specimen_stats::other_genomic.nullable().asc().nulls_first(),
            )),
            SortOrder::Descending => Box::new((
                accession_events::collection_repository_id
                    .nullable()
                    .desc()
                    .nulls_last(),
                collection_events::event_date.nullable().desc().nulls_last(),
                specimen_stats::other_genomic.nullable().desc().nulls_last(),
            )),
        }
    }


    pub trait DynamicSort: Sized {
        fn dynamic_sort(self, sorting: Sort<Sortable>) -> diesel::helper_types::Order<Self, SortExpression>
        where
            Self: diesel::query_dsl::methods::OrderDsl<SortExpression>,
        {
            match sorting.sortable {
                Sortable::Status => self.order(by_status(sorting.order)),
                Sortable::Voucher => todo!(),
                Sortable::Institution => todo!(),
                Sortable::Country => self.order(by_country(sorting.order)),
                Sortable::CollectionDate => self.order(by_collection_date(sorting.order)),
                Sortable::MetadataScore => todo!(),
                Sortable::Genomes => self.order(by_genomes(sorting.order)),
                Sortable::Loci => self.order(by_loci(sorting.order)),
                Sortable::GenomicData => self.order(by_genomic_data(sorting.order)),
            }
        }
    }

    impl<T> DynamicSort for T {}
}
