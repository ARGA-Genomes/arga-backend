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
  'higher_taxon',

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
  'subfamilia',
  'subordo',
  'regio',

  'species',
  'subspecies',
  'infraspecies',

  'aggregate_genera',
  'aggregate_species',
  'cohort',
  'subcohort',
  'division',
  'infraclass',
  'infraorder',
  'section',
  'subdivision',

  'incertae_sedis',
  'special_form',
  'unranked'
);


CREATE TABLE taxa (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets NOT NULL,
    parent_id uuid REFERENCES taxa,

    status taxonomic_status NOT NULL,
    rank taxonomic_rank NOT NULL,

    scientific_name varchar NOT NULL,
    canonical_name varchar NOT NULL,
    authorship varchar,

    nomenclatural_code varchar NOT NULL,
    citation varchar,
    vernacular_names text[],
    description text,
    remarks text,

    created_at timestamp with time zone NOT NULL DEFAULT current_timestamp,
    updated_at timestamp with time zone NOT NULL DEFAULT current_timestamp
);

-- defer the contraint until the transaction is committed. this allows bulk inserts that reference
-- each other without having to worry about the order of insertions
ALTER TABLE taxa ALTER CONSTRAINT taxa_parent_id_fkey DEFERRABLE INITIALLY DEFERRED;

CREATE INDEX taxa_parent_id ON taxa (parent_id);
CREATE UNIQUE INDEX taxa_unique_name ON taxa (scientific_name, dataset_id);
