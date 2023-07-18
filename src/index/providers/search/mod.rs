use chrono::NaiveDateTime;
use tantivy::collector::{Count, TopDocs};
use tantivy::query::QueryParser;
use tantivy::{Index, IndexReader, ReloadPolicy, TantivyError, Document};
use tantivy::schema::{Schema, TEXT, STORED, Field, SchemaBuilder, STRING};
use tracing::error;
use uuid::Uuid;

use crate::database::models::TaxonomicStatus;
use crate::index::lists::Pagination;


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("tantivy index error")]
    Tantivy(#[from] tantivy::TantivyError),

    #[error("tantivy query error")]
    QueryError(#[from] tantivy::query::QueryParserError),
}

#[derive(Debug)]
pub struct FinalResult {
    pub(crate) results: Vec<SearchItem>,
    pub(crate) total: i32
}


#[derive(Debug)]
pub enum SearchItem {
    Species(SpeciesItem),
    Genome(GenomeItem),
}

#[derive(Debug)]
pub struct SpeciesItem {
    pub name_id: Uuid,
    pub status: TaxonomicStatus,
    pub score: f32,

    pub canonical_name: Option<String>,
    pub subspecies: Vec<String>,
    pub synonyms: Vec<String>,
    pub common_names: Vec<String>,

    pub kingdom: Option<String>,
    pub phylum: Option<String>,
    pub class: Option<String>,
    pub order: Option<String>,
    pub family: Option<String>,
    pub genus: Option<String>,
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
    pub reference_genome: bool,
    pub release_date: Option<NaiveDateTime>,
}


#[derive(Debug, Clone)]
struct CommonFields {
    name_id: Field,
    status: Field,
    canonical_name: Field,
}

#[derive(Debug, Clone)]
struct TaxonFields {
    subspecies: Field,
    synonyms: Field,
    common_names: Field,
    kingdom: Field,
    phylum: Field,
    class: Field,
    order: Field,
    family: Field,
    genus: Field,
}

#[derive(Debug, Clone)]
struct GenomeFields {
    accession: Field,
    genome_rep: Field,
    data_source: Field,
    level: Field,
    reference_genome: Field,
    release_date: Field,
}


#[derive(Clone)]
pub struct SearchIndex {
    index: Index,
    reader: IndexReader,

    common: CommonFields,
    taxon: TaxonFields,
    genome: GenomeFields,
}


impl SearchIndex {
    pub fn open() -> Result<SearchIndex, Error> {
        let schema = SearchIndex::schema()?;
        let index = Index::open_in_dir(".index")?;
        let reader = index.reader_builder().reload_policy(ReloadPolicy::OnCommit).try_into()?;

        let common = CommonFields {
            name_id: get_field(&schema, "name_id")?,
            status: get_field(&schema, "status")?,
            canonical_name: get_field(&schema, "canonical_name")?,
        };
        let taxon = TaxonFields {
            subspecies: get_field(&schema, "subspecies")?,
            synonyms: get_field(&schema, "synonyms")?,
            common_names: get_field(&schema, "common_names")?,
            kingdom: get_field(&schema, "kingdom")?,
            phylum: get_field(&schema, "phylum")?,
            class: get_field(&schema, "class")?,
            order: get_field(&schema, "order")?,
            family: get_field(&schema, "family")?,
            genus: get_field(&schema, "genus")?,
        };
        let genome = GenomeFields {
            accession: get_field(&schema, "accession")?,
            genome_rep: get_field(&schema, "genome_rep")?,
            data_source: get_field(&schema, "data_source")?,
            level: get_field(&schema, "level")?,
            reference_genome: get_field(&schema, "reference_genome")?,
            release_date: get_field(&schema, "release_date")?,
        };

        Ok(SearchIndex {
            index,
            reader,
            common,
            taxon,
            genome,
        })
    }

    pub fn schema() -> tantivy::Result<Schema> {
        // define the data we want to be search on
        let mut schema_builder = Schema::builder();

        // common fields
        schema_builder.add_text_field("data_type", STRING | STORED);
        schema_builder.add_text_field("name_id", STRING | STORED);
        schema_builder.add_text_field("status", STRING | STORED);
        schema_builder.add_text_field("canonical_name", TEXT | STORED);

        Self::taxon_schema(&mut schema_builder);
        Self::genome_schema(&mut schema_builder);

        let schema = schema_builder.build();
        Ok(schema)
    }

    pub fn taxon_schema(schema_builder: &mut SchemaBuilder) {
        schema_builder.add_text_field("subspecies", TEXT | STORED);
        schema_builder.add_text_field("synonyms", TEXT | STORED);
        schema_builder.add_text_field("common_names", TEXT | STORED);

        schema_builder.add_text_field("kingdom", STRING | STORED);
        schema_builder.add_text_field("phylum", STRING | STORED);
        schema_builder.add_text_field("class", STRING | STORED);
        schema_builder.add_text_field("order", STRING | STORED);
        schema_builder.add_text_field("family", STRING | STORED);
        schema_builder.add_text_field("genus", STRING | STORED);
    }

    pub fn genome_schema(schema_builder: &mut SchemaBuilder) {
        schema_builder.add_text_field("accession", STRING | STORED);
        schema_builder.add_text_field("genome_rep", STRING | STORED);
        schema_builder.add_text_field("data_source", TEXT | STORED);
        schema_builder.add_text_field("level", TEXT | STORED);
        schema_builder.add_bool_field("reference_genome", STORED);
        schema_builder.add_date_field("release_date", STORED);
    }

    pub fn species(&self, query: &str, pagination: Option<Pagination>) -> Result<FinalResult, Error> {
        let searcher = self.reader.searcher();
        let query = format!("data_type:taxon {query}");

        let mut offset = 0;
        let mut page_size = 20;

        if pagination.is_some() {
            offset = pagination.unwrap().page_size * (pagination.unwrap().page - 1);
            page_size = pagination.unwrap().page_size;
        }

        let mut query_parser = QueryParser::for_index(&self.index, vec![
            self.common.canonical_name,
            self.taxon.subspecies,
            self.taxon.synonyms,
            self.taxon.common_names,
        ]);
        // query_parser.set_field_boost(common_names, 50.0);
        query_parser.set_conjunction_by_default();
        let parsed_query = query_parser.parse_query(&query)?;

        let mut records = Vec::new();

        let top_docs = searcher.search(&parsed_query, &TopDocs::with_limit(page_size as usize).and_offset(offset as usize))?;
        let count = searcher.search(&parsed_query, &Count).unwrap();
        for (score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;

            if let Some(name_id) = get_uuid(&doc, self.common.name_id) {
                let status = match get_text(&doc, self.common.status) {
                    None => TaxonomicStatus::Invalid,
                    Some(value) => serde_json::from_str(&value).unwrap_or(TaxonomicStatus::Invalid),
                };

                let item = SpeciesItem {
                    name_id,
                    status,
                    score,
                    canonical_name: get_text(&doc, self.common.canonical_name),
                    subspecies: get_all_text(&doc, self.taxon.subspecies),
                    synonyms: get_all_text(&doc, self.taxon.synonyms),
                    common_names: get_all_text(&doc, self.taxon.common_names),
                    kingdom: get_text(&doc, self.taxon.kingdom),
                    phylum: get_text(&doc, self.taxon.phylum),
                    class: get_text(&doc, self.taxon.class),
                    order: get_text(&doc, self.taxon.order),
                    family: get_text(&doc, self.taxon.family),
                    genus: get_text(&doc, self.taxon.genus),
                };
                records.push(SearchItem::Species(item));
            }
        }

        Ok(FinalResult { results: records, total: count as i32 })
    }

    pub fn genomes(&self, query: &str, pagination: Option<Pagination>) -> Result<FinalResult, Error> {
        let searcher = self.reader.searcher();
        let query = format!("data_type:genome {query}");

        let mut offset = 0;
        let mut page_size = 20;

        if pagination.is_some() {
            offset = pagination.unwrap().page_size * (pagination.unwrap().page - 1);
            page_size = pagination.unwrap().page_size;
        }

        let mut query_parser = QueryParser::for_index(&self.index, vec![
            self.common.canonical_name,
            self.genome.accession,
        ]);
        query_parser.set_conjunction_by_default();
        let parsed_query = query_parser.parse_query(&query)?;

        let mut records = Vec::new();

        let top_docs = searcher.search(&parsed_query, &TopDocs::with_limit(page_size as usize).and_offset(offset as usize))?;
        let count = searcher.search(&parsed_query, &Count).unwrap();
        for (score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;

            if let Some(name_id) = get_uuid(&doc, self.common.name_id) {
                let status = match get_text(&doc, self.common.status) {
                    None => TaxonomicStatus::Invalid,
                    Some(value) => serde_json::from_str(&value).unwrap_or(TaxonomicStatus::Invalid),
                };

                let item = GenomeItem {
                    name_id,
                    status,
                    score,
                    canonical_name: get_text(&doc, self.common.canonical_name),
                    accession: get_text(&doc, self.genome.accession).unwrap_or_default(),
                    genome_rep: get_text(&doc, self.genome.genome_rep),
                    data_source: get_text(&doc, self.genome.data_source),
                    level: get_text(&doc, self.genome.level),
                    reference_genome: get_bool(&doc, self.genome.reference_genome).unwrap_or(false),
                    release_date: get_datetime(&doc, self.genome.release_date),
                };
                records.push(SearchItem::Genome(item));
            }
        }

        Ok(FinalResult { results: records, total: count as i32 })
    }
}


fn get_field(schema: &Schema, name: &str) -> Result<Field, Error> {
    let field = schema.get_field(name).ok_or(TantivyError::FieldNotFound(name.to_string()))?;
    Ok(field)
}

fn get_uuid(doc: &Document, field: Field) -> Option<Uuid> {
    match doc.get_first(field) {
        None => None,
        Some(value) => match Uuid::parse_str(value.as_text().unwrap_or_default()) {
            Ok(uuid) => Some(uuid),
            Err(err) => {
                error!(?err, ?value, "failed to parse name_id");
                None
            },
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

fn get_datetime(doc: &Document, field: Field) -> Option<NaiveDateTime> {
    match doc.get_first(field) {
        Some(value) => match value.as_date() {
            Some(dt) => NaiveDateTime::from_timestamp_opt(dt.into_timestamp_secs(), 0),
            None => None,
        },
        None => None,
    }
}
