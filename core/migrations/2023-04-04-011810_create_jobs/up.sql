CREATE TYPE job_status AS ENUM ('pending', 'initialized', 'running', 'completed', 'failed', 'dead');

CREATE TABLE jobs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    status job_status NOT NULL DEFAULT 'pending',
    worker varchar(255) NOT NULL,
    payload jsonb,

    created_at timestamp with time zone NOT NULL DEFAULT current_timestamp,
    updated_at timestamp with time zone NOT NULL DEFAULT current_timestamp
);
