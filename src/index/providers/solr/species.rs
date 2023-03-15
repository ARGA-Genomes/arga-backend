use async_trait::async_trait;
use serde::Deserialize;

use crate::index::species::{GetGenomicData, GenomicData};
use super::{Solr, Error};


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
}

impl From<SolrData> for GenomicData {
    fn from(source: SolrData) -> Self {
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
        }
    }
}
