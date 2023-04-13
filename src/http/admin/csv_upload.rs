use axum::{Router, BoxError, Json};
use axum::http::StatusCode;
use axum::body::Bytes;
use axum::routing::post;
use axum::extract::{DefaultBodyLimit, Multipart, State};
use diesel::deserialize::FromSql;
use diesel::serialize::ToSql;
use diesel::pg::{Pg, sql_types};
use diesel::{Insertable, FromSqlRow, AsExpression};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use tower_http::limit::RequestBodyLimitLayer;

use futures::{Stream, TryStreamExt};
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;
use tracing::debug;

use crate::http::Context;
use crate::http::error::Error;
use crate::index::providers::db::Database;
use crate::schema;


/// The REST gateway for the admin backend for basic CRUD operations
pub(crate) fn router() -> Router<Context> {
    Router::new()
        .route("/api/admin/queue", post(queue_csv))
        .route("/api/admin/upload", post(accept_csv))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(250 * 1024 * 1024))
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


#[derive(Debug, Deserialize, Serialize, Insertable)]
#[diesel(table_name = schema::jobs)]
struct NewJob {
    worker: String,
    payload: Option<ImportJobData>,
}

#[derive(Debug, Deserialize)]
struct QueueForm {
    name: String,
    description: Option<String>,
    file: String,
}

#[tracing::instrument(skip(db_provider))]
async fn queue_csv(
    State(db_provider): State<Database>,
    Json(form): Json<QueueForm>,
) -> Result<(), Error> {
    use schema::jobs::dsl::*;
    let mut conn = db_provider.pool.get().await.unwrap();

    if form.file.is_empty() {
        return Err(Error::MissingParam("file".into()));
    }
    if form.name.is_empty() {
        return Err(Error::MissingParam("name".into()));
    }


    let import_data = ImportJobData {
        name: form.name,
        description: form.description,
        tmp_name: format!("arga_admin_{}", form.file),
    };

    diesel::insert_into(jobs)
        .values(&NewJob {
            worker: "import_csv".into(),
            payload: Some(import_data),
        })
        .execute(&mut conn)
        .await.unwrap();

    Ok(())
}


#[tracing::instrument]
async fn accept_csv(mut multipart: Multipart) -> Result<String, Error> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        if let Some(file_name) = field.file_name() {
            let uuid = uuid::Uuid::new_v4();
            let tmp_name = format!("arga_admin_{}", uuid.to_string());
            debug!(?uuid, file_name, tmp_name, "Receiving file");
            stream_to_file(&tmp_name, field).await.unwrap();

            return Ok(uuid.to_string());
        }
    }

    Ok("".into())
}


async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    if !valid_path(path) {
        return Err((StatusCode::BAD_REQUEST, "Invalid path".to_owned()));
    }

    async {
        // get an async reader from the body stream
        let body = stream.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err));
        let reader = StreamReader::new(body);
        futures::pin_mut!(reader);

        // create the file on the server
        let tmp_path = std::env::var("ADMIN_TMP_UPLOAD_STORE").expect("No upload storage specified");
        let path = std::path::Path::new(&tmp_path).join(path);
        let mut file = BufWriter::new(File::create(path).await?);

        tokio::io::copy(&mut reader, &mut file).await?;

        Ok::<_, std::io::Error>(())
    }
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}


fn valid_path(path: &str) -> bool {
    let path = std::path::Path::new(path);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    components.count() == 1
}
