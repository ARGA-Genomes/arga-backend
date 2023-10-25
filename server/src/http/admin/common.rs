use std::collections::HashMap;

use axum::Json;
use serde::Serialize;

use crate::http::error::InternalError;
use crate::database::extensions::pagination;


#[derive(Debug, Serialize)]
pub struct Page<T> {
    total: i64,
    records: Vec<T>,
}

pub type PageResult<T> = Result<Json<Page<T>>, InternalError>;

impl<T> From<Vec<(T, i64)>> for Page<T> {
    fn from(value: Vec<(T, i64)>) -> Self {
        let page: pagination::Page<T> = value.into();
        Self {
            total: page.total,
            records: page.records,
        }
    }
}


pub fn parse_int_param(params: &HashMap<String, String>, name: &str, default: i64) -> i64 {
    let val = params.get(name).map(|val| val.parse::<i64>().unwrap_or(default)).unwrap_or(default);
    if val <= 0 { 1 } else { val }
}
