CREATE TABLE organism_logs (
    operation_id numeric PRIMARY KEY NOT NULL,
    parent_id numeric NOT NULL,
    entity_id varchar NOT NULL,
    dataset_version_id uuid REFERENCES dataset_versions ON DELETE CASCADE NOT NULL,
    action operation_action NOT NULL,
    atom jsonb DEFAULT '{}' NOT NULL
);

CREATE INDEX organism_logs_parent_id ON organism_logs (parent_id);
CREATE INDEX organism_logs_entity_id ON organism_logs (entity_id);
CREATE INDEX organism_logs_dataset_version_id ON organism_logs (dataset_version_id);



CREATE MATERIALIZED VIEW organism_entities AS
SELECT entity_id FROM organism_logs
GROUP BY entity_id
ORDER BY entity_id;

CREATE UNIQUE INDEX organism_entities_entity_id ON organism_entities (entity_id);
