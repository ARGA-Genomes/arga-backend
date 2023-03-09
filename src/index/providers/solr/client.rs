use tracing::debug;
use url::Url;
use serde::{Deserialize, de::DeserializeOwned};

use super::Error;


#[derive(Deserialize)]
struct SolrResult<T> {
    response: T,
}

#[derive(Deserialize)]
struct SolrGroupResult<T> {
    grouped: T,
}

#[derive(Deserialize)]
struct SolrFacetResult<T, R> {
    response: T,
    facet_counts: FacetCounts<R>,
}

#[derive(Deserialize)]
struct FacetCounts<R> {
    facet_pivot: R,
}


#[derive(Clone)]
pub struct SolrClient {
    host: String,
    client: reqwest::Client,
}

impl SolrClient {
    pub fn new(host: &str) -> SolrClient {
        SolrClient {
            host: String::from(host),
            client: reqwest::Client::new(),
        }
    }

    // TODO: this would probably be better as a builder to get some type safety
    // with the params but for now its easier to build more complex queries without
    // having to extend the client
    pub async fn select<'a, T>(&self, params: &Vec<(&str, &str)>) -> Result<T, Error>
        where T: DeserializeOwned + std::fmt::Debug
    {
        let grouped = params.contains(&("group", "true"));
        debug!(grouped);

        let base_url = format!("{}/select", self.host);
        let url = Url::parse_with_params(&base_url, params).unwrap();
        debug!(url = url.as_str());

        let resp = self.client.get(url.as_str()).send().await?;

        if grouped {
            let json = resp.json::<SolrGroupResult<T>>().await?;
            Ok(json.grouped)
        } else {
            let json = resp.json::<SolrResult<T>>().await?;
            Ok(json.response)
        }
    }

    pub async fn select_faceted<'a, T, R>(&self, params: &Vec<(&str, &str)>) -> Result<(T, R), Error>
        where T: DeserializeOwned + std::fmt::Debug,
              R: DeserializeOwned + std::fmt::Debug
    {
        let base_url = format!("{}/select", self.host);
        let url = Url::parse_with_params(&base_url, params).unwrap();
        debug!(url = url.as_str());

        let resp = self.client.get(url.as_str()).send().await?;

        let json = resp.json::<SolrFacetResult<T, R>>().await?;
        Ok((json.response, json.facet_counts.facet_pivot))
    }
}
