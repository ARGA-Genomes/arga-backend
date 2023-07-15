use std::path::{Path, PathBuf};
use std::collections::HashMap;

use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::Deserialize;
use stakker::*;
use tracing::{instrument, info, error};
use uuid::Uuid;

use crate::database::schema;
use crate::database::models::{Job, Name, TaxonSource, Taxon, TaxonomicStatus,
    // Regions, RegionType,
};


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub struct TaxaImporter {
    thread: PipedThread<Job, Job>,
}

impl TaxaImporter {
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
        info!("Running taxa importer");
        let tmp_path = std::env::var("ADMIN_TMP_UPLOAD_STORAGE").expect("No upload storage specified");

        if let Some(payload) = job.payload {
            match serde_json::from_value::<ImportJobData>(payload) {
                Ok(data) => {
                    let source = get_or_create_taxon_source(&data.name, &data.description, &data.url).unwrap();
                    let path = Path::new(&tmp_path).join(data.tmp_name);
                    import(path, &source).unwrap();
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
    Csv(#[from] csv::Error),

    #[error("an error occurred getting a database connection")]
    Pool(#[from] diesel::r2d2::PoolError),
}

#[derive(Debug, Deserialize)]
struct ImportJobData {
    name: String,
    description: Option<String>,
    url: Option<String>,
    tmp_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    scientific_name: String,
    // authority: Option<String>,
    canonical_name: Option<String>,
    // rank: Option<String>,

    kingdom: Option<String>,
    phylum: Option<String>,
    class: Option<String>,
    order: Option<String>,
    family: Option<String>,
    tribe: Option<String>,
    genus: Option<String>,

    superclass: Option<String>,
    superorder: Option<String>,
    superfamily: Option<String>,
    supertribe: Option<String>,

    subphylum: Option<String>,
    subclass: Option<String>,
    suborder: Option<String>,
    subfamily: Option<String>,
    subtribe: Option<String>,
    subgenus: Option<String>,
    // subspecies: Option<String>,

    // basionym_genus: Option<String>,
    // basionym_subgenus: Option<String>,
    // basionym_species: Option<String>,
    // basionym_subspecies: Option<String>,
    // basionym_canonical_name: Option<String>,
    // basionym_author: Option<String>,
    // basionym_year: Option<String>,

    specific_epithet: Option<String>,
    subspecific_epithet: Option<String>,

    species: Option<String>,
    genus_full: Option<String>,
    family_full: Option<String>,
    order_full: Option<String>,

    // name_according_to: Option<String>,
    // name_published_in: Option<String>,

    taxonomic_status: Option<String>,
    // taxon_remarks: Option<String>,
    // source: Option<String>,
    // source_url: Option<String>,
    // source_id: Option<String>,
}

#[derive(Debug, Queryable, Deserialize)]
struct NameMatch {
    id: Uuid,
    scientific_name: String,
}


pub fn get_or_create_taxon_source(name: &str, description: &Option<String>, url: &Option<String>) -> Result<TaxonSource, Error> {
    use schema::taxon_source;
    let pool = get_connection_pool();
    let mut conn = pool.get()?;

    if let Some(source) = taxon_source::table.filter(taxon_source::name.eq(name)).get_result(&mut conn).optional()? {
        return Ok(source);
    }

    let source = diesel::insert_into(taxon_source::table)
        .values((
            taxon_source::name.eq(name),
            taxon_source::description.eq(description),
            taxon_source::url.eq(url),
        ))
        .get_result(&mut conn)?;

    Ok(source)
}

#[instrument]
pub fn import(path: PathBuf, source: &TaxonSource) -> Result<(), Error> {
    info!("Getting database connection pool");
    let pool = &mut get_connection_pool();

    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    import_names(&records, pool)?;
    import_taxa(&records, source, pool)?;
    // import_regions(&records, pool)?;

    Ok(())
}


fn import_names(records: &Vec<Record>, pool: &mut PgPool) -> Result<(), Error> {
    use schema::names;

    let names = extract_names(&records);

    info!(total=records.len(), "Importing names");
    let imported: Vec<Result<usize, Error>> = names.par_chunks(10_000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(names::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=names.len(), total_imported, "Importing names finished");

    Ok(())
}


fn import_taxa(records: &Vec<Record>, source: &TaxonSource, pool: &mut PgPool) -> Result<(), Error> {
    use schema::taxa;

    let names = match_names(&records, pool);
    let taxa = extract_taxa(&source, &names, &records);
    // let taxa = extract_history(&source, &names, &records);
    // let taxa = extract_remarks(&source, &names, &records);

    // filter out unmatched specimens
    let taxa = taxa.into_iter().filter_map(|r| r).collect::<Vec<Taxon>>();

    // deduplicate taxa entries so that the same csv file for other imports
    // can be used for taxa imports
    // let total = taxa.len();
    // info!(total, "Deduplicating taxa");
    // taxa.par_sort_by(|a, b| a.scientific_name.cmp(&b.scientific_name));
    // taxa.dedup_by(|a, b| a.scientific_name.eq(&b.scientific_name));
    // info!(total, duplicates=total - taxa.len(), "Deduplicating taxa finished");

    info!(total=taxa.len(), "Importing taxa");
    let imported: Vec<Result<usize, Error>> = taxa.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(taxa::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(inserted_rows)
    }).collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total=taxa.len(), total_imported, "Importing taxa finished");

    Ok(())
}


fn match_names(records: &Vec<Record>, pool: &mut PgPool) -> HashMap<String, Uuid> {
    use schema::names;
    info!(total=records.len(), "Matching names");

    let matched: Vec<Result<Vec<NameMatch>, Error>> = records.par_chunks(50_000).map(|chunk| {
        let mut conn = pool.get()?;
        let scientific_names: Vec<&String> = chunk.iter().map(|row| &row.scientific_name).collect();

        let results = names::table
            .select((names::id, names::scientific_name))
            .filter(names::scientific_name.eq_any(scientific_names))
            .load::<NameMatch>(&mut conn)?;

        Ok::<Vec<NameMatch>, Error>(results)
    }).collect();

    let mut id_map: HashMap<String, Uuid> = HashMap::new();

    for chunk in matched {
        if let Ok(names) = chunk {
            for name_match in names {
                id_map.insert(name_match.scientific_name, name_match.id);
            }
        }
    }

    info!(total=records.len(), matched=id_map.len(), "Matching names finished");
    id_map
}


fn extract_names(records: &Vec<Record>) -> Vec<Name> {
    info!(total=records.len(), "Extracting names");

    let names = records.par_iter().map(|row| {
        let species_authority = extract_authority(&row.canonical_name, &row.species);

        Name {
            id: Uuid::new_v4(),
            scientific_name: row.scientific_name.clone(),
            canonical_name: row.canonical_name.clone(),
            authorship: species_authority,
        }
    }).collect::<Vec<Name>>();

    info!(names=names.len(), "Extracting names finished");
    names
}


fn extract_taxa(source: &TaxonSource, names: &HashMap<String, Uuid>, records: &Vec<Record>) -> Vec<Option<Taxon>> {
    info!(total=records.len(), "Extracting taxa");

    let taxa = records.par_iter().map(|row| {
        let order_authority = extract_authority(&row.order, &row.order_full);
        let family_authority = extract_authority(&row.family, &row.family_full);
        let genus_authority = extract_authority(&row.genus, &row.genus_full);

        // if genus isn't supplied try to extract it from the scientific name
        // by splitting on the first space and taking the first component
        let genus = match &row.genus {
            Some(genus) => Some(genus.clone()),
            None => extract_genus(&row.scientific_name),
        };

        // if genus isn't supplied try to extract it from the scientific name
        // by splitting on the first space and taking the first component
        let specific_epithet = match &row.specific_epithet {
            Some(specific_epithet) => Some(specific_epithet.clone()),
            None => extract_specific_epithet(&row.scientific_name),
        };

        // fallback to extracting the authority from the scientific name if a
        // species value isn't present
        let species_authority = match &row.species {
            Some(_) => extract_authority(&row.canonical_name, &row.species),
            None => extract_authority(&row.canonical_name, &Some(row.scientific_name.clone())),
        };

        match names.get(&row.scientific_name) {
            Some(name_id) => Some(Taxon {
                id: Uuid::new_v4(),
                source: source.id.clone(),
                name_id: name_id.clone(),

                status: str_to_taxonomic_status(&row.taxonomic_status),
                scientific_name: row.scientific_name.clone(),
                canonical_name: row.canonical_name.clone(),

                kingdom: row.kingdom.clone(),
                phylum: row.phylum.clone(),
                class: row.class.clone(),
                order: row.order.clone(),
                family: row.family.clone(),
                tribe: row.tribe.clone(),
                genus,
                specific_epithet,

                subphylum: row.subphylum.clone(),
                subclass: row.subclass.clone(),
                suborder: row.suborder.clone(),
                subfamily: row.subfamily.clone(),
                subtribe: row.subtribe.clone(),
                subgenus: row.subgenus.clone(),
                subspecific_epithet: row.subspecific_epithet.clone(),

                superclass: row.superclass.clone(),
                superorder: row.superorder.clone(),
                superfamily: row.superfamily.clone(),
                supertribe: row.supertribe.clone(),

                order_authority,
                family_authority,
                genus_authority,
                species_authority,

                // name_according_to: row.name_according_to.clone(),
                // name_published_in: row.name_published_in.clone(),
            }),
            None => None,
        }
    }).collect::<Vec<Option<Taxon>>>();

    info!(taxa=taxa.len(), "Extracting taxa finished");
    taxa
}


fn extract_genus(scientific_name: &str) -> Option<String> {
    match scientific_name.split_once(" ") {
        Some((genus, _rest)) => Some(genus.to_string()),
        None => None,
    }
}

fn extract_specific_epithet(scientific_name: &str) -> Option<String> {
    match scientific_name.split_once(" ") {
        Some((genus, rest)) => match rest.split_once(" ") {
            Some((specific_epithet, _rest)) => Some(specific_epithet.to_string()),
            None => None,
        }
        None => None,
    }
}

fn extract_authority(name: &Option<String>, full_name: &Option<String>) -> Option<String> {
    match (name, full_name) {
        (Some(name), Some(full_name)) => Some(full_name.trim_start_matches(name).trim().to_string()),
        _ => None
    }
}


// based on https://rs.gbif.org/vocabulary/gbif/taxonomic_status.xml
fn str_to_taxonomic_status(value: &Option<String>) -> TaxonomicStatus {
    match value {
        Some(status) => match status.to_lowercase().as_str() {
            "valid" => TaxonomicStatus::Valid,
            "valid name" => TaxonomicStatus::Valid,
            "accepted" => TaxonomicStatus::Valid,
            "accepted name" => TaxonomicStatus::Valid,

            "undescribed" => TaxonomicStatus::Undescribed,
            "species inquirenda" => TaxonomicStatus::SpeciesInquirenda,
            "hybrid" => TaxonomicStatus::Hybrid,

            "synonym" => TaxonomicStatus::Synonym,
            "junior synonym" => TaxonomicStatus::Synonym,
            "later synonym" => TaxonomicStatus::Synonym,


            "invalid" => TaxonomicStatus::Invalid,
            "invalid name" => TaxonomicStatus::Invalid,
            "unaccepted" => TaxonomicStatus::Invalid,
            "unaccepted name" => TaxonomicStatus::Invalid,

            _ => TaxonomicStatus::Invalid,
        },
        None => TaxonomicStatus::Invalid,
    }
}


fn get_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    let url = crate::database::get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    Pool::builder().build(manager).expect("Could not build connection pool")
}






// #[derive(Default)]
// struct RegionImport {
//     scientific_name: String,
//     ibra: Option<Vec<String>>,
//     imcra: Option<Vec<String>>,
// }

// #[instrument(skip(df, conn))]
// fn import_regions(df: &DataFrame, conn: &mut PgConnection) -> Result<(), Error> {
//     info!(height = df.height(), "Transforming");

//     let mut rows = Vec::with_capacity(df.height());
//     for _ in 0..df.height() {
//         rows.push(RegionImport::default());
//     }

//     let series = df.column("scientificName")?;
//     for (idx, value) in series.iter().enumerate() {
//         rows[idx].scientific_name = parse_string(&value).expect("scientificName is mandatory")
//     }

//     // set the optional fields for the name data. it wont overwrite existing names
//     // but new names will prserve these values indefinitely
//     let attr_names = df.get_column_names();
//     let attributes = find_attributes(&attr_names, conn)?;

//     for attribute in &attributes {
//         let series = df.column(&attribute.name)?;
//         info!(name = attribute.name, "Enumerating column");

//         match attribute.name.as_str() {
//             "ibraRegions" => for (idx, value) in series.iter().enumerate() {
//                 rows[idx].ibra = parse_array(&value);
//             },
//             "imcraRegions" => for (idx, value) in series.iter().enumerate() {
//                 rows[idx].imcra = parse_array(&value);
//             },
//             _ => {}
//         }
//     }

//     info!(total=rows.len(), "Importing regions");
//     use schema::{regions, names};

//     let mut total = 0;
//     for chunk in rows.chunks(10_000) {
//         info!(rows = chunk.len(), "Inserting into regions");

//         let mut id_map: HashMap<String, Uuid> = HashMap::new();
//         let all_names: Vec<&String> = rows.iter().map(|row| &row.scientific_name).collect();

//         let results = names::table
//             .select((names::id, names::scientific_name))
//             .filter(names::scientific_name.eq_any(all_names))
//             .load::<(Uuid, String)>(conn)?;

//         for (uuid, name) in results {
//             id_map.insert(name, uuid);
//         }

//         let mut values = Vec::new();
//         for row in chunk {
//             if let Some(uuid) = id_map.get(&row.scientific_name) {
//                 if let Some(value) = &row.ibra {
//                     values.push(Regions {
//                         id: Uuid::new_v4(),
//                         name_id: uuid.clone(),
//                         region_type: RegionType::Ibra,
//                         values: value.clone(),
//                     });
//                 }
//                 if let Some(value) = &row.imcra {
//                     values.push(Regions {
//                         id: Uuid::new_v4(),
//                         name_id: uuid.clone(),
//                         region_type: RegionType::Imcra,
//                         values: value.clone(),
//                     });
//                 }
//             }
//         }

//         let inserted_rows = diesel::insert_into(regions::table)
//             .values(values)
//             .execute(conn)?;

//         info!(inserted_rows, "Inserted into regions");
//         total += inserted_rows;
//     }

//     info!(total, "Finished importing regions");
//     Ok(())
// }
