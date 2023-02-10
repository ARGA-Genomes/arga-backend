use async_trait::async_trait;
use serde::Deserialize;

use crate::index::search::{Searchable, SearchResults, SearchFilterItem, SearchItem};
use super::{Solr, Error};


#[async_trait]
impl Searchable for Solr {
    type Error = Error;

    async fn filtered(&self, filters: &Vec<SearchFilterItem>) -> Result<SearchResults, Error> {
        // convert the filter items to a format that solr understands, specifically {key}:{value}
        let filters = filters.iter().map(|filter| filter_to_solr_filter(filter)).collect::<Vec<String>>();

        let mut params = vec![
            ("q", "*:*"),
            ("rows", "20"),
        ];

        // having multiple `fq` params is the same as using AND
        for filter in filters.iter() {
            params.push(("fq", filter));
        }

        tracing::debug!(?params);
        let results = self.client.select::<Results>(&params).await?;

        Ok(SearchResults {
            total: results.total,
            records: results.records,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Results {
    #[serde(rename(deserialize = "numFound"))]
    total: usize,
    #[serde(rename(deserialize = "docs"))]
    records: Vec<SearchItem>,
}


fn filter_to_solr_filter(filter: &SearchFilterItem) -> String {
    format!("{}:{}", &filter.field, &filter.value)
}
