use std::collections::HashMap;
use std::path::PathBuf;

use arga_core::models::{NomenclaturalAct, NomenclaturalActType};
use arga_core::schema;
use chrono::{DateTime, Utc};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use rayon::prelude::*;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use super::utils::date_time_from_str_opt;
use crate::error::Error;
use crate::matchers::name_matcher;

type PgPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Debug, Clone, Deserialize)]
struct Record {
    entity_id: String,
    acted_on: Option<String>,
    scientific_name: String,

    #[serde(deserialize_with = "nomenclatural_act_opt")]
    act: Option<NomenclaturalActType>,
    source_url: String,
    publication: String,
    publication_date: Option<String>,
}

/// Extract nomenclatural acts from a CSV file
pub fn extract(path: &PathBuf, pool: &mut PgPool) -> Result<Vec<NomenclaturalAct>, Error> {
    let mut records: Vec<Record> = Vec::new();
    for row in csv::Reader::from_path(&path)?.deserialize() {
        records.push(row?);
    }

    let names = name_matcher::name_map(pool)?;
    let publications = publications_map(pool)?;
    let acts = extract_acts(&records, &names, &publications);
    Ok(acts)
}

fn extract_acts(
    records: &Vec<Record>,
    names: &name_matcher::NameMap,
    publications: &PublicationMap,
) -> Vec<NomenclaturalAct> {
    info!(total = records.len(), "Extracting nomenclatural acts");

    let acts = records
        .par_iter()
        .map(|row| {
            let acted_on = row.acted_on.clone().unwrap_or("Biota".to_string());
            let acted_on = names.get(&acted_on);
            let name = names.get(&row.scientific_name);
            let publication = publications.get(&row.publication);

            match (acted_on, name, publication) {
                (Some(acted_on), Some(name), Some(publication)) => Some(NomenclaturalAct {
                    id: Uuid::new_v4(),
                    entity_id: row.entity_id.clone(),
                    name_id: name.id,
                    acted_on_id: acted_on.id,
                    publication_id: publication.clone(),
                    act: row.act.clone().unwrap_or(NomenclaturalActType::NameUsage),
                    source_url: row.source_url.clone(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }),
                _ => None,
            }
        })
        .collect::<Vec<Option<NomenclaturalAct>>>();

    let acts: Vec<NomenclaturalAct> = acts.into_iter().filter_map(|r| r).collect();

    info!(acts = acts.len(), "Extracting nomenclatural acts finished");
    acts
}


pub type PublicationMap = HashMap<String, Uuid>;

fn publications_map(pool: &mut PgPool) -> Result<PublicationMap, Error> {
    use schema::name_publications::dsl::*;
    let mut conn = pool.get()?;

    let records = name_publications
        .select((id, citation.assume_not_null()))
        .filter(citation.is_not_null())
        .load::<(Uuid, String)>(&mut conn)?;

    let mut map = HashMap::new();
    for (pub_id, pub_citation) in records {
        map.insert(pub_citation, pub_id);
    }

    Ok(map)
}


pub fn nomenclatural_act_opt<'de, D>(deserializer: D) -> Result<Option<NomenclaturalActType>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use NomenclaturalActType::*;

    let s: Option<String> = Deserialize::deserialize(deserializer)?;

    Ok(match s {
        None => None,
        Some(s) => match s.as_str() {
            "SpeciesNova" => Some(SpeciesNova),
            "SubspeciesNova" => Some(SubspeciesNova),
            "GenusSpeciesNova" => Some(GenusSpeciesNova),
            "CombinatioNova" => Some(CombinatioNova),
            "RevivedStatus" => Some(RevivedStatus),
            "NameUsage" => Some(NameUsage),
            "" => None,
            _ => None,
        },
    })
}
