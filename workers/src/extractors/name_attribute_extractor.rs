use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use csv::StringRecord;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use tracing::info;
use uuid::Uuid;

use arga_core::models::{NameAttribute, AttributeValueType, AttributeCategory};
use crate::error::Error;
use crate::matchers::dataset_matcher::{DatasetMap, match_datasets, DatasetRecord};
use crate::matchers::name_matcher::{match_records, NameRecord, NameMatch};

use super::utils::parse_naive_date_time;


type PgPool = Pool<ConnectionManager<PgConnection>>;
type MatchedRecords = Vec<(NameMatch, Record)>;


#[derive(Debug, Clone)]
pub enum AttributeValue {
    Boolean(bool),
    Integer(i64),
    Decimal(BigDecimal),
    String(String),
    Timestamp(NaiveDateTime),
}


#[derive(Debug, Clone)]
struct Record {
    scientific_name: Option<String>,
    canonical_name: Option<String>,
    dataset_id: Option<String>,
    attributes: HashMap<String, AttributeValue>,
}

impl From<Record> for NameRecord {
    fn from(value: Record) -> Self {
        Self {
            scientific_name: value.scientific_name,
            canonical_name: value.canonical_name,
        }
    }
}

impl From<Record> for DatasetRecord {
    fn from(value: Record) -> Self {
        Self { global_id: value.dataset_id.unwrap_or_default() }
    }
}


/// Extract conservation statuses from a CSV file
pub fn extract(path: PathBuf, pool: &mut PgPool) -> Result<Vec<NameAttribute>, Error> {
    let mut reader = csv::Reader::from_path(&path)?;
    let headers = reader.headers()?.clone();

    let mut records: Vec<Record> = Vec::new();
    for row in reader.into_records() {
        let row = row?;
        records.push(decompose_row(&headers, row)?);
    }

    // match the records to a dataset
    let datasets = match_datasets(&records, pool);

    // match the records to names in the database. this will filter out any names
    // that could not be matched
    let records = match_records(records, pool);
    let attrs = extract_attributes(&datasets, records)?;
    Ok(attrs)
}


fn decompose_row(headers: &StringRecord, row: StringRecord) -> Result<Record, Error> {
    let mut record = Record {
        scientific_name: None,
        canonical_name: None,
        dataset_id: None,
        attributes: HashMap::new(),
    };

    for (header, field) in headers.iter().zip(row.into_iter()) {
        match header {
            "scientific_name" => { record.scientific_name = Some(field.to_string()); },
            "canonical_name" => { record.canonical_name = Some(field.to_string()); },
            "dataset_id" => { record.dataset_id = Some(field.to_string()); },
            header => {
                if !field.trim().is_empty() {
                    record.attributes.insert(header.to_string(), infer_type(field));
                }
            },
        };
    }

    Ok(record)
}


fn infer_type(value: &str) -> AttributeValue {
    if value.to_lowercase() == "true" {
        AttributeValue::Boolean(true)
    }
    else if value.to_lowercase() == "false" {
        AttributeValue::Boolean(false)
    }
    else if let Ok(timestamp) = parse_naive_date_time(value) {
        AttributeValue::Timestamp(timestamp)
    }
    else if let Ok(integer) = str::parse::<i64>(value) {
        AttributeValue::Integer(integer)
    }
    else if let Ok(decimal) = BigDecimal::from_str(value) {
        AttributeValue::Decimal(decimal)
    }
    else {
        AttributeValue::String(value.to_string())
    }
}


fn extract_attributes(datasets: &DatasetMap, records: MatchedRecords) -> Result<Vec<NameAttribute>, Error> {
    info!(total=records.len(), "Extracting name attributes");

    let attrs: Result<Vec<Vec<NameAttribute>>, Error> = records.into_par_iter().map(|(name, row)| {
        let dataset_id = row.dataset_id.clone().unwrap_or_default();

        let attributes = match datasets.get(&dataset_id) {
            None => vec![],
            Some(dataset) => into_name_attribute(&dataset.id, &name, row)?,
        };

        Ok(attributes)
    }).collect();

    let attrs = attrs?;
    let attrs: Vec<NameAttribute> = attrs.into_iter().flatten().collect();

    info!(attributes=attrs.len(), "Extracting name attributes finished");
    Ok(attrs)
}


fn into_name_attribute(dataset_id: &Uuid, name: &NameMatch, row: Record) -> Result<Vec<NameAttribute>, Error> {
    let mut attributes = Vec::new();

    for (key, attr) in row.attributes {
        attributes.push(NameAttribute {
            id: Uuid::new_v4(),
            dataset_id: dataset_id.clone(),
            name_id: name.id.clone(),
            name: key,
            category: AttributeCategory::BushfireRecovery,
            value_type: match &attr {
                AttributeValue::Boolean(_) => AttributeValueType::Boolean,
                AttributeValue::Integer(_) => AttributeValueType::Integer,
                AttributeValue::Decimal(_) => AttributeValueType::Decimal,
                AttributeValue::String(_) => AttributeValueType::String,
                AttributeValue::Timestamp(_) => AttributeValueType::Timestamp,
            },
            value_bool: match attr {
                AttributeValue::Boolean(val) => Some(val),
                _ => None,
            },
            value_int: match attr {
                AttributeValue::Integer(val) => Some(val),
                _ => None,
            },
            value_decimal: match &attr {
                AttributeValue::Decimal(val) => Some(val.clone()),
                _ => None,
            },
            value_str: match &attr {
                AttributeValue::String(val) => Some(val.clone()),
                _ => None,
            },
            value_timestamp: match &attr {
                AttributeValue::Timestamp(val) => Some(val.clone()),
                _ => None,
            },
        });
    }

    Ok(attributes)
}
