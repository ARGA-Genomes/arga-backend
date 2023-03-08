CREATE TABLE media (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),

    media_id bigint,
    media_type varchar(255),
    format varchar(255),
    identifier varchar(255),
    "references" varchar(255),
    created timestamp with time zone,
    creator varchar(255),
    publisher varchar(255),
    license varchar(255),
    rights_holder varchar(255),
    catalog_number bigint
);
