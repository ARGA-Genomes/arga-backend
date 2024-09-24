ALTER TABLE sources DROP COLUMN reuse_pill;
ALTER TABLE sources DROP COLUMN access_pill;
ALTER TABLE sources DROP COLUMN content_type;

DROP TYPE IF EXISTS data_reuse_status;
DROP TYPE IF EXISTS access_rights_status;
DROP TYPE IF EXISTS source_content_type;