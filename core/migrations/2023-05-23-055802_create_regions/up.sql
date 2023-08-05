CREATE TYPE region_type AS ENUM ('ibra', 'imcra');

CREATE TABLE regions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name_id uuid REFERENCES names NOT NULL,
    region_type region_type NOT NULL,
    values text[] NOT NULL
);
