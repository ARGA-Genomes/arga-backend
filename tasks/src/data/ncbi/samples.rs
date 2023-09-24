use std::{path::PathBuf, collections::HashMap, fs::File};

use dotenvy::dotenv;
use memmap2::Mmap;
use quick_xml::{Reader, events::{Event, BytesStart}};
use serde::Serialize;
use tracing::{info, error, debug};
use rayon::prelude::*;
use polars::prelude::*;

use diesel::{prelude::*, r2d2::{ConnectionManager, Pool}};
use indicatif::{ProgressBar, ProgressStyle, ParallelProgressIterator, MultiProgress};
use uuid::Uuid;

use crate::schema;
use super::Error;


type PgPool = Pool<ConnectionManager<PgConnection>>;


#[derive(Queryable, Insertable, Debug, Default)]
#[diesel(table_name = schema::biosamples)]
pub struct Record {
    pub name_id: Uuid,
    pub accession: String,
    pub sra: Option<String>,

    pub submission_date: Option<String>,
    pub publication_date: Option<String>,
    pub last_update: Option<String>,

    pub title: Option<String>,
    pub owner: Option<String>,

    pub attributes: serde_json::Value,
}

#[derive(Debug, Default, Clone)]
pub struct BioSample {
    pub name_id: Uuid,
    pub accession: String,
    pub sra: Option<String>,

    pub submission_date: Option<String>,
    pub publication_date: Option<String>,
    pub last_update: Option<String>,

    pub title: Option<String>,
    pub taxonomy_id: Option<String>,
    pub taxonomy_name: Option<String>,
    pub organism_name: Option<String>,

    pub owner: Option<String>,

    pub attributes: Vec<Attribute>,
}

#[derive(Serialize, Debug, Default, Clone)]
pub struct Attribute {
    name: String,
    harmonized_name: Option<String>,
    value: Option<String>,
}

impl From<BioSample> for Record {
    fn from(value: BioSample) -> Self {
        Self {
            name_id: value.name_id,
            accession: value.accession,
            sra: value.sra,
            submission_date: value.submission_date,
            publication_date: value.publication_date,
            last_update: value.last_update,
            title: value.title,
            owner: value.owner,
            attributes: serde_json::to_value(value.attributes).unwrap(),
        }
    }
}


#[derive(Debug)]
enum BioSampleState {
    Root,
    Sample,

    Ids,
    Id(BioSampleId),

    Description,
    Title,
    Organism,
    OrganismName,

    Owner,
    OwnerName,

    Attributes,
    Attribute(Attribute),
}

#[derive(Debug)]
enum BioSampleId {
    BioSample,
    SRA,
}


enum ItemState {
    Opened,
    Closed,
}


pub fn import(path: PathBuf) -> Result<(), Error> {
    info!("Establishing database connection");
    let mut pool = get_connection_pool();

    // read the name map from the ncbi taxonomy import.
    // the reason we need this is because the the sheer amount of duplication
    // in the ncbi taxa which all contain different ids and references link
    // to those different ids. so we normalise it here by linking through name
    info!("Loading name map");
    let ncbi_names = CsvReader::from_path("ncbi_names.csv")?
        .has_header(true)
        .finish()?
        .lazy()
        .select(&[col("occurrenceID").alias("mapId"), col("scientificName").alias("ncbiName")])
        .collect()?;

    let name_map = CsvReader::from_path("ncbi_name_map.csv")?
        .has_header(true)
        .finish()?
        .lazy()
        .select(&[col("occurrenceID").alias("mapId"), col("scientificName").alias("alaName")])
        .collect()?;

    let joined = ncbi_names.join(&name_map, ["mapId"], ["mapId"], JoinType::Inner, None)?;

    let name_ids = load_taxon_map(&joined, &pool)?;
    info!(total=name_ids.len(), "Name map loaded");


    let path = path.join("biosample_set.xml");
    info!(?path, "Memory mapping file");
    let file = std::fs::File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let locations = analyse_biosample(&mmap)?;
    import_biosample_analyzed(&mmap, locations, &mut pool, &name_ids)?;

    Ok(())
}


