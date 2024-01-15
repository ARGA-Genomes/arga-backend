use arga_core::models::TaxonomicStatus;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::database::extensions::classification_filters::Classification;
use crate::database::extensions::filters::with_classification;
use crate::database::{schema, schema_gnl};
use crate::http::Error;

use super::PgPool;


const ACCEPTED_NAMES: [TaxonomicStatus; 6] = [
    TaxonomicStatus::Accepted,
    TaxonomicStatus::Undescribed,
    TaxonomicStatus::SpeciesInquirenda,
    TaxonomicStatus::ManuscriptName,
    TaxonomicStatus::Hybrid,
    TaxonomicStatus::Informal,
];


pub struct Overview {
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
        use schema_gnl::taxa_filter::dsl::*;
        let mut conn = self.pool.get().await?;

        let total: i64 = taxa_filter
            .filter(status.eq_any(ACCEPTED_NAMES))
            .filter(with_classification(&Classification::Kingdom("Animalia".to_string())))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(Overview {
            total,
        })
    }

    pub async fn plants(&self) -> Result<Overview, Error> {
        use schema_gnl::taxa_filter::dsl::*;
        let mut conn = self.pool.get().await?;

        let total: i64 = taxa_filter
            .filter(status.eq_any(ACCEPTED_NAMES))
            .filter(with_classification(&Classification::Kingdom("Plantae".to_string())))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(Overview {
            total,
        })
    }

    pub async fn fungi(&self) -> Result<Overview, Error> {
        use schema_gnl::taxa_filter::dsl::*;
        let mut conn = self.pool.get().await?;

        let total: i64 = taxa_filter
            .filter(status.eq_any(ACCEPTED_NAMES))
            .filter(with_classification(&Classification::Kingdom("Fungi".to_string())))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(Overview {
            total,
        })
    }

    pub async fn bacteria(&self) -> Result<Overview, Error> {
        use schema_gnl::taxa_filter::dsl::*;
        let mut conn = self.pool.get().await?;

        let total: i64 = taxa_filter
            .filter(status.eq_any(ACCEPTED_NAMES))
            .filter(with_classification(&Classification::Kingdom("Bacteria".to_string())))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(Overview {
            total,
        })
    }

    pub async fn protista(&self) -> Result<Overview, Error> {
        use schema_gnl::taxa_filter::dsl::*;
        let mut conn = self.pool.get().await?;

        let total: i64 = taxa_filter
            .filter(status.eq_any(ACCEPTED_NAMES))
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
}
