use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::{Index, IndexReader, ReloadPolicy, TantivyError};
use tantivy::schema::{Schema, TEXT, STORED, Field};
use tracing::error;
use uuid::Uuid;


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("tantivy index error")]
    Tantivy(#[from] tantivy::TantivyError),
}


#[derive(Debug)]
pub enum SearchItem {
    Species { uuid: Uuid, score: f32 },
    UndescribedSpecies { genus: String, score: f32 },
}


#[derive(Debug, Clone)]
struct SearchFields {
    name_id: Field,
    canonical_name: Field,
    subspecies: Field,
    // synonyms: Field,

    genus: Field,
    undescribed_species: Field,
}


#[derive(Clone)]
pub struct SearchIndex {
    index: Index,
    reader: IndexReader,
    fields: SearchFields,
}


impl SearchIndex {
    pub fn open() -> Result<SearchIndex, Error> {
        let schema = SearchIndex::schema()?;
        let index = Index::open_in_dir(".index")?;
        let reader = index.reader_builder().reload_policy(ReloadPolicy::OnCommit).try_into()?;

        let fields = SearchFields {
            name_id: get_field(&schema, "name_id")?,
            canonical_name: get_field(&schema, "canonical_name")?,
            subspecies: get_field(&schema, "subspecies")?,
            // synonyms: get_field(&schema, "synonyms")?,

            genus: get_field(&schema, "genus")?,
            undescribed_species: get_field(&schema, "undescribed_species")?,
        };

        Ok(SearchIndex {
            index,
            reader,
            fields,
        })
    }

    pub fn schema() -> tantivy::Result<Schema> {
        // define the data we want to be search on
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("name_id", TEXT | STORED);
        // schema_builder.add_text_field("common_names", TEXT | STORED);
        schema_builder.add_text_field("canonical_name", TEXT);
        schema_builder.add_text_field("subspecies", TEXT);
        schema_builder.add_text_field("synonyms", TEXT | STORED);

        schema_builder.add_text_field("genus", TEXT | STORED);
        schema_builder.add_text_field("undescribed_species", TEXT);

        let schema = schema_builder.build();

        Ok(schema)
    }

    pub fn species(&self, query: &str) -> Result<Vec<SearchItem>, Error> {
        let searcher = self.reader.searcher();

        let query_parser = QueryParser::for_index(&self.index, vec![
            self.fields.canonical_name,
            self.fields.subspecies,
            self.fields.genus,
            self.fields.undescribed_species,
        ]);
        // query_parser.set_field_boost(common_names, 50.0);
        let parsed_query = query_parser.parse_query(query).unwrap();

        let mut records = Vec::with_capacity(20);

        let top_docs = searcher.search(&parsed_query, &TopDocs::with_limit(20))?;
        for (score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;
            // let common_names = doc.get_all(common_names).map(|v| v.as_text().unwrap_or_default()).into_vec();

            // we have a species result
            if let Some(name_id) = doc.get_first(self.fields.name_id) {
                match Uuid::parse_str(name_id.as_text().unwrap_or_default()) {
                    Ok(uuid) => records.push(SearchItem::Species { uuid, score }),
                    Err(err) => error!(?err, ?name_id, "failed to parse name_id"),
                };
            }
            else if let Some(genus) = doc.get_first(self.fields.genus) {
                if let Some(genus) = genus.as_text() {
                    records.push(SearchItem::UndescribedSpecies { genus: genus.to_string(), score });
                }
            }
        }

        Ok(records)
    }
}


fn get_field(schema: &Schema, name: &str) -> Result<Field, Error> {
    let field = schema.get_field(name).ok_or(TantivyError::FieldNotFound(name.to_string()))?;
    Ok(field)
}
