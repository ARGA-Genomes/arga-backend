use async_trait::async_trait;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel::Queryable;
use tracing::instrument;

use crate::index::species::{self, GetSpecies, Taxonomy, GetRegions, GetMedia, GetSpecimens, GetConservationStatus};
use crate::index::providers::db::models::{Name, UserTaxon, RegionType, TaxonPhoto, Specimen};
use super::models::ConservationStatus;
use super::{Database, Error};


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

    #[instrument(skip(self))]
    async fn taxonomy(&self, name: &Name) -> Result<Taxonomy, Error> {
        let mut conn = self.pool.get().await?;

        let mut taxonomy = Taxonomy {
            scientific_name: name.scientific_name.clone(),
            canonical_name: name.canonical_name.clone(),
            authorship: name.authorship.clone(),
            ..Default::default()
        };

        use crate::schema::user_taxa;
        let taxa = user_taxa::table
            .filter(user_taxa::name_id.eq(name.id))
            .load::<UserTaxon>(&mut conn)
            .await?;

        for taxon in taxa {
            taxonomy.kingdom = taxon.kingdom;
            taxonomy.phylum = taxon.phylum;
            taxonomy.class = taxon.class;
            taxonomy.order = taxon.order;
            taxonomy.family = taxon.family;
            taxonomy.genus = taxon.genus;
        }

        Ok(taxonomy)
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

    async fn ibra(&self, name: &Name) -> Result<Vec<species::Region>, Error> {
        use crate::schema::regions;
        let mut conn = self.pool.get().await?;

        let regions = regions::table
            .select(regions::values)
            .filter(regions::name_id.eq(name.id))
            .filter(regions::region_type.eq(RegionType::Ibra))
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

    async fn imcra(&self, name: &Name) -> Result<Vec<species::Region>, Error> {
        use crate::schema::regions;
        let mut conn = self.pool.get().await?;

        let regions = regions::table
            .select(regions::values)
            .filter(regions::name_id.eq(name.id))
            .filter(regions::region_type.eq(RegionType::Imcra))
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

    async fn photos(&self, name: &Name) -> Result<Vec<species::Photo>, Error> {
        use crate::schema::taxon_photos::dsl::*;
        let mut conn = self.pool.get().await?;

        let records = taxon_photos
            .filter(name_id.eq(name.id))
            .load::<TaxonPhoto>(&mut conn)
            .await?;

        let mut photos = Vec::with_capacity(records.len());
        for record in records {
            photos.push(species::Photo {
                url: record.url,
                publisher: record.publisher,
                license: record.license,
                rights_holder: record.rights_holder,
                reference_url: record.source,
            });
        }

        Ok(photos)
    }
}


#[async_trait]
impl GetSpecimens for Database {
    type Error = Error;

    async fn specimens(&self, name: &Name) -> Result<Vec<species::Specimen>, Error> {
        use crate::schema::specimens::dsl::*;
        let mut conn = self.pool.get().await?;

        let records = specimens
            .filter(name_id.eq(name.id))
            .load::<Specimen>(&mut conn)
            .await?;

        let mut results = Vec::with_capacity(records.len());
        for record in records {
            results.push(species::Specimen {
                type_status: record.type_status,
                institution_name: record.institution_name,
                organism_id: record.organism_id,
                locality: record.locality,
                latitude: record.latitude,
                longitude: record.longitude,
                details: record.details,
                remarks: record.remarks,
            });
        }

        Ok(results)
    }
}


#[async_trait]
impl GetConservationStatus for Database {
    type Error = Error;

    async fn conservation_status(&self, name: &Name) -> Result<Vec<species::ConservationStatus>, Error> {
        use crate::schema::conservation_statuses::dsl::*;
        let mut conn = self.pool.get().await?;

        let records = conservation_statuses
            .filter(name_id.eq(name.id))
            .load::<ConservationStatus>(&mut conn)
            .await?;

        let records = records.into_iter().map(|r| r.into()).collect();
        Ok(records)
    }
}

impl From<ConservationStatus> for species::ConservationStatus {
    fn from(value: ConservationStatus) -> Self {
        Self {
            status: value.status,
            state: value.state,
            source: value.source,
        }
    }
}
