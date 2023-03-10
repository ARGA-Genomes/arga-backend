CREATE TABLE taxa (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),

    taxon_id bigint,
    dataset_id varchar(255),
    parent_name_usage_id varchar(255),
    accepted_name_usage_id varchar(255),
    original_name_usage_id varchar(255),

    scientific_name varchar(1000),
    scientific_name_authorship varchar(1000),
    canonical_name varchar(255),
    generic_name varchar(255),

    specific_epithet varchar(255),
    infraspecific_epithet varchar(255),

    taxon_rank text,
    name_according_to text,
    name_published_in text,
    taxonomic_status varchar(255),
    nomenclatural_status varchar(255),
    taxon_remarks text,

    kingdom varchar(255),
    phylum varchar(255),
    class varchar(255),
    "order" varchar(255),
    family varchar(255),
    genus varchar(255)
);