pub fn get_all_names(path: &PathBuf) -> Result<Vec<String>, Error> {
    info!(?path, "Memory mapping file");
    let file = std::fs::File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let locations = analyse_biosample(&mmap)?;

    info!(items=locations.len(), "Reading biosample file");
    let style = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {human_pos}/{human_len} @ {per_sec}").unwrap();
    let transform_bar = ProgressBar::new(locations.len() as u64).with_style(style.clone());

    let samples: Vec<String> = locations
        .into_par_iter()
        .progress_with(transform_bar.clone())
        .filter_map(|(start, end)| {
            let sample = process_item(&mmap, start, end).expect("Failed to process item");
            match sample.taxonomy_name {
                Some(name) => Some(name),
                None => None,
            }
        })
        .collect();

    transform_bar.finish();
    info!("Finished importing");

    Ok(samples)
}


pub fn analyse_biosample(mmap: &Mmap) -> Result<Vec<(usize, usize)>, Error> {
    info!("Analyzing file");

    let bytes_style = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} @ {bytes_per_sec}").unwrap();
    let bar = ProgressBar::new(mmap.len() as u64).with_style(bytes_style);

    let needle_open = "<BioSample ";
    let needle_close = "</BioSample>";

    let finder_open = memchr::memmem::Finder::new(needle_open);
    let finder_close = memchr::memmem::Finder::new(needle_close);

    let mut items: Vec<(usize, usize)> = Vec::with_capacity(100_000);
    let mut item_start = 0;
    let mut read_pos = 0;
    let mut state = ItemState::Closed;

    loop {
        if item_start >= mmap.len() {
            break;
        }

        match state {
            // look for opening tag
            ItemState::Closed => {
                match finder_open.find(&mmap[read_pos..mmap.len()]) {
                    Some(index) => {
                        item_start = read_pos + index;
                        state = ItemState::Opened;

                        read_pos = item_start + needle_open.len();
                        bar.set_position(read_pos as u64);
                    }
                    None => break
                };
            }
            // look for closing tag
            ItemState::Opened => {
                 match finder_close.find(&mmap[read_pos..mmap.len()]) {
                    Some(index) => {
                        let item_end = read_pos + index + needle_close.len();
                        items.push((item_start, item_end));
                        state = ItemState::Closed;

                        read_pos = item_end;
                        bar.set_position(read_pos as u64);
                    }
                    None => break
                };
            }
        }

    }

    bar.finish();
    info!(total=items.len(), "Finished analyzing");
    Ok(items)
}


pub fn import_biosample_analyzed(
    mmap: &Mmap,
    locations: Vec<(usize, usize)>,
    pool: &mut PgPool,
    name_ids: &HashMap<String, Uuid>,
) -> Result<(), Error>
{
    info!(items=locations.len(), "Importing biosample file");

    let style = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {human_pos}/{human_len} @ {per_sec}").unwrap();
    let bars = MultiProgress::new();
    let transform_bar = bars.add(ProgressBar::new(locations.len() as u64).with_style(style.clone()));
    let persist_bar = bars.add(ProgressBar::new(locations.len() as u64).with_style(style));

    locations.chunks(1_000_000).for_each(|chunk| {
        let samples: Vec<BioSample> = chunk
            .into_par_iter()
            .progress_with(transform_bar.clone())
            .map(|(start, end)| {
                let mut sample = process_item(mmap, *start, *end).unwrap();
                if let Some(name) = &sample.taxonomy_name {
                    if let Some(uuid) = name_ids.get(name) {
                        sample.name_id = uuid.clone();
                    }
                }
                sample
            })
            .collect();

        samples.par_chunks(1000).progress_with(persist_bar.clone()).for_each(|chunk| {
            persist_items(chunk.to_vec(), &mut pool.clone()).unwrap();
        });
    });


    transform_bar.finish();
    persist_bar.finish();
    info!("Finished importing");
    Ok(())
}


