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

diesel::table! {
    eav (object_id) {
        object_id -> Uuid,
        entity_id -> Uuid,
        attribute_id -> Uuid,
        value_id -> Uuid,
        name -> Varchar,
        data_type -> Varchar,
        value_string -> Nullable<Varchar>,
        value_text -> Nullable<Text>,
        value_integer -> Nullable<Int8>,
        value_boolean -> Nullable<Bool>,
        value_timestamp -> Nullable<Timestamptz>,
        value_array -> Nullable<Array<Nullable<Varchar>>>,
    }
}

diesel::table! {
    eav_strings (object_id) {
        object_id -> Uuid,
        entity_id -> Uuid,
        attribute_id -> Uuid,
        value_id -> Uuid,
        name -> Varchar,
        value -> Varchar,
    }
}

diesel::table! {
    eav_text (object_id) {
        object_id -> Uuid,
        entity_id -> Uuid,
        attribute_id -> Uuid,
        value_id -> Uuid,
        name -> Varchar,
        value -> Text,
    }
}

diesel::table! {
    eav_integers (object_id) {
        object_id -> Uuid,
        entity_id -> Uuid,
        attribute_id -> Uuid,
        value_id -> Uuid,
        name -> Varchar,
        value -> Int8,
    }
}

diesel::table! {
    eav_booleans (object_id) {
        object_id -> Uuid,
        entity_id -> Uuid,
        attribute_id -> Uuid,
        value_id -> Uuid,
        name -> Varchar,
        value -> Bool,
    }
}

diesel::table! {
    eav_timestamps (object_id) {
        object_id -> Uuid,
        entity_id -> Uuid,
        attribute_id -> Uuid,
        value_id -> Uuid,
        name -> Varchar,
        value -> Timestamptz,
    }
}

diesel::table! {
    eav_arrays (object_id) {
        object_id -> Uuid,
        entity_id -> Uuid,
        attribute_id -> Uuid,
        value_id -> Uuid,
        name -> Varchar,
        value -> Array<Nullable<Varchar>>,
    }
}

diesel::table! {
    user_taxa_objects (object_id) {
        object_id -> Uuid,
        entity_id -> Uuid,
        attribute_id -> Uuid,
        value_id -> Uuid,
        attribute_name -> Varchar,
        data_type -> Varchar,
        taxa_lists_id -> Nullable<Uuid>,
        scientific_name -> Nullable<Varchar>,
        scientific_name_authorship -> Nullable<Varchar>,
        canonical_name -> Nullable<Varchar>,
        taxon_rank -> Nullable<Text>,
        taxonomic_status -> Nullable<Varchar>,
    }
}

diesel::table! {
    ranked_taxa (id) {
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
        list_name -> Varchar,
        taxa_priority -> Integer,
    }
}

diesel::table! {
    common_names (id) {
        id -> Uuid,
        vernacular_name -> Varchar,
        vernacular_language -> Nullable<Varchar>,
        scientific_name -> Varchar,
        scientific_name_authorship -> Nullable<Varchar>,
        canonical_name -> Nullable<Varchar>,
        rank -> Nullable<Varchar>,
    }
}


use super::schema::{names, assemblies};

diesel::joinable!(ranked_taxa -> assemblies (name_id));
diesel::joinable!(ranked_taxa -> names (name_id));

diesel::allow_tables_to_appear_in_same_query!(
    gnl,
    eav,
    eav_strings,
    eav_text,
    eav_integers,
    eav_booleans,
    eav_timestamps,
    eav_arrays,
    ranked_taxa,
    common_names,
);

diesel::allow_tables_to_appear_in_same_query!(
    ranked_taxa,
    names,
);

diesel::allow_tables_to_appear_in_same_query!(
    ranked_taxa,
    assemblies,
);
