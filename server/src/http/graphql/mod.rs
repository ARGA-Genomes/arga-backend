pub mod common;
pub mod helpers;

pub mod maps;
pub mod overview;
pub mod search;
pub mod species;
pub mod stats;
// pub mod lists;
pub mod dataset;
pub mod source;
pub mod traces;
// pub mod assembly;
// pub mod assemblies;
pub mod dna_extract;
pub mod extensions;
pub mod marker;
pub mod markers;
pub mod names;
pub mod provenance;
pub mod sequence;
pub mod specimen;
pub mod subsample;
pub mod taxa;
pub mod taxon;

use async_graphql::extensions::Tracing;
use async_graphql::http::GraphiQLSource;
use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Router};

use self::common::{FilterItem, SearchFilterItem};
use self::dataset::Dataset;
// use self::assembly::Assembly;
// use self::assemblies::Assemblies;
use self::dna_extract::DnaExtract;
use self::extensions::ErrorLogging;
use self::maps::Maps;
use self::marker::Marker;
use self::markers::Markers;
use self::overview::Overview;
use self::provenance::Provenance;
use self::search::Search;
use self::sequence::Sequence;
use self::source::Source;
use self::species::Species;
use self::specimen::Specimen;
use self::stats::Statistics;
use self::subsample::Subsample;
use self::taxa::Taxa;
use self::taxon::Taxon;
use self::traces::Traces;
use super::error::Error;
use crate::http::Context as State;

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

    async fn search(&self, filters: Vec<SearchFilterItem>) -> Result<Search, Error> {
        Search::new(filters)
    }

    async fn species(&self, ctx: &Context<'_>, canonical_name: String) -> Result<Species, Error> {
        let state = ctx.data::<State>().unwrap();
        Species::new(&state.database, canonical_name).await
    }

    async fn stats(&self) -> Statistics {
        Statistics {}
    }

    async fn maps(&self, tolerance: Option<f32>) -> Maps {
        Maps { tolerance }
    }

    async fn sources(&self, ctx: &Context<'_>) -> Result<Vec<Source>, Error> {
        let state = ctx.data::<State>().unwrap();
        Source::all(&state.database).await
    }

    async fn source(
        &self,
        ctx: &Context<'_>,
        by: source::SourceBy,
        filters: Option<Vec<FilterItem>>,
    ) -> Result<Source, Error> {
        let state = ctx.data::<State>().unwrap();
        Source::new(&state.database, &by, filters.unwrap_or_default()).await
    }

    async fn dataset(&self, ctx: &Context<'_>, by: dataset::DatasetBy) -> Result<Dataset, Error> {
        let state = ctx.data::<State>().unwrap();
        Dataset::new(&state.database, &by).await
    }

    async fn traces(&self, uuid: String) -> Traces {
        let uuid = uuid::Uuid::parse_str(&uuid).unwrap();
        Traces { uuid }
    }

    // async fn assembly(&self, ctx: &Context<'_>, accession: String) -> Result<Assembly, Error> {
    //     let state = ctx.data::<State>().unwrap();
    //     Assembly::new(&state.database, &accession).await
    // }

    // async fn assemblies(&self) -> Assemblies {
    //     Assemblies {}
    // }

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

    async fn sequence(&self, ctx: &Context<'_>, by: sequence::SequenceBy) -> Result<Vec<Sequence>, Error> {
        let state = ctx.data::<State>().unwrap();
        Sequence::new(&state.database, &by).await
    }

    async fn taxon(&self, ctx: &Context<'_>, rank: taxon::TaxonRank, canonical_name: String) -> Result<Taxon, Error> {
        let state = ctx.data::<State>().unwrap();
        Taxon::new(&state.database, rank, canonical_name).await
    }

    async fn provenance(&self, ctx: &Context<'_>) -> Provenance {
        Provenance {}
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
