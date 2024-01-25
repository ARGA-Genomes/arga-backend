use arga_core::models::{TaxonomicStatus, ACCEPTED_NAMES};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;

use crate::database::extensions::classification_filters::Classification;
use crate::database::extensions::species_filters::with_classification;
use crate::database::{schema, schema_gnl};
use crate::http::Error;

use super::PgPool;


pub struct Overview {
    pub total: i64,
}

#[derive(Debug, Queryable, Clone, Deserialize)]
pub struct SourceOverview {
    pub id: uuid::Uuid,
    pub name: String,
    pub total: i64,
}


#[derive(Clone)]
pub struct OverviewProvider {
    pub pool: PgPool,
}

impl OverviewProvider {
    pub async fn all_species(&self) -> Result<Overview, Error> {
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;
        let total: i64 = species.count().get_result(&mut conn).await?;
        Ok(Overview { total })
    }

    pub async fn animals(&self) -> Result<Overview, Error> {
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;

        let total: i64 = species
            .filter(taxon_status.eq_any(ACCEPTED_NAMES))
            .filter(with_classification(&Classification::Kingdom("Animalia".to_string())))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(Overview {
            total,
        })
    }

    pub async fn plants(&self) -> Result<Overview, Error> {
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;

        let total: i64 = species
            .filter(taxon_status.eq_any(ACCEPTED_NAMES))
            .filter(with_classification(&Classification::Kingdom("Plantae".to_string())))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(Overview {
            total,
        })
    }

    pub async fn fungi(&self) -> Result<Overview, Error> {
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;

        let total: i64 = species
            .filter(taxon_status.eq_any(ACCEPTED_NAMES))
            .filter(with_classification(&Classification::Kingdom("Fungi".to_string())))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(Overview {
            total,
        })
    }

    pub async fn bacteria(&self) -> Result<Overview, Error> {
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;

        let total: i64 = species
            .filter(taxon_status.eq_any(ACCEPTED_NAMES))
            .filter(with_classification(&Classification::Kingdom("Bacteria".to_string())))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(Overview {
            total,
        })
    }

    pub async fn protista(&self) -> Result<Overview, Error> {
        use schema_gnl::species::dsl::*;
        let mut conn = self.pool.get().await?;

        let total: i64 = species
            .filter(taxon_status.eq_any(ACCEPTED_NAMES))
            .filter(with_classification(&Classification::Superkingdom("Protista".to_string())))
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

    pub async fn loci(&self) -> Result<Overview, Error> {
        use schema_gnl::markers::dsl::*;
        let mut conn = self.pool.get().await?;
        let total: i64 = markers.count().get_result(&mut conn).await?;

        Ok(Overview {
            total,
        })
    }

    pub async fn specimens(&self) -> Result<Overview, Error> {
        use schema::specimens::dsl::*;
        let mut conn = self.pool.get().await?;
        let total: i64 = specimens.count().get_result(&mut conn).await?;

        Ok(Overview {
            total,
        })
    }

    pub async fn sources(&self) -> Result<Vec<SourceOverview>, Error> {
        use schema::{sources, datasets, name_attributes};
        use diesel::dsl::count_star;

        let mut conn = self.pool.get().await?;

        let records = name_attributes::table
            .inner_join(datasets::table)
            .inner_join(sources::table.on(datasets::source_id.eq(sources::id)))
            .group_by((sources::id, sources::name))
            .select((sources::id, sources::name, count_star()))
            .filter(name_attributes::name.eq("last_updated"))
            .load::<SourceOverview>(&mut conn)
            .await?;

        Ok(records)
    }
}
