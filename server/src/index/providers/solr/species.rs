use tracing::warn;

use async_trait::async_trait;
use serde::Deserialize;

use crate::index::species::{GetGenomicData, GenomicData, GeoCoordinates, GetWholeGenomes, WholeGenome, AssociatedSequences};
use crate::database::models::Name;
use super::{Solr, Error};


#[async_trait]
impl GetWholeGenomes for Solr {
    type Error = Error;

    async fn full_genomes(&self, names: &Vec<Name>) -> Result<Vec<WholeGenome>, Error> {
        let filter = names_into_filter(&names);
        let filter = format!("scientificName:{filter}");

        // get all species that have a matched name. this filters
        // out records that couldn't be matched by the name-matching service
        let params = vec![
            ("q", "*:*"),
            ("rows", "20"),
            ("fq", &filter),
            ("fq", "taxonRank:species"),
            ("fq", r#"dynamicProperties_ncbi_genome_rep:"Full""#),
            ("fq", r#"dataResourceName:"NCBI Genome Genbank""#),
        ];

        tracing::debug!(?params);
        let results = self.client.select::<WholeGenomeResults>(&params).await?;
        let records = results.records.into_iter().map(|s| s.into()).collect();

        Ok(records)
    }

    async fn partial_genomes(&self, names: &Vec<Name>) -> Result<Vec<WholeGenome>, Error> {
        let filter = names_into_filter(&names);
        let filter = format!("scientificName:{filter}");

        // get all species that have a matched name. this filters
        // out records that couldn't be matched by the name-matching service
        let params = vec![
            ("q", "*:*"),
            ("rows", "20"),
            ("fq", &filter),
            ("fq", "taxonRank:species"),
            ("fq", r#"dynamicProperties_ncbi_genome_rep:"Partial""#),
        ];

        tracing::debug!(?params);
        let results = self.client.select::<WholeGenomeResults>(&params).await?;
        let records = results.records.into_iter().map(|s| s.into()).collect();

        Ok(records)
    }

    async fn reference_genomes(&self, names: &Vec<Name>) -> Result<Vec<WholeGenome>, Error> {
        let filter = names_into_filter(&names);
        let filter = format!("scientificName:{filter}");

        // get all species that have a matched name. this filters
        // out records that couldn't be matched by the name-matching service
        let params = vec![
            ("q", "*:*"),
            ("rows", "20"),
            ("fq", &filter),
            ("fq", "taxonRank:species"),
            ("fq", r#"dataResourceName:"NCBI Genome RefSeq""#),
            // ("fq", r#"dynamicProperties_ncbi_genome_rep:"Full" OR dynamicProperties_ncbi_genome_rep:"Partial""#),
        ];

        tracing::debug!(?params);
        let results = self.client.select::<WholeGenomeResults>(&params).await?;
        let records = results.records.into_iter().map(|s| s.into()).collect();

        Ok(records)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WholeGenomeResults {
    #[serde(rename(deserialize = "docs"))]
    records: Vec<WholeGenomeData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WholeGenomeData {
    id: String,
    data_resource_name: Option<String>,
    recorded_by: Option<Vec<String>>,
    license: Option<String>,
    provenance: Option<String>,
    event_date: Option<String>,
    occurrence_year: Option<Vec<String>>,
    other_catalog_numbers: Option<Vec<String>>,

    #[serde(rename(deserialize = "dynamicProperties_ncbi_assembly_accession"))]
    accession: Option<String>,
    #[serde(rename(deserialize = "dynamicProperties_geneticAccessionURI"))]
    accession_uri: Option<String>,
    #[serde(rename(deserialize = "dynamicProperties_ncbi_refseq_category"))]
    refseq_category: Option<String>,

    #[serde(rename(deserialize = "dynamicProperties_ncbi_nuccore"))]
    ncbi_nuccore: Option<String>,
    #[serde(rename(deserialize = "dynamicProperties_ncbi_bioproject"))]
    ncbi_bioproject: Option<String>,
    #[serde(rename(deserialize = "dynamicProperties_ncbi_biosample"))]
    ncbi_biosample: Option<String>,
    #[serde(rename(deserialize = "dynamicProperties_MIXS_0000005"))]
    mixs_0000005: Option<String>,
    #[serde(rename(deserialize = "dynamicProperties_MIXS_0000029"))]
    mixs_0000029: Option<String>,
    #[serde(rename(deserialize = "dynamicProperties_MIXS_0000026"))]
    mixs_0000026: Option<String>,

    #[serde(rename(deserialize = "dynamicProperties_ncbi_paired_asm_comp"))]
    paired_asm_comp: Option<String>,

    #[serde(rename(deserialize = "raw_recordedBy"))]
    raw_recorded_by: Option<String>,
    #[serde(rename(deserialize = "dynamicProperties_ncbi_release_type"))]
    ncbi_release_type: Option<String>,

    #[serde(rename(deserialize = "dynamicProperties_ncbi_genome_rep"))]
    ncbi_sequence_type: Option<String>,

    #[serde(rename(deserialize = "lat_long"))]
    lat_long: Option<String>,
}

impl From<WholeGenomeData> for WholeGenome {
    fn from(source: WholeGenomeData) -> Self {
        // coordinates are indexed as a string so we parse it and convert
        // to an f32 tuple for easier consumption
        let coordinates = match source.lat_long {
            None => None,
            // log parse failures but dont consider it a request error
            Some(value) => match parse_coordinates(&value) {
                Ok(coordinates) => coordinates,
                Err(error) => {
                    warn!(value, ?error, "failed to parse lat lng");
                    None
                }
            }
        };

        Self {
            is_reference_sequence: source.refseq_category == Some("representative genome".to_string()),

            id: source.id,
            r#type: source.ncbi_sequence_type,
            data_resource: source.data_resource_name,
            recorded_by: source.recorded_by,
            license: source.license,
            provenance: source.provenance,
            event_date: source.event_date,
            occurrence_year: source.occurrence_year,
            other_catalog_numbers: source.other_catalog_numbers,
            accession: source.accession,
            accession_uri: source.accession_uri,
            refseq_category: source.refseq_category,
            ncbi_nuccore: source.ncbi_nuccore,
            ncbi_bioproject: source.ncbi_bioproject,
            ncbi_biosample: source.ncbi_biosample,
            mixs_0000005: source.mixs_0000005,
            mixs_0000026: source.mixs_0000026,
            mixs_0000029: source.mixs_0000029,
            paired_asm_comp: source.paired_asm_comp,
            raw_recorded_by: source.raw_recorded_by,
            ncbi_release_type: source.ncbi_release_type,
            coordinates,
        }
    }
}


#[async_trait]
impl GetGenomicData for Solr {
    type Error = Error;

    async fn genomic_data(&self, canonical_name: &str) -> Result<Vec<GenomicData>, Error> {
        // TODO: although this isn't user input its possible that it does get
        // used like that at some point. it would be good to determine what kind
        // of sanitation solr needs, if any
        let filter = &format!(r#"raw_scientificName:"{canonical_name}""#);

        let params = vec![
            ("q", "*:*"),
            ("rows", "20"),
            ("fq", filter),
        ];

        tracing::debug!(?params);
        let results = self.client.select::<Results>(&params).await?;
        let data = results.records.into_iter().map(|s| s.into()).collect();
        Ok(data)
    }
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Results {
    #[serde(rename(deserialize = "docs"))]
    records: Vec<SolrData>,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SolrData {
    #[serde(rename(deserialize = "raw_scientificName"))]
    raw_scientific_name: Option<String>,
    basis_of_record: Option<String>,
    data_resource_name: Option<String>,
    recorded_by: Option<Vec<String>>,
    license: Option<String>,
    provenance: Option<String>,
    event_date: Option<String>,

    #[serde(rename(deserialize = "dynamicProperties_ncbi_assembly_accession"))]
    accession: Option<String>,
    #[serde(rename(deserialize = "dynamicProperties_geneticAccessionURI"))]
    accession_uri: Option<String>,
    #[serde(rename(deserialize = "dynamicProperties_ncbi_refseq_category"))]
    refseq_category: Option<String>,

    #[serde(rename(deserialize = "lat_long"))]
    lat_long: Option<String>,

    #[serde(rename(deserialize = "associatedSequences"))]
    associated_sequences: Option<String>,
}

impl From<SolrData> for GenomicData {
    fn from(source: SolrData) -> Self {
        // coordinates are indexed as a string so we parse it and convert
        // to an f32 tuple for easier consumption
        let coordinates = match source.lat_long {
            None => None,
            // log parse failures but dont consider it a request error
            Some(value) => match parse_coordinates(&value) {
                Ok(coordinates) => coordinates,
                Err(error) => {
                    warn!(value, ?error, "failed to parse lat lng");
                    None
                }
            }
        };

        let associated_sequences = match source.associated_sequences {
            None => None,
            Some(value) => match parse_associated_sequences(&value) {
                Ok(associated_sequences) => associated_sequences,
                Err(error) => {
                    warn!(value, ?error, "failed to parse");
                    None
                }
            }
        };

        Self {
            canonical_name: source.raw_scientific_name,
            r#type: source.basis_of_record,
            data_resource: source.data_resource_name,
            recorded_by: source.recorded_by,
            license: source.license,
            provenance: source.provenance,
            event_date: source.event_date,
            accession: source.accession,
            accession_uri: source.accession_uri,
            refseq_category: source.refseq_category,
            coordinates,
            associated_sequences
        }
    }
}

fn parse_coordinates(value: &str) -> Result<Option<GeoCoordinates>, anyhow::Error> {
    let coordinates: Vec<String> = Vec::from_iter(value.split(",").map(|s| s.to_string()));

    if coordinates.len() == 2 {
        Ok(Some(GeoCoordinates {
            latitude: coordinates[0].parse::<f32>()?,
            longitude: coordinates[1].parse::<f32>()?,
        }))
    } else {
        Ok(None)
    }
}

fn parse_associated_sequences(value: &str) -> Result<Option<AssociatedSequences>, anyhow::Error> {

    if value.contains("sequenceID") {

        let json_string = value.replace(r#"'"#, r#"""#);
        let list: [AssociatedSequences;1] = serde_json::from_str(&*json_string)?;
        let associated_sequences = list[0].clone();

        if !json_string.is_empty() {
            Ok(Some(AssociatedSequences {
                sequence_id: associated_sequences.sequence_id,
                genbank_accession: associated_sequences.genbank_accession,
                markercode: associated_sequences.markercode,
                nucleotides: associated_sequences.nucleotides,
            }))
        } else {
            Ok(None)
        }
    }else {
        Ok(None)
    }
}


fn names_into_filter(names: &Vec<Name>) -> String {
    // craft a single filter by joining them all with OR since the default
    // will treat it as an AND query
    let names = names.into_iter().map(|name| {
        format!("\"{}\"", name.canonical_name.clone().unwrap_or_else(|| name.scientific_name.clone()))
    }).collect::<Vec<String>>();
    names.join(" OR ")
}
