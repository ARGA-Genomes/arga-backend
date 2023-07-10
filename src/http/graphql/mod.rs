pub mod overview;
pub mod search;
pub mod family;
pub mod genus;
pub mod species;
pub mod stats;
pub mod maps;
pub mod lists;
pub mod traces;
pub mod assembly;
pub mod specimen;
pub mod markers;
pub mod extensions;

use axum::{Extension, Router};
use axum::response::IntoResponse;
use axum::routing::get;

use async_graphql::{Object, EmptySubscription, EmptyMutation, Schema, Context};
use async_graphql::extensions::Tracing;
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};


use crate::http::Context as State;
use crate::index::lists::{Filters, Pagination};
use self::overview::Overview;
use self::search::Search;
use self::family::Family;
use self::genus::Genus;
use self::species::Species;
use self::stats::Statistics;
use self::maps::Maps;
use self::lists::{Lists, FilterItem};
use self::extensions::ErrorLogging;
use self::traces::Traces;
use self::assembly::Assembly;
use self::specimen::Specimen;
use self::markers::Markers;

use super::error::Error;


pub type ArgaSchema = Schema<Query, EmptyMutation, EmptySubscription>;


/// The starting point for any GraphQL query.
///
/// This encapsulates all functionality available from the ARGA service.
pub struct Query;

#[Object]
impl Query {
    async fn overview(&self) -> Overview {
        Overview {}
    }

    async fn search(&self) -> Search {
        Search {}
    }

    async fn species(&self, ctx: &Context<'_>, canonical_name: String) -> Result<Species, Error> {
        let state = ctx.data::<State>().unwrap();
        Species::new(&state.database, canonical_name).await
    }

    async fn family(&self, family: String) -> Family {
        Family { family }
    }

    async fn genus(&self, genus: String) -> Genus {
        Genus { genus }
    }

    async fn stats(&self) -> Statistics {
        Statistics {}
    }

    async fn maps(&self, tolerance: Option<f32>) -> Maps {
        Maps { tolerance }
    }

    async fn lists(
        &self,
        ctx: &Context<'_>,
        name: String,
        filters: Option<Vec<FilterItem>>,
        pagination: Option<Pagination>,
    ) -> Result<Lists, Error>
    {
        let state = ctx.data::<State>().unwrap();

        let filters = match filters {
            Some(items) => Filters {
                items: items.into_iter().map(|item| item.into()).collect(),
            },
            None => Filters::default(),
        };

        let pagination = pagination.unwrap_or_else(|| Pagination { page: 1, page_size: 20 });

        Lists::new(&state.database, name, filters, pagination).await
    }

    async fn traces(&self, uuid: String) -> Traces {
        let uuid = uuid::Uuid::parse_str(&uuid).unwrap();
        Traces { uuid }
    }

    async fn assembly(&self, ctx: &Context<'_>, accession: String) -> Result<Assembly, Error> {
        let state = ctx.data::<State>().unwrap();
        Assembly::new(&state.database, &accession).await
    }

    async fn specimen(&self, ctx: &Context<'_>, specimen_id: String) -> Result<Specimen, Error> {
        let state = ctx.data::<State>().unwrap();
        Specimen::new(&state.database, &specimen_id).await
    }

    async fn markers(&self) -> Markers {
        Markers {}
    }
}

/// The GraphQL API.
///
/// Defines the graphql resolvers and sets up the context
/// and middleware. This is the entry point to our graphql api
/// like the root router does for http requests.
pub(crate) fn schema(state: State) -> ArgaSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(state)
        .extension(ErrorLogging)
        .extension(Tracing)
        .finish()
}

/// Handles graphql requests.
async fn graphql_handler(schema: Extension<ArgaSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

/// Adds the built-in graphql IDE when visiting with a browser.
/// This will likely be disabled in the future in favour of postman/insomnia.
async fn graphql_ide() -> impl IntoResponse {
    axum::response::Html(GraphiQLSource::build().endpoint("/api").finish())
}

/// The router enabling the graphql extension and passes
/// requests to the handler.
pub(crate) fn router(state: State) -> Router<State> {
    let schema = schema(state);
    Router::new()
        .route("/api", get(graphql_ide).post(graphql_handler))
        .layer(Extension(schema))
}