pub fn process_item(mmap: &memmap2::Mmap, start: usize, end: usize) -> Result<BioSample, Error> {
    use BioSampleState as State;

    let mut reader = Reader::from_reader(&mmap[start..end]);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut sample = BioSample::default();
    let mut state = State::Root;

    loop {
        state = match (state, reader.read_event_into(&mut buf)?) {
            // Attributes
            (State::Attributes, Event::Start(e)) if e.local_name().as_ref() == b"Attribute" => State::Attribute(Attribute {
                name: parse_attribute(&reader, &e, "attribute_name")?.unwrap(),
                harmonized_name: parse_attribute(&reader, &e, "harmonized_name")?,
                value: None,
            }),
            (State::Attribute(_), Event::End(e)) if e.local_name().as_ref() == b"Attribute" => State::Attributes,
            (State::Attribute(mut attr), Event::Text(text)) => {
                sample.attributes.push(Attribute {
                    name: attr.name.clone(),
                    harmonized_name: attr.harmonized_name.take(),
                    value: Some(text.unescape()?.into_owned()),
                });
                State::Attribute(attr)
            },


            // Ids children
            (State::Ids, Event::Start(e)) if e.local_name().as_ref() == b"Id" => {
                if let Some(db) = e.try_get_attribute("db")? {
                    match db.value.as_ref() {
                        b"BioSample" => State::Id(BioSampleId::BioSample),
                        b"SRA" => State::Id(BioSampleId::SRA),
                        _ => State::Ids,
                    }
                } else {
                    State::Ids
                }
            },
            (State::Id(BioSampleId::BioSample), Event::Text(text)) => {
                sample.accession = text.unescape()?.into_owned();
                State::Id(BioSampleId::BioSample)
            },
            (State::Id(BioSampleId::SRA), Event::Text(text)) => {
                sample.sra = Some(text.unescape()?.into_owned());
                State::Id(BioSampleId::SRA)
            },
            (State::Id(_), Event::End(e)) if e.local_name().as_ref() == b"Id" => State::Ids,


            // Description children
            (State::Description, Event::Start(e)) if e.local_name().as_ref() == b"Title" => State::Title,
            (State::Title, Event::End(e)) if e.local_name().as_ref() == b"Title" => State::Description,
            (State::Title, Event::Text(text)) => {
                sample.title = Some(text.unescape()?.into_owned());
                State::Title
            },

            (State::Description, Event::Start(e)) if e.local_name().as_ref() == b"Organism" => {
                sample.taxonomy_id = parse_attribute(&reader, &e, "taxonomy_id")?;
                sample.taxonomy_name = parse_attribute(&reader, &e, "taxonomy_name")?;
                State::Organism
            },
            (State::Description, Event::Empty(e)) if e.local_name().as_ref() == b"Organism" => {
                sample.taxonomy_id = parse_attribute(&reader, &e, "taxonomy_id")?;
                sample.taxonomy_name = parse_attribute(&reader, &e, "taxonomy_name")?;
                State::Description
            },
            (State::Organism, Event::End(e)) if e.local_name().as_ref() == b"Organism" => State::Description,


            // Organism children
            (State::Organism, Event::Start(e)) if e.local_name().as_ref() == b"OrganismName" => State::OrganismName,
            (State::OrganismName, Event::End(e)) if e.local_name().as_ref() == b"OrganismName" => State::Organism,
            (State::OrganismName, Event::Text(text)) => {
                sample.organism_name = Some(text.unescape()?.into_owned());
                State::OrganismName
            },


            // Owner children
            (State::Owner, Event::Start(e)) if e.local_name().as_ref() == b"Name" => State::OwnerName,
            (State::OwnerName, Event::End(e)) if e.local_name().as_ref() == b"Name" => State::Owner,
            (State::OwnerName, Event::Text(text)) => {
                sample.owner = Some(text.unescape()?.into_owned());
                State::OwnerName
            },


            // Sample children
            (State::Sample, Event::Start(e)) if e.local_name().as_ref() == b"Ids" => State::Ids,
            (State::Ids, Event::End(e)) if e.local_name().as_ref() == b"Ids" => State::Sample,

            (State::Sample, Event::Start(e)) if e.local_name().as_ref() == b"Description" => State::Description,
            (State::Description, Event::End(e)) if e.local_name().as_ref() == b"Description" => State::Sample,

            (State::Sample, Event::Start(e)) if e.local_name().as_ref() == b"Owner" => State::Owner,
            (State::Owner, Event::End(e)) if e.local_name().as_ref() == b"Owner" => State::Sample,

            (State::Sample, Event::Start(e)) if e.local_name().as_ref() == b"Attributes" => State::Attributes,
            (State::Attributes, Event::End(e)) if e.local_name().as_ref() == b"Attributes" => State::Sample,


            (State::Sample, Event::End(e)) if e.local_name().as_ref() == b"BioSample" => {
                State::Root
            },

            (State::Root, Event::Start(e)) if e.local_name().as_ref() == b"BioSample" => {
                sample.submission_date = parse_attribute(&reader, &e, "submission_date")?;
                sample.publication_date = parse_attribute(&reader, &e, "publication_date")?;
                sample.last_update = parse_attribute(&reader, &e, "last_update")?;
                State::Sample
            },


            (State::Root, Event::Eof) => break,
            (state, Event::Eof) => panic!("Unexpected end of file. Last state: {state:?}"),
            (state, _) => state,
        };
    }

    Ok(sample)
}


