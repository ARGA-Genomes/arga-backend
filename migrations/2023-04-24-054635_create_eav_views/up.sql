CREATE VIEW eav AS
SELECT
    objects.id AS object_id,
    entity_id,
    attribute_id,
    value_id,
    name,
    data_type,
    object_values_string.value AS value_string,
    object_values_text.value AS value_text,
    object_values_integer.value AS value_integer,
    object_values_boolean.value AS value_boolean,
    object_values_timestamp.value AS value_timestamp,
    object_values_array.value AS value_array
FROM objects
JOIN attributes ON attribute_id=attributes.id
LEFT JOIN object_values_string ON value_id=object_values_string.id
LEFT JOIN object_values_text ON value_id=object_values_text.id
LEFT JOIN object_values_integer ON value_id=object_values_integer.id
LEFT JOIN object_values_boolean ON value_id=object_values_boolean.id
LEFT JOIN object_values_timestamp ON value_id=object_values_timestamp.id
LEFT JOIN object_values_array ON value_id=object_values_array.id;


CREATE VIEW eav_strings AS
SELECT
    objects.id AS object_id,
    entity_id,
    attribute_id,
    value_id,
    name,
    value
FROM objects
JOIN attributes ON attribute_id=attributes.id
JOIN object_values_string ON value_id=object_values_string.id;


CREATE VIEW eav_text AS
SELECT
    objects.id AS object_id,
    entity_id,
    attribute_id,
    value_id,
    name,
    value
FROM objects
JOIN attributes ON attribute_id=attributes.id
JOIN object_values_text ON value_id=object_values_text.id;


CREATE VIEW eav_integers AS
SELECT
    objects.id AS object_id,
    entity_id,
    attribute_id,
    value_id,
    name,
    value
FROM objects
JOIN attributes ON attribute_id=attributes.id
JOIN object_values_integer ON value_id=object_values_integer.id;


CREATE VIEW eav_booleans AS
SELECT
    objects.id AS object_id,
    entity_id,
    attribute_id,
    value_id,
    name,
    value
FROM objects
JOIN attributes ON attribute_id=attributes.id
JOIN object_values_boolean ON value_id=object_values_boolean.id;


CREATE VIEW eav_timestamps AS
SELECT
    objects.id AS object_id,
    entity_id,
    attribute_id,
    value_id,
    name,
    value
FROM objects
JOIN attributes ON attribute_id=attributes.id
JOIN object_values_timestamp ON value_id=object_values_timestamp.id;


CREATE VIEW eav_arrays AS
SELECT
    objects.id AS object_id,
    entity_id,
    attribute_id,
    value_id,
    name,
    value
FROM objects
JOIN attributes ON attribute_id=attributes.id
JOIN object_values_array ON value_id=object_values_array.id;
