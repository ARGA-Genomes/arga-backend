use async_trait::async_trait;

use crate::database::models::Name;


#[async_trait]
pub trait GetNames {
    type Error;
    async fn find_by_canonical_name(&self, name: &str) -> Result<Vec<Name>, Self::Error>;
    async fn find_by_scientific_name(&self, name: &str) -> Result<Name, Self::Error>;
}
