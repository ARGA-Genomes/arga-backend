-- Modify "collection_events" table
ALTER TABLE "public"."collection_events" ADD COLUMN "material_sample_id" character varying NULL;
-- Modify "specimens" table
ALTER TABLE "public"."specimens" ADD COLUMN "specimen_id" character varying NULL;
