use std::collections::HashMap;
use std::path::PathBuf;

use arga_core::crdt::lww::Map;
use arga_core::crdt::{Frame, Version};
use arga_core::models::{
    Action,
    CollectionEventAtom,
    CollectionEventOperation,
    DatasetVersion,
    SpecimenAtom,
    SpecimenOperation,
};
use arga_core::schema;
use bigdecimal::BigDecimal;
use diesel::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use xxhash_rust::xxh3::Xxh3;

use crate::data::oplogger::get_pool;
use crate::data::Error;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    record_id: String,
    scientific_name: Option<String>,
    // canonical_name: Option<String>,
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
    // verbatim_lat_long: Option<String>,
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
                SpecimenAtom::IdentificationRemarks(value) => specimen.identification_remarks = Some(value),
            }
        }

        specimen
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct CollectionEvent {
    entity_id: String,
    specimen_id: String,

    event_date: Option<String>,
    event_time: Option<String>,
    collected_by: Option<String>,

    field_number: Option<String>,
    catalog_number: Option<String>,
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

    field_notes: Option<String>,
    remarks: Option<String>,
}

impl From<Map<CollectionEventAtom>> for CollectionEvent {
    fn from(value: Map<CollectionEventAtom>) -> Self {
        use CollectionEventAtom::*;

        let mut event = CollectionEvent {
            entity_id: value.entity_id,
            ..Default::default()
        };

        for val in value.atoms.into_values() {
            match val {
                Empty => {}
                SpecimenId(value) => event.specimen_id = value,
                EventDate(value) => event.event_date = Some(value),
                EventTime(value) => event.event_time = Some(value),
                CollectedBy(value) => event.collected_by = Some(value),
                FieldNumber(value) => event.field_number = Some(value),
                CatalogNumber(value) => event.catalog_number = Some(value),
                RecordNumber(value) => event.record_number = Some(value),
                IndividualCount(value) => event.individual_count = Some(value),
                OrganismQuantity(value) => event.organism_quantity = Some(value),
                OrganismQuantityType(value) => event.organism_quantity_type = Some(value),
                Sex(value) => event.sex = Some(value),
                GenotypicSex(value) => event.genotypic_sex = Some(value),
                PhenotypicSex(value) => event.phenotypic_sex = Some(value),
                LifeStage(value) => event.life_stage = Some(value),
                ReproductiveCondition(value) => event.reproductive_condition = Some(value),
                Behavior(value) => event.behavior = Some(value),
                EstablishmentMeans(value) => event.establishment_means = Some(value),
                DegreeOfEstablishment(value) => event.degree_of_establishment = Some(value),
                Pathway(value) => event.pathway = Some(value),
                OccurrenceStatus(value) => event.occurrence_status = Some(value),
                Preparation(value) => event.preparation = Some(value),
                OtherCatalogNumbers(value) => event.other_catalog_numbers = Some(value),
                EnvBroadScale(value) => event.env_broad_scale = Some(value),
                EnvLocalScale(value) => event.env_local_scale = Some(value),
                EnvMedium(value) => event.env_medium = Some(value),
                Habitat(value) => event.habitat = Some(value),
                RefBiomaterial(value) => event.ref_biomaterial = Some(value),
                SourceMatId(value) => event.source_mat_id = Some(value),
                SpecificHost(value) => event.specific_host = Some(value),
                Strain(value) => event.strain = Some(value),
                Isolate(value) => event.isolate = Some(value),
                FieldNotes(value) => event.field_notes = Some(value),
                Remarks(value) => event.remarks = Some(value),
            }
        }

        event
    }
}


pub struct SpecimenFrame {
    dataset_version_id: Uuid,
    entity_id: String,
    frame: Frame<SpecimenOperation>,
}

