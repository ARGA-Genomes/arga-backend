DROP INDEX nomenclatural_acts_name;
CREATE UNIQUE INDEX nomenclatural_acts_unique_name ON nomenclatural_acts (name_id);
