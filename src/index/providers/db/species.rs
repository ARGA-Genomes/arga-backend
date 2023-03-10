use async_trait::async_trait;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel::Queryable;

use crate::index::species::{self, GetSpecies, Taxonomy};
use super::{Database, Error, Taxon};


#[derive(Queryable, Debug)]
struct Distribution {
    pub locality: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub threat_status: Option<String>,
    pub source: Option<String>,
}

impl From<Distribution> for species::Distribution {
    fn from(source: Distribution) -> Self {
        Self {
            locality: source.locality,
            country: source.country,
            country_code: source.country_code,
            threat_status: source.threat_status,
            source: source.source
        }
    }
}


#[async_trait]
impl GetSpecies for Database {
    type Error = Error;

    async fn taxonomy(&self, name: &str) -> Result<Taxonomy, Error> {
        use crate::schema::taxa::dsl::*;
        let mut conn = self.pool.get().await?;

        let taxon = taxa
            .select((
                scientific_name_authorship,
                canonical_name,
                kingdom,
                phylum,
                class,
                order,
                family,
                genus,
            ))
            .filter(taxon_rank.eq("species"))
            .filter(canonical_name.eq(name))
            .first::<Taxon>(&mut conn).await?;

        Ok(Taxonomy::from(taxon))
    }

    async fn distribution(&self, name: &str) -> Result<Vec<species::Distribution>, Error> {
        use crate::schema::taxa::dsl::{taxa, canonical_name, taxon_id as taxa_taxon_id};
        use crate::schema::distribution::dsl::*;
        let mut conn = self.pool.get().await?;

        let rows = distribution
            .inner_join(taxa.on(taxon_id.eq(taxa_taxon_id)))
            .select((
                locality,
                country,
                country_code,
                threat_status,
                source,
            ))
            .filter(canonical_name.eq(name))
            .load::<Distribution>(&mut conn).await?;

        let dist = rows.into_iter().map(|r| r.into()).collect();
        Ok(dist)
    }
}


