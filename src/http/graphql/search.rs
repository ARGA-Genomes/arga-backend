use async_graphql::*;
use serde::{Serialize, Deserialize};

use crate::http::Error;
use crate::http::Context as State;


pub struct Search;

#[Object]
impl Search {
    /// Returns the amount of preserved specimens in the index
    async fn with_kingdom(&self, ctx: &Context<'_>, kingdom: String) -> Result<SearchResults, Error> {
        let state = ctx.data::<State>().unwrap();
        search_query(&format!(r#"kingdom:"{kingdom}""#), state).await
    }
}

async fn search_query(query: &str, state: &State) -> Result<SearchResults, Error> {
    match state.solr.select::<SearchResults>(&query, 20).await {
        Ok(results) => Ok(results),
        Err(e) => {
            let err = Err(crate::http::Error::Solr(e));
            tracing::error!(?err);
            err
        }
    }
}


#[derive(Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    #[serde(rename(deserialize = "numFound"))]
    total: usize,
    #[serde(rename(deserialize = "docs"))]
    records: Vec<SearchItem>,
}

#[derive(Debug, SimpleObject, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchItem {
    id: String,

    /// The scientific name given to this taxon
    scientific_name: Option<String>,
    /// The taxonomic genus
    genus: Option<String>,
    /// The taxonomic sub genus
    subgenus: Option<String>,
    /// The taxonomic kingdom
    kingdom: Option<String>,
    /// The taxonomic phylum
    phylum: Option<String>,
    /// The taxonomic family
    family: Option<String>,
    /// The taxonomic class
    class: Option<String>,

    species_group: Option<Vec<String>>,
    species_subgroup: Option<Vec<String>>,
    biome: Option<String>,

    event_date: Option<String>,
    event_time: Option<String>,
    license: Option<String>,

    recorded_by: Option<Vec<String>>,
    identified_by: Option<Vec<String>>,
}
