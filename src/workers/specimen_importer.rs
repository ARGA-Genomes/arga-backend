use std::path::{Path, PathBuf};
use std::collections::HashMap;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use polars::prelude::*;
use serde::Deserialize;
use stakker::*;
use tracing::{instrument, info, error};
use uuid::Uuid;

use crate::database::schema;
use crate::database::models::{Job, NameList, NameListType, Specimen, Event, CollectionEvent, Organism};

type PgPool = Pool<ConnectionManager<PgConnection>>;


pub struct SpecimenImporter {
    thread: PipedThread<Job, Job>,
}

impl SpecimenImporter {
    pub fn init(cx: CX![]) -> Option<Self> {
        let thread = PipedThread::spawn(
            fwd_to!([cx], recv() as (Job)),
            fwd_to!([cx], term() as (Option<String>)),
            cx,
            move |link| {
                while let Some(job) = link.recv() {
                    Self::process(job);
                }
            }
        );

        Some(Self {
            thread,
        })
    }

    pub fn import(&mut self, _cx: CX![], job: Job) {
        self.thread.send(job);
    }

    fn recv(&mut self, _cx: CX![], _job: Job) {

    }

    fn term(&self, cx: CX![], panic: Option<String>) {
        if let Some(msg) = panic {
            panic!("Unexpected thread failure: {}", msg);
        }
        cx.stop();
    }

