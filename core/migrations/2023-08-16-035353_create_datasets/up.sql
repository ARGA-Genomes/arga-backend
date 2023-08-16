CREATE TABLE datasets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),

    name varchar NOT NULL,
    description text,
    url varchar
);
