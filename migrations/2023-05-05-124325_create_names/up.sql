CREATE TABLE names (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    scientific_name varchar NOT NULL,
    canonical_name varchar,
    authorship varchar,
    rank varchar NOT NULL
);
COMMENT ON TABLE names IS 'All taxa names. Unique names used to associated attributes for specific taxonomic names';

CREATE UNIQUE INDEX names_scientific_name ON names (scientific_name);
CREATE INDEX names_scientific_name_rank ON names (scientific_name, rank);
CREATE INDEX names_canonical_name ON names (canonical_name);
CREATE INDEX names_canonical_name_rank ON names (canonical_name, rank);


CREATE TABLE name_properties (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id uuid REFERENCES names NOT NULL,
    attribute_id uuid REFERENCES attributes NOT NULL,
    value_id bigint NOT NULL
);
COMMENT ON TABLE name_properties IS 'The entity-attribute-value association table for names';
COMMENT ON COLUMN name_properties.id IS 'The object UUID. Having a UUID primary key allows external sources to generate and associate object in bulk';
COMMENT ON COLUMN name_properties.entity_id IS 'The entity ID. For names this is the unique scientific_name on the names table';
COMMENT ON COLUMN name_properties.attribute_id IS 'The attribute definition UUID. Links to the definition for this object association';
COMMENT ON COLUMN name_properties.value_id IS 'The value UUID. Used with the attribute definition data type to retrieve the correct value from an object values table';

CREATE UNIQUE INDEX name_unique_properties ON name_properties (entity_id, attribute_id, value_id);
CREATE INDEX name_properties_entity_id ON name_properties (entity_id);
CREATE INDEX name_properties_attribute_id ON name_properties (attribute_id);
CREATE INDEX name_properties_entity_id_attribute_id ON name_properties (entity_id, attribute_id);
