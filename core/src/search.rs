use chrono::{NaiveDate, NaiveDateTime};
use tantivy::collector::{Count, TopDocs};
use tantivy::query::QueryParser;
use tantivy::schema::{Field, STORED, STRING, Schema, SchemaBuilder, TEXT};
use tantivy::{Document, Index, IndexReader, ReloadPolicy, TantivyError};
use tracing::error;
use uuid::Uuid;

use super::models::TaxonomicStatus;


pub type SearchResult = Result<(Vec<SearchItem>, usize), Error>;


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("tantivy index error")]
    Tantivy(#[from] tantivy::TantivyError),

    #[error("tantivy query error")]
    QueryError(#[from] tantivy::query::QueryParserError),

    #[error("Parse error: {0}")]
    ParseError(String),
}

#[derive(Debug, Clone)]
pub enum DataType {
    Taxon,
    Genome,
    Locus,
    Specimen,
}


#[derive(Debug)]
pub enum SearchItem {
    Species(SpeciesItem),
    Genome(GenomeItem),
    Locus(LocusItem),
    Specimen(SpecimenItem),
}

#[derive(Debug)]
pub struct SpeciesItem {
    pub name_id: Uuid,
    pub status: TaxonomicStatus,
    pub score: f32,

    pub canonical_name: Option<String>,
    pub rank: Option<String>,
    pub subspecies: Vec<String>,
    pub synonyms: Vec<String>,
    pub common_names: Vec<String>,

    pub kingdom: Option<String>,
    pub phylum: Option<String>,
    pub class: Option<String>,
    pub order: Option<String>,
    pub family: Option<String>,
    pub genus: Option<String>,

    pub regnum: Option<String>,
    pub division: Option<String>,
    pub classis: Option<String>,
    pub ordo: Option<String>,
    pub familia: Option<String>,
}

#[derive(Debug)]
pub struct GenomeItem {
    pub name_id: Uuid,
    pub status: TaxonomicStatus,
    pub score: f32,

    pub canonical_name: Option<String>,
    pub accession: String,
    pub genome_rep: Option<String>,
    pub data_source: Option<String>,
    pub level: Option<String>,
    pub assembly_type: Option<String>,
    pub reference_genome: bool,
    pub release_date: Option<NaiveDate>,
    pub source_uri: Option<String>,
}

#[derive(Debug)]
pub struct LocusItem {
    pub name_id: Uuid,
    pub status: TaxonomicStatus,
    pub score: f32,

    pub canonical_name: Option<String>,
    pub accession: String,
    pub locus_type: Option<String>,
    pub data_source: Option<String>,
    pub voucher_status: Option<String>,
    pub event_date: Option<NaiveDateTime>,
    pub event_location: Option<String>,
}

#[derive(Debug)]
pub struct SpecimenItem {
    pub name_id: Uuid,
    pub status: TaxonomicStatus,
    pub score: f32,

    pub canonical_name: Option<String>,
    pub accession: String,
    pub data_source: Option<String>,
    pub institution_code: Option<String>,
    pub collection_code: Option<String>,
    pub recorded_by: Option<String>,
    pub identified_by: Option<String>,
    pub event_date: Option<NaiveDateTime>,
    pub event_location: Option<String>,
}


#[derive(Debug, Clone)]
struct CommonFields {
    data_type: Field,
    name_id: Field,
    status: Field,
    canonical_name: Field,
}

#[derive(Debug, Clone)]
struct TaxonFields {
    rank: Field,
    subspecies: Field,
    synonyms: Field,
    common_names: Field,
    kingdom: Field,
    phylum: Field,
    class: Field,
    order: Field,
    family: Field,
    genus: Field,
    regnum: Field,
    division: Field,
    classis: Field,
    ordo: Field,
    familia: Field,
}

#[derive(Debug, Clone)]
struct GenomeFields {
    accession: Field,
    genome_rep: Field,
    data_source: Field,
    level: Field,
    assembly_type: Field,
    reference_genome: Field,
    release_date: Field,
    source_uri: Field,
}

#[derive(Debug, Clone)]
struct LocusFields {
    accession: Field,
    locus_type: Field,
    data_source: Field,
    voucher_status: Field,
    event_date: Field,
    event_location: Field,
}

#[derive(Debug, Clone)]
struct SpecimenFields {
    accession: Field,
    data_source: Field,
    institution_code: Field,
    collection_code: Field,
    recorded_by: Field,
    identified_by: Field,
    event_date: Field,
    event_location: Field,
}


impl TryFrom<&str> for DataType {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Taxon" => Ok(DataType::Taxon),
            "Genome" => Ok(DataType::Genome),
            "Locus" => Ok(DataType::Locus),
            "Specimen" => Ok(DataType::Specimen),
            val => Err(Error::ParseError(format!("Unkown data type: {}", val).to_string())),
        }
    }
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Taxon => f.write_str("Taxon"),
            DataType::Genome => f.write_str("Genome"),
            DataType::Locus => f.write_str("Locus"),
            DataType::Specimen => f.write_str("Specimen"),
        }?;
        Ok(())
    }
}


