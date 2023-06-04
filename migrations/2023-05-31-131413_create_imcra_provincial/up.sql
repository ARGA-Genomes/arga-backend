CREATE TABLE imcra_provincial (
    ogc_fid integer PRIMARY KEY,
    pb_name varchar,
    pb_num integer,
    water_type varchar,
    area_km2 double precision,
    wkb_geometry geometry(MultiPolygon, 4283)
);

CREATE INDEX imcra_provincial_wkb_geometry_geom_idx ON imcra_provincial USING gist (wkb_geometry);
