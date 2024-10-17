CREATE TYPE taxonomic_act_type AS ENUM (
  'accepted',
  'unaccepted',
  'synonym',
  'homonym',
  'nomenclatural_synonym',
  'taxonomic_synonym',
  'replaced_synonym'
);

ALTER TABLE taxonomic_acts ADD COLUMN act taxonomic_act_type NOT NULL;
