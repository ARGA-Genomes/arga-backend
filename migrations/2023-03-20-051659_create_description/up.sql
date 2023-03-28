CREATE TABLE descriptions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),

    taxon_id bigint,
    type varchar(1000),
    language varchar(255),
    description text,
    source varchar(1000),
    creator varchar(3000),
    contributor varchar(255),
    license varchar(255)
);
