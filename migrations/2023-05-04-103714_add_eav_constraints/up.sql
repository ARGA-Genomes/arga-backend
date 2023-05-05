ALTER TABLE objects ADD FOREIGN KEY (entity_id) REFERENCES user_taxa;
ALTER TABLE objects ADD FOREIGN KEY (attribute_id) REFERENCES attributes;
