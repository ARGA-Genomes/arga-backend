use tracing::debug;
use url::Url;
use serde::{Deserialize, de::DeserializeOwned};

use super::Error;


static HOST: &str = "https://apis.ala.org.au/species";


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AlaResult<T> {
    search_results: T,
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
}
