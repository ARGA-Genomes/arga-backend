use async_graphql::*;
use uuid::Uuid;

use crate::database::{list_groups, Database};
use crate::http::Error;

use super::common::attributes::AttributeValueType;


pub struct ListGroups;

#[derive(SimpleObject)]
pub struct ListGroupDetails {
    pub name: String,
    pub value_type: AttributeValueType,
    pub value_bool: Option<bool>,
    pub value_int: Option<i64>,
    pub value_decimal: Option<String>,
    pub value_str: Option<String>,
    pub value_timestamp: Option<chrono::NaiveDateTime>, // or chrono::DateTime<chrono::Utc>
    pub source_id: Uuid,
    pub source_name: String,
}

impl From<list_groups::ListGroup> for ListGroupDetails {
    fn from(value: list_groups::ListGroup) -> Self {
        Self {
            name: value.name,
            value_type: value.value_type.into(),
            value_bool: value.value_bool,
            value_int: value.value_int,
            value_decimal: value.value_decimal.map(|d| d.to_string()),
            value_str: value.value_str,
            value_timestamp: value.value_timestamp,
            source_id: value.source_id,
            source_name: value.source_name,
        }
    }
}

impl ListGroups {
    pub async fn new(db: &Database, source_name: &String) -> Result<Vec<ListGroupDetails>, Error> {
        let groups = db.list_groups.find(&source_name).await?;
        let groups = groups.into_iter().map(|m| m.into()).collect();
        Ok(groups)
    }
}
