use async_trait::async_trait;
use polars::prelude::IntoVec;
use tantivy::{query::QueryParser, collector::TopDocs, TantivyError};

use crate::index::search::{FullTextSearch, FullTextSearchResult, FullTextSearchItem, TaxonItem, FullTextType};

use super::SearchIndex;


#[async_trait]
impl FullTextSearch for SearchIndex {
    type Error = super::Error;

    async fn full_text(&self, query: &str) -> Result<FullTextSearchResult, Self::Error> {
        let searcher = self.reader.searcher();

        let scientific_name = self.schema.get_field("scientific_name").ok_or(TantivyError::FieldNotFound("scientific_name".to_string()))?;
        let canonical_name = self.schema.get_field("canonical_name").ok_or(TantivyError::FieldNotFound("canonical_name".to_string()))?;
        let common_names = self.schema.get_field("common_names").ok_or(TantivyError::FieldNotFound("common_names".to_string()))?;

        let mut query_parser = QueryParser::for_index(&self.index, vec![
            common_names,
            canonical_name,
            scientific_name,
        ]);
        query_parser.set_field_boost(common_names, 50.0);
        let parsed_query = query_parser.parse_query(query).unwrap();

        let mut records = Vec::with_capacity(20);

        let top_docs = searcher.search(&parsed_query, &TopDocs::with_limit(20))?;
        for (score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;
            let scientific_name = doc.get_first(scientific_name).map(|v| v.as_text().unwrap_or_default());
            let canonical_name = doc.get_first(canonical_name).map(|v| v.as_text().unwrap_or_default());
            let common_names = doc.get_all(common_names).map(|v| v.as_text().unwrap_or_default()).into_vec();

            records.push(FullTextSearchItem::Taxon(TaxonItem {
                scientific_name: scientific_name.unwrap_or_default().to_string(),
                canonical_name: canonical_name.map(|v| v.to_string()),
                rank: None,
                taxonomic_status: None,
                common_names,
                score,
                r#type: FullTextType::Taxon,
            }));
        }

        Ok(FullTextSearchResult {
            records,
        })
    }
}
