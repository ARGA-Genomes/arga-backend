use chrono::{DateTime, Utc};
use diesel::{Queryable, Insertable};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use super::{schema, schema_gnl};


#[derive(Queryable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema::taxon_source)]
pub struct TaxonSource {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "schema::sql_types::TaxonomicStatus"]
pub enum TaxonomicStatus {
    Valid,
    Undescribed,
    SpeciesInquirenda,
    Hybrid,
    Synonym,
    Invalid,
}

impl Default for TaxonomicStatus {
    fn default() -> Self {
        TaxonomicStatus::Invalid
    }
}

#[derive(Queryable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema::taxa)]
pub struct Taxon {
    pub id: Uuid,
    pub source: Uuid,
    pub name_id: Uuid,

    pub status: TaxonomicStatus,
    pub scientific_name: String,
    pub canonical_name: Option<String>,

    pub kingdom: Option<String>,
    pub phylum: Option<String>,
    pub class: Option<String>,
    pub order: Option<String>,
    pub family: Option<String>,
    pub tribe: Option<String>,
    pub genus: Option<String>,
    pub specific_epithet: Option<String>,

    pub subphylum: Option<String>,
    pub subclass: Option<String>,
    pub suborder: Option<String>,
    pub subfamily: Option<String>,
    pub subtribe: Option<String>,
    pub subgenus: Option<String>,
    pub subspecific_epithet: Option<String>,

    pub superclass: Option<String>,
    pub superorder: Option<String>,
    pub superfamily: Option<String>,
    pub supertribe: Option<String>,

    pub order_authority: Option<String>,
    pub family_authority: Option<String>,
    pub genus_authority: Option<String>,
    pub species_authority: Option<String>,

    // pub name_according_to: Option<String>,
    // pub name_published_in: Option<String>,
}

#[derive(Queryable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema::taxon_history)]
pub struct TaxonHistory {
    pub id: Uuid,
    pub old_taxon_id: Uuid,
    pub new_taxon_id: Uuid,
    pub changed_by: Option<String>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Queryable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema::taxon_remarks)]
pub struct TaxonRemarks {
    pub id: Uuid,
    pub taxon_id: Uuid,
    pub remark: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Queryable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema_gnl::species)]
pub struct Species {
    pub id: Uuid,
    pub source: Uuid,
    pub name_id: Uuid,

    pub status: TaxonomicStatus,
    pub scientific_name: String,
    pub canonical_name: Option<String>,

    pub kingdom: Option<String>,
    pub phylum: Option<String>,
    pub class: Option<String>,
    pub order: Option<String>,
    pub family: Option<String>,
    pub tribe: Option<String>,
    pub genus: Option<String>,
    pub specific_epithet: Option<String>,

    pub subphylum: Option<String>,
    pub subclass: Option<String>,
    pub suborder: Option<String>,
    pub subfamily: Option<String>,
    pub subtribe: Option<String>,
    pub subgenus: Option<String>,
    pub subspecific_epithet: Option<String>,

    pub superclass: Option<String>,
    pub superorder: Option<String>,
    pub superfamily: Option<String>,
    pub supertribe: Option<String>,

    pub order_authority: Option<String>,
    pub family_authority: Option<String>,
    pub genus_authority: Option<String>,
    pub species_authority: Option<String>,

    pub subspecies: Option<Vec<String>>,
    pub window_rank: i64,
}

#[derive(Queryable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema_gnl::undescribed_species)]
pub struct UndescribedSpecies {
    pub genus: String,
    pub genus_authority: Option<String>,
    pub names: Vec<String>,
}


#[derive(Queryable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema::user_taxa_lists)]
pub struct UserTaxaList {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub priority: i32,
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


#[derive(Clone, Queryable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema_gnl::ranked_taxa)]
pub struct RankedTaxon {
    pub id: Uuid,
    pub taxa_lists_id: Uuid,
    pub name_id: Uuid,

    pub scientific_name: Option<String>,
    pub scientific_name_authorship: Option<String>,
    pub canonical_name: Option<String>,

    pub specific_epithet: Option<String>,
    pub infraspecific_epithet: Option<String>,
    pub taxon_rank: Option<String>,
    pub name_according_to: Option<String>,
    pub name_published_in: Option<String>,
    pub taxonomic_status: Option<String>,
    pub taxon_remarks: Option<String>,

    pub kingdom: Option<String>,
    pub phylum: Option<String>,
    pub class: Option<String>,
    pub order: Option<String>,
    pub family: Option<String>,
    pub genus: Option<String>,

    pub list_name: String,
    pub taxa_priority: i32,
}


#[derive(Clone, Queryable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::users)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}


#[derive(Debug, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "schema::sql_types::JobStatus"]
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



#[derive(Clone, Queryable, Insertable, Debug, Default, Serialize, Deserialize)]
#[diesel(table_name = schema::names)]
pub struct Name {
    pub id: Uuid,
    pub scientific_name: String,
    pub canonical_name: Option<String>,
    pub authorship: Option<String>,
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
}


#[derive(Clone, Debug, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "schema::sql_types::RegionType"]
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
#[ExistingTypePath = "schema::sql_types::NameListType"]
pub enum NameListType {
    Regions,
    ConservationStatus,
    Specimen,
    Marker,
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
    pub institution_code: Option<String>,
    pub collection_code: Option<String>,
    pub catalog_number: Option<String>,
    pub recorded_by: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::organisms)]
pub struct Organism {
    pub id: Uuid,
    pub name_id: Uuid,
    pub organism_id: Option<String>,
    pub organism_name: Option<String>,
    pub organism_scope: Option<String>,
    pub associated_organisms: Option<String>,
    pub previous_identifications: Option<String>,
    pub remarks: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::events)]
