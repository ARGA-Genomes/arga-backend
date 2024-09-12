CREATE TYPE data_reuse_status as ENUM (
    'limited',
    'unlimited',
    'none',
    'variable'
);

CREATE TYPE access_rights_status as ENUM (
    'open',
    'restricted',
    'conditional',
    'variable'
);

CREATE TYPE source_content_type as ENUM (
    'Taxonomic Backbone',
    'Ecological Traits',
    'Genomic Data',
    'Specimens',
    'Non-genomic Data',
    'Morphological Traits',
    'Biochemical Traits',
    'Mixed datatypes',
    'Functional Traits',
    'Ethnobiology'
);

ALTER TABLE sources ADD COLUMN reuse_pill data_reuse_status;
ALTER TABLE sources ADD COLUMN access_pill access_rights_status;
ALTER TABLE sources ADD COLUMN content_type source_content_type;