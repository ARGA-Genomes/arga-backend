pub mod common;
pub mod helpers;

pub mod dataset;
pub mod dna_extract;
pub mod extensions;
pub mod maps;
pub mod marker;
pub mod markers;
pub mod names;
pub mod organism;
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
use tracing::instrument;

use self::common::FilterItem;
use self::dataset::Dataset;
use self::dna_extract::DnaExtract;
use self::extensions::ErrorLogging;
use self::maps::Maps;
use self::marker::Marker;
use self::markers::Markers;
use self::organism::Organism;
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
use super::cache::CacheLayer;
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

    async fn search(&self) -> Search {
        Search {}
    }

    #[instrument(skip(self, ctx), fields(canonical_name = %canonical_name))]
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

    #[instrument(skip(self, ctx))]
    async fn sources(&self, ctx: &Context<'_>) -> Result<Vec<Source>, Error> {
        let state = ctx.data::<State>()?;
        Source::all(&state.database).await
    }

    #[instrument(skip(self, ctx), fields(source_by = ?by))]
    async fn source(
        &self,
        ctx: &Context<'_>,
        by: source::SourceBy,
        filters: Option<Vec<FilterItem>>,
    ) -> Result<Source, Error> {
        let state = ctx.data::<State>()?;
        Source::new(&state.database, &by, filters.unwrap_or_default()).await
    }

    #[instrument(skip(self, ctx), fields(dataset_by = ?by))]
    async fn dataset(&self, ctx: &Context<'_>, by: dataset::DatasetBy) -> Result<Dataset, Error> {
        let state = ctx.data::<State>()?;
        Dataset::new(&state.database, &by).await
    }

    #[instrument(skip(self, ctx), fields(organism_by = ?by))]
    async fn organism(&self, ctx: &Context<'_>, by: organism::OrganismBy) -> Result<Organism, Error> {
        let state = ctx.data::<State>()?;
        Organism::new(&state.database, &by).await
    }

    #[instrument(skip(self, ctx), fields(specimen_by = ?by))]
    async fn specimen(&self, ctx: &Context<'_>, by: specimen::SpecimenBy) -> Result<Specimen, Error> {
        let state = ctx.data::<State>()?;
        Specimen::new(&state.database, &by).await
    }

    #[instrument(skip(self, ctx), fields(accession = %accession))]
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

    #[instrument(skip(self, ctx), fields(subsample_by = ?by))]
    async fn subsample(&self, ctx: &Context<'_>, by: subsample::SubsampleBy) -> Result<Option<Subsample>, Error> {
        let state = ctx.data::<State>()?;
        Subsample::new(&state.database, &by).await
    }

    #[instrument(skip(self, ctx), fields(dna_extract_by = ?by))]
    async fn dna_extract(&self, ctx: &Context<'_>, by: dna_extract::DnaExtractBy) -> Result<Option<DnaExtract>, Error> {
        let state = ctx.data::<State>()?;
        DnaExtract::new(&state.database, &by).await
    }

    #[instrument(skip(self, ctx), fields(sequence_by = ?by))]
    async fn sequence(&self, ctx: &Context<'_>, by: sequence::SequenceBy) -> Result<Vec<Sequence>, Error> {
        let state = ctx.data::<State>()?;
        Sequence::new(&state.database, &by).await
    }

    #[instrument(skip(self, ctx), fields(taxon_by = ?by))]
    async fn taxon(
        &self,
        ctx: &Context<'_>,
        by: taxon::TaxonBy,
        filters: Option<Vec<FilterItem>>,
    ) -> Result<Taxon, Error> {
        let state = ctx.data::<State>()?;
        Taxon::new(&state.database, by, filters).await
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
    let schema = schema(state.clone());

    let mut router = Router::new()
        .route("/api", get(graphql_ide).post(graphql_handler))
        .layer(Extension(schema));

    // Add caching middleware if cache URL is configured
    if let Some(cache_url) = &state.config.cache_url {
        match CacheLayer::new(cache_url, state.config.cache_ttl, state.config.cache_skip_pattern.clone()) {
            Ok(cache_layer) => {
                match &state.config.cache_skip_pattern {
                    Some(pattern) => tracing::info!("Caching enabled for GraphQL API with skip pattern: {}", pattern),
                    None => tracing::info!("Caching enabled for GraphQL API with no skip pattern"),
                }
                router = router.layer(cache_layer);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize cache: {}", e);
            }
        }
    } else {
        tracing::info!("Caching disabled - no cache URL configured");
    }

    router
}
