CREATE TABLE accession_event_logs (
    operation_id numeric PRIMARY KEY NOT NULL,
    parent_id numeric NOT NULL,
    entity_id varchar NOT NULL,
    dataset_version_id uuid REFERENCES dataset_versions ON DELETE CASCADE NOT NULL,
    action operation_action NOT NULL,
    atom jsonb DEFAULT '{}' NOT NULL
);

CREATE INDEX accession_event_logs_parent_id ON accession_event_logs (parent_id);
CREATE INDEX accession_event_logs_entity_id ON accession_event_logs (entity_id);
CREATE INDEX accession_event_logs_dataset_version_id ON accession_event_logs (dataset_version_id);


CREATE MATERIALIZED VIEW accession_event_entities AS
SELECT entity_id FROM accession_event_logs GROUP BY entity_id ORDER BY entity_id;

CREATE UNIQUE INDEX accession_event_entities_entity_id ON accession_event_entities (entity_id);
