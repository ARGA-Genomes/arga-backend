use async_trait::async_trait;

use diesel::prelude::*;
use diesel::sql_types::{Text, Nullable};
use diesel_async::RunQueryDsl;
use tracing::{instrument, debug};
use uuid::Uuid;

use crate::index::lists::{GetListNames, GetListPhotos, Filters, Filter, FilterItem, Pagination, GetListStats, ListStats};

use super::{schema, Database, Error, PgPool};
use super::models::{NameList, Name, TaxonPhoto, Taxon};


sql_function!(fn lower(x: Nullable<Text>) -> Nullable<Text>);


#[derive(Clone)]
pub struct ListProvider {
    pub pool: PgPool,
}

impl ListProvider {
    pub async fn list_taxa(&self, list: &Vec<Name>) -> Result<Vec<Taxon>, Error> {
        use schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = list.iter().map(|n| n.id).collect();

        let records = taxa
            .filter(name_id.eq_any(name_ids))
            .load::<Taxon>(&mut conn)
            .await?;

        Ok(records)
    }
}


#[async_trait]
impl GetListNames for Database {
    type Error = Error;

    #[instrument(skip(self))]
    async fn list_names(&self, list: &NameList, filters: &Filters, pagination: &Pagination) -> Result<Vec<Name>, Self::Error> {
        use schema::{names, taxa, conservation_statuses};
        let mut conn = self.pool.get().await?;

        let offset = pagination.page_size * (pagination.page - 1);

        let mut query = conservation_statuses::table
            .inner_join(names::table)
            .inner_join(taxa::table.on(taxa::name_id.eq(names::id)))
            .select(names::all_columns)
            .filter(conservation_statuses::list_id.eq(list.id))
            .order_by(names::scientific_name)
            .offset(offset)
            .limit(pagination.page_size)
            .into_boxed();

        for item in filters.items.iter() {
            query = match item {
                FilterItem::Include(filter) => match filter {
                    Filter::Kingdom(value) => query.filter(lower(taxa::kingdom).eq(value.to_lowercase())),
                    Filter::Phylum(value) => query.filter(lower(taxa::phylum).eq(value.to_lowercase())),
                },
                FilterItem::Exclude(filter) => match filter {
                    Filter::Kingdom(value) => query.filter(lower(taxa::kingdom).ne(value.to_lowercase())),
                    Filter::Phylum(value) => query.filter(lower(taxa::phylum).ne(value.to_lowercase())),
                },
            };
        }

        debug!("Getting filtered names");
        let records = query
            .load::<Name>(&mut conn)
            .await?;

        Ok(records)
    }
}


#[async_trait]
impl GetListPhotos for Database {
    type Error = Error;

    async fn list_photos(&self, list: &Vec<Name>) -> Result<Vec<TaxonPhoto>, Self::Error> {
        use schema::taxon_photos::dsl::*;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = list.iter().map(|n| n.id).collect();

        let photos = taxon_photos
            .filter(name_id.eq_any(name_ids))
            .load::<TaxonPhoto>(&mut conn)
            .await?;

        Ok(photos)
    }
}


#[async_trait]
impl GetListStats for Database {
    type Error = Error;

    async fn list_stats(&self, list: &NameList, filters: &Filters) -> Result<ListStats, Self::Error> {
        use schema::{names, taxa, conservation_statuses};
        let mut conn = self.pool.get().await?;

        let mut query = conservation_statuses::table
            .inner_join(names::table)
            .inner_join(taxa::table.on(taxa::name_id.eq(names::id)))
            .select(diesel::dsl::count_star())
            .filter(conservation_statuses::list_id.eq(list.id))
            .into_boxed();

        for item in filters.items.iter() {
            query = match item {
                FilterItem::Include(filter) => match filter {
                    Filter::Kingdom(value) => query.filter(lower(taxa::kingdom).eq(value.to_lowercase())),
                    Filter::Phylum(value) => query.filter(lower(taxa::phylum).eq(value.to_lowercase())),
                },
                FilterItem::Exclude(filter) => match filter {
                    Filter::Kingdom(value) => query.filter(lower(taxa::kingdom).ne(value.to_lowercase())),
                    Filter::Phylum(value) => query.filter(lower(taxa::phylum).ne(value.to_lowercase())),
                },
            };
        }

        debug!("Getting filtered names");
        let total_records: i64 = query.get_result(&mut conn).await?;

        Ok(ListStats {
            total_records: total_records as usize,
        })
    }
}
