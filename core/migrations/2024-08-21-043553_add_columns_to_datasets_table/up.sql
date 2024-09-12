ALTER TABLE datasets ADD COLUMN reuse_pill data_reuse_status;
ALTER TABLE datasets ADD COLUMN access_pill access_rights_status;
ALTER TABLE datasets ADD COLUMN publication_year int;
ALTER TABLE datasets ADD COLUMN content_type source_content_type;