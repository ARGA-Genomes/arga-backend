use async_graphql::*;
use serde::{Serialize, Deserialize};

use crate::http::Error;
use crate::http::Context as State;


pub struct Overview;

#[Object]
impl Overview {
    /// Returns all specimens that are assigned to a specific taxon concept ID
    async fn categories(&self, ctx: &Context<'_>) -> Result<CategoryGroups, Error> {
        let state = ctx.data::<State>().unwrap();
        let query = format!(r#"*%3A*&fl=id&group=true&group.field=country&group.field=class&group.field=biome&group.field=kingdom&group.field=provenance"#);

        match state.solr.select_grouped::<CategoryGroups>(&query, 1).await {
            Ok(records) => Ok(records),
            Err(e) => {
                let err = Err(crate::http::Error::Solr(e));
                tracing::error!(?err);
                err
            }
        }
    }

    /// Returns the amount of preserved specimens in the index
    async fn preserved_specimens(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        summary_query(r#"basisOfRecord:"PRESERVED_SPECIMEN""#, state).await
    }

    /// Returns the amount of terrestrial specimens in the index
    async fn terrestrial(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        summary_query(r#"biome:"TERRESTRIAL""#, state).await
    }

    /// Returns the amount of marine specimens in the index
    async fn marine(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        summary_query(r#"biome:"MARINE""#, state).await
    }

    /// Returns the amount of specimens collected in Australia
    async fn in_australia(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        summary_query(r#"country:"Australia""#, state).await
    }

    /// Returns the amount of bacteria specimens in the index
    async fn bacteria(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        summary_query(r#"kingdom:"Bacteria""#, state).await
    }

    /// Returns the amount of published datasets in the index
    async fn published_datasets(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        summary_query(r#"provenance:"Published dataset""#, state).await
    }
}


async fn summary_query(query: &str, state: &State) -> Result<usize, Error> {
    match state.solr.select::<Summary>(&query, 1).await {
        Ok(summary) => Ok(summary.total),
        Err(e) => {
            let err = Err(crate::http::Error::Solr(e));
            tracing::error!(?err);
            err
        }
    }
}


#[derive(Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    #[serde(rename(deserialize = "numFound"))]
    total: usize,
}


#[derive(Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryGroups {
    country: Category,
    class: Category,
    biome: Category,
    kingdom: Category,
    provenance: Category,
}

#[derive(Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    /// The amount of matched records
    matches: usize,
    /// The amount of records ascribed to the category
    groups: Vec<Group>,
}

#[derive(Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    group_value: Option<String>,
    doclist: DocList,
}

#[derive(Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocList {
    #[serde(rename(deserialize = "numFound"))]
    total: usize,
}
