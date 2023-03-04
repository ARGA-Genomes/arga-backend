use async_trait::async_trait;
use uuid::Uuid;

use crate::index::species::{Species, Taxonomy, Distribution};
use super::{Database, Error};


#[async_trait]
impl Species for Database {
    type Error = Error;

    async fn taxonomy(&self, taxon_uuid: Uuid) -> Result<Taxonomy, Error> {
        let taxon = sqlx::query_as!(Taxon, r#"
SELECT canonical_name,
       scientific_name_authorship,
       kingdom,
       phylum,
       class,
       "order",
       family,
       genus
FROM taxa
WHERE id = $1
        "#, taxon_uuid).fetch_one(&self.pool).await?;

        Ok(Taxonomy::from(taxon))
    }

    async fn distribution(&self, taxon_uuid: Uuid) -> Result<Vec<Distribution>, Error> {
         let rows = sqlx::query_as!(Distribution, r#"
SELECT locality,
       country,
       country_code,
       threat_status,
       source
FROM distribution
JOIN taxa on taxa.taxon_id = distribution.taxon_id
WHERE taxa.id = $1
        "#, taxon_uuid).fetch_all(&self.pool).await?;

        Ok(rows)
    }
}


#[derive(Debug)]
struct Taxon {
    scientific_name_authorship: Option<String>,
    canonical_name: Option<String>,
    kingdom: Option<String>,
    phylum: Option<String>,
    class: Option<String>,
    order: Option<String>,
    family: Option<String>,
    genus: Option<String>,
}

impl From<Taxon> for Taxonomy {
    fn from(source: Taxon) -> Self {
        Self {
            canonical_name: source.canonical_name,
            authorship: source.scientific_name_authorship,

            kingdom: source.kingdom,
            phylum: source.phylum,
            class: source.class,
            order: source.order,
            family: source.family,
            genus: source.genus,
        }
    }
}
