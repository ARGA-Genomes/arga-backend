-- Create "library_assemblies" table
CREATE TABLE "public"."library_assemblies" (
 "library_entity_id" character varying NOT NULL,
 "assembly_entity_id" character varying NOT NULL,
 PRIMARY KEY ("library_entity_id", "assembly_entity_id"),
 CONSTRAINT "library_assemblies_assembly_entity_id_fkey" FOREIGN KEY ("assembly_entity_id") REFERENCES "public"."assemblies" ("entity_id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "library_assemblies_library_entity_id_fkey" FOREIGN KEY ("library_entity_id") REFERENCES "public"."libraries" ("entity_id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "library_assemblies_assembly_entity_id" to table: "library_assemblies"
CREATE INDEX "library_assemblies_assembly_entity_id" ON "public"."library_assemblies" ("assembly_entity_id");
-- Create index "library_assemblies_library_entity_id" to table: "library_assemblies"
CREATE INDEX "library_assemblies_library_entity_id" ON "public"."library_assemblies" ("library_entity_id");
