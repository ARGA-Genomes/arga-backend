/// Implements a pagination extension for diesel.
/// This is taken pretty much as is from the diesel guides at: https://diesel.rs/guides/extending-diesel.html
use diesel::pg::Pg;
use diesel::query_builder::{AstPass, Query, QueryFragment, QueryId};
use diesel::sql_types::BigInt;
use diesel::{PgConnection, QueryResult, RunQueryDsl};


pub struct FilteredPage<T, Options> {
    pub records: Vec<T>,
    pub total: i64,
    pub options: Options,
}

impl<T, Options> FilteredPage<T, Options> {
    pub fn new(records: Vec<(T, i64)>, options: Options) -> FilteredPage<T, Options> {
        let page = Page::from(records);
        FilteredPage {
            records: page.records,
            total: page.total,
            options,
        }
    }
}


pub struct Page<T> {
    pub records: Vec<T>,
    pub total: i64,
}

impl<T> From<Vec<(T, i64)>> for Page<T> {
    fn from(source: Vec<(T, i64)>) -> Self {
        let mut records = Vec::with_capacity(source.len());
        let mut total = 0;
        for (record, total_records) in source {
            records.push(record);
            total = total_records;
        }
        Page { records, total }
    }
}


const DEFAULT_PER_PAGE: i64 = 16;

pub trait Paginate: Sized {
    /// Paginate the query and return records in the page as well as the total
    /// amount of records. This will use a default page size of 10.
    fn paginate(self, page: i64) -> Paginated<Self>;
}

impl<T> Paginate for T {
    fn paginate(self, page: i64) -> Paginated<Self> {
        Paginated {
            query: self,
            per_page: DEFAULT_PER_PAGE,
            page,
            offset: (page - 1) * DEFAULT_PER_PAGE,
        }
    }
}

#[derive(Debug, Clone, Copy, QueryId)]
pub struct Paginated<T> {
    query: T,
    page: i64,
    per_page: i64,
    offset: i64,
}

impl<T> Paginated<T> {
    /// Define the amount of records present in a single page
    pub fn per_page(self, per_page: i64) -> Self {
        Paginated {
            per_page,
            offset: (self.page - 1) * per_page,
            ..self
        }
    }
}

// where the query gets generated
impl<T> QueryFragment<Pg> for Paginated<T>
where
    T: QueryFragment<Pg>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
        out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") as paged_query_with LIMIT ");
        out.push_bind_param::<BigInt, _>(&self.per_page)?;
        out.push_sql(" OFFSET ");
        out.push_bind_param::<BigInt, _>(&self.offset)?;
        Ok(())
    }
}


// implement the query trait to define the return type for the
// paginated query
impl<T: Query> Query for Paginated<T> {
    type SqlType = (T::SqlType, BigInt);
}

// this marker enables the paginated query to be executed as is
impl<T> RunQueryDsl<PgConnection> for Paginated<T> {}