    // #[instrument]
    fn process(job: Job) {
        info!("Running specimen importer");
        let tmp_path = std::env::var("ADMIN_TMP_UPLOAD_STORAGE").expect("No upload storage specified");

        if let Some(payload) = job.payload {
            match serde_json::from_value::<ImportJobData>(payload) {
                Ok(data) => {
                    let list = create_specimen_list(&data.name, &data.description).unwrap();
                    let path = Path::new(&tmp_path).join(data.tmp_name);
                    import(path, &list).unwrap();
                }
                Err(err) => {
                    error!(?err, "Invalid JSON payload");
                }
            }
        }
    }
}


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("an error occurred with the database connection")]
    Database(#[from] diesel::result::Error),

    #[error("an error occurred parsing the file")]
    Parsing(#[from] PolarsError),

    #[error("an error occurred parsing the file")]
    Csv(#[from] csv::Error),

    #[error("an error occurred getting a database connection")]
    Pool(#[from] diesel::r2d2::PoolError),
}

#[derive(Debug, Deserialize)]
struct ImportJobData {
    name: String,
    description: Option<String>,
    tmp_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    scientific_name: String,
    type_status: Option<String>,
    institution_name: Option<String>,
    institution_code: Option<String>,
    collection_code: Option<String>,
    catalog_number: Option<String>,
    recorded_by: Option<String>,
    #[serde(rename(deserialize = "organismID"))]
    organism_id: Option<String>,
    locality: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    details: Option<String>,
    remarks: Option<String>,

    // event block
    #[serde(rename(deserialize = "eventID"))]
    event_id: Option<String>,
    // #[serde(rename(deserialize = "parentEventID"))]
    // parent_event_id: Option<String>,
    field_number: Option<String>,
    event_date: Option<chrono::DateTime<chrono::Utc>>,
    habitat: Option<String>,
    sampling_protocol: Option<String>,
    sampling_size_value: Option<String>,
    sampling_size_unit: Option<String>,
    sampling_effort: Option<String>,
    field_notes: Option<String>,
    event_remarks: Option<String>,

    // occurrence block
    #[serde(rename(deserialize = "occurrenceID"))]
    occurrence_id: Option<String>,
    record_number: Option<String>,
    individual_count: Option<String>,
    organism_quantity: Option<String>,
    organism_quantity_type: Option<String>,
    sex: Option<String>,
    life_stage: Option<String>,
    reproductive_condition: Option<String>,
    behavior: Option<String>,
    establishment_means: Option<String>,
    pathway: Option<String>,
    occurrence_status: Option<String>,
    preparation: Option<String>,
    other_catalog_numbers: Option<String>,

    // organism block
    organism_name: Option<String>,
    organism_scope: Option<String>,
    associated_organisms: Option<String>,
    previous_identifications: Option<String>,
    organism_remarks: Option<String>,
}

#[derive(Debug, Queryable, Deserialize)]
struct NameMatch {
    id: Uuid,
    scientific_name: String,
    canonical_name: Option<String>,
}


pub fn create_specimen_list(list_name: &str, list_description: &Option<String>) -> Result<NameList, Error> {
    use schema::name_lists::dsl::*;
    let pool = get_connection_pool();
    let mut conn = pool.get()?;

    let name_list = diesel::insert_into(name_lists)
        .values((
            list_type.eq(NameListType::Specimen),
            name.eq(list_name),
            description.eq(list_description),
        ))
        .get_result(&mut conn)?;

    Ok(name_list)
}

#[instrument]
pub fn import(path: PathBuf, list: &NameList) -> Result<(), Error> {
    info!("Getting database connection pool");
    let pool = &mut get_connection_pool();

    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    import_specimens(records, list, pool)?;

    Ok(())
}


fn import_specimens(records: Vec<Record>, list: &NameList, pool: &mut PgPool) -> Result<(), Error> {
    use schema::{events, collection_events, specimens, organisms};

    let names = match_names(&records, pool);
    let specimens = extract_specimens(&list, &names,  &records);
    let organisms = extract_organisms(&records, &specimens);
    let events = extract_events(&records, &specimens);
    let collections = extract_collection_events(&records, &specimens, &events);


    // filter out unmatched specimens
    let specimens = specimens.into_iter().filter_map(|r| r).collect::<Vec<Specimen>>();

    info!(total=specimens.len(), "Importing specimens");
    let imported: Vec<Result<usize, Error>> = specimens.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(specimens::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=specimens.len(), total_imported, "Importing specimens finished");


    // filter out unmatched and dedup all organisms
    let organisms = organisms.into_iter().filter_map(|r| r).collect::<Vec<Organism>>();

    info!(total=organisms.len(), "Importing specimen organisms");
    let imported: Vec<Result<usize, Error>> = organisms.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(organisms::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=organisms.len(), imported=total_imported, "Importing specimen organisms finished");


    // filter out unmatched and dedup all events
    let events = events.into_iter().filter_map(|r| r).collect::<Vec<Event>>();

    info!(total=events.len(), "Importing specimen events");
    let imported: Vec<Result<usize, Error>> = events.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(events::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=events.len(), imported=total_imported, "Importing specimen events finished");


    // filter out unmatched collection events. because a collection event always
    // describes one specimen we don't dedupe them
    let collections = collections.into_iter().filter_map(|r| r).collect::<Vec<CollectionEvent>>();

    info!(total=collections.len(), "Importing specimen collection events");
    let imported: Vec<Result<usize, Error>> = collections.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(collection_events::table)
            .values(chunk)
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=collections.len(), total_imported, "Importing specimen collection events finished");

    Ok(())
}


fn match_names(records: &Vec<Record>, pool: &mut PgPool) -> HashMap<String, Uuid> {
    use schema::names;
    info!(total=records.len(), "Matching names");

    let matched: Vec<Result<Vec<NameMatch>, Error>> = records.par_chunks(50_000).map(|chunk| {
        let mut conn = pool.get()?;
        let all_names: Vec<&String> = chunk.iter().map(|row| &row.scientific_name).collect();

        let results = names::table
            .select((names::id, names::scientific_name, names::canonical_name))
            .filter(names::canonical_name.eq_any(all_names))
            .load::<NameMatch>(&mut conn)?;

        Ok::<Vec<NameMatch>, Error>(results)
    }).collect();

    let mut id_map: HashMap<String, Uuid> = HashMap::new();

    for chunk in matched {
        if let Ok(names) = chunk {
            for name_match in names {
                let name = name_match.canonical_name.unwrap_or(name_match.scientific_name);
                id_map.insert(name, name_match.id);
            }
        }
    }

    info!(total=records.len(), matched=id_map.len(), "Matching names finished");
    id_map
}


fn extract_organisms(records: &Vec<Record>, specimens: &Vec<Option<Specimen>>) -> Vec<Option<Organism>> {
    info!(total=records.len(), "Extracting organisms");

    let organisms = records.par_iter().zip(specimens).map(|(row, specimen)| {
        match (specimen, &row.organism_id) {
            (Some(specimen), Some(organism_id)) => Some(Organism {
                id: Uuid::new_v4(),
                name_id: specimen.name_id.clone(),
                organism_id: Some(organism_id.clone()),
                organism_name: row.organism_name.clone(),
                organism_scope: row.organism_scope.clone(),
                associated_organisms: row.associated_organisms.clone(),
                previous_identifications: row.previous_identifications.clone(),
                remarks: row.organism_remarks.clone(),
            }),
            _ => None,
        }
    }).collect::<Vec<Option<Organism>>>();

    info!(organisms=organisms.len(), "Extracting organisms finished");
    organisms
}


fn extract_events(records: &Vec<Record>, specimens: &Vec<Option<Specimen>>) -> Vec<Option<Event>> {
    info!(total=records.len(), "Extracting events");

    let events = records.par_iter().zip(specimens).map(|(row, specimen)| {
        match specimen {
            Some(_) => Some(Event {
                id: Uuid::new_v4(),
                parent_event_id: None,
                event_id: row.event_id.clone(),
                field_number: row.field_number.clone(),
                event_date: row.event_date.map(|d| d.date_naive()).clone(),
                habitat: row.habitat.clone(),
                sampling_protocol: row.sampling_protocol.clone(),
                sampling_size_value: row.sampling_size_value.clone(),
                sampling_size_unit: row.sampling_size_unit.clone(),
                sampling_effort: row.sampling_effort.clone(),
                field_notes: row.field_notes.clone(),
                event_remarks: row.event_remarks.clone(),
            }),
            None => None,
        }
    }).collect::<Vec<Option<Event>>>();

    info!(events=events.len(), "Extracting events finished");
    events
}


fn extract_collection_events(
    records: &Vec<Record>,
    specimens: &Vec<Option<Specimen>>,
    events: &Vec<Option<Event>>,
) -> Vec<Option<CollectionEvent>>
{
    info!(total=records.len(), "Extracting collection events");

    let collections = (records, specimens, events).into_par_iter().map(|(row, specimen, event)| {
        match (specimen, event) {
            (Some(specimen), Some(event)) => Some(CollectionEvent {
                id: Uuid::new_v4(),
                event_id: event.id.clone(),
                specimen_id: specimen.id.clone(),
                organism_id: None,

                occurrence_id: row.occurrence_id.clone(),
                catalog_number: row.catalog_number.clone(),
                record_number: row.record_number.clone(),
                individual_count: row.individual_count.clone(),
                organism_quantity: row.organism_quantity.clone(),
                organism_quantity_type: row.organism_quantity_type.clone(),
                sex: row.sex.clone(),
                life_stage: row.life_stage.clone(),
                reproductive_condition: row.reproductive_condition.clone(),
                behavior: row.behavior.clone(),
                establishment_means: row.establishment_means.clone(),
                pathway: row.pathway.clone(),
                occurrence_status: row.occurrence_status.clone(),
                preparation: row.preparation.clone(),
                other_catalog_numbers: row.other_catalog_numbers.clone(),
            }),
            _ => None,
        }
    }).collect::<Vec<Option<CollectionEvent>>>();

    info!(collection_events=collections.len(), "Extracting collection events finished");
    collections
}


fn extract_specimens(list: &NameList, names: &HashMap<String, Uuid>, records: &Vec<Record>) -> Vec<Option<Specimen>> {
    info!(total=records.len(), "Extracting specimens");

    let specimens = records.par_iter().map(|row| {
        match names.get(&row.scientific_name) {
            Some(name_id) => Some(Specimen {
                id: Uuid::new_v4(),
                list_id: list.id.clone(),
                name_id: name_id.clone(),
                type_status: row.type_status.clone().unwrap_or("unspecified".to_string()),
                institution_name: row.institution_name.clone(),
                institution_code: row.institution_code.clone(),
                collection_code: row.collection_code.clone(),
                catalog_number: row.catalog_number.clone(),
                recorded_by: row.recorded_by.clone(),
                organism_id: row.organism_id.clone(),
                locality: row.locality.clone(),
                latitude: row.latitude,
                longitude: row.longitude,
                details: row.details.clone(),
                remarks: row.remarks.clone(),
            }),
            None => None,
        }
    }).collect::<Vec<Option<Specimen>>>();

    info!(specimens=specimens.len(), "Extracting specimens finished");
    specimens
}


fn get_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    let url = crate::database::get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    Pool::builder().build(manager).expect("Could not build connection pool")
}
