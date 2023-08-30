use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::database::schema;
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

    pub async fn genomes(&self) -> Result<Overview, Error> {
        use schema::assemblies;
        let mut conn = self.pool.get().await?;
        let total: i64 = assemblies::table.count().get_result(&mut conn).await?;

        Ok(Overview {
            total,
        })
    }

    pub async fn whole_genomes(&self) -> Result<Overview, Error> {
        use schema::assemblies::dsl::*;
        let mut conn = self.pool.get().await?;
        let total: i64 = assemblies
            .filter(genome_rep.eq("Full"))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(Overview {
            total,
        })
    }

    pub async fn partial_genomes(&self) -> Result<Overview, Error> {
        use schema::assemblies::dsl::*;
        let mut conn = self.pool.get().await?;
        let total: i64 = assemblies
            .filter(genome_rep.eq("Partial"))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(Overview {
            total,
        })
    }

    pub async fn markers(&self) -> Result<Overview, Error> {
        use schema::markers;
        let mut conn = self.pool.get().await?;
        let total: i64 = markers::table.count().get_result(&mut conn).await?;

        Ok(Overview {
            total,
        })
    }
}
