CREATE TABLE nomenclatural_acts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name varchar NOT NULL,
    source_url varchar,
    citation varchar,
    example varchar
);

CREATE UNIQUE INDEX nomenclatural_acts_unique_name ON nomenclatural_acts (name);
