use std::collections::HashMap;
use std::{path::PathBuf, fs::File};
use std::io::prelude::*;

use indicatif::{ProgressStyle, ParallelProgressIterator};
use itertools::izip;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{info, error};

use arga_core::schema;
use crate::data::Error;
use crate::data::ncbi::biosamples::{Progress, MultiProgressIterator};


type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type NameMap = HashMap<String, NameMatch>;


#[derive(Debug, Clone, Queryable, Deserialize)]
pub struct NameMatch {
    pub scientific_name: String,
    pub canonical_name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Record {
    processid: String,
    trace_ids: String,
    trace_names: String,
    trace_links: String,
    species_name: Option<String>,
}

#[derive(Debug)]
struct Trace {
    processid: String,
    id: String,
    name: String,
    link: String,
}

#[derive(Debug, Clone, Default, Serialize)]
struct Abif {
    a_analyzed: Option<String>,
    c_analyzed: Option<String>,
    g_analyzed: Option<String>,
    t_analyzed: Option<String>,
}

impl From<abif::Abif> for Abif {
    fn from(value: abif::Abif) -> Self {
        Self {
            a_analyzed: value.a_analyzed().map(|arr| arr.iter().map(|&v| v.to_string()).collect::<Vec<String>>().join(",")),
            c_analyzed: value.c_analyzed().map(|arr| arr.iter().map(|&v| v.to_string()).collect::<Vec<String>>().join(",")),
            g_analyzed: value.g_analyzed().map(|arr| arr.iter().map(|&v| v.to_string()).collect::<Vec<String>>().join(",")),
            t_analyzed: value.t_analyzed().map(|arr| arr.iter().map(|&v| v.to_string()).collect::<Vec<String>>().join(",")),
        }
    }
}


pub fn name_map(pool: &mut PgPool) -> Result<NameMap, Error> {
    use schema::names::dsl::*;
    info!("Creating name map");

    let mut conn = pool.get()?;

    let results = names
        .select((scientific_name, canonical_name))
        .load::<NameMatch>(&mut conn)?;

    let mut map = NameMap::new();
    for name_match in results {
        map.insert(name_match.scientific_name.clone(), name_match.clone());
        map.insert(name_match.canonical_name.clone(), name_match);
    }

    info!(total=map.len(), "Creating name map finished");
    Ok(map)
}


pub fn download(path: PathBuf, output: PathBuf) -> Result<(), Error> {
    let mut reader = csv::Reader::from_path(&path)?;

    info!("Getting all currently imported names");
    let url = arga_core::get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    let mut pool = Pool::builder().build(manager)?;

    let names = name_map(&mut pool)?;


    info!(?path, "Deserializing CSV file");
    let mut traces = Vec::new();

    // process the csv file and extract combined trace info
    for row in reader.deserialize() {
        let record: Record = row?;

        if !names.contains_key(&record.species_name.unwrap_or_default()) {
            continue;
        }

        let ids: Vec<&str> = record.trace_ids.split("|").collect();
        let names: Vec<&str> = record.trace_names.split("|").filter(|name| name != &record.processid).collect();
        let links: Vec<&str> = record.trace_links.split("|").collect();

        if ids.len() != names.len() || ids.len() != links.len() {
            error!(?ids, ?names, ?links, "Trace array dimension mismatch");
        }
        assert!(ids.len() == names.len() && ids.len() == links.len());

        for (id, name, link) in izip!(ids, names, links) {
            traces.push(Trace {
                processid: record.processid.clone(),
                id: id.to_owned(),
                name: name.to_owned(),
                link: link.to_owned(),
            });
        }
    }


    // download trace files in parallel
    info!(?output, "Downloading");
    let style = ProgressStyle::with_template("[{elapsed_precise}/{eta}] {bar:40.cyan/blue} {human_pos}/{human_len} @ {per_sec}").unwrap();

    let failed: Vec<(Trace, Error)> = traces.into_par_iter().progress_with_style(style).filter_map(|trace| {
        match download_file(&output, &trace) {
            Ok(_) => None,
            Err(err) => Some((trace, err)),
        }
    }).collect();

    for (trace, err) in failed {
        error!(?err, ?trace, "Download failed");
    }

    Ok(())
}



fn download_file(output: &PathBuf, trace: &Trace) -> Result<(), Error> {
    let path = output.join(&trace.processid).join(&trace.id);
    std::fs::create_dir_all(&path)?;

    let mut file = File::create(path.join(&trace.name))?;

    let body = ureq::get(&trace.link).call()?;
    let mut buffer = vec![];
    body.into_reader().read_to_end(&mut buffer)?;
    file.write_all(&buffer)?;

    Ok(())
}



pub fn process(input: PathBuf, dir: PathBuf) -> Result<(), Error> {
    info!(?input, ?dir, "Parsing all trace files");

    let traces = extract_traces(&input)?;
    let mut writer = csv::Writer::from_path("traces.csv")?;

    let bars = Progress::new();
    let parse_bar = bars.add("Parsing", traces.len());

    // parse trace files in parallel
    info!(?dir, "Parsing");

    for chunk in traces.chunks(100_000) {
        let results: Vec<abif::Abif> = chunk
            .into_par_iter()
            .progress_with(parse_bar.clone())
            .filter_map(|trace| {
                match parse_trace(&trace, &dir) {
                    Ok(abif) => Some(abif),
                    Err(_err) => {
                        // error!(?err, "Failed to parse trace file");
                        None
                    },
            }
        }).collect();

        for trace in results.into_iter().multiprogress_with(&bars, "Persisting traces") {
            writer.serialize::<Abif>(trace.into())?;
        }
    }

    Ok(())
}


fn parse_trace(trace: &Trace, dir: &PathBuf) -> Result<abif::Abif, Error> {
    let path = dir.join(&trace.processid).join(&trace.id).join(&trace.name);
    let file = File::open(path)?;
    let abif = abif::Abif::read(file)?;
    Ok(abif)
}


fn extract_traces(path: &PathBuf) -> Result<Vec<Trace>, Error> {
    let mut reader = csv::Reader::from_path(&path)?;

    info!("Getting all currently imported names");
    let url = arga_core::get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    let mut pool = Pool::builder().build(manager)?;

    let names = name_map(&mut pool)?;

    info!(?path, "Deserializing CSV file");
    let mut traces = Vec::new();

    // process the csv file and extract combined trace info
    for row in reader.deserialize() {
        let record: Record = row?;

        if !names.contains_key(&record.species_name.unwrap_or_default()) {
            continue;
        }

        let ids: Vec<&str> = record.trace_ids.split("|").collect();
        let names: Vec<&str> = record.trace_names.split("|").filter(|name| name != &record.processid).collect();
        let links: Vec<&str> = record.trace_links.split("|").collect();

        if ids.len() != names.len() || ids.len() != links.len() {
            error!(?ids, ?names, ?links, "Trace array dimension mismatch");
        }
        assert!(ids.len() == names.len() && ids.len() == links.len());

        for (id, name, link) in izip!(ids, names, links) {
            traces.push(Trace {
                processid: record.processid.clone(),
                id: id.to_owned(),
                name: name.to_owned(),
                link: link.to_owned(),
            });
        }
    }

    Ok(traces)
}
