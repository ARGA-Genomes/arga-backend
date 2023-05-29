CREATE TYPE name_list_type AS ENUM ('regions', 'conservation_status');

CREATE TABLE name_lists (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    list_type name_list_type NOT NULL,
    name varchar NOT NULL,
    description text
);
