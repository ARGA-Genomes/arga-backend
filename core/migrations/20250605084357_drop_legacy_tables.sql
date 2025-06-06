-- Drop index "nomenclatrural_acts_name" from table: "nomenclatural_acts"
DROP INDEX "public"."nomenclatrural_acts_name";
-- Drop index "nomenclatural_acts_unique_entity" from table: "nomenclatural_acts"
DROP INDEX "public"."nomenclatural_acts_unique_entity";
-- Create index "nomenclatural_acts_entity" to table: "nomenclatural_acts"
CREATE UNIQUE INDEX "nomenclatural_acts_entity" ON "public"."nomenclatural_acts" ("entity_id");
-- Drop "assembly_stats" table
DROP TABLE "public"."assembly_stats";
-- Drop "biosamples" table
DROP TABLE "public"."biosamples";
-- Drop "ecology" table
DROP TABLE "public"."ecology";
-- Drop "indigenous_knowledge" table
DROP TABLE "public"."indigenous_knowledge";
-- Drop "organisms" table
DROP TABLE "public"."organisms";
-- Drop "trace_files" table
DROP TABLE "public"."trace_files";
-- Drop "assemblies" table
DROP TABLE "public"."assemblies";
-- Drop "taxon_history" table
DROP TABLE "public"."taxon_history";
-- Drop "name_publications" table
DROP TABLE "public"."name_publications";
