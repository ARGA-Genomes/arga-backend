CREATE TYPE operation_action AS ENUM (
  'create',
  'update',
  'delete'
);


CREATE TABLE operation_logs (
    operation_id numeric PRIMARY KEY NOT NULL,
    reference_id numeric NOT NULL,
    object_id varchar NOT NULL,
    action operation_action NOT NULL,
    atom jsonb DEFAULT '{}' NOT NULL
);
