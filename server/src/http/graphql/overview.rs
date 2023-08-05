use async_graphql::*;

use crate::http::Error;
use crate::http::Context as State;
use crate::index::overview::{Overview as OverviewTrait, OverviewCategory};


pub struct Overview;

#[Object]
impl Overview {
    /// Returns the amount of genomic records for animals in the index
    async fn animals(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.animals().await?.total)
    }

    async fn genomes(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.genomes().await?.total)
    }

    async fn markers(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.markers().await?.total)
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

    /// Returns the amount of agriculture, aquaculture and commercial species in the index
    async fn agricultural_and_aquaculture_and_commercial(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.solr.total(OverviewCategory::AgriculturalAndAquacultureAndCommercial).await?)
    }

    /// Returns the amount of bioSecurity and pest in the index
    async fn bio_security_and_pest(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.solr.total(OverviewCategory::BioSecurityAndPest).await?)
    }

    /// Returns the amount of marine specimens in the index
    async fn marine(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.solr.total(OverviewCategory::Marine).await?)
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

    /// Returns the amount of threatened species in the index
    async fn threatened_species(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.solr.total(OverviewCategory::ThreatenedSpecies).await?)
    }

    /// Returns the amount of whole genomes in the index
    async fn whole_genomes(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.whole_genomes().await?.total)
    }
    /// Returns the amount of partial genomes in the index
    async fn partial_genomes(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.partial_genomes().await?.total)
    }
    /// Returns the amount of organelles in the index
    async fn organelles(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.solr.total(OverviewCategory::Organelles).await?)
    }
    /// Returns the amount of barcodes in the index
    async fn barcodes(&self, ctx: &Context<'_>) -> Result<i64, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.database.overview.markers().await?.total)
    }

    /// Returns the amount of records
    async fn all_records(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.solr.total(OverviewCategory::AllRecords).await?)
    }

    /// Returns the amount of species
    async fn all_species(&self, ctx: &Context<'_>) -> Result<usize, Error> {
        let state = ctx.data::<State>().unwrap();
        Ok(state.solr.total(OverviewCategory::AllSpecies).await?)
    }
}
