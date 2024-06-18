-- replace the table with one that serves a different purpose
ALTER TABLE taxon_history DROP COLUMN act_id;
DROP TABLE nomenclatural_acts;

CREATE TYPE nomenclatural_act_type AS ENUM (
  'species_nova',
  'subspecies_nova',
  'genus_species_nova',
  'combinatio_nova',
  'revived_status'
);


CREATE TABLE nomenclatural_acts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),

    entity_id varchar NOT NULL,
    publication_id uuid REFERENCES name_publications NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    acted_on_id uuid REFERENCES names NOT NULL,

    act nomenclatural_act_type NOT NULL,
    source_url varchar NOT NULL,

    created_at timestamp with time zone NOT NULL DEFAULT current_timestamp,
    updated_at timestamp with time zone NOT NULL DEFAULT current_timestamp
);
COMMENT ON TABLE nomenclatural_acts IS 'Name definitions and redefinitions. Any act on a name';
COMMENT ON COLUMN nomenclatural_acts.entity_id IS 'The entity in the logs that this record is reduced from';
COMMENT ON COLUMN nomenclatural_acts.name_id IS 'The name that has been defined or changed';
COMMENT ON COLUMN nomenclatural_acts.acted_on_id IS 'The name that is being affected by this act';
COMMENT ON COLUMN nomenclatural_acts.act IS 'The specific act being performed by this record';

CREATE UNIQUE INDEX nomenclatural_acts_unique_name ON nomenclatural_acts (name_id);
CREATE UNIQUE INDEX nomenclatural_acts_unique_entity ON nomenclatural_acts (entity_id);
CREATE INDEX nomenclatural_acts_acted_on ON nomenclatural_acts (acted_on_id);
