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
  'hyporder',
  'minorder',
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
  'higher_taxon',

  'aggregate_genera',
  'aggregate_species',
  'cohort',
  'subcohort',
  'division',
  'incertae_sedis',
  'infraclass',
  'infraorder',
  'section',
  'subdivision',

  'regnum',
  'familia',
  'classis',
  'ordo',
  'varietas',
  'forma',
  'subclassis',
  'superordo',
  'sectio',
  'nothovarietas',
  'subvarietas',
  'series',
  'infraspecies',
  'subfamilia',
  'subordo',
  'special_form',
  'regio'
);


CREATE TABLE classifications (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets NOT NULL,
    parent_id uuid REFERENCES classifications NOT NULL,

    taxon_id serial,
    rank taxonomic_rank NOT NULL,
    accepted_name_usage varchar,
    original_name_usage varchar,
    scientific_name varchar NOT NULL,
    scientific_name_authorship varchar,
    canonical_name varchar NOT NULL,
    nomenclatural_code varchar NOT NULL,
    status taxonomic_status NOT NULL,

    citation varchar,
    vernacular_names text[],
    alternative_names text[],
    description text,
    remarks text
);


CREATE INDEX classifications_parent_id ON classifications (parent_id);
CREATE UNIQUE INDEX classifications_unique_taxon_id ON classifications (taxon_id);
CREATE UNIQUE INDEX classifications_unique_name_rank ON classifications (scientific_name, rank);

ALTER SEQUENCE classifications_taxon_id_seq RESTART WITH 5000000;
ALTER TABLE classifications ALTER CONSTRAINT classifications_parent_id_fkey DEFERRABLE INITIALLY DEFERRED;
