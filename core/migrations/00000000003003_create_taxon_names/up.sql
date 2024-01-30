CREATE TABLE taxon_names (
    taxon_id uuid REFERENCES taxa NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    PRIMARY KEY(taxon_id, name_id)
);
