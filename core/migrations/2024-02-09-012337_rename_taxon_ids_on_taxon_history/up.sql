ALTER TABLE taxon_history RENAME COLUMN old_taxon_id TO acted_on;
ALTER TABLE taxon_history RENAME COLUMN new_taxon_id TO taxon_id;
