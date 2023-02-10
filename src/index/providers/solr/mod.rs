pub mod client;
pub mod filters;

use thiserror::Error;

use client::SolrClient;


#[derive(Error, Debug)]
pub enum Error {
    #[error("internal request error")]
    Request(#[from] reqwest::Error),
}


#[derive(Clone)]
pub struct Solr {
    client: SolrClient,
}


impl Solr {
    pub fn new(client: SolrClient) -> Solr {
        Solr { client }
    }
}

