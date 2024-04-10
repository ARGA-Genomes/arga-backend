CREATE TABLE specimen_logs (
    operation_id numeric PRIMARY KEY NOT NULL,
    parent_id numeric NOT NULL,
    entity_id varchar NOT NULL,
    dataset_version_id uuid REFERENCES dataset_versions ON DELETE CASCADE NOT NULL,
    action operation_action NOT NULL,
    atom jsonb DEFAULT '{}' NOT NULL
);

CREATE INDEX specimen_logs_parent_id ON specimen_logs (parent_id);
CREATE INDEX specimen_logs_entity_id ON specimen_logs (entity_id);
CREATE INDEX specimen_logs_dataset_version_id ON specimen_logs (dataset_version_id);
