use tantivy::{Index, IndexReader, ReloadPolicy};
use tantivy::schema::{Schema, TEXT, STORED};

pub mod search;


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("tantivy index error")]
    Tantivy(#[from] tantivy::TantivyError),
}


#[derive(Clone)]
pub struct SearchIndex {
    schema: Schema,
    index: Index,
    reader: IndexReader,
}


impl SearchIndex {
    pub fn open() -> Result<SearchIndex, Error> {
        let schema = SearchIndex::schema()?;
        let index = Index::open_in_dir(".index")?;
        let reader = index.reader_builder().reload_policy(ReloadPolicy::OnCommit).try_into()?;

        Ok(SearchIndex {
            schema,
            index,
            reader,
        })
    }

    pub fn schema() -> tantivy::Result<Schema> {
        // define the data we want to be search on
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("scientific_name", TEXT | STORED);
        schema_builder.add_text_field("canonical_name", TEXT);
        schema_builder.add_text_field("rank", TEXT);
        schema_builder.add_text_field("common_names", TEXT | STORED);

        let schema = schema_builder.build();

        Ok(schema)
    }
}
