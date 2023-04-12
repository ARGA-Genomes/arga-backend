CREATE TYPE attribute_data_type AS ENUM ('string', 'text', 'integer', 'boolean', 'timestamp', 'array');

CREATE TABLE attributes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar(255) NOT NULL UNIQUE,
    data_type attribute_data_type NOT NULL,
    description text,
    reference_url varchar(255)
);
COMMENT ON TABLE attributes IS 'Attribute definitions that can be associated with objects like taxa records';
COMMENT ON COLUMN attributes.id IS 'The attribute UUID. We use UUID to allow external source to generate and reference new attributes';
COMMENT ON COLUMN attributes.name IS 'The unique name of an attribute';
COMMENT ON COLUMN attributes.data_type IS 'The value type of the attribute. Used to query the correct object values table';
COMMENT ON COLUMN attributes.description IS 'What the attribute represents';
COMMENT ON COLUMN attributes.reference_url IS 'A link to the official definition of the attribute';


CREATE TABLE objects (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id uuid NOT NULL,
    attribute_id uuid NOT NULL,
    value_id uuid NOT NULL
);
COMMENT ON TABLE objects IS 'The entity-attribute-value association table';
COMMENT ON COLUMN objects.id IS 'The object UUID. Having a UUID primary key allows external sources to generate and associate object in bulk';
COMMENT ON COLUMN objects.attribute_id IS 'The attribute definition UUID. Links to the definition for this object association';
COMMENT ON COLUMN objects.value_id IS 'The value UUID. Used with the attribute definition data type to retrieve the correct value from an object values table';


CREATE TABLE object_values_string (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    value varchar(255) NOT NULL
);

CREATE TABLE object_values_text  (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    value text NOT NULL
);

CREATE TABLE object_values_integer (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    value bigint NOT NULL
);

CREATE TABLE object_values_boolean (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    value boolean NOT NULL
);

CREATE TABLE object_values_timestamp (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    value timestamp with time zone NOT NULL
);

CREATE TABLE object_values_array (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    value text[] NOT NULL
);
