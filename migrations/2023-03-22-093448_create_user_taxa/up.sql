CREATE TABLE user_taxa (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    taxa_lists_id uuid NOT NULL,

    scientific_name varchar(1000),
    scientific_name_authorship varchar(1000),
    canonical_name varchar(255),

    specific_epithet varchar(255),
    infraspecific_epithet varchar(255),

    taxon_rank text,
    name_according_to text,
    name_published_in text,
    taxonomic_status varchar(255),
    taxon_remarks text,

    kingdom varchar(255),
    phylum varchar(255),
    class varchar(255),
    "order" varchar(255),
    family varchar(255),
    genus varchar(255)
);
