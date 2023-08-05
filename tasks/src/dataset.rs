use diesel::{Insertable, FromSqlRow, AsExpression};
use diesel::pg::{Pg, sql_types};
use diesel::serialize::ToSql;
use diesel::deserialize::FromSql;
use diesel_async::RunQueryDsl;

use arga_backend::database::{schema, Database};
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
    name: String,
    description: Option<String>,
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
pub async fn import(worker: &str, name: &str, path: &str) -> Result<(), anyhow::Error> {
    use schema::jobs;

    let db_host = arga_backend::database::get_database_url();
    let database = Database::connect(&db_host).await?;
    let mut conn = database.pool.get().await?;

    let import_data = ImportJobData {
        name: name.to_string(),
        description: None,
        tmp_name: path.to_string(),
    };

    let id = diesel::insert_into(jobs::table)
        .values(&NewJob {
            worker: worker.to_string(),
            payload: Some(import_data),
        })
        .returning(jobs::id)
        .get_result::<Uuid>(&mut conn)
        .await?;

    info!(?id, name, worker, path, "Job queued");
    Ok(())
}
