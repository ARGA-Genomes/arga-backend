use std::collections::HashMap;
use std::path::PathBuf;

use arga_core::models::{Atom, Operation, TaxonomicStatus};
use arga_core::schema;
use chrono::{DateTime, Utc};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use xxhash_rust::xxh3::Xxh3;

use crate::data::oplogger::LWWMap;
use crate::data::ParseError;
use crate::data::{oplogger::ObjectFrame, Error};

use super::{hlc::HybridTimestamp, Version};

type PgPool = Pool<ConnectionManager<PgConnection>>;

fn parse_date_time(value: &str) -> Result<DateTime<Utc>, ParseError> {
    if let Ok(datetime) = DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%#z") {
        return Ok(datetime.into());
    }
    if let Ok(datetime) = DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.3f%#z") {
        return Ok(datetime.into());
    }
    Ok(DateTime::parse_from_rfc3339(value)?.into())
}

fn date_time_from_str<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    parse_date_time(&s).map_err(serde::de::Error::custom)
}

fn taxonomic_status_from_str<'de, D>(deserializer: D) -> Result<TaxonomicStatus, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    str_to_taxonomic_status(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, Deserialize)]
struct Record {
    acted_on: String,
    scientific_name: String,
    #[serde(deserialize_with = "taxonomic_status_from_str")]
    taxonomic_status: TaxonomicStatus,
    nomenclatural_act: String,
    source_url: Option<String>,
    publication: Option<String>,

    #[serde(deserialize_with = "date_time_from_str")]
    created_at: DateTime<Utc>,
    #[serde(deserialize_with = "date_time_from_str")]
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Default, Serialize)]
pub struct OriginalDescription {
    pub acted_on: String,
    pub scientific_name: String,
    pub taxonomic_status: TaxonomicStatus,
    pub nomenclatural_act: String,
    pub source_url: Option<String>,
    pub publication: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub entity_id: String,
}

impl From<LWWMap> for OriginalDescription {
    fn from(value: LWWMap) -> Self {
        let mut desc = OriginalDescription {
            entity_id: value.entity_id,
            ..Default::default()
        };

        for val in value.map.into_values() {
            match val {
                Atom::Empty => {}
                Atom::ScientificName { value } => desc.scientific_name = value,
                Atom::ActedOn { value } => desc.acted_on = value,
                Atom::TaxonomicStatus(status) => desc.taxonomic_status = status,
                Atom::NomenclaturalAct { value } => desc.nomenclatural_act = value,
                Atom::SourceUrl { value } => desc.source_url = Some(value),
                Atom::Publication { value } => desc.publication = Some(value),
                Atom::CreatedAt(dt) => desc.created_at = dt,
                Atom::UpdatedAt(dt) => desc.updated_at = dt,
            }
        }

        desc
    }
}

pub struct NomenclaturalActs {
    pub path: PathBuf,
    pub dataset_id: Uuid,
}

impl NomenclaturalActs {
    pub fn original_descriptions(&self) -> Result<Vec<Operation>, Error> {
        let mut records: Vec<Record> = Vec::new();
        for row in csv::Reader::from_path(&self.path)?.deserialize() {
            records.push(row?);
        }

        let ts: HybridTimestamp = Utc::now().into();
        let mut version = Version(ts);
        let mut operations = Vec::new();

        for record in records.into_iter() {
            // the uniqueness of a nomenclatural act is the acted on taxon,
            // the scientific name, and the act itself.
            let mut hasher = Xxh3::new();
            hasher.update(record.acted_on.as_bytes());
            hasher.update(record.scientific_name.as_bytes());
            let hash = hasher.digest();

            let mut object = ObjectFrame::new(self.dataset_id, version, hash.to_string());

            if record.acted_on == record.scientific_name {
                object.update(Atom::ActedOn {
                    value: "Biota".to_string(),
                });
            } else {
                object.update(Atom::ActedOn {
                    value: record.acted_on,
                });
            }

            object.update(Atom::ScientificName {
                value: record.scientific_name,
            });

            object.update(Atom::TaxonomicStatus(record.taxonomic_status));
            object.update(Atom::NomenclaturalAct {
                value: record.nomenclatural_act,
            });
            object.update(Atom::CreatedAt(record.created_at));
            object.update(Atom::UpdatedAt(record.updated_at));

            if let Some(source_url) = record.source_url {
                object.update(Atom::SourceUrl { value: source_url });
            }

            if let Some(publication) = record.publication {
                object.update(Atom::Publication { value: publication });
            }

            operations.extend(object.operations);
            version = object.previous;
        }

        Ok(operations)
    }
}

pub fn process(path: PathBuf, dataset_id: String) -> Result<(), Error> {
    let dataset_id = find_dataset_id(dataset_id)?;
    let acts = NomenclaturalActs { path, dataset_id };
    let records = acts.original_descriptions()?;
    import_operations(records)?;
    Ok(())
}

