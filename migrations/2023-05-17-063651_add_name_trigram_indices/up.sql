CREATE INDEX vernacular_names_trgrm ON vernacular_names USING GIST (vernacular_name gist_trgm_ops);
CREATE INDEX names_trgrm ON names USING GIST (canonical_name gist_trgm_ops);
