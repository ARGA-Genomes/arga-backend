use std::collections::HashMap;
use std::path::PathBuf;

use arga_core::crdt::lww::Map;
use arga_core::crdt::{Frame, Version};
use arga_core::models::{Action, DatasetVersion, NomenclaturalActAtom, NomenclaturalActOperation, TaxonomicStatus};
use arga_core::schema;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use xxhash_rust::xxh3::Xxh3;

use crate::data::oplogger::get_pool;
use crate::data::{Error, ParseError};

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

impl From<Map<NomenclaturalActAtom>> for OriginalDescription {
    fn from(value: Map<NomenclaturalActAtom>) -> Self {
        use NomenclaturalActAtom::*;

        let mut desc = OriginalDescription {
            entity_id: value.entity_id,
            ..Default::default()
        };

        for val in value.atoms.into_values() {
            match val {
                Empty => {}
                ScientificName(value) => desc.scientific_name = value,
                ActedOn(value) => desc.acted_on = value,
                TaxonomicStatus(status) => desc.taxonomic_status = status,
                NomenclaturalAct(value) => desc.nomenclatural_act = value,
                SourceUrl(value) => desc.source_url = Some(value),
                Publication(value) => desc.publication = Some(value),
                CreatedAt(value) => desc.created_at = value,
                UpdatedAt(value) => desc.updated_at = value,

                _ => {}
            }
        }

        desc
    }
}

pub struct NomenclaturalActFrame {
    dataset_version_id: Uuid,
    entity_id: String,
    frame: Frame<NomenclaturalActOperation>,
}

impl NomenclaturalActFrame {
    pub fn create(dataset_version_id: Uuid, entity_id: String, last_version: Version) -> NomenclaturalActFrame {
        let mut frame = Frame::new(last_version);

        frame.push(NomenclaturalActOperation {
            operation_id: frame.next.into(),
            parent_id: frame.current.into(),
            entity_id: entity_id.clone(),
            dataset_version_id,
            action: Action::Create,
            atom: NomenclaturalActAtom::Empty,
        });

        NomenclaturalActFrame {
            dataset_version_id,
            entity_id,
            frame,
        }
    }

    pub fn push(&mut self, atom: NomenclaturalActAtom) {
        let operation_id: BigDecimal = self.frame.next.into();
        let parent_id = self
            .frame
            .operations
            .last()
            .map(|op| op.operation_id.clone())
            .unwrap_or(operation_id.clone());

        let op = NomenclaturalActOperation {
            operation_id,
            parent_id,
            dataset_version_id: self.dataset_version_id,
            entity_id: self.entity_id.clone(),
            action: Action::Update,
            atom,
        };

        self.frame.push(op);
    }

    pub fn operations(&self) -> &Vec<NomenclaturalActOperation> {
        &self.frame.operations
    }
}

pub struct NomenclaturalActs {
    pub path: PathBuf,
    pub dataset_version_id: Uuid,
}

impl NomenclaturalActs {
    pub fn original_descriptions(&self) -> Result<Vec<NomenclaturalActOperation>, Error> {
        use NomenclaturalActAtom::*;

        let mut records: Vec<Record> = Vec::new();
        for row in csv::Reader::from_path(&self.path)?.deserialize() {
            records.push(row?);
        }

        let mut last_version = Version::new();
        let mut operations: Vec<NomenclaturalActOperation> = Vec::new();

        for record in records.into_iter() {
            // the uniqueness of a nomenclatural act is the acted on taxon,
            // the scientific name, and the act itself.
            let mut hasher = Xxh3::new();
            hasher.update(record.acted_on.as_bytes());
            hasher.update(record.scientific_name.as_bytes());
            let hash = hasher.digest();

            let mut frame = NomenclaturalActFrame::create(self.dataset_version_id, hash.to_string(), last_version);

            if record.acted_on == record.scientific_name {
                frame.push(ActedOn("Biota".to_string()));
            }
            else {
                frame.push(ActedOn(record.acted_on));
            }

            frame.push(ScientificName(record.scientific_name));
            frame.push(TaxonomicStatus(record.taxonomic_status));
            frame.push(NomenclaturalAct(record.nomenclatural_act));
            frame.push(CreatedAt(record.created_at));
            frame.push(UpdatedAt(record.updated_at));

            if let Some(source_url) = record.source_url {
                frame.push(SourceUrl(source_url));
            }

            if let Some(publication) = record.publication {
                frame.push(Publication(publication));
            }

            last_version = frame.frame.current;
            operations.extend(frame.frame.operations);
        }

        Ok(operations)
    }
}