#[derive(Clone)]
pub struct SearchIndex {
    index: Index,
    reader: IndexReader,

    common: CommonFields,
    taxon: TaxonFields,
    genome: GenomeFields,
    locus: LocusFields,
    specimen: SpecimenFields,
}

impl SearchIndex {
    pub fn open() -> Result<SearchIndex, Error> {
        let schema = SearchIndex::schema()?;
        let index = Index::open_in_dir(".index")?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;

        let common = CommonFields {
            data_type: get_field(&schema, "data_type")?,
            name_id: get_field(&schema, "name_id")?,
            status: get_field(&schema, "status")?,
            canonical_name: get_field(&schema, "canonical_name")?,
        };
        let taxon = TaxonFields {
            rank: get_field(&schema, "rank")?,
            subspecies: get_field(&schema, "subspecies")?,
            synonyms: get_field(&schema, "synonyms")?,
            common_names: get_field(&schema, "common_names")?,
            kingdom: get_field(&schema, "kingdom")?,
            phylum: get_field(&schema, "phylum")?,
            class: get_field(&schema, "class")?,
            order: get_field(&schema, "order")?,
            family: get_field(&schema, "family")?,
            genus: get_field(&schema, "genus")?,
            regnum: get_field(&schema, "regnum")?,
            division: get_field(&schema, "division")?,
            classis: get_field(&schema, "classis")?,
            ordo: get_field(&schema, "ordo")?,
            familia: get_field(&schema, "familia")?,
        };
        let genome = GenomeFields {
            accession: get_field(&schema, "accession")?,
            genome_rep: get_field(&schema, "genome_rep")?,
            data_source: get_field(&schema, "data_source")?,
            level: get_field(&schema, "level")?,
            assembly_type: get_field(&schema, "assembly_type")?,
            reference_genome: get_field(&schema, "reference_genome")?,
            release_date: get_field(&schema, "release_date")?,
            source_uri: get_field(&schema, "source_uri")?,
        };
        let locus = LocusFields {
            accession: get_field(&schema, "accession")?,
            locus_type: get_field(&schema, "locus_type")?,
            data_source: get_field(&schema, "data_source")?,
            voucher_status: get_field(&schema, "voucher_status")?,
            event_date: get_field(&schema, "event_date")?,
            event_location: get_field(&schema, "event_location")?,
        };
        let specimen = SpecimenFields {
            accession: get_field(&schema, "accession")?,
            data_source: get_field(&schema, "data_source")?,
            institution_code: get_field(&schema, "institution_code")?,
            collection_code: get_field(&schema, "collection_code")?,
            recorded_by: get_field(&schema, "recorded_by")?,
            identified_by: get_field(&schema, "identified_by")?,
            event_date: get_field(&schema, "event_date")?,
            event_location: get_field(&schema, "event_location")?,
        };

        Ok(SearchIndex {
            index,
            reader,
            common,
            taxon,
            genome,
            locus,
            specimen,
        })
    }

