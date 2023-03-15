pub mod overview;
pub mod search;
pub mod family;
pub mod genus;
pub mod species;
pub mod stats;
pub mod extensions;

use axum::{Extension, Router};
use axum::response::IntoResponse;
use axum::routing::get;

use async_graphql::{Object, EmptySubscription, EmptyMutation, Schema};
use async_graphql::extensions::Tracing;
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};

use crate::features::Features;
use crate::http::Context;
use self::overview::Overview;
use self::search::Search;
use self::family::Family;
use self::genus::Genus;
use self::species::Species;
use self::stats::Statistics;
use self::extensions::ErrorLogging;


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

    async fn species(&self, canonical_name: String) -> Species {
        Species { canonical_name }
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
}

/// The GraphQL API.
///
/// Defines the graphql resolvers and sets up the context
/// and middleware. This is the entry point to our graphql api
/// like the root router does for http requests.
pub(crate) fn schema(context: Context) -> ArgaSchema {
    let with_tracing = context.features.is_enabled(Features::OpenTelemetry);

    let mut builder = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(context)
        .extension(ErrorLogging);

    if let Ok(true) = with_tracing {
        tracing::info!("Enabling graphql tracing extension");
        builder = builder.extension(Tracing);
    }

    builder.finish()
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
pub(crate) fn router(context: Context) -> Router<Context> {
    let schema = schema(context);
    Router::new()
        .route("/api", get(graphql_ide).post(graphql_handler))
        .layer(Extension(schema))
}
