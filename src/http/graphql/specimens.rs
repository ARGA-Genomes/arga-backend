use async_graphql::*;
use serde::{Serialize, Deserialize};


pub struct Specimens;

#[Object]
impl Specimens {
    /// Returns all specimens that are assigned to a specific taxon concept ID
    async fn taxon_concept_id(&self, ctx: &Context<'_>, id: String) -> Result<SpecimenList, crate::http::Error> {
        let state = ctx.data::<crate::http::Context>().unwrap();
        let query = format!(r#"taxonConceptID:"{id}""#);

        match state.solr.select::<SpecimenList>(&query, 50).await {
            Ok(records) => Ok(records),
            Err(e) => {
                let err = Err(crate::http::Error::Solr(e));
                tracing::error!(?err);
                err
            }
        }
    }
}


#[derive(Debug, SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecimenList {
    #[serde(rename(deserialize = "numFound"))]
    total: usize,
    #[serde(rename(deserialize = "docs"))]
    records: Vec<Specimen>,
}

#[derive(Debug, SimpleObject, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Specimen {
    id: String,

    #[serde(rename(deserialize = "occurrenceID"))]
    occurrence_id: String,
    #[serde(rename(deserialize = "taxonConceptID"))]
    taxon_concept_id: Option<String>,
    #[serde(rename(deserialize = "genusID"))]
    genus_id: Option<String>,
    #[serde(rename(deserialize = "kingdomID"))]
    kingdom_id: Option<String>,
    #[serde(rename(deserialize = "phylumID"))]
    phylum_id: Option<String>,
    #[serde(rename(deserialize = "familyID"))]
    family_id: Option<String>,
    #[serde(rename(deserialize = "orderID"))]
    order_id: Option<String>,
    #[serde(rename(deserialize = "classID"))]
    class_id: Option<String>,

    /// The scientific name given to this taxon
    scientific_name: Option<String>,
    /// The taxonomic genus
    genus: Option<String>,
    /// The taxonomic sub genus
    subgenus: Option<String>,
    /// The taxonomic kingdom
    kingdom: Option<String>,
    /// The taxonomic phylum
    phylum: Option<String>,
    /// The taxonomic family
    family: Option<String>,
    /// The taxonomic class
    class: Option<String>,

    species_group: Option<Vec<String>>,
    species_subgroup: Option<Vec<String>>,
    biome: Option<String>,

    locality: Option<String>,
    state_province: Option<String>,
    country: Option<String>,
    country_code: Option<String>,

    event_date: Option<String>,
    event_time: Option<String>,
    license: Option<String>,

    lat_long: Option<String>,
    geohash: Option<String>,
    location: Option<String>,
    quad: Option<String>,
    packed_quad: Option<String>,
    geospatial_issues: Vec<String>,

    occurrence_status: Option<String>,
    occurrence_remarks: Option<String>,
    basis_of_record: Option<String>,
    scientific_name_authorship: Option<String>,

    data_resource_uid: Option<String>,
    data_resource_name: Option<String>,
    record_number: Option<String>,
    provenance: Option<String>,
    recorded_by: Option<Vec<String>>,
    identified_by: Option<Vec<String>>,

    assertions: Option<Vec<String>>,
}
