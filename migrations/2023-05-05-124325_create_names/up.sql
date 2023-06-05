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