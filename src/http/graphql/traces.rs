use async_graphql::*;
use tracing::instrument;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::http::Error;
use crate::http::Context as State;

use crate::database::models::TraceFile;
use crate::database::schema;



#[derive(Clone, Debug, SimpleObject)]
pub struct Analyzed {
    pub g: Vec<i32>,
    pub a: Vec<i32>,
    pub t: Vec<i32>,
    pub c: Vec<i32>,
}


pub struct Traces {
    pub uuid: Uuid,
}

#[Object]
impl Traces {
    #[instrument(skip(self, ctx))]
    async fn analyzed(&self, ctx: &Context<'_>) -> Result<Analyzed, Error> {
        let state = ctx.data::<State>().unwrap();
        let mut conn = state.db_provider.pool.get().await?;

        use schema::trace_files::dsl::*;
        let trace = trace_files.filter(id.eq(self.uuid)).get_result::<TraceFile>(&mut conn).await?;

        Ok(Analyzed {
            g: trace.analyzed_g.unwrap().into_iter().map(|v| v.unwrap_or_default()).collect(),
            a: trace.analyzed_a.unwrap().into_iter().map(|v| v.unwrap_or_default()).collect(),
            t: trace.analyzed_t.unwrap().into_iter().map(|v| v.unwrap_or_default()).collect(),
            c: trace.analyzed_c.unwrap().into_iter().map(|v| v.unwrap_or_default()).collect(),
        })
    }
}
