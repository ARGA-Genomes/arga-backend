CREATE TYPE taxonomic_act_type AS ENUM (
  'unaccepted',
  'synonym',
  'homonym',
  'nomenclatural_synonym',
  'taxonomic_synonym',
  'replaced_synonym'
);


CREATE TABLE taxonomic_acts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),

    entity_id varchar NOT NULL,
    taxon_id uuid REFERENCES taxa NOT NULL,
    accepted_taxon_id uuid REFERENCES taxa,

    act taxonomic_act_type NOT NULL,
    source_url varchar,

    created_at timestamp with time zone NOT NULL DEFAULT current_timestamp,
    updated_at timestamp with time zone NOT NULL DEFAULT current_timestamp
);
COMMENT ON TABLE taxonomic_acts IS 'An act within a specific taxonomic system';
COMMENT ON COLUMN taxonomic_acts.entity_id IS 'The entity in the logs that this record is reduced from';
COMMENT ON COLUMN taxonomic_acts.taxon_id IS 'The taxon that is being affected by this act';
COMMENT ON COLUMN taxonomic_acts.accepted_taxon_id IS 'The taxon that is considered currently accepted in the system';
COMMENT ON COLUMN taxonomic_acts.act IS 'The specific act being performed by this record';

CREATE UNIQUE INDEX taxonomic_acts_unique_entity ON taxonomic_acts (entity_id);
CREATE INDEX taxonomic_acts_unique_taxon ON taxonomic_acts (taxon_id);
CREATE INDEX taxonomic_acts_accepted_taxon ON taxonomic_acts (accepted_taxon_id);
