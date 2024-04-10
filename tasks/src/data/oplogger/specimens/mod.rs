use std::collections::HashMap;
use std::path::PathBuf;

use arga_core::crdt::lww::Map;
use arga_core::crdt::{Frame, Version};
use arga_core::models::{Action, DatasetVersion, SpecimenAtom, SpecimenOperation};
use arga_core::schema;
use bigdecimal::BigDecimal;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use xxhash_rust::xxh3::Xxh3;

use crate::data::oplogger::get_pool;
use crate::data::Error;

type PgPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Debug, Clone, Deserialize)]
struct Record {
    record_id: String,
    scientific_name: Option<String>,
    canonical_name: Option<String>,

    type_status: Option<String>,
    institution_name: Option<String>,
    institution_code: Option<String>,
    collection_code: Option<String>,
    catalog_number: Option<String>,
    collected_by: Option<String>,
    identified_by: Option<String>,
    identified_date: Option<String>,
    organism_id: Option<String>,
    material_sample_id: Option<String>,
    details: Option<String>,
    remarks: Option<String>,
    identification_remarks: Option<String>,

    // location block
    locality: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
    state_province: Option<String>,
    county: Option<String>,
    municipality: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    verbatim_lat_long: Option<String>,
    elevation: Option<f64>,
    depth: Option<f64>,
    elevation_accuracy: Option<f64>,
    depth_accuracy: Option<f64>,
    location_source: Option<String>,

    // collection event block
    event_date: Option<String>,
    event_time: Option<String>,
    field_number: Option<String>,
    field_notes: Option<String>,
    record_number: Option<String>,
    individual_count: Option<String>,
    organism_quantity: Option<String>,
    organism_quantity_type: Option<String>,
    sex: Option<String>,
    genotypic_sex: Option<String>,
    phenotypic_sex: Option<String>,
    life_stage: Option<String>,
    reproductive_condition: Option<String>,
    behavior: Option<String>,
    establishment_means: Option<String>,
    degree_of_establishment: Option<String>,
    pathway: Option<String>,
    occurrence_status: Option<String>,
    preparation: Option<String>,
    other_catalog_numbers: Option<String>,
    env_broad_scale: Option<String>,
    env_local_scale: Option<String>,
    env_medium: Option<String>,
    habitat: Option<String>,
    ref_biomaterial: Option<String>,
    source_mat_id: Option<String>,
    specific_host: Option<String>,
    strain: Option<String>,
    isolate: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Specimen {
    entity_id: String,
    record_id: String,
    scientific_name: Option<String>,
    canonical_name: Option<String>,

    material_sample_id: Option<String>,
    organism_id: Option<String>,

    institution_name: Option<String>,
    institution_code: Option<String>,
    collection_code: Option<String>,
    recorded_by: Option<String>,
    identified_by: Option<String>,
    identified_date: Option<String>,

    type_status: Option<String>,
    locality: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
    state_province: Option<String>,
    county: Option<String>,
    municipality: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    elevation: Option<f64>,
    depth: Option<f64>,
    elevation_accuracy: Option<f64>,
    depth_accuracy: Option<f64>,
    location_source: Option<String>,

    details: Option<String>,
    remarks: Option<String>,
    identification_remarks: Option<String>,
}

impl From<Map<SpecimenAtom>> for Specimen {
    fn from(value: Map<SpecimenAtom>) -> Self {
        let mut specimen = Specimen {
            entity_id: value.entity_id,
            ..Default::default()
        };

        for val in value.atoms.into_values() {
            match val {
                SpecimenAtom::Empty => {}
                SpecimenAtom::RecordId(value) => specimen.record_id = value,
                SpecimenAtom::MaterialSampleId(value) => specimen.material_sample_id = Some(value),
                SpecimenAtom::OrganismId(value) => specimen.organism_id = Some(value),
                SpecimenAtom::ScientificName(value) => specimen.scientific_name = Some(value),
                SpecimenAtom::InstitutionName(value) => specimen.institution_name = Some(value),
                SpecimenAtom::InstitutionCode(value) => specimen.institution_code = Some(value),
                SpecimenAtom::CollectionCode(value) => specimen.collection_code = Some(value),
                SpecimenAtom::RecordedBy(value) => specimen.recorded_by = Some(value),
                SpecimenAtom::IdentifiedBy(value) => specimen.identified_by = Some(value),
                SpecimenAtom::IdentifiedDate(value) => specimen.identified_date = Some(value),
                SpecimenAtom::TypeStatus(value) => specimen.type_status = Some(value),
                SpecimenAtom::Locality(value) => specimen.locality = Some(value),
                SpecimenAtom::Country(value) => specimen.country = Some(value),
                SpecimenAtom::CountryCode(value) => specimen.country_code = Some(value),
                SpecimenAtom::StateProvince(value) => specimen.state_province = Some(value),
                SpecimenAtom::County(value) => specimen.county = Some(value),
                SpecimenAtom::Municipality(value) => specimen.municipality = Some(value),
                SpecimenAtom::Latitude(value) => specimen.latitude = Some(value),
                SpecimenAtom::Longitude(value) => specimen.longitude = Some(value),
                SpecimenAtom::Elevation(value) => specimen.elevation = Some(value),
                SpecimenAtom::Depth(value) => specimen.depth = Some(value),
                SpecimenAtom::ElevationAccuracy(value) => specimen.elevation_accuracy = Some(value),
                SpecimenAtom::DepthAccuracy(value) => specimen.depth_accuracy = Some(value),
                SpecimenAtom::LocationSource(value) => specimen.location_source = Some(value),
                SpecimenAtom::Details(value) => specimen.details = Some(value),
                SpecimenAtom::Remarks(value) => specimen.remarks = Some(value),
                SpecimenAtom::IdentificationRemarks(value) => {
                    specimen.identification_remarks = Some(value)
                }
            }
        }

        specimen
    }
}

pub struct SpecimenFrame {
    dataset_version_id: Uuid,
    entity_id: String,
    frame: Frame<SpecimenOperation>,
}

impl SpecimenFrame {
    pub fn create(
        dataset_version_id: Uuid,
        entity_id: String,
        last_version: Version,
    ) -> SpecimenFrame {
        let mut frame = Frame::new(last_version);

        frame.push(SpecimenOperation {
            operation_id: frame.next.into(),
            parent_id: frame.current.into(),
            dataset_version_id,
            entity_id: entity_id.clone(),
            action: Action::Create,
            atom: SpecimenAtom::Empty,
        });

        SpecimenFrame {
            dataset_version_id,
            entity_id,
            frame,
        }
    }

