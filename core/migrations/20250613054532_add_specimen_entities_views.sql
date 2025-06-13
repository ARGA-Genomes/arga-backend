-- These views are a small cache that represent all entites in various log tables.
-- By having them in a materialized view we don't have to do a 'group by' in the log table
-- making any queries requiring a lookup to be much more efficient.

CREATE MATERIALIZED VIEW collection_event_entities AS
SELECT entity_id FROM collection_event_logs GROUP BY entity_id ORDER BY entity_id;
CREATE UNIQUE INDEX collection_event_entities_entity_id ON collection_event_entities (entity_id);

CREATE MATERIALIZED VIEW organism_entities AS
SELECT entity_id FROM organism_logs GROUP BY entity_id ORDER BY entity_id;
CREATE UNIQUE INDEX organism_entities_entity_id ON organism_entities (entity_id);

CREATE MATERIALIZED VIEW accession_event_entities AS
SELECT entity_id FROM accession_event_logs GROUP BY entity_id ORDER BY entity_id;
CREATE UNIQUE INDEX accession_event_entities_entity_id ON accession_event_entities (entity_id);
