CREATE TABLE taxon_source (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar NOT NULL,
    description varchar,
    url varchar
);

CREATE UNIQUE INDEX taxon_source_name ON taxon_source (name);


CREATE TYPE taxonomic_status AS ENUM (
  'valid',
  'undescribed',
  'species_inquirenda',
  'hybrid',
  'synonym',
  'invalid'
);


CREATE TABLE taxa (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    source uuid REFERENCES taxon_source NOT NULL,
    name_id uuid REFERENCES names NOT NULL,

    status taxonomic_status NOT NULL,
    scientific_name varchar NOT NULL,
    canonical_name varchar,

    kingdom varchar,
    phylum varchar,
    class varchar,
    "order" varchar,
    family varchar,
    tribe varchar,
    genus varchar,
    specific_epithet varchar,

    subphylum varchar,
    subclass varchar,
    suborder varchar,
    subfamily varchar,
    subtribe varchar,
    subgenus varchar,
    subspecific_epithet varchar,

    superclass varchar,
    superorder varchar,
    superfamily varchar,
    supertribe varchar,

    order_authority varchar,
    family_authority varchar,
    genus_authority varchar,
    species_authority varchar

    -- name_according_to varchar,
    -- name_published_in varchar
);

CREATE UNIQUE INDEX taxa_unique_name ON taxa (scientific_name);


CREATE TABLE taxon_history (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    old_taxon_id uuid REFERENCES taxa NOT NULL,
    new_taxon_id uuid REFERENCES taxa NOT NULL,

    changed_by varchar,
    reason varchar,

    created_at timestamp with time zone NOT NULL DEFAULT current_timestamp
);


CREATE TABLE taxon_remarks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    taxon_id uuid REFERENCES taxa NOT NULL,
    remark varchar NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT current_timestamp
);
