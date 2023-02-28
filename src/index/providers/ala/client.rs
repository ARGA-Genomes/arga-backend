use tracing::debug;
use url::Url;
use serde::{Deserialize, de::DeserializeOwned};

use super::Error;


static HOST: &str = "https://api.ala.org.au/species";


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AlaResult<T> {
    search_results: T,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AlaAutoCompleteResult<T> {
    auto_complete_list: T,
}

#[derive(Clone)]
pub struct AlaClient {
    client: reqwest::Client,
}

impl AlaClient {
    pub fn new() -> AlaClient {
        AlaClient {
            client: reqwest::Client::new(),
        }
    }

    pub async fn search<T>(&self, params: &Vec<(&str, &str)>) -> Result<T, Error>
        where T: DeserializeOwned + std::fmt::Debug
    {
        let base_url = format!("{}/search", HOST);
        let url = Url::parse_with_params(&base_url, params).unwrap();
        debug!(url = url.as_str());

        let resp = self.client.get(url.as_str()).send().await?;
        let json = resp.json::<AlaResult<T>>().await?;
        Ok(json.search_results)
    }

    #[tracing::instrument(skip(self))]
    pub async fn auto<T>(&self, query: &str, idx_type: &str, limit: usize) -> Result<T, Error>
        where T: DeserializeOwned + std::fmt::Debug
    {
        let limit = limit.to_string();
        let params = vec![
            ("q", query),
            ("idxType", idx_type),
            ("limit", &limit),
        ];

        let base_url = format!("{}/search/auto", HOST);
        let url = Url::parse_with_params(&base_url, params).unwrap();
        debug!(url = url.as_str());

        let json = self.get::<AlaAutoCompleteResult<T>>(url.as_str()).await?;
        Ok(json.auto_complete_list)
    }


    #[tracing::instrument(skip(self))]
    async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T, Error> {
        let resp = {
            let span = tracing::span!(tracing::Level::TRACE, "http");
            let _enter = span.enter();
            self.client.get(url).send().await?
        };
        {
            let span = tracing::span!(tracing::Level::TRACE, "json_parsing");
            let _enter = span.enter();
            let json = resp.json::<T>().await?;
            Ok(json)
        }
    }
}