pub struct Event {
    pub id: Uuid,
    pub parent_event_id: Option<Uuid>,
    pub event_id: Option<String>,
    pub field_number: Option<String>,
    pub event_date: Option<chrono::NaiveDate>,
    pub habitat: Option<String>,
    pub sampling_protocol: Option<String>,
    pub sampling_size_value: Option<String>,
    pub sampling_size_unit: Option<String>,
    pub sampling_effort: Option<String>,
    pub field_notes: Option<String>,
    pub event_remarks: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::collection_events)]
pub struct CollectionEvent {
    pub id: Uuid,
    pub event_id: Uuid,
    pub specimen_id: Uuid,
    pub organism_id: Option<Uuid>,

    pub occurrence_id: Option<String>,
    pub catalog_number: Option<String>,
    pub record_number: Option<String>,
    pub individual_count: Option<String>,
    pub organism_quantity: Option<String>,
    pub organism_quantity_type: Option<String>,
    pub sex: Option<String>,
    pub life_stage: Option<String>,
    pub reproductive_condition: Option<String>,
    pub behavior: Option<String>,
    pub establishment_means: Option<String>,
    pub degree_of_establishment: Option<String>,
    pub pathway: Option<String>,
    pub occurrence_status: Option<String>,
    pub preparation: Option<String>,
    pub other_catalog_numbers: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::sequencing_events)]
pub struct SequencingEvent {
    pub id: Uuid,
    pub event_id: Uuid,
    pub specimen_id: Uuid,
    pub organism_id: Option<Uuid>,

    pub sequence_id: Option<String>,
    pub genbank_accession: Option<String>,
    pub target_gene: Option<String>,
    pub dna_sequence: Option<String>,
}

#[derive(Clone, Queryable, Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = schema::sequencing_run_events)]
pub struct SequencingRunEvent {
    pub id: Uuid,
    pub sequencing_event_id: Uuid,

    pub trace_id: Option<String>,
    pub trace_name: Option<String>,
    pub trace_link: Option<String>,
    pub sequencing_date: Option<chrono::NaiveDateTime>,
    pub sequencing_center: Option<String>,
    pub target_gene: Option<String>,
    pub direction: Option<String>,
    pub pcr_primer_name_forward: Option<String>,
    pub pcr_primer_name_reverse: Option<String>,
    pub sequence_primer_forward_name: Option<String>,
    pub sequence_primer_reverse_name: Option<String>,
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


#[derive(Debug, Queryable, Insertable, Default, Clone)]
#[diesel(table_name = schema::assemblies)]
pub struct Assembly {
    pub id: Uuid,
    pub name_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    pub accession: String,
    pub bioproject_id: Option<String>,
    pub biosample_id: Option<String>,
    pub material_sample_id: Option<String>,
    pub nuccore: Option<String>,
    pub refseq_category: Option<String>,
    pub specific_host: Option<String>,
    pub clone_strain: Option<String>,
    pub version_status: Option<String>,
    pub contam_screen_input: Option<String>,
    pub release_type: Option<String>,
    pub genome_rep: Option<String>,
    pub gbrs_paired_asm: Option<String>,
    pub paired_asm_comp: Option<String>,
    pub excluded_from_refseq: Option<String>,
    pub relation_to_type_material: Option<String>,
    pub asm_not_live_date: Option<String>,
    pub other_catalog_numbers: Option<String>,
    pub recorded_by: Option<String>,
    pub genetic_accession_uri: Option<String>,
    pub event_date: Option<String>,
}

#[derive(Debug, Queryable, Insertable, Default)]
#[diesel(table_name = schema::assembly_stats)]
pub struct AssemblyStats {
    pub id: Uuid,
    pub assembly_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    pub total_length: Option<i32>,
    pub spanned_gaps: Option<i32>,
    pub unspanned_gaps: Option<i32>,
    pub region_count: Option<i32>,
    pub scaffold_count: Option<i32>,
    pub scaffold_n50: Option<i32>,
    pub scaffold_l50: Option<i32>,
    pub scaffold_n75: Option<i32>,
    pub scaffold_n90: Option<i32>,
    pub contig_count: Option<i32>,
    pub contig_n50: Option<i32>,
    pub contig_l50: Option<i32>,
    pub total_gap_length: Option<i32>,
    pub molecule_count: Option<i32>,
    pub top_level_count: Option<i32>,
    pub component_count: Option<i32>,
    pub gc_perc: Option<i32>,
}


#[derive(Debug, Queryable, Insertable, Default)]
#[diesel(table_name = schema::biosamples)]
pub struct BioSample {
    pub id: Uuid,
    pub name_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    pub accession: String,
    pub sra: Option<String>,
    pub submission_date: Option<String>,
    pub publication_date: Option<String>,
    pub last_update: Option<String>,
    pub title: Option<String>,
    pub owner: Option<String>,
    pub attributes: Option<serde_json::Value>,
}


#[derive(Debug, Clone, Queryable, Insertable, Default)]
#[diesel(table_name = schema::markers)]
pub struct Marker {
    pub id: Uuid,
    pub name_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    pub accession: String,
    pub material_sample_id: Option<String>,
    pub gb_acs: Option<String>,
    pub marker_code: Option<String>,
    pub nucleotide: Option<String>,
    pub recorded_by: Option<String>,

    pub list_id: Uuid,
    pub version: Option<String>,
    pub basepairs: Option<i64>,
    pub type_: Option<String>,
    pub shape: Option<String>,
    pub source_url: Option<String>,
    pub fasta_url: Option<String>,
    pub extra_data: Option<serde_json::Value>,
}
