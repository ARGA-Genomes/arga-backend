use diesel::{Queryable, Insertable};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::schema;
use crate::schema_gnl;


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
    pub id: Uuid,
    pub taxa_lists_id: Uuid,
    pub name_id: Uuid,

    // http://rs.tdwg.org/dwc/terms/scientificName
    pub scientific_name: Option<String>,
    // http://rs.tdwg.org/dwc/terms/scientificNameAuthorship
    pub scientific_name_authorship: Option<String>,
    // http://rs.gbif.org/terms/1.0/canonicalName
    pub canonical_name: Option<String>,

    // http://rs.tdwg.org/dwc/terms/specificEpithet
    pub specific_epithet: Option<String>,
    // http://rs.tdwg.org/dwc/terms/infraspecificEpithet
    pub infraspecific_epithet: Option<String>,
    // http://rs.tdwg.org/dwc/terms/taxonRank
    pub taxon_rank: Option<String>,
    // http://rs.tdwg.org/dwc/terms/nameAccordingTo
    pub name_according_to: Option<String>,
    // http://rs.tdwg.org/dwc/terms/namePublishedIn
    pub name_published_in: Option<String>,
    // http://rs.tdwg.org/dwc/terms/taxonomicStatus
    pub taxonomic_status: Option<String>,
    // http://rs.tdwg.org/dwc/terms/taxonRemarks
    pub taxon_remarks: Option<String>,

    // http://rs.tdwg.org/dwc/terms/kingdom
    pub kingdom: Option<String>,
    // http://rs.tdwg.org/dwc/terms/phylum
    pub phylum: Option<String>,
    // http://rs.tdwg.org/dwc/terms/class
    pub class: Option<String>,
    // http://rs.tdwg.org/dwc/terms/order
    pub order: Option<String>,
    // http://rs.tdwg.org/dwc/terms/family
    pub family: Option<String>,
    // http://rs.tdwg.org/dwc/terms/genus
    pub genus: Option<String>,
}


#[derive(Clone, Queryable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::users)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}



#[derive(Clone, Queryable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema_gnl::gnl)]
pub struct ArgaTaxon {
    pub id: Uuid,

    // http://rs.tdwg.org/dwc/terms/scientificName
    pub scientific_name: Option<String>,
    // http://rs.tdwg.org/dwc/terms/scientificNameAuthorship
    pub scientific_name_authorship: Option<String>,
    // http://rs.gbif.org/terms/1.0/canonicalName
    pub canonical_name: Option<String>,

    // http://rs.tdwg.org/dwc/terms/specificEpithet
    pub specific_epithet: Option<String>,
    // http://rs.tdwg.org/dwc/terms/infraspecificEpithet
    pub infraspecific_epithet: Option<String>,
    // http://rs.tdwg.org/dwc/terms/taxonRank
    pub taxon_rank: Option<String>,
    // http://rs.tdwg.org/dwc/terms/nameAccordingTo
    pub name_according_to: Option<String>,
    // http://rs.tdwg.org/dwc/terms/namePublishedIn
    pub name_published_in: Option<String>,
    // http://rs.tdwg.org/dwc/terms/taxonomicStatus
    pub taxonomic_status: Option<String>,
    // http://rs.tdwg.org/dwc/terms/taxonRemarks
    pub taxon_remarks: Option<String>,

    // http://rs.tdwg.org/dwc/terms/kingdom
    pub kingdom: Option<String>,
    // http://rs.tdwg.org/dwc/terms/phylum
    pub phylum: Option<String>,
    // http://rs.tdwg.org/dwc/terms/class
    pub class: Option<String>,
    // http://rs.tdwg.org/dwc/terms/order
    pub order: Option<String>,
    // http://rs.tdwg.org/dwc/terms/family
    pub family: Option<String>,
    // http://rs.tdwg.org/dwc/terms/genus
    pub genus: Option<String>,

    pub source: Option<String>,
    pub taxa_lists_id: Option<Uuid>,
}



#[derive(Debug, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::JobStatus"]
pub enum JobStatus {
    Pending,
    Initialized,
    Running,
    Completed,
    Failed,
    Dead,
}

#[derive(Queryable, Debug, Deserialize)]
#[diesel(table_name = schema::jobs)]
pub struct Job {
    pub id: Uuid,
    pub status: JobStatus,
    pub worker: String,
    pub payload: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}


#[derive(Debug, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::AttributeDataType"]
pub enum AttributeDataType {
    String,
    Text,
    Integer,
    Boolean,
    Timestamp,
    Array,
}

pub enum AttributeDataValue {
    String(String),
    Text(String),
    Integer(i64),
    Boolean(bool),
    Timestamp(chrono::DateTime<chrono::Utc>),
    Array(Vec<String>),
}

#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct Attribute {
    pub id: Uuid,
    pub name: String,
    pub data_type: AttributeDataType,
    pub description: Option<String>,
    pub reference_url: Option<String>,
}

pub trait AttributeParser {
    fn parse(&self, value: &Attribute) -> Option<AttributeDataValue>;
}


#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = schema::objects)]
pub struct Object {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub attribute_id: Uuid,
    pub value_id: Uuid,
}

