use async_trait::async_trait;
use serde::Deserialize;

use crate::index::overview::{Overview, OverviewCategory};
use super::{Solr, Error};


#[async_trait]
impl Overview for Solr {
    type Error = Error;

    async fn total(&self, category: OverviewCategory) -> Result<usize, Error> {
        let query = match category {
            OverviewCategory::Animals => "kingdom:Animalia",
            OverviewCategory::Plants => "kingdom:Plantae",
            OverviewCategory::Fungi => "kingdom:Fungi",
            OverviewCategory::AgriculturalAndPest => "*:*",
            OverviewCategory::MarineAndAquaculture => r#"biome:"MARINE""#,
            OverviewCategory::AllSpecies => "*:*",
            OverviewCategory::PreservedSpecimens => r#"basisOfRecord:"PRESERVED_SPECIMEN""#,
            OverviewCategory::TerrestrialBiodiversity => r#"biome:"TERRESTRIAL""#,
            OverviewCategory::ThreatenedSpecies => "*:*",
            OverviewCategory::Genome =>  r#"dynamicProperties_ncbi_genome_rep:"Full" OR dynamicProperties_ncbi_genome_rep:"Partial" OR dataResourceName:*RefSeq*"#,
            OverviewCategory::Organelles => "*:*", //TODO: to fix once the data is ready
            OverviewCategory::Barcodes => r#"dataProviderName:"Barcode of Life""#
        };

        let params = vec![
            ("q", query),
            ("rows", "1"),
        ];

        tracing::debug!(?params);
        let results = self.client.select::<Total>(&params).await?;
        Ok(results.total)
    }
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Total {
    #[serde(rename(deserialize = "numFound"))]
    total: usize,
}
