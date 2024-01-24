CREATE TABLE names (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    scientific_name varchar NOT NULL,
    canonical_name varchar NOT NULL,
    authorship varchar
);
COMMENT ON TABLE names IS 'All taxa names. Unique names used to associate attributes and data for specific taxonomic names';

CREATE UNIQUE INDEX names_scientific_name ON names (scientific_name);
CREATE INDEX names_canonical_name ON names (canonical_name);
