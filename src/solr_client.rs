use thiserror::Error;
use serde::{de::DeserializeOwned, Deserialize};


#[derive(Error, Debug)]
pub enum Error {
    #[error("internal request error")]
    Request(#[from] reqwest::Error),
}


#[derive(Deserialize)]
struct SolrResult<T> {
    response: T,
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

    pub async fn select<'a, T>(&self, query: &str, rows: usize) -> Result<T, Error>
        where T: DeserializeOwned + std::fmt::Debug
    {
        let url = format!("{}/solr/biocache/select?q={query}&rows={rows}", self.host);
        let resp = self.client.get(url).send().await?;
        let json = resp.json::<SolrResult<T>>().await;
        Ok(json?.response)
    }
}
