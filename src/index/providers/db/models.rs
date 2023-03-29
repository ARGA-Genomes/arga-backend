use diesel::{Queryable, Insertable};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::schema;


#[derive(Clone, Queryable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema::taxa)]
pub struct Taxon {
    id: Uuid,

    taxon_id: Option<i64>,
    // http://rs.tdwg.org/dwc/terms/datasetID
    dataset_id: Option<String>,
    // http://rs.tdwg.org/dwc/terms/parentNameUsageID
    parent_name_usage_id: Option<String>,
    // http://rs.tdwg.org/dwc/terms/acceptedNameUsageID
    accepted_name_usage_id: Option<String>,
    // http://rs.tdwg.org/dwc/terms/originalNameUsageID
    original_name_usage_id: Option<String>,

    // http://rs.tdwg.org/dwc/terms/scientificName
    scientific_name: Option<String>,
    // http://rs.tdwg.org/dwc/terms/scientificNameAuthorship
    scientific_name_authorship: Option<String>,
    // http://rs.gbif.org/terms/1.0/canonicalName
    canonical_name: Option<String>,
    // http://rs.tdwg.org/dwc/terms/genericName
    generic_name: Option<String>,

    // http://rs.tdwg.org/dwc/terms/specificEpithet
    specific_epithet: Option<String>,
    // http://rs.tdwg.org/dwc/terms/infraspecificEpithet
    infraspecific_epithet: Option<String>,
    // http://rs.tdwg.org/dwc/terms/taxonRank
    taxon_rank: Option<String>,
    // http://rs.tdwg.org/dwc/terms/nameAccordingTo
    name_according_to: Option<String>,
    // http://rs.tdwg.org/dwc/terms/namePublishedIn
    name_published_in: Option<String>,
    // http://rs.tdwg.org/dwc/terms/taxonomicStatus
    taxonomic_status: Option<String>,
    // http://rs.tdwg.org/dwc/terms/nomenclaturalStatus
    nomenclatural_status: Option<String>,
    // http://rs.tdwg.org/dwc/terms/taxonRemarks
    taxon_remarks: Option<String>,

    // http://rs.tdwg.org/dwc/terms/kingdom
    kingdom: Option<String>,
    // http://rs.tdwg.org/dwc/terms/phylum
    phylum: Option<String>,
    // http://rs.tdwg.org/dwc/terms/class
    class: Option<String>,
    // http://rs.tdwg.org/dwc/terms/order
    order: Option<String>,
    // http://rs.tdwg.org/dwc/terms/family
    family: Option<String>,
    // http://rs.tdwg.org/dwc/terms/genus
    genus: Option<String>,
}

#[derive(Queryable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema::distribution)]
pub struct Distribution {
    id: Uuid,

    taxon_id: Option<i64>,
    location_id: Option<String>,
    locality: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
    location_remarks: Option<String>,
    establishment_means: Option<String>,
    life_stage: Option<String>,
    occurrence_status: Option<String>,
    threat_status: Option<String>,
    source: Option<String>,
}


#[derive(Queryable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema::user_taxa_lists)]
pub struct UserTaxaList {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema::user_taxa)]
pub struct UserTaxon {
    id: Uuid,
    taxa_lists_id: Uuid,

    // http://rs.tdwg.org/dwc/terms/scientificName
    scientific_name: Option<String>,
    // http://rs.tdwg.org/dwc/terms/scientificNameAuthorship
    scientific_name_authorship: Option<String>,
    // http://rs.gbif.org/terms/1.0/canonicalName
    canonical_name: Option<String>,

    // http://rs.tdwg.org/dwc/terms/specificEpithet
    specific_epithet: Option<String>,
    // http://rs.tdwg.org/dwc/terms/infraspecificEpithet
    infraspecific_epithet: Option<String>,
    // http://rs.tdwg.org/dwc/terms/taxonRank
    taxon_rank: Option<String>,
    // http://rs.tdwg.org/dwc/terms/nameAccordingTo
    name_according_to: Option<String>,
    // http://rs.tdwg.org/dwc/terms/namePublishedIn
    name_published_in: Option<String>,
    // http://rs.tdwg.org/dwc/terms/taxonomicStatus
    taxonomic_status: Option<String>,
    // http://rs.tdwg.org/dwc/terms/taxonRemarks
    taxon_remarks: Option<String>,

    // http://rs.tdwg.org/dwc/terms/kingdom
    kingdom: Option<String>,
    // http://rs.tdwg.org/dwc/terms/phylum
    phylum: Option<String>,
    // http://rs.tdwg.org/dwc/terms/class
    class: Option<String>,
    // http://rs.tdwg.org/dwc/terms/order
    order: Option<String>,
    // http://rs.tdwg.org/dwc/terms/family
    family: Option<String>,
    // http://rs.tdwg.org/dwc/terms/genus
    genus: Option<String>,
}


#[derive(Clone, Queryable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::users)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}



#[derive(Clone, Queryable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema::gnl)]
pub struct ArgaTaxon {
    id: Uuid,

    // http://rs.tdwg.org/dwc/terms/scientificName
    scientific_name: Option<String>,
    // http://rs.tdwg.org/dwc/terms/scientificNameAuthorship
    scientific_name_authorship: Option<String>,
    // http://rs.gbif.org/terms/1.0/canonicalName
    canonical_name: Option<String>,

    // http://rs.tdwg.org/dwc/terms/specificEpithet
    specific_epithet: Option<String>,
    // http://rs.tdwg.org/dwc/terms/infraspecificEpithet
    infraspecific_epithet: Option<String>,
    // http://rs.tdwg.org/dwc/terms/taxonRank
    taxon_rank: Option<String>,
    // http://rs.tdwg.org/dwc/terms/nameAccordingTo
    name_according_to: Option<String>,
    // http://rs.tdwg.org/dwc/terms/namePublishedIn
    name_published_in: Option<String>,
    // http://rs.tdwg.org/dwc/terms/taxonomicStatus
    taxonomic_status: Option<String>,
    // http://rs.tdwg.org/dwc/terms/taxonRemarks
    taxon_remarks: Option<String>,

    // http://rs.tdwg.org/dwc/terms/kingdom
    kingdom: Option<String>,
    // http://rs.tdwg.org/dwc/terms/phylum
    phylum: Option<String>,
    // http://rs.tdwg.org/dwc/terms/class
    class: Option<String>,
    // http://rs.tdwg.org/dwc/terms/order
    order: Option<String>,
    // http://rs.tdwg.org/dwc/terms/family
    family: Option<String>,
    // http://rs.tdwg.org/dwc/terms/genus
    genus: Option<String>,

    source: Option<String>,
    taxa_lists_id: Option<Uuid>,
}
