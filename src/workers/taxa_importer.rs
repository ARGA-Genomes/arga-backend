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
use crate::database::models::{
    Job,
    UserTaxon,
    UserTaxaList,
    Name,
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
                    let taxa_list = create_taxa_list(&data.name, &data.description).unwrap();
                    let path = Path::new(&tmp_path).join(data.tmp_name);
                    import(path, &taxa_list).unwrap();
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
    tmp_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    scientific_name: String,
    authority: Option<String>,
    canonical_name: Option<String>,
    rank: Option<String>,

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
    subspecies: Option<String>,

    // basionym_genus: Option<String>,
    // basionym_subgenus: Option<String>,
    // basionym_species: Option<String>,
    // basionym_subspecies: Option<String>,
    // basionym_canonical_name: Option<String>,
    // basionym_author: Option<String>,
    // basionym_year: Option<String>,

    specific_epithet: Option<String>,
    subspecific_epithet: Option<String>,

    name_according_to: Option<String>,
    name_published_in: Option<String>,

    taxonomic_status: Option<String>,
    taxon_remarks: Option<String>,
    // source: Option<String>,
}

#[derive(Debug, Queryable, Deserialize)]
struct NameMatch {
    id: Uuid,
    scientific_name: String,
}


pub fn create_taxa_list(list_name: &str, list_description: &Option<String>) -> Result<UserTaxaList, Error> {
    use schema::user_taxa_lists::dsl::*;
    let pool = get_connection_pool();
    let mut conn = pool.get()?;

    let taxa_list = diesel::insert_into(user_taxa_lists)
        .values((
            name.eq(list_name),
            description.eq(list_description),
        ))
        .get_result(&mut conn)?;

    Ok(taxa_list)
}

#[instrument]
pub fn import(path: PathBuf, list: &UserTaxaList) -> Result<(), Error> {
    info!("Getting database connection pool");
    let pool = &mut get_connection_pool();

    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    import_names(&records, pool)?;
    import_taxa(&records, list, pool)?;
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
    info!(total=names.len(), total_imported, "Importing specimens finished");

    Ok(())
}


fn import_taxa(records: &Vec<Record>, list: &UserTaxaList, pool: &mut PgPool) -> Result<(), Error> {
    use schema::user_taxa;

    let names = match_names(&records, pool);
    let taxa = extract_taxa(&list, &names, &records);

    // filter out unmatched specimens
    let mut taxa = taxa.into_iter().filter_map(|r| r).collect::<Vec<UserTaxon>>();

    // deduplicate taxa entries so that the same csv file for other imports
    // can be used for taxa imports
    let total = taxa.len();
    info!(total, "Deduplicating taxa");
    taxa.par_sort_by(|a, b| a.scientific_name.cmp(&b.scientific_name));
    taxa.dedup_by(|a, b| a.scientific_name.eq(&b.scientific_name));
    info!(total, duplicates=total - taxa.len(), "Deduplicating taxa finished");

    info!(total=taxa.len(), "Importing taxa");
    let imported: Vec<Result<usize, Error>> = taxa.par_chunks(1000).map(|chunk| {
        let mut conn = pool.get()?;
        let inserted_rows = diesel::insert_into(user_taxa::table)
            .values(chunk)
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
        Name {
            id: Uuid::new_v4(),
            scientific_name: row.scientific_name.clone(),
            canonical_name: row.canonical_name.clone(),
            authorship: row.authority.clone(),
            rank: row.rank.clone().unwrap_or_else(|| derive_taxon_rank(&row)),
        }
    }).collect::<Vec<Name>>();

    info!(names=names.len(), "Extracting names finished");
    names
}


fn extract_taxa(list: &UserTaxaList, names: &HashMap<String, Uuid>, records: &Vec<Record>) -> Vec<Option<UserTaxon>> {
    info!(total=records.len(), "Extracting taxa");

    let taxa = records.par_iter().map(|row| {
        match names.get(&row.scientific_name) {
            Some(name_id) => Some(UserTaxon {
                id: Uuid::new_v4(),
                taxa_lists_id: list.id.clone(),
                name_id: name_id.clone(),
                scientific_name: Some(row.scientific_name.clone()),
                scientific_name_authorship: row.authority.clone(),
                canonical_name: row.canonical_name.clone(),
                specific_epithet: row.specific_epithet.clone(),
                infraspecific_epithet: row.subspecific_epithet.clone(),
                taxon_rank: Some(row.rank.clone().unwrap_or_else(|| derive_taxon_rank(&row))),
                name_according_to: row.name_according_to.clone(),
                name_published_in: row.name_published_in.clone(),
                taxonomic_status: row.taxonomic_status.clone(),
                taxon_remarks: row.taxon_remarks.clone(),
                kingdom: row.kingdom.clone(),
                phylum: row.phylum.clone(),
                class: row.class.clone(),
                order: row.order.clone(),
                family: row.family.clone(),
                genus: row.genus.clone(),
            }),
            None => None,
        }
    }).collect::<Vec<Option<UserTaxon>>>();

    info!(taxa=taxa.len(), "Extracting taxa finished");
    taxa
}


fn derive_taxon_rank(record: &Record) -> String {
    let rank = if record.subspecies.is_some() {
        "subspecies"
    } else if record.subspecific_epithet.is_some() {
        "subspecies"
    } else if record.specific_epithet.is_some() {
        "species"
    } else if record.canonical_name.is_some() {
        "species"
    } else if record.subgenus.is_some() {
        "subgenus"
    } else if record.genus.is_some() {
        "genus"
    } else if record.subtribe.is_some() {
        "subtribe"
    } else if record.tribe.is_some() {
        "tribe"
    } else if record.supertribe.is_some() {
        "supertribe"
    } else if record.subfamily.is_some() {
        "subfamily"
    } else if record.family.is_some() {
        "family"
    } else if record.superfamily.is_some() {
        "superfamily"
    } else if record.suborder.is_some() {
        "suborder"
    } else if record.order.is_some() {
        "order"
    } else if record.superorder.is_some() {
        "superorder"
    } else if record.subclass.is_some() {
        "subclass"
    } else if record.class.is_some() {
        "class"
    } else if record.superclass.is_some() {
        "superclass"
    } else if record.subphylum.is_some() {
        "subphylum"
    } else if record.phylum.is_some() {
        "phylum"
    } else if record.kingdom.is_some() {
        "kingdom"
    } else {
        "unranked"
    };

    String::from(rank)
}


fn get_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    let url = crate::database::get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    Pool::builder().build(manager).expect("Could not build connection pool")
}
