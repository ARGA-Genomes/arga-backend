pub mod overview;
pub mod specimens;
pub mod search;
pub mod extensions;

use axum::{Extension, Router};
use axum::response::IntoResponse;
use axum::routing::get;

use async_graphql::{Object, EmptySubscription, EmptyMutation, Schema};
use async_graphql::extensions::Tracing;
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};

use crate::http::Context;
use self::overview::Overview;
use self::specimens::Specimens;
use self::search::Search;
use self::extensions::ErrorLogging;


pub type ArgaSchema = Schema<Query, EmptyMutation, EmptySubscription>;


pub struct Query;

#[Object]
impl Query {
    async fn overview(&self) -> Overview {
        Overview {}
    }
    async fn specimens(&self) -> Specimens {
        Specimens {}
    }
    async fn search(&self) -> Search {
        Search {}
    }
}

pub(crate) fn schema(context: Context) -> ArgaSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(context)
        .extension(ErrorLogging)
        .extension(Tracing)
        .finish()
}

async fn graphql_handler(
    schema: Extension<ArgaSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}


async fn graphql_ide() -> impl IntoResponse {
    axum::response::Html(GraphiQLSource::build().endpoint("/api").finish())
}


pub(crate) fn router(context: Context) -> Router<Context> {
    let schema = schema(context);
    Router::new()
        .route("/api", get(graphql_ide).post(graphql_handler))
        .layer(Extension(schema))
}
