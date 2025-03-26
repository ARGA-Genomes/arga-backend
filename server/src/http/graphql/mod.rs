pub mod common;
pub mod helpers;

pub mod dataset;
pub mod dna_extract;
pub mod extensions;
pub mod maps;
pub mod marker;
pub mod markers;
pub mod names;
pub mod overview;
pub mod provenance;
pub mod search;
pub mod sequence;
pub mod source;
pub mod species;
pub mod specimen;
pub mod stats;
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
use super::error::Error;
use crate::database::extensions::species_filters::NameAttributeFilter;
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
        let state = ctx.data::<State>()?;
        Species::new(&state.database, canonical_name).await
    }

    async fn stats(&self) -> Statistics {
        Statistics {}
    }

    async fn maps(&self, tolerance: Option<f32>) -> Maps {
        Maps { tolerance }
    }

    async fn sources(&self, ctx: &Context<'_>) -> Result<Vec<Source>, Error> {
        let state = ctx.data::<State>()?;
        Source::all(&state.database).await
    }

    async fn source(
        &self,
        ctx: &Context<'_>,
        by: source::SourceBy,
        filters: Option<Vec<FilterItem>>,
        species_attribute: Option<NameAttributeFilter>,
    ) -> Result<Source, Error> {
        let state = ctx.data::<State>()?;
        Source::new(&state.database, &by, filters.unwrap_or_default(), species_attribute).await
    }

    async fn dataset(&self, ctx: &Context<'_>, by: dataset::DatasetBy) -> Result<Dataset, Error> {
        let state = ctx.data::<State>()?;
        Dataset::new(&state.database, &by).await
    }

    async fn specimen(&self, ctx: &Context<'_>, by: specimen::SpecimenBy) -> Result<Specimen, Error> {
        let state = ctx.data::<State>()?;
        Specimen::new(&state.database, &by).await
    }

    async fn marker(&self, ctx: &Context<'_>, accession: String) -> Result<Marker, Error> {
        let state = ctx.data::<State>()?;
        Marker::new(&state.database, &accession).await
    }

    async fn markers(&self) -> Markers {
        Markers {}
    }

    async fn taxa(&self, filters: Vec<taxa::TaxaFilter>) -> Result<Taxa, Error> {
        Taxa::new(filters)
    }

    async fn subsample(&self, ctx: &Context<'_>, by: subsample::SubsampleBy) -> Result<Option<Subsample>, Error> {
        let state = ctx.data::<State>()?;
        Subsample::new(&state.database, &by).await
    }

    async fn dna_extract(&self, ctx: &Context<'_>, by: dna_extract::DnaExtractBy) -> Result<Option<DnaExtract>, Error> {
        let state = ctx.data::<State>()?;
        DnaExtract::new(&state.database, &by).await
    }

    async fn sequence(&self, ctx: &Context<'_>, by: sequence::SequenceBy) -> Result<Vec<Sequence>, Error> {
        let state = ctx.data::<State>()?;
        Sequence::new(&state.database, &by).await
    }

    async fn taxon(&self, ctx: &Context<'_>, by: taxon::TaxonBy) -> Result<Taxon, Error> {
        let state = ctx.data::<State>()?;
        Taxon::new(&state.database, by).await
    }

    async fn provenance(&self) -> Provenance {
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
