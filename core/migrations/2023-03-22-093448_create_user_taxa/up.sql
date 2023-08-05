CREATE TABLE user_taxa_lists (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar NOT NULL,
    description text
);

CREATE TABLE user_taxa (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    taxa_lists_id uuid REFERENCES user_taxa_lists NOT NULL,
    name_id uuid REFERENCES names NOT NULL,

    scientific_name varchar,
    scientific_name_authorship varchar,
    canonical_name varchar,

    specific_epithet varchar,
    infraspecific_epithet varchar,

    taxon_rank text,
    name_according_to text,
    name_published_in text,
    taxonomic_status varchar,
    taxon_remarks text,

    kingdom varchar,
    phylum varchar,
    class varchar,
    "order" varchar,
    family varchar,
    genus varchar
);

CREATE INDEX user_taxa_list_id ON user_taxa (taxa_lists_id);
