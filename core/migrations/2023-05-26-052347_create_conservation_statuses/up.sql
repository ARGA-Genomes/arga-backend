CREATE TABLE conservation_statuses (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    status varchar NOT NULL,
    state varchar,
    source varchar
);
