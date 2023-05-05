use async_trait::async_trait;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel::Queryable;
use uuid::Uuid;

use crate::index::species::{self, GetSpecies, Taxonomy, GetRegions, GetMedia};
use crate::index::providers::db::models::Media;
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
            source: source.source,
        }
    }
}


#[async_trait]
impl GetSpecies for Database {
    type Error = Error;

    async fn taxonomy(&self, name: &str) -> Result<Taxonomy, Error> {
        use crate::schema_gnl::gnl::dsl::*;
        let mut conn = self.pool.get().await?;

        let taxon = gnl
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


#[async_trait]
impl GetRegions for Database {
    type Error = Error;

    async fn ibra(&self, name: &str) -> Result<Vec<species::Region>, Error> {
        use crate::schema_gnl::eav_arrays::dsl::{eav_arrays, entity_id, value, name as attr};
        use crate::schema_gnl::gnl::dsl::*;
        let mut conn = self.pool.get().await?;

        let regions = eav_arrays
            .inner_join(gnl.on(entity_id.eq(id)))
            .select(value)
            .filter(attr.eq("ibraRegions"))
            .filter(canonical_name.eq(name))
            .load::<Vec<Option<String>>>(&mut conn)
            .await?;

        let mut filtered = Vec::new();
        for region in regions.concat() {
            if let Some(name) = region {
                filtered.push(species::Region {
                    name,
                });
            }
        }

        filtered.sort();
        filtered.dedup();
        Ok(filtered)
    }

    async fn imcra(&self, name: &str) -> Result<Vec<species::Region>, Error> {
        use crate::schema_gnl::eav_arrays::dsl::{eav_arrays, entity_id, value, name as attr};
        use crate::schema_gnl::gnl::dsl::*;
        let mut conn = self.pool.get().await?;

        let regions = eav_arrays
            .inner_join(gnl.on(entity_id.eq(id)))
            .select(value)
            .filter(attr.eq("imcraRegions"))
            .filter(canonical_name.eq(name))
            .load::<Vec<Option<String>>>(&mut conn)
            .await?;

        let mut filtered = Vec::new();
        for region in regions.concat() {
            if let Some(name) = region {
                filtered.push(species::Region {
                    name,
                });
            }
        }

        filtered.sort();
        filtered.dedup();
        Ok(filtered)
    }
}


#[async_trait]
impl GetMedia for Database {
    type Error = Error;

    async fn photos(&self, name: &str) -> Result<Vec<species::Photo>, Error> {
        use crate::schema_gnl::eav_strings::dsl::{eav_strings, entity_id, value, name as attr};
        use crate::schema_gnl::gnl::dsl::{gnl, canonical_name, id as gnl_id};
        let mut conn = self.pool.get().await?;

        let uuids = eav_strings
            .inner_join(gnl.on(entity_id.eq(gnl_id)))
            .select(value)
            .filter(attr.eq("curatedMainImage"))
            .filter(canonical_name.eq(name))
            .load::<String>(&mut conn)
            .await?;

        let uuids = uuids.iter().map(|uuid| Uuid::parse_str(uuid).unwrap()).collect::<Vec<Uuid>>();

        use crate::schema::media::dsl::*;
        let records = media
            .filter(id.eq_any(uuids))
            .load::<Media>(&mut conn)
            .await?;

        let mut photos = Vec::with_capacity(records.len());
        for record in records {
            if let Some(url) = record.identifier {
                photos.push(species::Photo {
                    url,
                    publisher: record.publisher,
                    license: record.license,
                    rights_holder: record.rights_holder,
                    reference_url: record.references,
                })
            }
        }

        Ok(photos)
    }
}
