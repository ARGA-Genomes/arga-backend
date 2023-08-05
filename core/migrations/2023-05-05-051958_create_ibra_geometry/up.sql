CREATE TABLE ibra (
    ogc_fid integer PRIMARY KEY,
    reg_code_7 character varying,
    reg_name_7 character varying,
    hectares double precision,
    sq_km double precision,
    rec_id integer,
    reg_code_6 character varying,
    reg_name_6 character varying,
    reg_no_61 double precision,
    feat_id character varying,
    shape_leng double precision,
    shape_area double precision,
    wkb_geometry geometry(MultiPolygon,4283)
);

CREATE INDEX ibra_wkb_geometry_geom_idx ON ibra USING gist (wkb_geometry);
