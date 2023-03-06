use async_trait::async_trait;
use uuid::Uuid;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel::Queryable;

use crate::index::species::{self, Species, Taxonomy};
use super::{Database, Error};


#[derive(Queryable, Debug)]
struct Taxon {
    scientific_name_authorship: Option<String>,
    canonical_name: Option<String>,
    kingdom: Option<String>,
    phylum: Option<String>,
    class: Option<String>,
    order: Option<String>,
    family: Option<String>,
    genus: Option<String>,
}

impl From<Taxon> for Taxonomy {
    fn from(source: Taxon) -> Self {
        Self {
            canonical_name: source.canonical_name,
            authorship: source.scientific_name_authorship,

            kingdom: source.kingdom,
            phylum: source.phylum,
            class: source.class,
            order: source.order,
            family: source.family,
            genus: source.genus,
        }
    }
}


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
impl Species for Database {
    type Error = Error;

    async fn taxonomy(&self, taxon_uuid: Uuid) -> Result<Taxonomy, Error> {
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
            .filter(id.eq(taxon_uuid))
            .first::<Taxon>(&mut conn).await?;

        Ok(Taxonomy::from(taxon))
    }

    async fn distribution(&self, taxon_uuid: Uuid) -> Result<Vec<species::Distribution>, Error> {
        use crate::schema::taxa::dsl::{taxa, id as taxa_id, taxon_id as taxa_taxon_id};
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
            .filter(taxa_id.eq(taxon_uuid))
            .load::<Distribution>(&mut conn).await?;

        let dist = rows.into_iter().map(|r| r.into()).collect();
        Ok(dist)
    }
}


