CREATE TYPE taxonomic_status AS ENUM (
  'accepted',
  'undescribed',
  'species_inquirenda',
  'manuscript_name',
  'hybrid',
  'synonym',
  'unaccepted',
  'informal',
  'placeholder'
);


CREATE TABLE taxa (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    parent_taxon_id uuid REFERENCES classifications,

    status taxonomic_status NOT NULL,
    scientific_name varchar NOT NULL,
    canonical_name varchar NOT NULL,

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
);

CREATE UNIQUE INDEX taxa_unique_name ON taxa (scientific_name, dataset_id);


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
