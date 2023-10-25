use std::collections::HashMap;
use std::path::Path;

use axum::body::Bytes;
use axum::extract::{State, Query, Multipart};
use axum::{Json, BoxError, Router};
use axum::http::StatusCode;
use axum::routing::{get, post};

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use tracing::debug;
use uuid::Uuid;

use futures::{Stream, TryStreamExt};
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;

use crate::database::extensions::Paginate;
use crate::http::Context;
use crate::http::error::{InternalError, Error};
use crate::database::{schema, Database};
use crate::database::models::{Name, TaxonPhoto, AdminMedia};

use super::common::{PageResult, parse_int_param};


async fn media(
    Query(params): Query<HashMap<String, String>>,
    State(database): State<Database>,
) -> PageResult<AdminMedia>
{
    use schema::{names, admin_media};
    let mut conn = database.pool.get().await?;

    let page = parse_int_param(&params, "page", 1);
    let per_page = parse_int_param(&params, "page_size", 20);
    let name = params.get("scientific_name").expect("must provide a scientific name parameter");

    let page = admin_media::table
        .inner_join(names::table)
        .filter(names::scientific_name.eq(name))
        .order_by(admin_media::reference_url)
        .select(admin_media::all_columns)
        .paginate(page)
        .per_page(per_page)
        .load::<(AdminMedia, i64)>(&mut conn)
        .await?;

    Ok(Json(page.into()))
}


async fn main_media(
    Query(params): Query<HashMap<String, String>>,
    State(db_provider): State<Database>,
) -> Result<Json<TaxonPhoto>, InternalError>
{
    let mut conn = db_provider.pool.get().await?;

    let name = params.get("scientific_name").expect("must provide a scientific name parameter");

    use schema::{names, taxon_photos};
    let photo = taxon_photos::table
        .select(taxon_photos::all_columns)
        .inner_join(names::table)
        .filter(names::scientific_name.eq(name))
        .get_result::<TaxonPhoto>(&mut conn)
        .await?;

    Ok(Json(photo))
}


#[derive(Deserialize, Debug)]
struct SetMainMedia {
    url: String,
    scientific_name: String,
    source: Option<String>,
    publisher: Option<String>,
    license: Option<String>,
    rights_holder: Option<String>,
}

async fn upsert_main_media(
    State(db_provider): State<Database>,
    Json(form): Json<SetMainMedia>,
) -> Result<(), InternalError>
{
    // link the main photo as an attribute against the taxa
    use schema::{names, taxon_photos};
    let mut conn = db_provider.pool.get().await?;

    debug!(?form, "setting main image");

    let name: Name = names::table
        .filter(names::scientific_name.eq(form.scientific_name))
        .get_result(&mut conn)
        .await?;

    // delete any previous main images
    diesel::delete(taxon_photos::table)
        .filter(taxon_photos::name_id.eq(name.id))
        .execute(&mut conn)
        .await?;

    // add a taxa photo entry linked to the name
    let photo = TaxonPhoto {
        id: Uuid::new_v4(),
        name_id: name.id,
        url: form.url,
        source: form.source,
        publisher: form.publisher,
        license: form.license,
        rights_holder: form.rights_holder,
    };

    diesel::insert_into(taxon_photos::table)
        .values(&photo)
        .execute(&mut conn)
        .await?;

    debug!(?photo, ?name, "main image set");

    Ok(())
}


#[derive(Deserialize, Debug)]
struct UploadMainImage {
    file: String,
    scientific_name: String,
    source: Option<String>,
    publisher: Option<String>,
    license: Option<String>,
    rights_holder: Option<String>,
}

async fn upload_main_image(
    State(db_provider): State<Database>,
    Json(form): Json<UploadMainImage>,
) -> Result<(), InternalError>
{
    // link the main photo as an attribute against the taxa
    use schema::{names, taxon_photos};
    let mut conn = db_provider.pool.get().await?;

    debug!(?form, "setting main image");

    let name: Name = names::table
        .filter(names::scientific_name.eq(form.scientific_name))
        .get_result(&mut conn)
        .await?;

    // move the temporary image file to the image store
    let filename = format!("arga_admin_{}", form.file);
    let outpath = format!("{}.jpg", form.file);
    let tmp_path = std::env::var("ADMIN_TMP_UPLOAD_STORAGE").expect("No upload storage specified");
    let img_path = std::env::var("ADMIN_IMAGE_UPLOAD_STORAGE").expect("No upload storage specified");
    std::fs::copy(Path::new(&tmp_path).join(&filename), Path::new(&img_path).join("assets").join(&outpath))?;

    // delete any previous main images
    diesel::delete(taxon_photos::table)
        .filter(taxon_photos::name_id.eq(name.id))
        .execute(&mut conn)
        .await?;

    // add a taxa photo entry linked to the name
    let photo = TaxonPhoto {
        id: Uuid::new_v4(),
        name_id: name.id,
        url: format!("https://app.arga.org.au/assets/{}.jpg", form.file),
        source: form.source,
        publisher: form.publisher,
        license: form.license,
        rights_holder: form.rights_holder,
    };

    diesel::insert_into(taxon_photos::table)
        .values(&photo)
        .execute(&mut conn)
        .await?;

    debug!(?photo, ?name, "main image set");

    Ok(())
}


#[tracing::instrument]
async fn accept_image(mut multipart: Multipart) -> Result<String, Error> {
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
        let tmp_path = std::env::var("ADMIN_TMP_UPLOAD_STORAGE").expect("No upload storage specified");
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


/// The REST gateway for the admin backend for basic CRUD operations
pub(crate) fn router() -> Router<Context> {
    Router::new()
        .route("/api/admin/media", get(media))
        .route("/api/admin/media/main", get(main_media))
        .route("/api/admin/media/main", post(upsert_main_media))
        .route("/api/admin/media/upload", post(accept_image))
        .route("/api/admin/media/upload_main_image", post(upload_main_image))
}
