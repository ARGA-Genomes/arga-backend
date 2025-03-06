use arga_core::models::AttributeValueType;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{database::schema, http::Error};

use super::PgPool;

#[derive(Queryable)]
pub struct ListGroup {
    pub name: String,
    pub value_type: AttributeValueType,
    pub value_bool: Option<bool>,
    pub value_int: Option<i64>,
    pub value_decimal: Option<bigdecimal::BigDecimal>,
    pub value_str: Option<String>,
    pub value_timestamp: Option<chrono::NaiveDateTime>, // or chrono::DateTime<chrono::Utc>
    pub source_id: Uuid,
    pub source_name: String,
}

#[derive(Clone)]
pub struct ListGroupProvider {
    pub pool: PgPool,
}

impl ListGroupProvider {
    pub async fn find(&self, source_name: &str) -> Result<Vec<ListGroup>, Error> {
        use schema::{datasets, name_attributes, sources};

        let mut conn = self.pool.get().await?;

        let records = name_attributes::table
            .inner_join(datasets::table)
            .inner_join(sources::table.on(sources::id.eq(datasets::source_id)))
            .filter(sources::name.eq(source_name))
            .select((
                name_attributes::name,
                name_attributes::value_type,
                name_attributes::value_bool,
                name_attributes::value_int,
                name_attributes::value_decimal,
                name_attributes::value_str,
                name_attributes::value_timestamp,
                datasets::source_id,
                sources::name,
            ))
            .load::<ListGroup>(&mut conn)
            .await?;

        Ok(records)
    }
}
