CREATE TABLE indigenous_knowledge (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,

    name varchar NOT NULL,
    food_use boolean NOT NULL,
    medicinal_use boolean NOT NULL,
    cultural_connection boolean NOT NULL,

    last_updated timestamp with time zone NOT NULL,
    source_url varchar
);
