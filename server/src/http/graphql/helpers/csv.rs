use arga_core::models::Species;
use async_graphql::*;
use base64::Engine;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;


#[derive(Debug, Serialize)]
pub struct ClassificationCsv {
    pub kingdom: Option<String>,
    pub phylum: Option<String>,
    pub class: Option<String>,
    pub order: Option<String>,
    pub family: Option<String>,
    pub genus: Option<String>,
}

pub fn normalize_classification(json: Value) -> ClassificationCsv {
    // Helper function: Given a list of possible keys, return the first found string value.
    fn get_field(val: &Value, keys: &[&str]) -> Option<String> {
        for &key in keys {
            if let Some(field_value) = val.get(key) {
                if let Some(s) = field_value.as_str() {
                    return Some(s.to_string());
                }
            }
        }
        // Return nothing if none of the keys are found.
        None
    }

    ClassificationCsv {
        kingdom: get_field(&json, &["regnum", "kingdom"]),
        phylum: get_field(&json, &["division", "phylum"]),
        class: get_field(&json, &["classis", "class"]),
        order: get_field(&json, &["ordo", "order"]),
        family: get_field(&json, &["familia", "family"]),
        genus: get_field(&json, &["genus"]),
    }
}

#[derive(Debug, Serialize)]
pub struct SpeciesCsv<'a> {
    pub id: Uuid,
    pub scientific_name: String,
    pub canonical_name: String,
    pub vernacular_names: String,
    pub authorship: String,
    pub status: String,
    pub rank: String,
    pub genomes: i64,
    pub loci: i64,
    pub specimens: i64,
    pub libraries: i64,
    pub total_genomic: i64,

    // Classification
    pub kingdom: Option<String>,
    pub phylum: Option<String>,
    pub class: Option<String>,
    pub order: Option<String>,
    pub family: Option<String>,
    pub genus: Option<String>,

    // Meta
    pub date_downloaded: &'a str,
}

/// This takes a collection of Species objects, and converts them into a CSV representation that is compressed by Brotli & base64 encoded
pub async fn species(species: Vec<Species>) -> Result<String, Error> {
    let date_downloaded = chrono::prelude::Local::now().to_string();

    let species_csv: Vec<SpeciesCsv> = species
        .into_iter()
        .map(|s| {
            let classification = normalize_classification(s.classification);

            SpeciesCsv {
                id: s.id,
                scientific_name: s.scientific_name,
                canonical_name: s.canonical_name,
                vernacular_names: s.vernacular_names.unwrap_or(vec![]).join(","),
                authorship: s.authorship.unwrap_or(String::from(",")),
                status: s.status.to_string(),
                rank: s.rank.to_string(),
                genomes: s.genomes,
                loci: s.loci,
                specimens: s.specimens,
                libraries: s.other,
                total_genomic: s.total_genomic,

                // Classification
                kingdom: classification.kingdom,
                phylum: classification.phylum,
                class: classification.class,
                order: classification.order,
                family: classification.family,
                genus: classification.genus,

                // Other meta
                date_downloaded: &date_downloaded,
            }
        })
        .collect();

    // Create CSV from species vector
    let mut wtr = csv::Writer::from_writer(vec![]);
    for sp in species_csv {
        wtr.serialize(sp).map_err(|e| Error::new(e.to_string()))?;
    }
    let csv_bytes = wtr.into_inner().map_err(|e| Error::new(e.to_string()))?;

    // Compress CSV bytes using Brotli
    use std::io::Write;
    let mut encoder = brotli2::write::BrotliEncoder::new(Vec::new(), 5);
    encoder.write_all(&csv_bytes).map_err(|e| Error::new(e.to_string()))?;
    let compressed_bytes = encoder.finish().map_err(|e| Error::new(e.to_string()))?;

    // Encode the compressed data as base64
    let encoded_string = base64::prelude::BASE64_STANDARD.encode(&compressed_bytes);
    Ok(encoded_string)
}
