CREATE MATERIALIZED VIEW tissue_entities AS
SELECT entity_id FROM tissue_logs GROUP BY entity_id ORDER BY entity_id;
CREATE UNIQUE INDEX tissue_entities_entity_id ON tissue_entities (entity_id);
