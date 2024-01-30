CREATE TYPE attribute_category AS ENUM (
    'bushfire_recovery'
);

CREATE TYPE attribute_value_type AS ENUM (
    'boolean',
    'integer',
    'decimal',
    'string',
    'timestamp'
);

CREATE TABLE name_attributes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets NOT NULL,
    name_id uuid REFERENCES names NOT NULL,

    name varchar NOT NULL,
    category attribute_category NOT NULL,
    value_type attribute_value_type NOT NULL,

    value_bool boolean,
    value_int bigint,
    value_decimal decimal,
    value_str varchar,
    value_timestamp timestamp without time zone
);

CREATE UNIQUE INDEX name_attributes_unique_name ON name_attributes (dataset_id, name_id, name);
