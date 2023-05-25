use async_trait::async_trait;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::index::lists::{GetListNames, GetListTaxa, GetListPhotos};
use crate::index::providers::db::models::UserTaxon;

use super::{Database, Error};
use super::models::{UserTaxaList, Name, TaxonPhoto};


#[async_trait]
impl GetListNames for Database {
    type Error = Error;

    async fn list_names(&self, list: &UserTaxaList) -> Result<Vec<Name>, Self::Error> {
        use crate::schema::names;
        use crate::schema::user_taxa;
        let mut conn = self.pool.get().await?;

        let records = user_taxa::table
            .inner_join(names::table)
            .select(names::all_columns)
            .filter(user_taxa::taxa_lists_id.eq(list.id))
            .order_by(names::scientific_name)
            .limit(20)
            .load::<Name>(&mut conn)
            .await?;

        Ok(records)
    }
}

#[async_trait]
impl GetListTaxa for Database {
    type Error = Error;

    async fn list_taxa(&self, list: &Vec<Name>) -> Result<Vec<UserTaxon>, Self::Error> {
        use crate::schema::user_taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = list.iter().map(|n| n.id).collect();

        let records = user_taxa
            .filter(name_id.eq_any(name_ids))
            .load::<UserTaxon>(&mut conn)
            .await?;

        Ok(records)
    }
}

#[async_trait]
impl GetListPhotos for Database {
    type Error = Error;

    async fn list_photos(&self, list: &Vec<Name>) -> Result<Vec<TaxonPhoto>, Self::Error> {
        use crate::schema::taxon_photos::dsl::*;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = list.iter().map(|n| n.id).collect();

        let photos = taxon_photos
            .filter(name_id.eq_any(name_ids))
            .load::<TaxonPhoto>(&mut conn)
            .await?;

        Ok(photos)
    }
}
