use std::path::PathBuf;

use arga_core::models::{CollectionEvent, Dataset, Specimen};
use arga_core::schema;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use rayon::prelude::*;
use tracing::info;

use crate::error::Error;
use crate::extractors::collection_extractor;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub fn import(path: PathBuf, dataset: &Dataset, context: &Vec<Dataset>, pool: &mut PgPool) -> Result<(), Error> {
    info!("Extracting collection events");

    let extractor = collection_extractor::extract(path, dataset, context, pool)?;

    for extract in extractor {
        let extract = extract?;
        // the extractors generate UUIDs and associate all records in the extract
        // so we must import them in a specific order to not trigger referential integrity
        // errors in the database.
        // right now we don't want to cross polinate datasets when it comes to linking
        // specimens or events to other specimens so this approach works for us as that
        // means every collection import should always create new records
        import_specimens(extract.specimens, pool)?;
        import_collection_events(extract.collection_events, pool)?;
    }

    info!("Import finished");
    Ok(())
}


fn import_specimens(records: Vec<Specimen>, pool: &mut PgPool) -> Result<(), Error> {
    use diesel::upsert::excluded;
    use schema::specimens::dsl::*;

    info!(total = records.len(), "Importing specimens");
    let imported: Vec<Result<usize, Error>> = records
        .par_chunks(1000)
        .map(|chunk| {
            let mut conn = pool.get()?;
            let inserted_rows = diesel::insert_into(specimens)
                .values(chunk)
                .on_conflict(id)
                .do_update()
                .set((
                    dataset_id.eq(excluded(dataset_id)),
                    name_id.eq(excluded(name_id)),
                    record_id.eq(excluded(record_id)),
                    material_sample_id.eq(excluded(material_sample_id)),
                    organism_id.eq(excluded(organism_id)),
                    institution_name.eq(excluded(institution_name)),
                    institution_code.eq(excluded(institution_code)),
                    collection_code.eq(excluded(collection_code)),
                    recorded_by.eq(excluded(recorded_by)),
                    identified_by.eq(excluded(identified_by)),
                    identified_date.eq(excluded(identified_date)),
                    type_status.eq(excluded(type_status)),
                    locality.eq(excluded(locality)),
                    country.eq(excluded(country)),
                    country_code.eq(excluded(country_code)),
                    state_province.eq(excluded(state_province)),
                    county.eq(excluded(county)),
                    municipality.eq(excluded(municipality)),
                    latitude.eq(excluded(latitude)),
                    longitude.eq(excluded(longitude)),
                    elevation.eq(excluded(elevation)),
                    depth.eq(excluded(depth)),
                    elevation_accuracy.eq(excluded(elevation_accuracy)),
                    depth_accuracy.eq(excluded(depth_accuracy)),
                    location_source.eq(excluded(location_source)),
                    details.eq(excluded(details)),
                    remarks.eq(excluded(remarks)),
                    identification_remarks.eq(excluded(identification_remarks)),
                    entity_id.eq(excluded(entity_id)),
                ))
                .execute(&mut conn)?;
            Ok(inserted_rows)
        })
        .collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total = records.len(), total_imported, "Importing specimens finished");

    Ok(())
}

fn import_collection_events(collections: Vec<CollectionEvent>, pool: &mut PgPool) -> Result<(), Error> {
    use diesel::upsert::excluded;
    use schema::collection_events::dsl::*;

    info!(total = collections.len(), "Importing specimen collection events");
    let imported: Vec<Result<usize, Error>> = collections
        .par_chunks(1000)
        .map(|chunk| {
            let mut conn = pool.get()?;
            let inserted_rows = diesel::insert_into(collection_events)
                .values(chunk)
                .on_conflict(id)
                .do_update()
                .set((
                    dataset_id.eq(excluded(dataset_id)),
                    specimen_id.eq(excluded(specimen_id)),
                    event_date.eq(excluded(event_date)),
                    event_time.eq(excluded(event_time)),
                    collected_by.eq(excluded(collected_by)),
                    field_number.eq(excluded(field_number)),
                    catalog_number.eq(excluded(catalog_number)),
                    record_number.eq(excluded(record_number)),
                    individual_count.eq(excluded(individual_count)),
                    organism_quantity.eq(excluded(organism_quantity)),
                    organism_quantity_type.eq(excluded(organism_quantity_type)),
                    sex.eq(excluded(sex)),
                    genotypic_sex.eq(excluded(genotypic_sex)),
                    phenotypic_sex.eq(excluded(phenotypic_sex)),
                    life_stage.eq(excluded(life_stage)),
                    reproductive_condition.eq(excluded(reproductive_condition)),
                    behavior.eq(excluded(behavior)),
                    establishment_means.eq(excluded(establishment_means)),
                    degree_of_establishment.eq(excluded(degree_of_establishment)),
                    pathway.eq(excluded(pathway)),
                    occurrence_status.eq(excluded(occurrence_status)),
                    preparation.eq(excluded(preparation)),
                    other_catalog_numbers.eq(excluded(other_catalog_numbers)),
                    env_broad_scale.eq(excluded(env_broad_scale)),
                    env_local_scale.eq(excluded(env_local_scale)),
                    env_medium.eq(excluded(env_medium)),
                    habitat.eq(excluded(habitat)),
                    ref_biomaterial.eq(excluded(ref_biomaterial)),
                    source_mat_id.eq(excluded(source_mat_id)),
                    specific_host.eq(excluded(specific_host)),
                    strain.eq(excluded(strain)),
                    isolate.eq(excluded(isolate)),
                    field_notes.eq(excluded(field_notes)),
                    remarks.eq(excluded(remarks)),
                    entity_id.eq(excluded(entity_id)),
                ))
                .execute(&mut conn)?;
            Ok(inserted_rows)
        })
        .collect();

    let mut total_imported = 0;
    for chunk_total in imported {
        total_imported += chunk_total?;
    }
    info!(total = collections.len(), total_imported, "Importing specimen collection events finished");

    Ok(())
}