pub fn reduce() -> Result<(), Error> {
    reduce_operations()
}

fn find_dataset_id(dataset_id: String) -> Result<Uuid, Error> {
    use schema::datasets::dsl::*;

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    let uuid = datasets
        .filter(global_id.eq(dataset_id))
        .select(id)
        .get_result::<Uuid>(&mut conn)?;

    Ok(uuid)
}

fn import_operations(records: Vec<Operation>) -> Result<(), Error> {
    use schema::operation_logs::dsl::*;

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    for chunk in records.chunks(1000) {
        diesel::insert_into(operation_logs)
            .values(chunk)
            .execute(&mut conn)?;
    }

    Ok(())
}

fn reduce_operations() -> Result<(), Error> {
    use schema::operation_logs::dsl::*;

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    let operations = operation_logs
        .order(operation_id.asc())
        .load::<Operation>(&mut conn)?;

    let mut grouped_ops: HashMap<String, Vec<Operation>> = HashMap::new();
    for op in operations.into_iter() {
        grouped_ops
            .entry(op.object_id.clone())
            .or_default()
            .push(op)
    }

    let mut entities = HashMap::new();

    for (key, ops) in grouped_ops.into_iter() {
        let mut map = LWWMap::new(key.clone());
        map.reduce(&ops);
        entities.insert(key, map);
    }

    let mut writer = csv::Writer::from_writer(std::io::stdout());

    for map in entities.into_values() {
        writer.serialize(OriginalDescription::from(map))?;
    }

    Ok(())
}

fn get_pool() -> Result<PgPool, Error> {
    let url = arga_core::get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    let pool = Pool::builder().build(manager)?;
    Ok(pool)
}

// based on https://rs.gbif.org/vocabulary/gbif/taxonomic_status.xml
fn str_to_taxonomic_status(value: &str) -> Result<TaxonomicStatus, ParseError> {
    match value.to_lowercase().as_str() {
        "valid" => Ok(TaxonomicStatus::Accepted),
        "valid name" => Ok(TaxonomicStatus::Accepted),
        "accepted" => Ok(TaxonomicStatus::Accepted),
        "accepted name" => Ok(TaxonomicStatus::Accepted),

        "undescribed" => Ok(TaxonomicStatus::Undescribed),
        "species inquirenda" => Ok(TaxonomicStatus::SpeciesInquirenda),
        "manuscript name" => Ok(TaxonomicStatus::ManuscriptName),
        "hybrid" => Ok(TaxonomicStatus::Hybrid),

        "synonym" => Ok(TaxonomicStatus::Synonym),
        "junior synonym" => Ok(TaxonomicStatus::Synonym),
        "later synonym" => Ok(TaxonomicStatus::Synonym),

        "invalid" => Ok(TaxonomicStatus::Unaccepted),
        "invalid name" => Ok(TaxonomicStatus::Unaccepted),
        "unaccepted" => Ok(TaxonomicStatus::Unaccepted),
        "unaccepted name" => Ok(TaxonomicStatus::Unaccepted),
        // "excluded" => Ok(TaxonomicStatus::Unaccepted),
        "informal" => Ok(TaxonomicStatus::Informal),
        "informal name" => Ok(TaxonomicStatus::Informal),

        "placeholder" => Ok(TaxonomicStatus::Placeholder),

        "basionym" => Ok(TaxonomicStatus::Basionym),
        "nomenclatural synonym" => Ok(TaxonomicStatus::NomenclaturalSynonym),
        "taxonomic synonym" => Ok(TaxonomicStatus::TaxonomicSynonym),
        "replaced synonym" => Ok(TaxonomicStatus::ReplacedSynonym),

        "orthographic variant" => Ok(TaxonomicStatus::OrthographicVariant),
        "misapplied" => Ok(TaxonomicStatus::Misapplied),
        "excluded" => Ok(TaxonomicStatus::Excluded),
        "alternative name" => Ok(TaxonomicStatus::AlternativeName),

        "pro parte misapplied" => Ok(TaxonomicStatus::ProParteMisapplied),
        "pro parte taxonomic synonym" => Ok(TaxonomicStatus::ProParteTaxonomicSynonym),

        "doubtful misapplied" => Ok(TaxonomicStatus::DoubtfulMisapplied),
        "doubtful taxonomic synonym" => Ok(TaxonomicStatus::DoubtfulTaxonomicSynonym),
        "doubtful pro parte misapplied" => Ok(TaxonomicStatus::DoubtfulProParteMisapplied),
        "doubtful pro parte taxonomic synonym" => {
            Ok(TaxonomicStatus::DoubtfulProParteTaxonomicSynonym)
        }

        val => Err(ParseError::InvalidValue(val.to_string())),
    }
}
