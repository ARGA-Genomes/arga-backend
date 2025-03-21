use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;

use super::PgPool;
use crate::database::extensions::classification_filters::Classification;
use crate::database::extensions::species_filters::with_accepted_classification;
use crate::database::schema_gnl;
use crate::http::Error;


pub struct Overview {
    pub total: i64,
}


#[derive(Debug, Queryable, Clone, Deserialize)]
pub struct OverviewRecord {
    pub name: String,
    pub total: i64,
}


#[derive(Clone)]
pub struct OverviewProvider {
    pub pool: PgPool,
}


impl OverviewProvider {
    pub async fn classification(&self, by: &Classification) -> Result<Overview, Error> {
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;

        let total: i64 = species
            .filter(with_accepted_classification(by))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(Overview { total })
    }

    pub async fn sequences(&self) -> Result<Overview, Error> {
        use schema_gnl::overview::dsl::*;
        let mut conn = self.pool.get().await?;
        let count: i64 = overview
            .filter(category.eq("data_type"))
            .filter(name.eq("sequences"))
            .select(total)
            .get_result(&mut conn)
            .await?;

        Ok(Overview { total: count })
    }

    pub async fn whole_genomes(&self) -> Result<Overview, Error> {
        use schema_gnl::overview::dsl::*;
        let mut conn = self.pool.get().await?;
        let count: i64 = overview
            .filter(category.eq("data_type"))
            .filter(name.eq("whole_genomes"))
            .select(total)
            .get_result(&mut conn)
            .await?;

        Ok(Overview { total: count })
    }

    pub async fn loci(&self) -> Result<Overview, Error> {
        use schema_gnl::overview::dsl::*;
        let mut conn = self.pool.get().await?;
        let count: i64 = overview
            .filter(category.eq("data_type"))
            .filter(name.eq("loci"))
            .select(total)
            .get_result(&mut conn)
            .await?;

        Ok(Overview { total: count })
    }

    pub async fn specimens(&self) -> Result<Overview, Error> {
        use schema_gnl::overview::dsl::*;
        let mut conn = self.pool.get().await?;
        let count: i64 = overview
            .filter(category.eq("data_type"))
            .filter(name.eq("specimens"))
            .select(total)
            .get_result(&mut conn)
            .await?;

        Ok(Overview { total: count })
    }

    pub async fn sources(&self) -> Result<Vec<OverviewRecord>, Error> {
        use schema_gnl::overview::dsl::*;
        let mut conn = self.pool.get().await?;
        let records = overview
            .filter(category.eq("source"))
            .select((name, total))
            .load::<OverviewRecord>(&mut conn)
            .await?;

        Ok(records)
    }

    pub async fn datasets(&self) -> Result<Vec<OverviewRecord>, Error> {
        use schema_gnl::overview::dsl::*;
        let mut conn = self.pool.get().await?;
        let records = overview
            .filter(category.eq("dataset"))
            .select((name, total))
            .load::<OverviewRecord>(&mut conn)
            .await?;

        Ok(records)
    }
}
