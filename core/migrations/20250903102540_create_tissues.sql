-- Create "tissues" table
CREATE TABLE "public"."tissues" (
 "entity_id" character varying NOT NULL,
 "specimen_id" character varying NOT NULL,
 "material_sample_id" character varying NOT NULL,
 "identification_verified" boolean NULL,
 "reference_material" boolean NULL,
 "custodian" character varying NULL,
 "institution" character varying NULL,
 "institution_code" character varying NULL,
 "sampling_protocol" character varying NULL,
 "tissue_type" character varying NULL,
 "disposition" character varying NULL,
 "fixation" character varying NULL,
 "storage" character varying NULL,
 PRIMARY KEY ("entity_id"),
 CONSTRAINT "tissues_material_sample_id_fkey" FOREIGN KEY ("material_sample_id") REFERENCES "public"."specimens" ("entity_id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "tissues_specimen_id_fkey" FOREIGN KEY ("specimen_id") REFERENCES "public"."specimens" ("entity_id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "tissues_material_sample_id" to table: "tissues"
CREATE INDEX "tissues_material_sample_id" ON "public"."tissues" ("material_sample_id");
-- Create index "tissues_specimen_id" to table: "tissues"
CREATE INDEX "tissues_specimen_id" ON "public"."tissues" ("specimen_id");