    pub fn schema() -> tantivy::Result<Schema> {
        // define the data we want to be search on
        let mut schema_builder = Schema::builder();

        Self::common_schema(&mut schema_builder);
        Self::taxon_schema(&mut schema_builder);
        Self::genome_schema(&mut schema_builder);
        Self::locus_schema(&mut schema_builder);
        Self::specimen_schema(&mut schema_builder);

        let schema = schema_builder.build();
        Ok(schema)
    }

    pub fn common_schema(schema_builder: &mut SchemaBuilder) {
        schema_builder.add_text_field("data_type", STRING | STORED);
        schema_builder.add_text_field("name_id", STRING | STORED);
        schema_builder.add_text_field("status", STRING | STORED);
        schema_builder.add_text_field("canonical_name", TEXT | STORED);

        schema_builder.add_text_field("accession", STRING | STORED);
        schema_builder.add_text_field("data_source", TEXT | STORED);
    }

    pub fn taxon_schema(schema_builder: &mut SchemaBuilder) {
        schema_builder.add_text_field("rank", TEXT | STORED);
        schema_builder.add_text_field("subspecies", TEXT | STORED);
        schema_builder.add_text_field("synonyms", TEXT | STORED);
        schema_builder.add_text_field("common_names", TEXT | STORED);

        schema_builder.add_text_field("kingdom", STRING | STORED);
        schema_builder.add_text_field("phylum", STRING | STORED);
        schema_builder.add_text_field("class", STRING | STORED);
        schema_builder.add_text_field("order", STRING | STORED);
        schema_builder.add_text_field("family", STRING | STORED);
        schema_builder.add_text_field("genus", STRING | STORED);

        schema_builder.add_text_field("regnum", STRING | STORED);
        schema_builder.add_text_field("division", STRING | STORED);
        schema_builder.add_text_field("classis", STRING | STORED);
        schema_builder.add_text_field("ordo", STRING | STORED);
        schema_builder.add_text_field("familia", STRING | STORED);
    }

    pub fn genome_schema(schema_builder: &mut SchemaBuilder) {
        schema_builder.add_text_field("genome_rep", STRING | STORED);
        schema_builder.add_text_field("level", TEXT | STORED);
        schema_builder.add_text_field("assembly_type", TEXT | STORED);
        schema_builder.add_bool_field("reference_genome", STORED);
        schema_builder.add_date_field("release_date", STORED);
        schema_builder.add_text_field("source_uri", STRING | STORED);
    }

    pub fn locus_schema(schema_builder: &mut SchemaBuilder) {
        schema_builder.add_text_field("locus_type", STRING | STORED);
        schema_builder.add_text_field("voucher_status", STRING | STORED);
        schema_builder.add_date_field("event_date", STORED);
        schema_builder.add_text_field("event_location", STORED);
    }

    pub fn specimen_schema(schema_builder: &mut SchemaBuilder) {
        schema_builder.add_text_field("institution_code", STRING | STORED);
        schema_builder.add_text_field("collection_code", TEXT | STORED);
        schema_builder.add_text_field("recorded_by", TEXT | STORED);
        schema_builder.add_text_field("identified_by", TEXT | STORED);
    }

    pub fn taxonomy(&self, query: &str, page: usize, per_page: usize) -> SearchResult {
        let query = format!("data_type:{} {query}", DataType::Taxon);
        self.all(&query, page, per_page)
    }

    pub fn genomes(&self, query: &str, page: usize, per_page: usize) -> SearchResult {
        let query = format!("data_type:{} {query}", DataType::Genome);
        self.all(&query, page, per_page)
    }

    pub fn loci(&self, query: &str, page: usize, per_page: usize) -> SearchResult {
        let query = format!("data_type:{} {query}", DataType::Locus);
        self.all(&query, page, per_page)
    }

    pub fn specimens(&self, query: &str, page: usize, per_page: usize) -> SearchResult {
        let query = format!("data_type:{} {query}", DataType::Specimen);
        self.all(&query, page, per_page)
    }

