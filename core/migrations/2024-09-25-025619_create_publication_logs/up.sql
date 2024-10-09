CREATE TABLE publication_logs (
    operation_id numeric PRIMARY KEY NOT NULL,
    parent_id numeric NOT NULL,
    entity_id varchar NOT NULL,
    dataset_version_id uuid REFERENCES dataset_versions ON DELETE CASCADE NOT NULL,
    action operation_action NOT NULL,
    atom jsonb DEFAULT '{}' NOT NULL
);

CREATE INDEX publication_logs_parent_id ON publication_logs (parent_id);
CREATE INDEX publication_logs_entity_id ON publication_logs (entity_id);
CREATE INDEX publication_logs_dataset_version_id ON publication_logs (dataset_version_id);
