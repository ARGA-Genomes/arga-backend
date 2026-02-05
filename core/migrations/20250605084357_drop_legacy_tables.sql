-- Drop index "nomenclatrural_acts_name" from table: "nomenclatural_acts"
DROP INDEX IF EXISTS "public"."nomenclatrural_acts_name";
-- Drop index "nomenclatural_acts_unique_entity" from table: "nomenclatural_acts"
DROP INDEX IF EXISTS "public"."nomenclatural_acts_unique_entity";
-- Create index "nomenclatural_acts_entity" to table: "nomenclatural_acts"
CREATE UNIQUE INDEX "nomenclatural_acts_entity" ON "public"."nomenclatural_acts" ("entity_id");
-- Drop "assembly_stats" table
DROP TABLE IF EXISTS "public"."assembly_stats";
-- Drop "biosamples" table
DROP TABLE IF EXISTS "public"."biosamples";
-- Drop "ecology" table
DROP TABLE IF EXISTS "public"."ecology";
-- Drop "indigenous_knowledge" table
DROP TABLE IF EXISTS "public"."indigenous_knowledge";
-- Drop "organisms" table
DROP TABLE IF EXISTS "public"."organisms";
-- Drop "trace_files" table
DROP TABLE IF EXISTS "public"."trace_files";
-- Drop "assemblies" table
DROP TABLE IF EXISTS "public"."assemblies";
-- Drop "taxon_history" table
DROP TABLE IF EXISTS "public"."taxon_history";
-- Drop "name_publications" table
DROP TABLE IF EXISTS "public"."name_publications";
