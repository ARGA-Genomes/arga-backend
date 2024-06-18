DROP TABLE nomenclatural_acts;
DROP TYPE nomenclatural_act_type;

-- switch back to the old table
CREATE TABLE nomenclatural_acts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar NOT NULL,
    source_url varchar,
    citation varchar,
    example varchar
);

CREATE UNIQUE INDEX nomenclatural_acts_unique_name ON nomenclatural_acts (name);

-- this doesn't completely revert the change as act_id is NOT NULL originally but
-- this should work well enough as a pathway back
ALTER TABLE taxon_history ADD COLUMN act_id uuid REFERENCES nomenclatural_acts;
