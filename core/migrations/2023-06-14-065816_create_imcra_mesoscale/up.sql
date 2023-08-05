CREATE TABLE imcra_mesoscale (
    ogc_fid integer PRIMARY KEY,
    meso_name varchar,
    meso_num integer,
    meso_abbr varchar,
    water_type varchar,
    area_km2 double precision,
    wkb_geometry geometry(MultiPolygon, 4283)
);

CREATE INDEX imcra_mesoscale_wkb_geometry_geom_idx ON imcra_mesoscale USING gist (wkb_geometry);
