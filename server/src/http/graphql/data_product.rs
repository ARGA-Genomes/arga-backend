use async_graphql::*;

use super::common::{Agent, DataProductDetails, Publication};
use crate::database::models;
use crate::http::{Context as State, Error};


#[derive(MergedObject)]
pub struct DataProduct(DataProductDetails, DataProductQuery);

impl DataProduct {
    pub fn from_record(product: models::DataProduct) -> DataProduct {
        let details = product.clone().into();
        let query = DataProductQuery { product };
        DataProduct(details, query)
    }
}

impl From<models::DataProduct> for DataProduct {
    fn from(value: models::DataProduct) -> Self {
        Self::from_record(value)
    }
}


struct DataProductQuery {
    product: models::DataProduct,
}

#[Object]
impl DataProductQuery {
    async fn publication(&self, ctx: &Context<'_>) -> Result<Option<Publication>, Error> {
        let state = ctx.data::<State>()?;

        let publication = match &self.product.publication_id {
            None => None,
            Some(publication_id) => Some(state.database.publications.find_by_id(publication_id).await?.into()),
        };

        Ok(publication)
    }

    async fn custodian(&self, ctx: &Context<'_>) -> Result<Option<Agent>, Error> {
        let state = ctx.data::<State>()?;

        let agent = match &self.product.custodian {
            None => None,
            Some(agent_id) => Some(state.database.agents.find_by_id(agent_id).await?.into()),
        };

        Ok(agent)
    }
}
