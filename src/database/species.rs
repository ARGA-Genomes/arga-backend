use async_trait::async_trait;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel::Queryable;
use tracing::instrument;
use uuid::Uuid;

use crate::index::species::{self, GetSpecies, Taxonomy, GetRegions, GetMedia, GetSpecimens, GetConservationStatus, GetTraceFiles};
use super::{schema, Database, Error};
use super::models::{Name, UserTaxon, RegionType, TaxonPhoto, Specimen, TraceFile, ConservationStatus};


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
        use schema::{user_taxa, user_taxa_lists};
        let mut conn = self.pool.get().await?;

        let taxon = user_taxa::table
            .inner_join(user_taxa_lists::table)
            .select(user_taxa::all_columns)
            .filter(user_taxa::name_id.eq(name.id))
            .order(user_taxa_lists::priority)
            .first::<UserTaxon>(&mut conn)
            .await?;

        let taxonomy = Taxonomy {
            vernacular_group: derive_vernacular_group(&taxon),
            scientific_name: taxon.scientific_name.unwrap_or_else(|| name.scientific_name.clone()),
            canonical_name: taxon.canonical_name,
            authorship: taxon.scientific_name_authorship,
            kingdom: taxon.kingdom,
            phylum: taxon.phylum,
            class: taxon.class,
            order: taxon.order,
            family: taxon.family,
            genus: taxon.genus,
        };
        Ok(taxonomy)
    }

    #[instrument(skip(self))]
    async fn taxa(&self, names: &Vec<Name>) -> Result<Vec<Taxonomy>, Error> {
        use schema::{user_taxa, user_taxa_lists};
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let records = user_taxa::table
            .inner_join(user_taxa_lists::table)
            .select(user_taxa::all_columns)
            .filter(user_taxa::name_id.eq_any(name_ids))
            .order(user_taxa_lists::priority)
            .load::<UserTaxon>(&mut conn)
            .await?;

        let mut taxa = Vec::with_capacity(records.len());
        for taxon in records {
            taxa.push(Taxonomy {
                vernacular_group: derive_vernacular_group(&taxon),
                scientific_name: taxon.scientific_name.unwrap_or_default(),
                canonical_name: taxon.canonical_name,
                authorship: taxon.scientific_name_authorship,
                kingdom: taxon.kingdom,
                phylum: taxon.phylum,
                class: taxon.class,
                order: taxon.order,
                family: taxon.family,
                genus: taxon.genus,
            })
        }

        Ok(taxa)
    }

    async fn distribution(&self, name: &str) -> Result<Vec<species::Distribution>, Error> {
        use schema::taxa::dsl::{taxa, canonical_name, taxon_id as taxa_taxon_id};
        use schema::distribution::dsl::*;
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
        use schema::regions;
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
        use schema::regions;
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
        use schema::taxon_photos::dsl::*;
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
        use schema::specimens::dsl::*;
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
                institution_code: record.institution_code,
                collection_code: record.collection_code,
                catalog_number: record.catalog_number,
                recorded_by: record.recorded_by,
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
        use schema::conservation_statuses::dsl::*;
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


#[async_trait]
impl GetTraceFiles for Database {
    type Error = Error;

    async fn trace_files(&self, names: &Vec<Name>) -> Result<Vec<species::TraceFile>, Error> {
        use schema::trace_files::dsl::*;
        let mut conn = self.pool.get().await?;

        let name_ids: Vec<Uuid> = names.iter().map(|n| n.id).collect();

        let records = trace_files
            .filter(name_id.eq_any(name_ids))
            .load::<TraceFile>(&mut conn)
            .await?;

        let records = records.into_iter().map(|r| r.into()).collect();
        Ok(records)
    }
}

impl From<TraceFile> for species::TraceFile {
    fn from(value: TraceFile) -> Self {
        Self {
            id: value.id.to_string(),
            metadata: value.metadata,

            peak_locations_user: value.peak_locations_user.map(from_int_array),
            peak_locations_basecaller: value.peak_locations_basecaller.map(from_int_array),
            quality_values_user: value.quality_values_user.map(from_int_array),
            quality_values_basecaller: value.quality_values_basecaller.map(from_int_array),
            sequences_user: value.sequences_user.map(from_int_array),
            sequences_basecaller: value.sequences_basecaller.map(from_int_array),

            measurements_voltage: value.measurements_voltage.map(from_int_array),
            measurements_current: value.measurements_current.map(from_int_array),
            measurements_power: value.measurements_power.map(from_int_array),
            measurements_temperature: value.measurements_temperature.map(from_int_array),

            analyzed_g: value.analyzed_g.map(from_int_array),
            analyzed_a: value.analyzed_a.map(from_int_array),
            analyzed_t: value.analyzed_t.map(from_int_array),
            analyzed_c: value.analyzed_c.map(from_int_array),

            raw_g: value.raw_g.map(from_int_array),
            raw_a: value.raw_a.map(from_int_array),
            raw_t: value.raw_t.map(from_int_array),
            raw_c: value.raw_c.map(from_int_array),

        }
    }
}

fn from_int_array(values: Vec<Option<i32>>) -> Vec<i32> {
    values.into_iter().map(|v| v.unwrap_or_default()).collect()
}


fn derive_vernacular_group(taxon: &UserTaxon) -> Option<String> {
    match taxon.kingdom.as_ref().map(|k| k.as_str()) {
        Some("Archaea") => Some("bacteria".into()),
        Some("Bacteria") => Some("bacteria".into()),
        Some("Protozoa") => Some("protists and other unicellular organisms".into()),
        Some("Fungi") => Some("mushrooms and other fungi".into()),
        Some("Animalia") => match taxon.phylum.as_ref().map(|k| k.as_str()) {
            Some("Mollusca") => Some("molluscs".into()),
            Some("Arthropoda") => match taxon.class.as_ref().map(|k| k.as_str()) {
                Some("Insecta") => Some("insects".into()),
                _ => None,
            }
            Some("Chordata") => match taxon.class.as_ref().map(|k| k.as_str()) {
                Some("Amphibia") => Some("frogs and other amphibians".into()),
                Some("Aves") => Some("birds".into()),
                Some("Mammalia") => Some("mammals".into()),
                _ => None,
            }
            _ => None,
        }
        Some("Chromista") => Some("seaweeds and other algae".into()),
        Some("Plantae") => match taxon.phylum.as_ref().map(|k| k.as_str()) {
            Some("Rhodophyta") => Some("seaweeds and other algae".into()),
            _ => Some("higher plants".into()),
        }
        _ => None,
    }
}
