use diesel::*;
use diesel::RunQueryDsl;
use diesel::pg::{Pg, sql_types};
use diesel::serialize::ToSql;
use diesel::deserialize::FromSql;
use diesel::r2d2::{ConnectionManager, Pool};

use arga_core::schema;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;


#[derive(Debug, Deserialize, Serialize, Insertable)]
#[diesel(table_name = schema::jobs)]
struct NewJob {
    worker: String,
    payload: Option<ImportJobData>,
}

#[derive(Debug, Deserialize, Serialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = sql_types::Jsonb)]
struct ImportJobData {
    dataset: String,
    isolation_context: Vec<String>,
    tmp_name: String,
}

impl ToSql<sql_types::Jsonb, Pg> for ImportJobData {
    fn to_sql(&self, out: &mut diesel::serialize::Output<Pg>) -> diesel::serialize::Result {
        let value = serde_json::to_value(self)?;
        <serde_json::Value as ToSql<sql_types::Jsonb, Pg>>::to_sql(&value, &mut out.reborrow())
    }
}

impl FromSql<sql_types::Jsonb, Pg> for ImportJobData {
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        let value = <serde_json::Value as FromSql<sql_types::Jsonb, Pg>>::from_sql(bytes)?;
        Ok(serde_json::from_value::<ImportJobData>(value)?)
    }
}


/// Queue an import job for a dataset.
pub fn import(worker: &str, dataset: &str, context: &Vec<String>, path: &str) {
    use schema::jobs;

    let url = arga_core::get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    let pool = Pool::builder().build(manager).expect("Could not build connection pool");
    let mut conn = pool.get().expect("Could not checkout connection");

    let import_data = ImportJobData {
        dataset: dataset.to_string(),
        isolation_context: context.clone(),
        tmp_name: path.to_string(),
    };

    let id = diesel::insert_into(jobs::table)
        .values(&NewJob {
            worker: worker.to_string(),
            payload: Some(import_data),
        })
        .returning(jobs::id)
        .get_result::<Uuid>(&mut conn)
        .unwrap();

    info!(?id, dataset, ?context, worker, path, "Job queued");
}
