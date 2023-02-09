use axum::async_trait;
use async_graphql::SimpleObject;
use serde::{Serialize, Deserialize};


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct TaxonomyFilters {
    pub kingdom: Option<Filter>,
    pub phylum: Option<Filter>,
    pub class: Option<Filter>,
    pub family: Option<Filter>,
    pub genus: Option<Filter>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct Filter {
    pub total_matches: usize,
    pub values: Vec<FilterOption>,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct FilterOption {
    pub matches: usize,
    pub value: Option<String>,
}

#[async_trait]
pub trait Filterable {
    type Error;

    async fn taxonomy_filters(&self) -> Result<TaxonomyFilters, Self::Error>;
}