pub fn process(path: PathBuf, dataset_version: DatasetVersion) -> Result<(), Error> {
    let acts = NomenclaturalActs {
        path,
        dataset_version_id: dataset_version.id,
    };
    let records = acts.original_descriptions()?;
    let reduced = reduce_operations(records)?;

    import_operations(reduced)?;
    Ok(())
}

pub fn reduce() -> Result<(), Error> {
    reduce_acts()
}

fn import_operations(records: Vec<NomenclaturalActOperation>) -> Result<(), Error> {
    use schema::nomenclatural_act_logs::dsl::*;

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    for chunk in records.chunks(1000) {
        diesel::insert_into(nomenclatural_act_logs)
            .values(chunk)
            .execute(&mut conn)?;
    }

    Ok(())
}

fn merge_operations(
    records: Vec<NomenclaturalActOperation>,
) -> Result<HashMap<String, Vec<NomenclaturalActOperation>>, Error> {
    use schema::nomenclatural_act_logs::dsl::*;

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    let operations = nomenclatural_act_logs
        .order(operation_id.asc())
        .load::<NomenclaturalActOperation>(&mut conn)?;

    let mut grouped_ops: HashMap<String, Vec<NomenclaturalActOperation>> = HashMap::new();
    for op in operations.into_iter() {
        grouped_ops.entry(op.entity_id.clone()).or_default().push(op)
    }

    for op in records.into_iter() {
        grouped_ops.entry(op.entity_id.clone()).or_default().push(op)
    }

    Ok(grouped_ops)
}

fn reduce_operations(records: Vec<NomenclaturalActOperation>) -> Result<Vec<NomenclaturalActOperation>, Error> {
    let entities = merge_operations(records)?;
    let mut merged_ops = Vec::new();

    for (key, ops) in entities.into_iter() {
        let mut map = Map::new(key);
        let reduced = map.reduce(&ops);
        merged_ops.extend(reduced);
    }

    Ok(merged_ops)
}

fn reduce_acts() -> Result<(), Error> {
    let entities = merge_operations(vec![])?;
    let mut acts = Vec::new();

    for (key, ops) in entities.into_iter() {
        let mut map = Map::new(key);
        map.reduce(&ops);
        acts.push(OriginalDescription::from(map));
    }

    let mut writer = csv::Writer::from_writer(std::io::stdout());

    for act in acts {
        writer.serialize(act)?;
    }

    Ok(())
}

// fn reduce_operations() -> Result<(), Error> {
//     use schema::operation_logs::dsl::*;

//     let pool = get_pool()?;
//     let mut conn = pool.get()?;

//     let operations = operation_logs
//         .order(operation_id.asc())
//         .load::<Operation>(&mut conn)?;

//     let mut grouped_ops: HashMap<String, Vec<Operation>> = HashMap::new();
//     for op in operations.into_iter() {
//         grouped_ops
//             .entry(op.object_id.clone())
//             .or_default()
//             .push(op)
//     }

//     let mut entities = HashMap::new();

//     for (key, ops) in grouped_ops.into_iter() {
//         let mut map = LWWMap::new(key.clone());
//         map.reduce(&ops);
//         entities.insert(key, map);
//     }

//     let mut writer = csv::Writer::from_writer(std::io::stdout());

//     for map in entities.into_values() {
//         writer.serialize(OriginalDescription::from(map))?;
//     }

//     Ok(())
// }

// based on https://rs.gbif.org/vocabulary/gbif/taxonomic_status.xml
pub fn str_to_taxonomic_status(value: &str) -> Result<TaxonomicStatus, ParseError> {
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
        "doubtful pro parte taxonomic synonym" => Ok(TaxonomicStatus::DoubtfulProParteTaxonomicSynonym),

        val => Err(ParseError::InvalidValue(val.to_string())),
    }
}
