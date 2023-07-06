use async_trait::async_trait;
use uuid::Uuid;

use crate::database::models::Name;


#[async_trait]
pub trait GetNames {
    type Error;
    async fn find_by_name_id(&self, uuid: &Uuid) -> Result<Name, Self::Error>;
    async fn find_by_canonical_name(&self, name: &str) -> Result<Vec<Name>, Self::Error>;
    async fn find_by_scientific_name(&self, name: &str) -> Result<Name, Self::Error>;
    async fn find_by_assembly_id(&self, uuid: &Uuid) -> Result<Name, Self::Error>;
}
