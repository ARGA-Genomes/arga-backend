use arga_core::models;
use async_graphql::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[graphql(remote = "models::PublicationType")]
pub enum PublicationType {
    Book,
    BookChapter,
    JournalArticle,
    JournalVolume,
    ProceedingsPaper,
    Url,
}

#[derive(SimpleObject)]
pub struct Publication {
    pub entity_id: String,
    pub title: Option<String>,
    pub authors: Option<Vec<String>>,
    pub published_year: Option<i32>,
    pub published_date: Option<DateTime<Utc>>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub doi: Option<String>,
    pub source_urls: Option<Vec<String>>,
    pub publication_type: Option<PublicationType>,
    pub citation: Option<String>,
}

impl From<models::Publication> for Publication {
    fn from(value: models::Publication) -> Self {
        Self {
            entity_id: value.entity_id,
            title: value.title,
            authors: value.authors.map(|i| i.into_iter().filter_map(|v| v).collect()),
            published_year: value.published_year,
            published_date: value.published_date,
            language: value.language,
            publisher: value.publisher,
            doi: value.doi,
            source_urls: value.source_urls.map(|i| i.into_iter().filter_map(|v| v).collect()),
            publication_type: value.publication_type.map(|t| t.into()),
            citation: value.citation,
        }
    }
}
