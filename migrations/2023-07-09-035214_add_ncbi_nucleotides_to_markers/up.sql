ALTER TABLE markers
ADD COLUMN list_id uuid NOT NULL,
ADD COLUMN version varchar,
ADD COLUMN basepairs bigint,
ADD COLUMN type varchar,
ADD COLUMN shape varchar,
ADD COLUMN source_url varchar,
ADD COLUMN fasta_url varchar,
ADD COLUMN extra_data jsonb;
