CREATE TABLE conservation_statuses (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    list_id uuid REFERENCES name_lists NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    status varchar NOT NULL,
    state varchar,
    source varchar
);
