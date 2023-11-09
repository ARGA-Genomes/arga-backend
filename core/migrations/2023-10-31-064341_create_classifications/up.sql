CREATE TYPE taxonomic_rank AS ENUM (
  'domain',
  'superkingdom',
  'kingdom',
  'subkingdom',
  'phylum',
  'subphylum',
  'superclass',
  'class',
  'subclass',
  'superorder',
  'order',
  'suborder',
  'superfamily',
  'family',
  'subfamily',
  'supertribe',
  'tribe',
  'subtribe',
  'genus',
  'subgenus',
  'species',
  'subspecies',
  'unranked',
  'higher_taxon'
);


CREATE TABLE classifications (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets NOT NULL,
    parent_id uuid REFERENCES classifications NOT NULL,

    taxon_id varchar NOT NULL,
    rank taxonomic_rank NOT NULL,
    accepted_name_usage varchar NOT NULL,
    original_name_usage varchar NOT NULL,
    scientific_name varchar NOT NULL,
    scientific_name_authorship varchar NOT NULL,
    canonical_name varchar NOT NULL,
    nomenclatural_code varchar NOT NULL,
    status taxonomic_status NOT NULL,

    citation varchar,
    vernacular_names text[],
    alternative_names text[],
    description text,
    remarks text
);


CREATE UNIQUE INDEX classifications_unique_taxon_id ON classifications (taxon_id);
CREATE UNIQUE INDEX classifications_unique_name ON classifications (scientific_name);
