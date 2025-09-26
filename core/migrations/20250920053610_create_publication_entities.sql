CREATE MATERIALIZED VIEW publication_entities AS
SELECT entity_id FROM publication_logs GROUP BY entity_id ORDER BY entity_id;
CREATE UNIQUE INDEX publication_entities_entity_id ON publication_entities (entity_id);
