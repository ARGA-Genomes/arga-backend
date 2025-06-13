-- Create "organisms" table
CREATE TABLE "public"."organisms" (
 "entity_id" character varying NOT NULL,
 "name_id" uuid NOT NULL,
 "organism_id" character varying NOT NULL,
 "sex" character varying NULL,
 "genotypic_sex" character varying NULL,
 "phenotypic_sex" character varying NULL,
 "life_stage" character varying NULL,
 "reproductive_condition" character varying NULL,
 "behavior" character varying NULL,
 PRIMARY KEY ("entity_id"),
 CONSTRAINT "organisms_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);

-- Drop index "specimens_name_id" from table: "specimens"
DROP INDEX "public"."specimens_name_id";

-- Modify "subsamples" table by removing the specimen_id constraint so we can change the specimens primary key
ALTER TABLE "public"."subsamples" DROP CONSTRAINT "subsamples_specimen_id_fkey";

-- Drop foreign key constraints as they require the referenced key to be unique and we can't drop the
-- primary key constraint unless all foreign keys are also dropped
ALTER TABLE "public"."collection_events" DROP CONSTRAINT "collection_events_specimen_id_fkey";
ALTER TABLE "public"."accession_events" DROP CONSTRAINT "accession_events_specimen_id_fkey";

-- Drop views that depend on the specimen id
DROP MATERIALIZED VIEW overview;
DROP MATERIALIZED VIEW specimen_stats;
DROP VIEW genomic_components;
DROP VIEW markers;
DROP VIEW whole_genomes;

-- Modify "specimens" table
ALTER TABLE "public"."specimens"
DROP CONSTRAINT "specimens_pkey",
DROP CONSTRAINT "specimens_name_id_fkey",
DROP COLUMN "id",
DROP COLUMN "dataset_id",
DROP COLUMN "record_id",
DROP COLUMN "material_sample_id",
ALTER COLUMN "organism_id" SET NOT NULL,
DROP COLUMN "institution_name",
DROP COLUMN "institution_code",
DROP COLUMN "collection_code",
DROP COLUMN "recorded_by",
DROP COLUMN "identified_by",
DROP COLUMN "identified_date",
DROP COLUMN "type_status",
DROP COLUMN "locality",
DROP COLUMN "country",
DROP COLUMN "country_code",
DROP COLUMN "state_province",
DROP COLUMN "county",
DROP COLUMN "municipality",
DROP COLUMN "latitude",
DROP COLUMN "longitude",
DROP COLUMN "elevation",
DROP COLUMN "depth",
DROP COLUMN "elevation_accuracy",
DROP COLUMN "depth_accuracy",
DROP COLUMN "location_source",
DROP COLUMN "details",
DROP COLUMN "remarks",
DROP COLUMN "identification_remarks",
ALTER COLUMN "entity_id" SET NOT NULL,
ADD PRIMARY KEY ("entity_id"),
ADD CONSTRAINT "specimens_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
ADD CONSTRAINT "specimens_organism_id_fkey" FOREIGN KEY ("organism_id") REFERENCES "public"."organisms" ("entity_id") ON UPDATE NO ACTION ON DELETE CASCADE;

-- Modify "accession_events" table
ALTER TABLE "public"."accession_events"
DROP CONSTRAINT "accession_events_pkey",
DROP COLUMN "id",
DROP COLUMN "dataset_id",
ALTER COLUMN "specimen_id" TYPE character varying,
ALTER COLUMN "event_date" TYPE date USING ("event_date"::text::date),
ALTER COLUMN "event_time" TYPE time USING ("event_time"::text::time),
DROP COLUMN "accession",
DROP COLUMN "material_sample_id",
ALTER COLUMN "entity_id" SET NOT NULL,
ADD COLUMN "name_id" uuid NOT NULL,
ADD COLUMN "collection_repository_id" character varying NULL,
ADD COLUMN "collection_repository_code" character varying NULL,
ADD COLUMN "disposition" character varying NULL,
ADD COLUMN "preparation" character varying NULL,
ADD COLUMN "prepared_by" character varying NULL,
ADD COLUMN "identified_by" character varying NULL,
ADD COLUMN "identified_date" date NULL,
ADD COLUMN "identification_remarks" character varying NULL,
ADD COLUMN "other_catalog_numbers" character varying NULL,
ADD PRIMARY KEY ("entity_id"),
ADD
CONSTRAINT "accession_events_specimen_id_fkey" FOREIGN KEY ("specimen_id") REFERENCES "public"."specimens" ("entity_id") ON UPDATE NO ACTION ON DELETE CASCADE, ADD
CONSTRAINT "accession_events_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE CASCADE;

-- Create index "accession_events_name_id" to table: "accession_events"
CREATE INDEX "accession_events_name_id" ON "public"."accession_events" ("name_id");

-- Modify "collection_events" table
ALTER TABLE "public"."collection_events"
DROP CONSTRAINT "collection_events_pkey",
DROP COLUMN "id",
DROP COLUMN "dataset_id",
ALTER COLUMN "specimen_id" TYPE character varying,
ALTER COLUMN "event_date" TYPE date USING ("event_date"::text::date),
ALTER COLUMN "event_time" TYPE time USING ("event_time"::text::time),
DROP COLUMN "field_number",
DROP COLUMN "catalog_number",
DROP COLUMN "record_number",
DROP COLUMN "sex",
DROP COLUMN "genotypic_sex",
DROP COLUMN "phenotypic_sex",
DROP COLUMN "life_stage",
DROP COLUMN "reproductive_condition",
DROP COLUMN "behavior",
DROP COLUMN "establishment_means",
DROP COLUMN "degree_of_establishment",
DROP COLUMN "pathway",
DROP COLUMN "occurrence_status",
DROP COLUMN "other_catalog_numbers",
DROP COLUMN "env_broad_scale",
DROP COLUMN "env_local_scale",
DROP COLUMN "env_medium",
DROP COLUMN "ref_biomaterial",
DROP COLUMN "source_mat_id",
DROP COLUMN "remarks",
ALTER COLUMN "entity_id" SET NOT NULL,
ADD COLUMN "name_id" uuid NOT NULL,
ADD COLUMN "organism_id" character varying NOT NULL,
ADD COLUMN "field_collecting_id" character varying NULL,
ADD COLUMN "collection_remarks" character varying NULL,
ADD COLUMN "identified_by" character varying NULL,
ADD COLUMN "identified_date" date NULL,
ADD COLUMN "identification_remarks" character varying NULL,
ADD COLUMN "locality" character varying NULL,
ADD COLUMN "country" character varying NULL,
ADD COLUMN "country_code" character varying NULL,
ADD COLUMN "state_province" character varying NULL,
ADD COLUMN "county" character varying NULL,
ADD COLUMN "municipality" character varying NULL,
ADD COLUMN "latitude" double precision NULL,
ADD COLUMN "longitude" double precision NULL,
ADD COLUMN "elevation" double precision NULL,
ADD COLUMN "depth" double precision NULL,
ADD COLUMN "elevation_accuracy" double precision NULL,
ADD COLUMN "depth_accuracy" double precision NULL,
ADD COLUMN "location_source" character varying NULL,
ADD COLUMN "environment_broad_scale" character varying NULL,
ADD COLUMN "environment_local_scale" character varying NULL,
ADD COLUMN "environment_medium" character varying NULL,
ADD PRIMARY KEY ("entity_id"),
ADD CONSTRAINT "collection_events_specimen_id_fkey" FOREIGN KEY ("specimen_id") REFERENCES "public"."specimens" ("entity_id") ON UPDATE NO ACTION ON DELETE CASCADE,
ADD CONSTRAINT "collection_events_name_id_fkey" FOREIGN KEY ("name_id") REFERENCES "public"."names" ("id") ON UPDATE NO ACTION ON DELETE CASCADE,
ADD CONSTRAINT "collection_events_organism_id_fkey" FOREIGN KEY ("organism_id") REFERENCES "public"."organisms" ("entity_id") ON UPDATE NO ACTION ON DELETE CASCADE;

-- Create index "collection_events_field_collecting_id" to table: "collection_events"
CREATE INDEX "collection_events_field_collecting_id" ON "public"."collection_events" ("field_collecting_id");

-- Create index "collection_events_name_id" to table: "collection_events"
CREATE INDEX "collection_events_name_id" ON "public"."collection_events" ("name_id");

-- Create index "collection_events_organism_id" to table: "collection_events"
CREATE INDEX "collection_events_organism_id" ON "public"."collection_events" ("organism_id");

-- Modify "subsamples" table by adding specimen constraint back
ALTER TABLE "public"."subsamples" ALTER COLUMN "specimen_id" TYPE character varying,
ADD CONSTRAINT "subsamples_specimen_id_fkey" FOREIGN KEY ("specimen_id") REFERENCES "public"."specimens" ("entity_id") ON UPDATE NO ACTION ON DELETE CASCADE;
