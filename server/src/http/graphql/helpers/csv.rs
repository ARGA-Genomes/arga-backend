use arga_core::models::Species;
use async_graphql::*;
use base64::Engine;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::http::graphql::taxon::RankSummary;


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

pub fn encode_to_csv_brotli_base64<T: Serialize>(records: &[T]) -> Result<String, Error> {
    // Create CSV from records
    let mut wtr = csv::Writer::from_writer(Vec::new());
    for record in records {
        wtr.serialize(record).map_err(|e| Error::new(e.to_string()))?;
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

#[derive(Debug, Serialize)]
pub struct SpeciesCsv {
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
}

#[derive(Debug, Serialize)]
pub struct RankSummaryCsv {
    pub lower_rank_total: i64,
    pub lower_rank_genomes: i64,
    pub lower_rank_genomic_data: i64,
    pub species_total: i64,
    pub species_genomes: i64,
    pub species_genomic_data: i64,
}

/// This takes a collection of Species objects, and converts them into a CSV representation that is compressed by Brotli & base64 encoded
pub async fn species(species: Vec<Species>) -> Result<String, Error> {
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
            }
        })
        .collect();

    // Encode species_csv into CSV, compress with Brotli, and encode to base64
    encode_to_csv_brotli_base64(&species_csv)
}

/// This takes two rank summaries, and converts them into a CSV representation that is compressed by Brotli & base64 encoded
pub async fn rank_summaries(rank_summary: RankSummary, species_summary: RankSummary) -> Result<String, Error> {
    // Encode species_csv into CSV, compress with Brotli, and encode to base64
    encode_to_csv_brotli_base64(&[RankSummaryCsv {
        lower_rank_total: rank_summary.total,
        lower_rank_genomes: rank_summary.genomes,
        lower_rank_genomic_data: rank_summary.genomic_data,
        species_total: species_summary.total,
        species_genomes: species_summary.genomes,
        species_genomic_data: species_summary.genomic_data,
    }])
}


/// Generic version: takes any Vec of serializable records and produces a Brotli-compressed, base64-encoded CSV string
pub async fn generic<T: Serialize>(records: Vec<T>) -> Result<String, Error> {
    // Encode records into CSV, compress with Brotli, and encode to base64
    encode_to_csv_brotli_base64(&records)
}
