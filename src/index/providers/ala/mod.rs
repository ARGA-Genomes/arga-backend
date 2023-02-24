pub mod client;
pub mod search;

use thiserror::Error;

use client::AlaClient;


#[derive(Error, Debug)]
pub enum Error {
    #[error("internal request error")]
    Request(#[from] reqwest::Error),
}


#[derive(Clone)]
pub struct Ala {
    client: AlaClient,
}


impl Ala {
    pub fn new() -> Ala {
        Ala { client: AlaClient::new() }
    }
}
