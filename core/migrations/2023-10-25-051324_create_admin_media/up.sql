CREATE TABLE admin_media (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name_id uuid REFERENCES names NOT NULL,
    image_source varchar NOT NULL,
    url varchar NOT NULL,
    width integer,
    height integer,
    reference_url varchar,
    title varchar,
    description varchar,
    source varchar,
    creator varchar,
    publisher varchar,
    license varchar,
    rights_holder varchar
);
