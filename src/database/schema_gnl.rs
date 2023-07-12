diesel::table! {
    species (id) {
        id -> Uuid,
        source -> Uuid,
        name_id -> Uuid,
        status -> crate::database::schema::sql_types::TaxonomicStatus,
        scientific_name -> Varchar,
        canonical_name -> Nullable<Varchar>,
        kingdom -> Nullable<Varchar>,
        phylum -> Nullable<Varchar>,
        class -> Nullable<Varchar>,
        order -> Nullable<Varchar>,
        family -> Nullable<Varchar>,
        tribe -> Nullable<Varchar>,
        genus -> Nullable<Varchar>,
        specific_epithet -> Nullable<Varchar>,
        subphylum -> Nullable<Varchar>,
        subclass -> Nullable<Varchar>,
        suborder -> Nullable<Varchar>,
        subfamily -> Nullable<Varchar>,
        subtribe -> Nullable<Varchar>,
        subgenus -> Nullable<Varchar>,
        subspecific_epithet -> Nullable<Varchar>,
        superclass -> Nullable<Varchar>,
        superorder -> Nullable<Varchar>,
        superfamily -> Nullable<Varchar>,
        supertribe -> Nullable<Varchar>,
        order_authority -> Nullable<Varchar>,
        family_authority -> Nullable<Varchar>,
        genus_authority -> Nullable<Varchar>,
        species_authority -> Nullable<Varchar>,
        subspecies -> Nullable<Array<Text>>,
        window_rank -> BigInt,
    }
}

diesel::table! {
    undescribed_species (genus) {
        genus -> Varchar,
        genus_authority -> Nullable<Varchar>,
        names -> Array<Text>,
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
    }
}


use super::schema::{names, assemblies};

diesel::joinable!(ranked_taxa -> assemblies (name_id));
diesel::joinable!(ranked_taxa -> names (name_id));

diesel::allow_tables_to_appear_in_same_query!(
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
