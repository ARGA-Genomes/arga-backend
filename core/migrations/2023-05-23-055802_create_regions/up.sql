CREATE TYPE region_type AS ENUM ('ibra', 'imcra', 'state', 'drainage_basin');

CREATE TABLE regions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    dataset_id uuid REFERENCES datasets ON DELETE CASCADE NOT NULL,
    name_id uuid REFERENCES names NOT NULL,
    region_type region_type NOT NULL,
    values text[] NOT NULL
);
