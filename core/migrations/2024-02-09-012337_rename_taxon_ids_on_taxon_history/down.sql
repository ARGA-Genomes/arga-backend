ALTER TABLE taxon_history RENAME COLUMN acted_on TO old_taxon_id;
ALTER TABLE taxon_history RENAME COLUMN taxon_id TO new_taxon_id;
