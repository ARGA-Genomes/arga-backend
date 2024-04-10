CREATE TABLE collection_event_logs (
    operation_id numeric PRIMARY KEY NOT NULL,
    parent_id numeric REFERENCES collection_event_logs NOT NULL,
    entity_id varchar NOT NULL,
    dataset_version_id uuid REFERENCES dataset_versions ON DELETE CASCADE NOT NULL,
    action operation_action NOT NULL,
    atom jsonb DEFAULT '{}' NOT NULL
);

CREATE INDEX collection_event_logs_parent_id ON collection_event_logs (parent_id);
CREATE INDEX collection_event_logs_entity_id ON collection_event_logs (entity_id);
