use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use serde::Serialize;

use crate::database::Database;
use crate::database::models;
use crate::database::schema;
use crate::http::Error;
use crate::http::Context as State;
use crate::index::names::GetNames;


#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct MarkerDetails {
    pub id: String,
    pub accession: String,
    pub material_sample_id: Option<String>,
    pub gb_acs: Option<String>,
    pub marker_code: Option<String>,
    pub nucleotide: Option<String>,
    pub recorded_by: Option<String>,
    pub version: Option<String>,
    pub basepairs: Option<i64>,
    pub type_: Option<String>,
    pub shape: Option<String>,
    pub source_url: Option<String>,
    pub fasta_url: Option<String>,
    pub extra_data: Option<serde_json::Value>,
}

impl From<models::Marker> for MarkerDetails {
    fn from(value: models::Marker) -> Self {
        Self {
            id: value.id.to_string(),
            accession: value.accession,
            material_sample_id: value.material_sample_id,
            gb_acs: value.gb_acs,
            marker_code: value.marker_code,
            nucleotide: value.nucleotide,
            recorded_by: value.recorded_by,
            version: value.version,
            basepairs: value.basepairs,
            type_: value.type_,
            shape: value.shape,
            source_url: value.source_url,
            fasta_url: value.fasta_url,
            extra_data: value.extra_data,
        }
    }
}



#[derive(MergedObject)]
pub struct Marker(MarkerDetails, MarkerQuery);

impl Marker {
    pub async fn new(db: &Database, accession: &str) -> Result<Marker, Error> {
        let query = MarkerQuery::new(db, accession).await?;
        Ok(Marker(query.marker.clone().into(), query))
    }
}


struct MarkerQuery {
    marker: models::Marker,
}

#[Object]
impl MarkerQuery {
    #[graphql(skip)]
    pub async fn new(db: &Database, accession: &str) -> Result<MarkerQuery, Error> {
        use schema::markers;
        let mut conn = db.pool.get().await?;
        let marker = markers::table
            .filter(markers::accession.eq(accession))
            .get_result::<models::Marker>(&mut conn)
            .await?;

        Ok(MarkerQuery {
            marker,
        })
    }

    async fn canonical_name(&self, ctx: &Context<'_>) -> Result<Option<String>, Error> {
        let state = ctx.data::<State>().unwrap();
        let name = state.database.find_by_name_id(&self.marker.name_id).await?;
        Ok(name.canonical_name)
    }

    async fn details(&self) -> MarkerDetails {
        self.marker.clone().into()
    }
}