impl SpecimenFrame {
    pub fn create(dataset_version_id: Uuid, entity_id: String, last_version: Version) -> SpecimenFrame {
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

            let mut frame = SpecimenFrame::create(self.dataset_version_id, hash.to_string(), last_version);
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


struct CollectionEventFrame {
    dataset_version_id: Uuid,
    entity_id: String,
    frame: Frame<CollectionEventOperation>,
}

impl CollectionEventFrame {
    pub fn create(dataset_version_id: Uuid, entity_id: String, last_version: Version) -> CollectionEventFrame {
        let mut frame = Frame::new(last_version);

        frame.push(CollectionEventOperation {
            operation_id: frame.next.into(),
            parent_id: frame.current.into(),
            dataset_version_id,
            entity_id: entity_id.clone(),
            action: Action::Create,
            atom: CollectionEventAtom::Empty,
        });

        CollectionEventFrame {
            dataset_version_id,
            entity_id,
            frame,
        }
    }

    pub fn push(&mut self, atom: CollectionEventAtom) {
        let operation_id: BigDecimal = self.frame.next.into();
        let parent_id = self
            .frame
            .operations
            .last()
            .map(|op| op.operation_id.clone())
            .unwrap_or(operation_id.clone());

        let op = CollectionEventOperation {
            operation_id,
            parent_id,
            dataset_version_id: self.dataset_version_id,
            entity_id: self.entity_id.clone(),
            action: Action::Update,
            atom,
        };

        self.frame.push(op);
    }
}

pub struct CollectionEvents {
    pub path: PathBuf,
    pub dataset_version_id: Uuid,
}

impl CollectionEvents {
    pub fn events(&self) -> Result<Vec<CollectionEventOperation>, Error> {
        use CollectionEventAtom::*;

        let mut records: Vec<Record> = Vec::new();
        for row in csv::Reader::from_path(&self.path)?.deserialize() {
            records.push(row?);
        }

        let mut last_version = Version::new();
        let mut operations: Vec<CollectionEventOperation> = Vec::new();

        for record in records.into_iter() {
            // the uniqueness of a specimen is by the record id which should be
            // a globally unique id if not an accession number
            let mut hasher = Xxh3::new();
            hasher.update(record.record_id.as_bytes());
            let hash = hasher.digest();

            let mut frame = CollectionEventFrame::create(self.dataset_version_id, hash.to_string(), last_version);
            frame.push(SpecimenId(record.record_id));

            if let Some(value) = record.event_date {
                frame.push(EventDate(value));
            }
            if let Some(value) = record.event_time {
                frame.push(EventTime(value));
            }
            if let Some(value) = record.collected_by {
                frame.push(CollectedBy(value));
            }
            if let Some(value) = record.field_number {
                frame.push(FieldNumber(value));
            }
            if let Some(value) = record.catalog_number {
                frame.push(CatalogNumber(value));
            }
            if let Some(value) = record.record_number {
                frame.push(RecordNumber(value));
            }
            if let Some(value) = record.individual_count {
                frame.push(IndividualCount(value));
            }
            if let Some(value) = record.organism_quantity {
                frame.push(OrganismQuantity(value));
            }
            if let Some(value) = record.organism_quantity_type {
                frame.push(OrganismQuantityType(value));
            }
            if let Some(value) = record.sex {
                frame.push(Sex(value));
            }
            if let Some(value) = record.genotypic_sex {
                frame.push(GenotypicSex(value));
            }
            if let Some(value) = record.phenotypic_sex {
                frame.push(PhenotypicSex(value));
            }
            if let Some(value) = record.life_stage {
                frame.push(LifeStage(value));
            }
            if let Some(value) = record.reproductive_condition {
                frame.push(ReproductiveCondition(value));
            }
            if let Some(value) = record.behavior {
                frame.push(Behavior(value));
            }
            if let Some(value) = record.establishment_means {
                frame.push(EstablishmentMeans(value));
            }
            if let Some(value) = record.degree_of_establishment {
                frame.push(DegreeOfEstablishment(value));
            }
            if let Some(value) = record.pathway {
                frame.push(Pathway(value));
            }
            if let Some(value) = record.occurrence_status {
                frame.push(OccurrenceStatus(value));
            }
            if let Some(value) = record.preparation {
                frame.push(Preparation(value));
            }
            if let Some(value) = record.other_catalog_numbers {
                frame.push(OtherCatalogNumbers(value));
            }
            if let Some(value) = record.env_broad_scale {
                frame.push(EnvBroadScale(value));
            }
            if let Some(value) = record.env_local_scale {
                frame.push(EnvLocalScale(value));
            }
            if let Some(value) = record.env_medium {
                frame.push(EnvMedium(value));
            }
            if let Some(value) = record.habitat {
                frame.push(Habitat(value));
            }
            if let Some(value) = record.ref_biomaterial {
                frame.push(RefBiomaterial(value));
            }
            if let Some(value) = record.source_mat_id {
                frame.push(SourceMatId(value));
            }
            if let Some(value) = record.specific_host {
                frame.push(SpecificHost(value));
            }
            if let Some(value) = record.strain {
                frame.push(Strain(value));
            }
            if let Some(value) = record.isolate {
                frame.push(Isolate(value));
            }
            if let Some(value) = record.field_notes {
                frame.push(FieldNotes(value));
            }
            if let Some(value) = record.remarks {
                frame.push(Remarks(value));
            }

            last_version = frame.frame.current;
            operations.extend(frame.frame.operations);
        }

        Ok(operations)
    }
}


pub fn process(path: PathBuf, dataset_version: DatasetVersion) -> Result<(), Error> {
    use schema::{collection_event_logs, specimen_logs};

    let specimens = Specimens {
        path: path.clone(),
        dataset_version_id: dataset_version.id,
    };

    let collections = CollectionEvents {
        path,
        dataset_version_id: dataset_version.id,
    };

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    let specimen_ops = specimen_logs::table
        .order(specimen_logs::operation_id.asc())
        .load::<SpecimenOperation>(&mut conn)?;

    let records = specimens.specimens()?;
    let reduced = reduce_operations(specimen_ops, records)?;
    import_specimens(reduced)?;


    let collection_ops = collection_event_logs::table
        .order(collection_event_logs::operation_id.asc())
        .load::<CollectionEventOperation>(&mut conn)?;

    let records = collections.events()?;
    let reduced = reduce_operations(collection_ops, records)?;
    import_collections(reduced)?;

    Ok(())
}

fn import_specimens(records: Vec<SpecimenOperation>) -> Result<(), Error> {
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

fn import_collections(records: Vec<CollectionEventOperation>) -> Result<(), Error> {
    use schema::collection_event_logs::dsl::*;

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    for chunk in records.chunks(1000) {
        diesel::insert_into(collection_event_logs)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
    }

    Ok(())
}

fn merge_ops<T, A>(existing_ops: Vec<T>, new_ops: Vec<T>) -> Result<HashMap<String, Vec<T>>, Error>
where
    T: arga_core::models::LogOperation<A>,
{
    let mut grouped: HashMap<String, Vec<T>> = HashMap::new();

    for op in existing_ops.into_iter() {
        grouped.entry(op.id().clone()).or_default().push(op);
    }

    for op in new_ops.into_iter() {
        grouped.entry(op.id().clone()).or_default().push(op);
    }

    Ok(grouped)
}

fn reduce_operations<T, A>(existing_ops: Vec<T>, new_ops: Vec<T>) -> Result<Vec<T>, Error>
where
    A: ToString + Clone + PartialEq,
    T: arga_core::models::LogOperation<A> + Clone,
{
    let entities = merge_ops(existing_ops, new_ops)?;
    let mut merged = Vec::new();

    for (key, ops) in entities.into_iter() {
        let mut map = Map::new(key);
        let reduced = map.reduce(&ops);
        merged.extend(reduced);
    }

    Ok(merged)
}


pub fn reduce_specimens() -> Result<(), Error> {
    use schema::specimen_logs::dsl::*;

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    let ops = specimen_logs
        .order(operation_id.asc())
        .load::<SpecimenOperation>(&mut conn)?;

    let entities = merge_ops(ops, vec![])?;
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

pub fn reduce_collections() -> Result<(), Error> {
    use schema::collection_event_logs::dsl::*;

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    let ops = collection_event_logs
        .order(operation_id.asc())
        .load::<CollectionEventOperation>(&mut conn)?;

    let entities = merge_ops(ops, vec![])?;
    let mut collections = Vec::new();

    for (key, ops) in entities.into_iter() {
        let mut map = Map::new(key);
        map.reduce(&ops);
        collections.push(CollectionEvent::from(map));
    }

    let mut writer = csv::Writer::from_writer(std::io::stdout());

    for collection in collections {
        writer.serialize(collection)?;
    }

    Ok(())
}
