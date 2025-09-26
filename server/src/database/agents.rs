use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use super::{Error, PgPool, schema};
use crate::database::models::Agent;


#[derive(Clone)]
pub struct AgentProvider {
    pub pool: PgPool,
}

impl AgentProvider {
    pub async fn find_by_id(&self, entity_id: &str) -> Result<Agent, Error> {
        use schema::agents;
        let mut conn = self.pool.get().await?;

        let agent = agents::table
            .filter(agents::entity_id.eq(entity_id))
            .select(Agent::as_select())
            .get_result::<Agent>(&mut conn)
            .await;

        if let Err(diesel::result::Error::NotFound) = agent {
            return Err(Error::NotFound(entity_id.to_string()));
        }

        Ok(agent?)
    }
}
