use async_graphql::*;

use super::common::species::DataType;
use super::taxon::Taxon;
use crate::database::extensions::filters::Filter;
use crate::database::extensions::taxa_filters;
use crate::http::{Context as State, Error};


/// Available filters when retrieving taxa.
#[derive(Debug, OneofObject)]
pub enum TaxaFilter {
    ScientificName(String),
    CanonicalName(String),
    VernacularGroup(String),
    HasData(DataType),
}


pub struct Taxa {
    filters: Vec<Filter>,
    taxa_filters: Vec<taxa_filters::TaxaFilter>,
}


#[Object]
impl Taxa {
    #[graphql(skip)]
    pub fn new(filters: Vec<TaxaFilter>) -> Result<Taxa, Error> {
        let taxa_filters = filters.into_iter().map(|f| f.into()).collect();

        Ok(Taxa {
            filters: vec![],
            taxa_filters,
            // filters: convert_filters(filters)?,
        })
    }

    async fn records(&self, ctx: &Context<'_>) -> Result<Vec<Taxon>, Error> {
        let state = ctx.data::<State>()?;
        let records = state.database.taxa.find(&self.taxa_filters).await?;
        let taxa = records.into_iter().map(|r| Taxon::init(r)).collect();
        Ok(taxa)
    }

    // async fn species(&self, ctx: &Context<'_>, page: i64, per_page: i64) -> Result<Page<SpeciesCard>, Error> {
    //     let state = ctx.data::<State>()?;
    //     let helper = SpeciesHelper::new(&state.database);

    //     let page = state.database.taxa.species(&vec![], page, per_page).await?;
    //     let cards = helper.filtered_cards(page.records).await?;

    //     Ok(Page {
    //         records: cards,
    //         total: page.total,
    //     })
    // }

    async fn filter_options(&self) -> FilterOptions {
        FilterOptions {
            filters: self.filters.clone(),
        }
    }
}

pub struct FilterOptions {
    filters: Vec<Filter>,
}


#[Object]
impl FilterOptions {
    async fn ecology(&self, ctx: &Context<'_>) -> Result<Vec<String>, Error> {
        let state = ctx.data::<State>()?;
        let options = state.database.taxa.ecology_options(&self.filters).await?;
        Ok(options)
    }

    async fn ibra(&self, ctx: &Context<'_>) -> Result<Vec<String>, Error> {
        let state = ctx.data::<State>()?;
        let options = state.database.taxa.ibra_options(&self.filters).await?;
        Ok(options)
    }

    async fn imcra(&self, ctx: &Context<'_>) -> Result<Vec<String>, Error> {
        let state = ctx.data::<State>()?;
        let options = state.database.taxa.imcra_options(&self.filters).await?;
        Ok(options)
    }

    async fn state(&self, ctx: &Context<'_>) -> Result<Vec<String>, Error> {
        let state = ctx.data::<State>()?;
        let options = state.database.taxa.state_options(&self.filters).await?;
        Ok(options)
    }

    async fn drainage_basin(&self, ctx: &Context<'_>) -> Result<Vec<String>, Error> {
        let state = ctx.data::<State>()?;
        let options = state.database.taxa.drainage_basin_options(&self.filters).await?;
        Ok(options)
    }
}


impl From<TaxaFilter> for taxa_filters::TaxaFilter {
    fn from(value: TaxaFilter) -> Self {
        use taxa_filters::DataFilter::*;
        use taxa_filters::TaxaFilter as Filter;
        use taxa_filters::TaxonFilter::*;

        match value {
            TaxaFilter::ScientificName(value) => Filter::Taxon(ScientificName(value)),
            TaxaFilter::CanonicalName(value) => Filter::Taxon(CanonicalName(value)),
            TaxaFilter::VernacularGroup(value) => Filter::Taxon(VernacularGroup(value)),
            TaxaFilter::HasData(value) => Filter::Data(HasData(value.into())),
        }
    }
}