    pub fn all(&self, query: &str, page: usize, per_page: usize) -> SearchResult {
        let searcher = self.reader.searcher();
        let offset = per_page * page.checked_sub(1).unwrap_or(0);

        // set the fields that the query should search on
        let mut query_parser = QueryParser::for_index(
            &self.index,
            vec![
                self.common.canonical_name,
                self.taxon.subspecies,
                self.taxon.synonyms,
                self.taxon.common_names,
                self.genome.accession,
                self.genome.level,
                self.genome.assembly_type,
                self.genome.data_source,
                self.locus.accession,
                self.locus.locus_type,
                self.locus.data_source,
                self.specimen.accession,
                self.specimen.data_source,
                self.specimen.institution_code,
                self.specimen.collection_code,
                self.specimen.recorded_by,
                self.taxon.kingdom,
                self.taxon.phylum,
                self.taxon.class,
                self.taxon.order,
                self.taxon.family,
                self.taxon.genus,
                self.taxon.regnum,
                self.taxon.division,
                self.taxon.classis,
                self.taxon.ordo,
                self.taxon.familia,
            ],
        );

        let query = format!(
            "(data_type:{}^100.0 OR data_type:{}^50.0 OR data_type:{}^10.0 OR data_type:{}) {query}",
            DataType::Taxon,
            DataType::Genome,
            DataType::Locus,
            DataType::Specimen,
        );

        query_parser.set_conjunction_by_default();
        let parsed_query = query_parser.parse_query(&query)?;

        let mut records = Vec::new();

        let top_docs = searcher.search(&parsed_query, &TopDocs::with_limit(per_page).and_offset(offset))?;
        let count = searcher.search(&parsed_query, &Count)?;

        for (score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;

            let data_type = get_data_type(&doc, self.common.data_type);
            let name_id = get_uuid(&doc, self.common.name_id);

            // this should always unwrap but we cannot guarantee that the index isn't
            // corrupted or wrongly used, so only process results that have all mandatory fields
            if let (Some(data_type), Some(name_id)) = (data_type, name_id) {
                let status = match get_text(&doc, self.common.status) {
                    None => TaxonomicStatus::Unaccepted,
                    Some(value) => serde_json::from_str(&value).unwrap_or(TaxonomicStatus::Unaccepted),
                };

                let item = match data_type {
                    DataType::Taxon => SearchItem::Species(SpeciesItem {
                        name_id,
                        status,
                        score,
                        canonical_name: get_text(&doc, self.common.canonical_name),
                        rank: get_text(&doc, self.taxon.rank),
                        subspecies: get_all_text(&doc, self.taxon.subspecies),
                        synonyms: get_all_text(&doc, self.taxon.synonyms),
                        common_names: get_all_text(&doc, self.taxon.common_names),
                        kingdom: get_text(&doc, self.taxon.kingdom),
                        phylum: get_text(&doc, self.taxon.phylum),
                        class: get_text(&doc, self.taxon.class),
                        order: get_text(&doc, self.taxon.order),
                        family: get_text(&doc, self.taxon.family),
                        genus: get_text(&doc, self.taxon.genus),
                        regnum: get_text(&doc, self.taxon.regnum),
                        division: get_text(&doc, self.taxon.division),
                        classis: get_text(&doc, self.taxon.classis),
                        ordo: get_text(&doc, self.taxon.ordo),
                        familia: get_text(&doc, self.taxon.familia),
                    }),
                    DataType::Genome => SearchItem::Genome(GenomeItem {
                        name_id,
                        status,
                        score,
                        canonical_name: get_text(&doc, self.common.canonical_name),
                        accession: get_text(&doc, self.genome.accession).unwrap_or_default(),
                        genome_rep: get_text(&doc, self.genome.genome_rep),
                        data_source: get_text(&doc, self.genome.data_source),
                        level: get_text(&doc, self.genome.level),
                        assembly_type: get_text(&doc, self.genome.assembly_type),
                        reference_genome: get_bool(&doc, self.genome.reference_genome).unwrap_or(false),
                        release_date: get_date(&doc, self.genome.release_date),
                        source_uri: get_text(&doc, self.genome.source_uri),
                    }),
                    DataType::Locus => SearchItem::Locus(LocusItem {
                        name_id,
                        status,
                        score,
                        canonical_name: get_text(&doc, self.common.canonical_name),
                        accession: get_text(&doc, self.locus.accession).unwrap_or_default(),
                        locus_type: get_text(&doc, self.locus.locus_type),
                        data_source: get_text(&doc, self.locus.data_source),
                        voucher_status: get_text(&doc, self.locus.voucher_status),
                        event_date: get_datetime(&doc, self.locus.event_date),
                        event_location: get_text(&doc, self.locus.event_location),
                    }),
                    DataType::Specimen => SearchItem::Specimen(SpecimenItem {
                        name_id,
                        status,
                        score,
                        canonical_name: get_text(&doc, self.common.canonical_name),
                        accession: get_text(&doc, self.specimen.accession).unwrap_or_default(),
                        data_source: get_text(&doc, self.specimen.data_source),
                        institution_code: get_text(&doc, self.specimen.institution_code),
                        collection_code: get_text(&doc, self.specimen.collection_code),
                        recorded_by: get_text(&doc, self.specimen.recorded_by),
                        identified_by: get_text(&doc, self.specimen.identified_by),
                        event_date: get_datetime(&doc, self.specimen.event_date),
                        event_location: get_text(&doc, self.specimen.event_location),
                    }),
                };

                records.push(item);
            }
        }

        Ok((records, count))
    }
}


