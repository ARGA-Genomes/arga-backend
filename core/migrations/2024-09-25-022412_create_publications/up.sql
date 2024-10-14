CREATE TYPE publication_type AS ENUM (
  'book',
  'book_chapter',
  'journal_article',
  'journal_volume',
  'proceedings_paper',
  'url'
);

CREATE TABLE publications (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id varchar NOT NULL,

    title varchar NOT NULL,
    authors text[] NOT NULL,
    published_year int NOT NULL,
    published_date timestamp with time zone,
    language varchar,
    publisher varchar,
    doi varchar,
    source_urls text[],
    publication_type publication_type,
    citation varchar,

    -- these are timestamps from the dataset, not our own timestamps
    record_created_at timestamp with time zone,
    record_updated_at timestamp with time zone,

    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL
);

-- each entity is a globally unique publication so we ensure that there is only one
-- record that can be reduced from the publication logs
CREATE UNIQUE INDEX publications_entity_id ON publications (entity_id);
