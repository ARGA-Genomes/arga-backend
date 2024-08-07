use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::data::Error;


#[derive(Debug, Clone, Deserialize)]
struct Record {
    id: String,
    sample_id: Option<String>,   // this is a bpa id in practice. eg: 102.100.100/352899
    specimen_id: Option<String>, // bpa format of a vouchered specimen id. eg: WAM R102627
    voucher_number: Option<String>,
    voucher_herbarium_catalog_number: Option<String>,
    // tissue_number: Option<String>, // the tissue number of the vouchered specimen. eg: ABTC119950
    // voucher_or_tissue_number: Option<String>, // eg: R102627
    sample_type: Option<String>,
    title: Option<String>,
    sex: Option<String>,
    genotypic_sex: Option<String>,
    phenotypic_sex: Option<String>,
    lifestage: Option<String>,
    life_stage: Option<String>,
    wild_captive: Option<String>,
    ancillary_notes: Option<String>,
    collector_sample_id: Option<String>,
    collection_date: Option<String>,
    living_collections_event_date: Option<String>,
    habitat: Option<String>,
    collection_method: Option<String>,
    collection_location: Option<String>,
    location_id: Option<String>,
    country: Option<String>,
    state_or_territory: Option<String>,
    state_or_region: Option<String>,
    location_text: Option<String>,
    geo_loc_name: Option<String>,
    elev: Option<String>,
    depth: Option<String>,
    depth_upper: Option<String>,
    depth_lower: Option<String>,
    location_notes: Option<String>,
    sample_site_location_description: Option<String>,
    decimal_latitude_public: Option<String>,
    latitude: Option<String>,
    decimal_longitude_public: Option<String>,
    longitude: Option<String>,
    coord_uncertainty_metres: Option<String>,
    lat_lon: Option<String>,
    location_info_restricted: Option<String>,
    location_generalisation: Option<String>,
    type_status: Option<String>,
    identified_by: Option<String>,
    id_vetting_by: Option<String>,
    id_vetting_date: Option<String>,
    taxon_id: Option<String>,
    scientific_name: Option<String>,
    species_name: Option<String>,
    kingdom: Option<String>,
    phylum: Option<String>,
    class: Option<String>,
    order: Option<String>,
    family: Option<String>,
    genus: Option<String>,
    subspecies_or_variant: Option<String>,
    scientific_name_authorship: Option<String>,
    common_name: Option<String>,
    scientific_name_notes: Option<String>,
    species_complex: Option<String>,
    subspecies: Option<String>,
    env_broad_scale: Option<String>,
    env_local_scale: Option<String>,
    env_medium: Option<String>,
    strain_or_isolate: Option<String>,
    host_common_name: Option<String>,
    host_type: Option<String>,
    host_family: Option<String>,
    host_scientific_name: Option<String>,
    specific_host: Option<String>,
    original_source_host_species: Option<String>,
    host_location: Option<String>,
    host_age: Option<String>,
    host_sex: Option<String>,
    host_state: Option<String>,
    host_disease_outcome: Option<String>,
    nagoya_protocol_compliance: Option<String>,
    nagoya_protocol_permit_number: Option<String>,
    collection_permit: Option<String>,
    collector: Option<String>,
    voucher_herbarium_collector_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CollectionEvent {
    id: String,
    record_id: String,
    material_sample_id: Option<String>,
    r#type: Option<String>,
    record_title_text: Option<String>,
    sex: Option<String>,
    genotypic_sex: Option<String>,
    phenotypic_sex: Option<String>,
    life_stage: Option<String>,
    degree_of_establishment: Option<String>,
    organism_id: Option<String>,
    organism_remarks: Option<String>,
    field_number: Option<String>,
    collection_date: Option<String>,
    habitat: Option<String>,
    event_remarks: Option<String>,
    location: Option<String>,
    location_id: Option<String>,
    country: Option<String>,
    state: Option<String>,
    location_text: Option<String>,
    geo_loc_name: Option<String>,
    elevation: Option<f64>,
    depth: Option<f64>,
    minimum_depth_in_meters: Option<f64>,
    maximum_depth_in_meters: Option<f64>,
    location_notes: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    coordinate_uncertainty_in_metres: Option<String>,
    lat_lon: Option<String>,
    location_info_restricted: Option<String>,
    location_generalisation: Option<String>,
    type_status: Option<String>,
    identified_by: Option<String>,
    date_identified: Option<String>,
    taxon_id: Option<String>,
    scientific_name: Option<String>,
    kingdom: Option<String>,
    phylum: Option<String>,
    class: Option<String>,
    order: Option<String>,
    family: Option<String>,
    genus: Option<String>,
    infraspecicific_epithet: Option<String>,
    scientific_name_authorship: Option<String>,
    vernacular_name: Option<String>,
    taxon_remarks: Option<String>,
    subspecies: Option<String>,
    env_broad_scale: Option<String>,
    env_local_scale: Option<String>,
    env_medium: Option<String>,
    isolate: Option<String>,
    host_common_name: Option<String>,
    host_family: Option<String>,
    host_scientific_name: Option<String>,
    host_location: Option<String>,
    host_age: Option<String>,
    host_sex: Option<String>,
    host_state: Option<String>,
    host_disease_outcome: Option<String>,
    nagoya_protocol_compliance: Option<String>,
    nagoya_protocol_permit_number: Option<String>,
    collection_permit: Option<String>,
    collected_by: Option<String>,
}

pub fn normalise(path: &PathBuf) -> Result<(), Error> {
    let mut reader = csv::Reader::from_path(&path)?;
    let mut writer = csv::Writer::from_path("collections.csv")?;

    for row in reader.deserialize() {
        let record: Record = row?;

        let material_sample_id = record
            .voucher_number
            .or(record.voucher_herbarium_catalog_number)
            .or(record.specimen_id);

        let record_id = material_sample_id
            .clone()
            .or(record.sample_id.clone())
            .unwrap_or(record.id.clone());

        let life_stage = record.life_stage.or(record.lifestage);
        let collection_date = record.collection_date.or(record.living_collections_event_date);
        let state = record.state_or_territory.or(record.state_or_region);
        let location_notes = record.location_notes.or(record.sample_site_location_description);
        let latitude = record.decimal_latitude_public.or(record.latitude);
        let longitude = record.decimal_longitude_public.or(record.longitude);
        let identified_by = record.identified_by.or(record.id_vetting_by);
        let scientific_name = record.scientific_name.or(record.species_name);
        let taxon_remarks = record.scientific_name_notes.or(record.species_complex);
        let host_common_name = record.host_common_name.or(record.host_type);
        let host_scientific_name = record
            .host_scientific_name
            .or(record.specific_host)
            .or(record.original_source_host_species);
        let collected_by = record.collector.or(record.voucher_herbarium_collector_id);

        // let material_sample_id = record.specimen_id.or(record.tissue_number).or(record.voucher_or_tissue_number);
        // let record_id = material_sample_id.clone().or(record.sample_id).unwrap_or(record.id.clone());

        let event = CollectionEvent {
            id: record.id.clone(),
            record_id,
            material_sample_id,
            r#type: record.sample_type,
            record_title_text: record.title,
            sex: record.sex,
            genotypic_sex: record.genotypic_sex,
            phenotypic_sex: record.phenotypic_sex,
            life_stage,
            degree_of_establishment: record.wild_captive,
            organism_id: None,
            organism_remarks: record.ancillary_notes,
            field_number: record.collector_sample_id,
            collection_date,
            habitat: record.habitat,
            event_remarks: record.collection_method,
            location: record.collection_location,
            location_id: record.location_id,
            country: record.country,
            state,
            location_text: record.location_text,
            geo_loc_name: record.geo_loc_name,
            elevation: parse_f64(record.elev),
            depth: parse_f64(record.depth),
            minimum_depth_in_meters: parse_f64(record.depth_upper),
            maximum_depth_in_meters: parse_f64(record.depth_lower),
            location_notes,
            latitude: parse_f64(latitude),
            longitude: parse_f64(longitude),
            coordinate_uncertainty_in_metres: record.coord_uncertainty_metres,
            lat_lon: record.lat_lon,
            location_info_restricted: record.location_info_restricted,
            location_generalisation: record.location_generalisation,
            type_status: record.type_status,
            identified_by,
            date_identified: record.id_vetting_date,
            taxon_id: record.taxon_id,
            scientific_name,
            kingdom: record.kingdom,
            phylum: record.phylum,
            class: record.class,
            order: record.order,
            family: record.family,
            genus: record.genus,
            infraspecicific_epithet: record.subspecies_or_variant,
            scientific_name_authorship: record.scientific_name_authorship,
            vernacular_name: record.common_name,
            taxon_remarks,
            subspecies: record.subspecies,
            env_broad_scale: record.env_broad_scale,
            env_local_scale: record.env_local_scale,
            env_medium: record.env_medium,
            isolate: record.strain_or_isolate,
            host_common_name,
            host_family: record.host_family,
            host_scientific_name,
            host_location: record.host_location,
            host_age: record.host_age,
            host_sex: record.host_sex,
            host_state: record.host_state,
            host_disease_outcome: record.host_disease_outcome,
            nagoya_protocol_compliance: record.nagoya_protocol_compliance,
            nagoya_protocol_permit_number: record.nagoya_protocol_permit_number,
            collection_permit: record.collection_permit,
            collected_by,
        };

        writer.serialize(event)?;
    }

    Ok(())
}


fn parse_f64(value: Option<String>) -> Option<f64> {
    match value {
        Some(v) => str::parse::<f64>(&v).ok(),
        None => None,
    }
}
