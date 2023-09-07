CREATE TABLE ecology (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    values text[] NOT NULL
);