#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = schema::object_values_string)]
pub struct ObjectValueString {
    pub id: Uuid,
    pub value: String,
}

#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = schema::object_values_text)]
pub struct ObjectValueText {
    pub id: Uuid,
    pub value: String,
}

#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = schema::object_values_array)]
pub struct ObjectValueArray {
    pub id: Uuid,
    pub value: Vec<String>,
}


#[derive(Queryable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema::media)]
pub struct Media {
    pub id: Uuid,

    pub media_id: Option<i64>,
    pub media_type: Option<String>,
    pub format: Option<String>,
    pub identifier: Option<String>,
    pub references: Option<String>,
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    pub creator: Option<String>,
    pub publisher: Option<String>,
    pub license: Option<String>,
    pub rights_holder: Option<String>,
    pub catalog_number: Option<i64>,
}

#[derive(Queryable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema::media_observations)]
pub struct MediaObservation {
    id: Uuid,

    media_id: Option<i64>,
    scientific_name: Option<String>,
    basis_of_record: Option<String>,
    institution_code: Option<String>,
    collection_code: Option<String>,
    dataset_name: Option<String>,
    captive: Option<String>,
    event_date: Option<chrono::DateTime<chrono::Utc>>,
    license: Option<String>,
    rights_holder: Option<String>,
}


#[derive(Debug, Queryable)]
#[diesel(table_name = schema_gnl::eav_strings)]
pub struct ObjectString {
    pub object_id: Uuid,
    pub entity_id: Uuid,
    pub attribute_id: Uuid,
    pub value_id: Uuid,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Queryable)]
#[diesel(table_name = schema_gnl::eav_arrays)]
pub struct ObjectArray {
    pub object_id: Uuid,
    pub entity_id: Uuid,
    pub attribute_id: Uuid,
    pub value_id: Uuid,
    pub name: String,
    pub value: Vec<String>,
}


#[derive(Clone, Queryable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema::names)]
pub struct Name {
    pub id: Uuid,
    pub scientific_name: String,
    pub canonical_name: Option<String>,
    pub authorship: Option<String>,
    pub rank: String,
}

#[derive(Clone, Queryable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema_gnl::common_names)]
pub struct CommonName {
    pub id: Uuid,
    pub vernacular_name: String,
    pub vernacular_language: Option<String>,
    pub scientific_name: String,
    pub scientific_name_authorship: Option<String>,
    pub canonical_name: Option<String>,
    pub rank: Option<String>,
}


#[derive(Clone, Debug, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::RegionType"]
pub enum RegionType {
    Ibra,
    Imcra,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::regions)]
pub struct Regions {
    pub id: Uuid,
    pub name_id: Uuid,
    pub region_type: RegionType,
    pub values: Vec<String>,
}


#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::taxon_photos)]
pub struct TaxonPhoto {
    pub id: Uuid,
    pub name_id: Uuid,
    pub url: String,
    pub source: Option<String>,
    pub publisher: Option<String>,
    pub license: Option<String>,
    pub rights_holder: Option<String>,
}


#[derive(Clone, Debug, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::NameListType"]
pub enum NameListType {
    Regions,
    ConservationStatus,
    Specimen,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::name_lists)]
pub struct NameList {
    pub id: Uuid,
    pub list_type: NameListType,
    pub name: String,
    pub description: Option<String>,
}


#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::conservation_statuses)]
pub struct ConservationStatus {
    pub id: Uuid,
    pub list_id: Uuid,
    pub name_id: Uuid,
    pub status: String,
    pub state: Option<String>,
    pub source: Option<String>,
}


#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::specimens)]
pub struct Specimen {
    pub id: Uuid,
    pub list_id: Uuid,
    pub name_id: Uuid,
    pub type_status: String,
    pub institution_name: Option<String>,
    pub organism_id: Option<String>,
    pub locality: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub details: Option<String>,
    pub remarks: Option<String>,
}


// postgres arrays allows nulls to be entered into an array
// so diesel will treat it as an array of optional numbers.
// we shorten the type here for readability
pub type IntArray = Vec<Option<i32>>;

#[derive(Clone, Queryable, Debug, Serialize, Deserialize)]
pub struct TraceFile {
    pub id: Uuid,
    pub name_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    pub metadata: serde_json::Value,

    pub peak_locations_user: Option<IntArray>,
    pub peak_locations_basecaller: Option<IntArray>,
    pub quality_values_user: Option<IntArray>,
    pub quality_values_basecaller: Option<IntArray>,
    pub sequences_user: Option<IntArray>,
    pub sequences_basecaller: Option<IntArray>,

    pub measurements_voltage: Option<IntArray>,
    pub measurements_current: Option<IntArray>,
    pub measurements_power: Option<IntArray>,
    pub measurements_temperature: Option<IntArray>,

    pub analyzed_g: Option<IntArray>,
    pub analyzed_a: Option<IntArray>,
    pub analyzed_t: Option<IntArray>,
    pub analyzed_c: Option<IntArray>,

    pub raw_g: Option<IntArray>,
    pub raw_a: Option<IntArray>,
    pub raw_t: Option<IntArray>,
    pub raw_c: Option<IntArray>,
}
