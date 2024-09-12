DROP TYPE data_reuse_status;
DROP TYPE access_rights_status;
DROP TYPE source_content_type;

ALTER TABLE sources DROP COLUMN reuse_pill;
ALTER TABLE sources DROP COLUMN access_pill;
ALTER TABLE sources DROP COLUMN content_type;