    pub fn push(&mut self, atom: SpecimenAtom) {
        let operation_id: BigDecimal = self.frame.next.into();
        let parent_id = self
            .frame
            .operations
            .last()
            .map(|op| op.operation_id.clone())
            .unwrap_or(operation_id.clone());

        let op = SpecimenOperation {
            operation_id,
            parent_id,
            dataset_version_id: self.dataset_version_id,
            entity_id: self.entity_id.clone(),
            action: Action::Update,
            atom,
        };

        self.frame.push(op);
    }

    pub fn operations(&self) -> &Vec<SpecimenOperation> {
        &self.frame.operations
    }
}

pub struct Specimens {
    pub path: PathBuf,
    pub dataset_version_id: Uuid,
}

impl Specimens {
    pub fn specimens(&self) -> Result<Vec<SpecimenOperation>, Error> {
        let mut records: Vec<Record> = Vec::new();
        for row in csv::Reader::from_path(&self.path)?.deserialize() {
            records.push(row?);
        }

        let mut last_version = Version::new();
        let mut operations: Vec<SpecimenOperation> = Vec::new();

        for record in records.into_iter() {
            // the uniqueness of a specimen is by the record id which should be
            // a globally unique id if not an accession number
            let mut hasher = Xxh3::new();
            hasher.update(record.record_id.as_bytes());
            let hash = hasher.digest();

            let mut frame =
                SpecimenFrame::create(self.dataset_version_id, hash.to_string(), last_version);
            // let mut object = ObjectFrame::new(self.dataset_id, version, hash.to_string());

            frame.push(SpecimenAtom::RecordId(record.record_id));

            if let Some(value) = record.material_sample_id {
                frame.push(SpecimenAtom::MaterialSampleId(value));
            }
            if let Some(value) = record.organism_id {
                frame.push(SpecimenAtom::OrganismId(value));
            }
            if let Some(value) = record.scientific_name {
                frame.push(SpecimenAtom::ScientificName(value));
            }
            if let Some(value) = record.institution_name {
                frame.push(SpecimenAtom::InstitutionName(value));
            }
            if let Some(value) = record.institution_code {
                frame.push(SpecimenAtom::InstitutionCode(value));
            }
            if let Some(value) = record.collection_code {
                frame.push(SpecimenAtom::CollectionCode(value));
            }
            // if let Some(value) = record.recorded_by {
            //     frame.push(SpecimenAtom::RecordedBy(value));
            // }
            if let Some(value) = record.identified_by {
                frame.push(SpecimenAtom::IdentifiedBy(value));
            }
            if let Some(value) = record.identified_date {
                frame.push(SpecimenAtom::IdentifiedDate(value));
            }
            if let Some(value) = record.type_status {
                frame.push(SpecimenAtom::TypeStatus(value));
            }
            if let Some(value) = record.locality {
                frame.push(SpecimenAtom::Locality(value));
            }
            if let Some(value) = record.country {
                frame.push(SpecimenAtom::Country(value));
            }
            if let Some(value) = record.country_code {
                frame.push(SpecimenAtom::CountryCode(value));
            }
            if let Some(value) = record.state_province {
                frame.push(SpecimenAtom::StateProvince(value));
            }
            if let Some(value) = record.county {
                frame.push(SpecimenAtom::County(value));
            }
            if let Some(value) = record.municipality {
                frame.push(SpecimenAtom::Municipality(value));
            }
            if let Some(value) = record.latitude {
                frame.push(SpecimenAtom::Latitude(value));
            }
            if let Some(value) = record.longitude {
                frame.push(SpecimenAtom::Longitude(value));
            }
            if let Some(value) = record.elevation {
                frame.push(SpecimenAtom::Elevation(value));
            }
            if let Some(value) = record.depth {
                frame.push(SpecimenAtom::Depth(value));
            }
            if let Some(value) = record.elevation_accuracy {
                frame.push(SpecimenAtom::ElevationAccuracy(value));
            }
            if let Some(value) = record.depth_accuracy {
                frame.push(SpecimenAtom::DepthAccuracy(value));
            }
            if let Some(value) = record.location_source {
                frame.push(SpecimenAtom::LocationSource(value));
            }
            if let Some(value) = record.details {
                frame.push(SpecimenAtom::Details(value));
            }
            if let Some(value) = record.remarks {
                frame.push(SpecimenAtom::Remarks(value));
            }
            if let Some(value) = record.identification_remarks {
                frame.push(SpecimenAtom::IdentificationRemarks(value));
            }

            last_version = frame.frame.current;
            operations.extend(frame.frame.operations);
        }

        Ok(operations)
    }
}

pub fn process(path: PathBuf, dataset_version: DatasetVersion) -> Result<(), Error> {
    let specimens = Specimens {
        path,
        dataset_version_id: dataset_version.id,
    };
    let records = specimens.specimens()?;
    let reduced = reduce_operations(records)?;

    import_operations(reduced)?;
    Ok(())
}

pub fn reduce() -> Result<(), Error> {
    reduce_specimens()
}

fn import_operations(records: Vec<SpecimenOperation>) -> Result<(), Error> {
    use schema::specimen_logs::dsl::*;

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    for chunk in records.chunks(1000) {
        diesel::insert_into(specimen_logs)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
    }

    Ok(())
}

fn merge_operations(
    records: Vec<SpecimenOperation>,
) -> Result<HashMap<String, Vec<SpecimenOperation>>, Error> {
    use schema::specimen_logs::dsl::*;

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    let operations = specimen_logs
        .order(operation_id.asc())
        .load::<SpecimenOperation>(&mut conn)?;

    let mut grouped_ops: HashMap<String, Vec<SpecimenOperation>> = HashMap::new();
    for op in operations.into_iter() {
        grouped_ops
            .entry(op.entity_id.clone())
            .or_default()
            .push(op)
    }

    for op in records.into_iter() {
        grouped_ops
            .entry(op.entity_id.clone())
            .or_default()
            .push(op)
    }

    Ok(grouped_ops)
}

fn reduce_operations(records: Vec<SpecimenOperation>) -> Result<Vec<SpecimenOperation>, Error> {
    let entities = merge_operations(records)?;
    let mut merged_ops = Vec::new();

    for (key, ops) in entities.into_iter() {
        let mut map = Map::new(key);
        let reduced = map.reduce(&ops);
        merged_ops.extend(reduced);
    }

    Ok(merged_ops)
}

fn reduce_specimens() -> Result<(), Error> {
    let entities = merge_operations(vec![])?;
    let mut specimens = Vec::new();

    for (key, ops) in entities.into_iter() {
        let mut map = Map::new(key);
        map.reduce(&ops);
        specimens.push(Specimen::from(map));
    }

    let mut writer = csv::Writer::from_writer(std::io::stdout());

    for specimen in specimens {
        writer.serialize(specimen)?;
    }

    Ok(())
}
