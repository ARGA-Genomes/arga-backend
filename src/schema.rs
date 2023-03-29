// @generated automatically by Diesel CLI.

diesel::table! {
    descriptions (id) {
        id -> Uuid,
        taxon_id -> Nullable<Int8>,
        #[sql_name = "type"]
        type_ -> Nullable<Varchar>,
        language -> Nullable<Varchar>,
        description -> Nullable<Text>,
        source -> Nullable<Varchar>,
        creator -> Nullable<Varchar>,
        contributor -> Nullable<Varchar>,
        license -> Nullable<Varchar>,
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
    gnl (id) {
        id -> Uuid,
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
        source -> Nullable<Varchar>,
        taxa_lists_id -> Nullable<Uuid>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    descriptions,
    distribution,
    media,
    media_observations,
    taxa,
    types_and_specimen,
    user_taxa,
    user_taxa_lists,
    users,
);
