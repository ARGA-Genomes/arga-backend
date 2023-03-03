use thiserror;
use serde::Deserialize;
use serde::Serialize;

use heck::ToShoutySnakeCase;


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to get the feature flag environment variable")]
    EnvVar(#[from] std::env::VarError)
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Features {
    OpenTelemetry
}

impl From<Features> for String {
    fn from(source: Features) -> Self {
        match source {
            Features::OpenTelemetry => "OpenTelemetry",
        }.to_string()
    }
}



#[derive(Debug, Clone)]
pub struct FeatureClient;


impl FeatureClient {
    pub fn new() -> FeatureClient {
        FeatureClient {}
    }

    pub fn is_enabled(&self, feature: Features) -> Result<bool, Error> {
        let var_name = String::from(feature).to_shouty_snake_case();
        Ok(std::env::var(var_name)? == "1")
    }
}
