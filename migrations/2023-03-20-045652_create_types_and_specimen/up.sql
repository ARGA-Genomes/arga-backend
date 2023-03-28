CREATE TABLE types_and_specimen (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),

    taxon_id bigint,
    designation_type varchar(255),
    designated_by varchar(3000),
    scientific_name varchar(255),
    taxon_rank varchar(255),
    source varchar(255)
);
