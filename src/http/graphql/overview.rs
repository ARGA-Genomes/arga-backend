use async_graphql::*;

use crate::http::Error;
use crate::http::Context as State;
use crate::index::overview::{Overview as OverviewTrait, OverviewCategory};


pub struct Overview;

#[Object]
impl Overview {
    /// Returns the amount of genomic records for animals in the index
    async fn animals(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.solr.total(OverviewCategory::Animals).await?)
    }

    /// Returns the amount of genomic records for plants in the index
    async fn plants(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.solr.total(OverviewCategory::Plants).await?)
    }

    /// Returns the amount of genomic records for fungi in the index
    async fn fungi(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.solr.total(OverviewCategory::Fungi).await?)
    }

    /// Returns the amount of bacteria specimens in the index
    async fn bacteria(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.solr.total(OverviewCategory::AgriculturalAndPest).await?)
    }

    /// Returns the amount of marine specimens in the index
    async fn marine(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.solr.total(OverviewCategory::MarineAndAquaculture).await?)
    }

    /// Returns the amount of specimens collected in Australia
    async fn in_australia(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.solr.total(OverviewCategory::AllSpecies).await?)
    }

    /// Returns the amount of preserved specimens in the index
    async fn preserved_specimens(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.solr.total(OverviewCategory::PreservedSpecimens).await?)
    }

    /// Returns the amount of terrestrial specimens in the index
    async fn terrestrial(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.solr.total(OverviewCategory::TerrestrialBiodiversity).await?)
    }

    /// Returns the amount of published datasets in the index
    async fn published_datasets(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.solr.total(OverviewCategory::ThreatenedSpecies).await?)
    }
}
