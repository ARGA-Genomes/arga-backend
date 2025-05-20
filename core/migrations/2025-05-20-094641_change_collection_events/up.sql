CREATE MATERIALIZED VIEW collection_event_entities AS
SELECT entity_id FROM collection_event_logs
GROUP BY entity_id
ORDER BY entity_id;

CREATE UNIQUE INDEX collection_event_entities_entity_id ON collection_event_entities (entity_id);


DROP TABLE collection_events;

CREATE TABLE collection_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id varchar NOT NULL,
    field_collecting_id varchar NOT NULL,

    name_id uuid REFERENCES names ON DELETE CASCADE NOT NULL,
    organism_id uuid REFERENCES organisms ON DELETE CASCADE NOT NULL,
    specimen_id uuid REFERENCES specimens ON DELETE CASCADE,

    event_date date,
    event_time time without time zone,
    collected_by varchar,
    collection_remarks varchar,
    identified_by varchar,
    identified_date date,
    identification_remarks varchar,

    locality varchar,
    country varchar,
    country_code varchar,
    state_province varchar,
    county varchar,
    municipality varchar,
    latitude float,
    longitude float,
    elevation float,
    depth float,
    elevation_accuracy float,
    depth_accuracy float,
    location_source varchar,

    preparation varchar,
    environment_broad_scale varchar,
    environment_local_scale varchar,
    environment_medium varchar,
    habitat varchar,
    specific_host varchar,
    individual_count varchar,
    organism_quantity varchar,
    organism_quantity_type varchar,

    strain varchar,
    isolate varchar,
    field_notes varchar
);

CREATE UNIQUE INDEX collection_events_entity_id ON collection_events (entity_id);
CREATE INDEX collection_events_field_collecting_id ON collection_events (field_collecting_id);
CREATE INDEX collection_events_name_id ON collection_events (name_id);
CREATE INDEX collection_events_organism_id ON collection_events (organism_id);
CREATE INDEX collection_events_specimen_id ON collection_events (specimen_id);