fn persist_items(samples: Vec<BioSample>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::biosamples::dsl::*;

    let mut conn = pool.get().unwrap();
    let values = samples
        .into_iter()
        .filter(|s| !s.name_id.is_nil())
        .map(|s| s.to_owned().into())
        .collect::<Vec<Record>>();

    match diesel::insert_into(biosamples).values(values).execute(&mut conn) {
        Ok(_) => Ok(()),
        Err(err) => {
            error!(?err, "Failed to persist bulk items");
            Err(err.into())
        },
    }
}


fn load_name_map(df: &DataFrame, pool: &PgPool) -> Result<HashMap<String, Uuid>, Error> {
    use schema::names;
    let mut conn = pool.get().unwrap();

    let mut name_ids: HashMap<String, Uuid> = HashMap::new();

    let mut names = Vec::new();
    for value in df.column("alaName")?.iter() {
        names.push(parse_string(&value).unwrap());
    }
    names.sort();
    names.dedup();

    for chunk in names.chunks(50_000) {
        let results = names::table
            .select((names::id, names::scientific_name))
            .filter(names::scientific_name.eq_any(chunk))
            .load(&mut conn)?;

        for (uuid, name) in results {
            name_ids.insert(name, uuid);
        }
    }

    Ok(name_ids)
}

fn load_taxon_map(df: &DataFrame, pool: &PgPool) -> Result<HashMap<String, Uuid>, Error> {
    let name_ids = load_name_map(&df, &pool)?;

    #[derive(Default)]
    struct Row {
        ncbi_name: String,
        name_id: Option<Uuid>,
    }

    let mut rows = Vec::with_capacity(df.height());
    for _ in 0..df.height() {
        rows.push(Row::default());
    }

    for (idx, value) in df.column("ncbiName")?.iter().enumerate() {
        rows[idx].ncbi_name = parse_string(&value).expect("ncbiName must be present");
    }

    for (idx, value) in df.column("alaName")?.iter().enumerate() {
        let name = parse_string(&value).expect("alaName must be present");
        rows[idx].name_id = match name_ids.get(&name) {
            Some(name_id) => Some(name_id.clone()),
            None => None,
        }
    }

    let mut map = HashMap::new();
    for row in rows {
        if let Some(name_id) = row.name_id {
            map.insert(row.ncbi_name, name_id);
        }
    }

    Ok(map)
}


fn parse_attribute<R>(reader: &Reader<R>, event: &BytesStart, name: &str) -> Result<Option<String>, Error> {
    Ok(match event.try_get_attribute(name)? {
        Some(value) => Some(value.decode_and_unescape_value(reader)?.into_owned()),
        None => None,
    })
}

fn parse_string(value: &AnyValue) -> Option<String> {
    match value {
        AnyValue::Utf8(text) => {
            if text.trim().is_empty() {
                None
            } else {
                Some(text.to_string())
            }
        },
        _ => None,
    }
}

fn parse_i64(value: &AnyValue) -> Option<i64> {
    match value {
        AnyValue::Int64(number) => Some(*number),
        _ => None,
    }
}

fn get_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}