fn get_field(schema: &Schema, name: &str) -> Result<Field, Error> {
    let field = schema
        .get_field(name)
        .ok_or(TantivyError::FieldNotFound(name.to_string()))?;
    Ok(field)
}

fn get_data_type(doc: &Document, field: Field) -> Option<DataType> {
    match doc.get_first(field) {
        None => None,
        Some(value) => match value.as_text() {
            Some(val) => match DataType::try_from(val) {
                Ok(data_type) => Some(data_type),
                Err(err) => {
                    error!(?err, "Failed to read data_type");
                    None
                }
            },
            None => None,
        },
    }
}

fn get_uuid(doc: &Document, field: Field) -> Option<Uuid> {
    match doc.get_first(field) {
        None => None,
        Some(value) => match Uuid::parse_str(value.as_text().unwrap_or_default()) {
            Ok(uuid) => Some(uuid),
            Err(err) => {
                error!(?err, ?value, "failed to parse name_id");
                None
            }
        },
    }
}

fn get_text(doc: &Document, field: Field) -> Option<String> {
    match doc.get_first(field) {
        Some(value) => value.as_text().map(|v| v.to_string()),
        None => None,
    }
}

fn get_all_text(doc: &Document, field: Field) -> Vec<String> {
    let mut values = Vec::new();
    for value in doc.get_all(field) {
        if let Some(text) = value.as_text() {
            values.push(text.to_string());
        }
    }
    values
}

fn get_bool(doc: &Document, field: Field) -> Option<bool> {
    match doc.get_first(field) {
        Some(value) => value.as_bool(),
        None => None,
    }
}

fn get_date(doc: &Document, field: Field) -> Option<NaiveDate> {
    match doc.get_first(field) {
        Some(value) => match value.as_date() {
            Some(dt) => chrono::DateTime::from_timestamp(dt.into_timestamp_secs(), 0).map(|dt| dt.date_naive()),
            None => None,
        },
        None => None,
    }
}

fn get_datetime(doc: &Document, field: Field) -> Option<NaiveDateTime> {
    match doc.get_first(field) {
        Some(value) => match value.as_date() {
            Some(dt) => chrono::DateTime::from_timestamp(dt.into_timestamp_secs(), 0).map(|dt| dt.naive_utc()),
            None => None,
        },
        None => None,
    }
}
