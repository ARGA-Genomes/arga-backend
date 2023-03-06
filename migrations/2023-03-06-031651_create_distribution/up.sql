CREATE TABLE distribution (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),

    taxon_id bigint,
    location_id text,
    locality text,
    country varchar(255),
    country_code varchar(255),
    location_remarks text,
    establishment_means varchar(255),
    life_stage varchar(255),
    occurrence_status varchar(255),
    threat_status varchar(255),
    "source" text
);
