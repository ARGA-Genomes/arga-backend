pub mod common;
pub mod helpers;

pub mod overview;
pub mod search;
pub mod class;
pub mod order;
pub mod family;
pub mod genus;
pub mod species;
pub mod stats;
pub mod maps;
pub mod lists;
pub mod sources;
pub mod dataset;
pub mod traces;
pub mod assembly;
pub mod assemblies;
pub mod specimen;
pub mod marker;
pub mod markers;
pub mod taxa;
pub mod subsample;
pub mod dna_extract;
pub mod sequence;
pub mod extensions;

use axum::{Extension, Router};
use axum::response::IntoResponse;
use axum::routing::get;

use async_graphql::{Object, EmptySubscription, EmptyMutation, Schema, Context};
use async_graphql::extensions::Tracing;
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};


use crate::http::Context as State;
use self::common::FilterItem;
use self::overview::Overview;
use self::search::Search;
use self::class::Class;
use self::order::Order;
use self::family::Family;
use self::genus::Genus;
use self::species::Species;
use self::stats::Statistics;
use self::maps::Maps;
use self::sources::Source;
use self::dataset::Dataset;
use self::extensions::ErrorLogging;
use self::traces::Traces;
use self::assembly::Assembly;
use self::assemblies::Assemblies;
use self::specimen::Specimen;
use self::marker::Marker;
use self::markers::Markers;
use self::taxa::Taxa;
use self::subsample::Subsample;
use self::dna_extract::DnaExtract;
use self::sequence::Sequence;

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

    async fn class(&self, class: String) -> Class {
        Class { class }
    }

    async fn order(&self, order: String) -> Order {
        Order { order }
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

    async fn sources(&self, ctx: &Context<'_>) -> Result<Vec<Source>, Error> {
        let state = ctx.data::<State>().unwrap();
        let records = state.database.sources.all_records().await?;
        let sources = records.into_iter().map(|record| Source::new(record)).collect();
        Ok(sources)
    }

    async fn dataset(&self, ctx: &Context<'_>, name: String) -> Result<Dataset, Error> {
        let state = ctx.data::<State>().unwrap();
        Dataset::new(&state.database, &name).await
    }

    async fn traces(&self, uuid: String) -> Traces {
        let uuid = uuid::Uuid::parse_str(&uuid).unwrap();
        Traces { uuid }
    }

    async fn assembly(&self, ctx: &Context<'_>, accession: String) -> Result<Assembly, Error> {
        let state = ctx.data::<State>().unwrap();
        Assembly::new(&state.database, &accession).await
    }

    async fn assemblies(&self) -> Assemblies {
        Assemblies {}
    }

    async fn specimen(&self, ctx: &Context<'_>, by: specimen::SpecimenBy) -> Result<Specimen, Error> {
        let state = ctx.data::<State>().unwrap();
        Specimen::new(&state.database, &by).await
    }

    async fn marker(&self, ctx: &Context<'_>, accession: String) -> Result<Marker, Error> {
        let state = ctx.data::<State>().unwrap();
        Marker::new(&state.database, &accession).await
    }

    async fn markers(&self) -> Markers {
        Markers {}
    }

    async fn taxa(&self, filters: Vec<FilterItem>) -> Result<Taxa, Error> {
        Taxa::new(filters)
    }

    async fn subsample(&self, ctx: &Context<'_>, by: subsample::SubsampleBy) -> Result<Option<Subsample>, Error> {
        let state = ctx.data::<State>().unwrap();
        Subsample::new(&state.database, &by).await
    }

    async fn dna_extract(&self, ctx: &Context<'_>, by: dna_extract::DnaExtractBy) -> Result<Option<DnaExtract>, Error> {
        let state = ctx.data::<State>().unwrap();
        DnaExtract::new(&state.database, &by).await
    }

    async fn sequence(&self, ctx: &Context<'_>, by: sequence::SequenceBy) -> Result<Option<Sequence>, Error> {
        let state = ctx.data::<State>().unwrap();
        Sequence::new(&state.database, &by).await
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
