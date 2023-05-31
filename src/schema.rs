// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "attribute_data_type"))]
    pub struct AttributeDataType;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "geometry"))]
    pub struct Geometry;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "job_status"))]
    pub struct JobStatus;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "name_list_type"))]
    pub struct NameListType;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "region_type"))]
    pub struct RegionType;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AttributeDataType;

    attributes (id) {
        id -> Uuid,
        name -> Varchar,
        data_type -> AttributeDataType,
        description -> Nullable<Text>,
        reference_url -> Nullable<Varchar>,
    }
}

diesel::table! {
    conservation_statuses (id) {
        id -> Uuid,
        list_id -> Uuid,
        name_id -> Uuid,
        status -> Varchar,
        state -> Nullable<Varchar>,
        source -> Nullable<Varchar>,
    }
}

diesel::table! {
    distribution (id) {
        id -> Uuid,
        taxon_id -> Nullable<Int8>,
        location_id -> Nullable<Text>,
        locality -> Nullable<Text>,
        country -> Nullable<Varchar>,
        country_code -> Nullable<Varchar>,
        location_remarks -> Nullable<Text>,
        establishment_means -> Nullable<Varchar>,
        life_stage -> Nullable<Varchar>,
        occurrence_status -> Nullable<Varchar>,
        threat_status -> Nullable<Varchar>,
        source -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Geometry;

    ibra (ogc_fid) {
        ogc_fid -> Int4,
        reg_code_7 -> Nullable<Varchar>,
        reg_name_7 -> Nullable<Varchar>,
        hectares -> Nullable<Float8>,
        sq_km -> Nullable<Float8>,
        rec_id -> Nullable<Int4>,
        reg_code_6 -> Nullable<Varchar>,
        reg_name_6 -> Nullable<Varchar>,
        reg_no_61 -> Nullable<Float8>,
        feat_id -> Nullable<Varchar>,
        shape_leng -> Nullable<Float8>,
        shape_area -> Nullable<Float8>,
        wkb_geometry -> Nullable<Geometry>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::JobStatus;

    jobs (id) {
        id -> Uuid,
        status -> JobStatus,
        worker -> Varchar,
        payload -> Nullable<Jsonb>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    media (id) {
        id -> Uuid,
        media_id -> Nullable<Int8>,
        media_type -> Nullable<Varchar>,
        format -> Nullable<Varchar>,
        identifier -> Nullable<Varchar>,
        references -> Nullable<Varchar>,
        created -> Nullable<Timestamptz>,
        creator -> Nullable<Varchar>,
        publisher -> Nullable<Varchar>,
        license -> Nullable<Varchar>,
        rights_holder -> Nullable<Varchar>,
        catalog_number -> Nullable<Int8>,
    }
}

diesel::table! {
    media_observations (id) {
        id -> Uuid,
        media_id -> Nullable<Int8>,
        scientific_name -> Nullable<Varchar>,
        basis_of_record -> Nullable<Varchar>,
        institution_code -> Nullable<Varchar>,
        collection_code -> Nullable<Varchar>,
        dataset_name -> Nullable<Varchar>,
        captive -> Nullable<Varchar>,
        event_date -> Nullable<Timestamptz>,
        license -> Nullable<Varchar>,
        rights_holder -> Nullable<Varchar>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::NameListType;

    name_lists (id) {
        id -> Uuid,
        list_type -> NameListType,
        name -> Varchar,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    name_vernacular_names (name_id, vernacular_name_id) {
        name_id -> Uuid,
        vernacular_name_id -> Int8,
    }
}

diesel::table! {
    names (id) {
        id -> Uuid,
        scientific_name -> Varchar,
        canonical_name -> Nullable<Varchar>,
        authorship -> Nullable<Varchar>,
        rank -> Varchar,
    }
}

diesel::table! {
    object_values_array (id) {
        id -> Uuid,
        value -> Array<Nullable<Text>>,
    }
}

diesel::table! {
    object_values_boolean (id) {
        id -> Uuid,
        value -> Bool,
    }
}

diesel::table! {
    object_values_integer (id) {
        id -> Uuid,
        value -> Int8,
    }
}

diesel::table! {
    object_values_string (id) {
        id -> Uuid,
        value -> Varchar,
    }
}

diesel::table! {
    object_values_text (id) {
        id -> Uuid,
        value -> Text,
    }
}

diesel::table! {
    object_values_timestamp (id) {
        id -> Uuid,
        value -> Timestamptz,
    }
}

diesel::table! {
    objects (id) {
        id -> Uuid,
        entity_id -> Uuid,
        attribute_id -> Uuid,
        value_id -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RegionType;

    regions (id) {
        id -> Uuid,
        name_id -> Uuid,
        region_type -> RegionType,
        values -> Array<Nullable<Text>>,
    }
}

diesel::table! {
    spatial_ref_sys (srid) {
        srid -> Int4,
        auth_name -> Nullable<Varchar>,
        auth_srid -> Nullable<Int4>,
        srtext -> Nullable<Varchar>,
        proj4text -> Nullable<Varchar>,
    }
}

diesel::table! {
    specimens (id) {
        id -> Uuid,
        list_id -> Uuid,
        name_id -> Uuid,
        type_status -> Varchar,
        institution_name -> Nullable<Varchar>,
        organism_id -> Nullable<Varchar>,
        locality -> Nullable<Varchar>,
        latitude -> Nullable<Float8>,
        longitude -> Nullable<Float8>,
        details -> Nullable<Varchar>,
        remarks -> Nullable<Varchar>,
    }
}

diesel::table! {
    taxa (id) {
        id -> Uuid,
        taxon_id -> Nullable<Int8>,
        dataset_id -> Nullable<Varchar>,
        parent_name_usage_id -> Nullable<Varchar>,
        accepted_name_usage_id -> Nullable<Varchar>,
        original_name_usage_id -> Nullable<Varchar>,
        scientific_name -> Nullable<Varchar>,
        scientific_name_authorship -> Nullable<Varchar>,
        canonical_name -> Nullable<Varchar>,
        generic_name -> Nullable<Varchar>,
        specific_epithet -> Nullable<Varchar>,
        infraspecific_epithet -> Nullable<Varchar>,
        taxon_rank -> Nullable<Text>,
        name_according_to -> Nullable<Text>,
        name_published_in -> Nullable<Text>,
        taxonomic_status -> Nullable<Varchar>,
        nomenclatural_status -> Nullable<Varchar>,
        taxon_remarks -> Nullable<Text>,
        kingdom -> Nullable<Varchar>,
        phylum -> Nullable<Varchar>,
        class -> Nullable<Varchar>,
        order -> Nullable<Varchar>,
        family -> Nullable<Varchar>,
        genus -> Nullable<Varchar>,
    }
}

diesel::table! {
    taxon_photos (id) {
        id -> Uuid,
        name_id -> Uuid,
        url -> Varchar,
        source -> Nullable<Varchar>,
        publisher -> Nullable<Varchar>,
        license -> Nullable<Varchar>,
        rights_holder -> Nullable<Varchar>,
    }
}

diesel::table! {
    types_and_specimen (id) {
        id -> Uuid,
        taxon_id -> Nullable<Int8>,
        designation_type -> Nullable<Varchar>,
        designated_by -> Nullable<Varchar>,
        scientific_name -> Nullable<Varchar>,
        taxon_rank -> Nullable<Varchar>,
        source -> Nullable<Varchar>,
    }
}

diesel::table! {
    user_taxa (id) {
        id -> Uuid,
        taxa_lists_id -> Uuid,
        name_id -> Uuid,
        scientific_name -> Nullable<Varchar>,
        scientific_name_authorship -> Nullable<Varchar>,
        canonical_name -> Nullable<Varchar>,
        specific_epithet -> Nullable<Varchar>,
        infraspecific_epithet -> Nullable<Varchar>,
        taxon_rank -> Nullable<Text>,
        name_according_to -> Nullable<Text>,
        name_published_in -> Nullable<Text>,
        taxonomic_status -> Nullable<Varchar>,
        taxon_remarks -> Nullable<Text>,
        kingdom -> Nullable<Varchar>,
        phylum -> Nullable<Varchar>,
        class -> Nullable<Varchar>,
        order -> Nullable<Varchar>,
        family -> Nullable<Varchar>,
        genus -> Nullable<Varchar>,
    }
}

diesel::table! {
    user_taxa_lists (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        name -> Varchar,
        email -> Varchar,
        user_role -> Varchar,
        password_hash -> Varchar,
        password_salt -> Varchar,
    }
}

diesel::table! {
    vernacular_names (id) {
        id -> Int8,
        vernacular_name -> Varchar,
        language -> Nullable<Varchar>,
    }
}

diesel::joinable!(conservation_statuses -> name_lists (list_id));
diesel::joinable!(conservation_statuses -> names (name_id));
diesel::joinable!(name_vernacular_names -> names (name_id));
diesel::joinable!(name_vernacular_names -> vernacular_names (vernacular_name_id));
diesel::joinable!(regions -> names (name_id));
diesel::joinable!(specimens -> name_lists (list_id));
diesel::joinable!(specimens -> names (name_id));
diesel::joinable!(taxon_photos -> names (name_id));
diesel::joinable!(user_taxa -> names (name_id));
diesel::joinable!(user_taxa -> user_taxa_lists (taxa_lists_id));

diesel::allow_tables_to_appear_in_same_query!(
    attributes,
    conservation_statuses,
    distribution,
    ibra,
    jobs,
    media,
    media_observations,
    name_lists,
    name_vernacular_names,
    names,
    object_values_array,
    object_values_boolean,
    object_values_integer,
    object_values_string,
    object_values_text,
    object_values_timestamp,
    objects,
    regions,
    spatial_ref_sys,
    specimens,
    taxa,
    taxon_photos,
    types_and_specimen,
    user_taxa,
    user_taxa_lists,
    users,
    vernacular_names,
);
