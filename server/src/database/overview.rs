use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::database::{schema, schema_gnl};
use crate::http::Error;

use super::PgPool;


pub struct Overview {
    pub total: i64,
}


#[derive(Clone)]
pub struct OverviewProvider {
    pub pool: PgPool,
}

impl OverviewProvider {
    pub async fn all_species(&self) -> Result<Overview, Error> {
        use schema::names::dsl::*;
        let mut conn = self.pool.get().await?;
        let total: i64 = names.count().get_result(&mut conn).await?;
        Ok(Overview { total })
    }

    pub async fn animals(&self) -> Result<Overview, Error> {
        use schema::{assemblies, names, taxa};
        let mut conn = self.pool.get().await?;

        let total: i64 = assemblies::table
            .inner_join(names::table)
            .inner_join(taxa::table.on(names::id.eq(taxa::name_id)))
            .filter(taxa::kingdom.eq("Animalia"))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(Overview {
            total,
        })
    }

    pub async fn plants(&self) -> Result<Overview, Error> {
        use schema::{assemblies, names, taxa};
        let mut conn = self.pool.get().await?;

        let total: i64 = assemblies::table
            .inner_join(names::table)
            .inner_join(taxa::table.on(names::id.eq(taxa::name_id)))
            .filter(taxa::kingdom.eq("Plantae"))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(Overview {
            total,
        })
    }

    pub async fn fungi(&self) -> Result<Overview, Error> {
        use schema::{assemblies, names, taxa};
        let mut conn = self.pool.get().await?;

        let total: i64 = assemblies::table
            .inner_join(names::table)
            .inner_join(taxa::table.on(names::id.eq(taxa::name_id)))
            .filter(taxa::kingdom.eq("Fungi"))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(Overview {
            total,
        })
    }

    pub async fn sequences(&self) -> Result<Overview, Error> {
        use schema::sequences::dsl::*;
        let mut conn = self.pool.get().await?;
        let total: i64 = sequences.count().get_result(&mut conn).await?;

        Ok(Overview {
            total,
        })
    }

    pub async fn whole_genomes(&self) -> Result<Overview, Error> {
        use schema_gnl::whole_genomes::dsl::*;
        let mut conn = self.pool.get().await?;
        let total: i64 = whole_genomes.count().get_result(&mut conn).await?;

        Ok(Overview {
            total,
        })
    }

    pub async fn markers(&self) -> Result<Overview, Error> {
        use schema_gnl::markers::dsl::*;
        let mut conn = self.pool.get().await?;
        let total: i64 = markers.count().get_result(&mut conn).await?;

        Ok(Overview {
            total,
        })
    }
}
