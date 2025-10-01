-- Create "libraries" table
CREATE TABLE "public"."libraries" (
 "entity_id" character varying NOT NULL,
 "extract_id" character varying NOT NULL,
 "species_name_id" bigint NOT NULL,
 "publication_id" character varying NULL,
 "library_id" character varying NOT NULL,
 "event_date" date NULL,
 "event_time" time NULL,
 "prepared_by" character varying NULL,
 "concentration" double precision NULL,
 "concentration_unit" character varying NULL,
 "pcr_cycles" integer NULL,
 "layout" character varying NULL,
 "selection" character varying NULL,
 "bait_set_name" character varying NULL,
 "bait_set_reference" character varying NULL,
 "construction_protocol" character varying NULL,
 "source" character varying NULL,
 "insert_size" character varying NULL,
 "design_description" character varying NULL,
 "strategy" character varying NULL,
 "index_tag" character varying NULL,
 "index_dual_tag" character varying NULL,
 "index_oligo" character varying NULL,
 "index_dual_oligo" character varying NULL,
 "location" character varying NULL,
 "remarks" character varying NULL,
 "dna_treatment" character varying NULL,
 "number_of_libraries_pooled" integer NULL,
 "pcr_replicates" integer NULL,
 PRIMARY KEY ("entity_id"),
 CONSTRAINT "libraries_extract_id_fkey" FOREIGN KEY ("extract_id") REFERENCES "public"."dna_extracts" ("entity_id") ON UPDATE NO ACTION ON DELETE CASCADE,
 CONSTRAINT "libraries_prepared_by_fkey" FOREIGN KEY ("prepared_by") REFERENCES "public"."agents" ("entity_id") ON UPDATE NO ACTION ON DELETE NO ACTION,
 CONSTRAINT "libraries_publication_id_fkey" FOREIGN KEY ("publication_id") REFERENCES "public"."publications" ("entity_id") ON UPDATE NO ACTION ON DELETE NO ACTION
);
-- Create "library_logs" table
CREATE TABLE "public"."library_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "library_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "library_logs_dataset_version_id" to table: "library_logs"
CREATE INDEX "library_logs_dataset_version_id" ON "public"."library_logs" ("dataset_version_id");
-- Create index "library_logs_entity_id" to table: "library_logs"
CREATE INDEX "library_logs_entity_id" ON "public"."library_logs" ("entity_id");
-- Create index "library_logs_parent_id" to table: "library_logs"
CREATE INDEX "library_logs_parent_id" ON "public"."library_logs" ("parent_id");




-- manual materialized view creation
CREATE MATERIALIZED VIEW library_entities AS
SELECT entity_id FROM library_logs GROUP BY entity_id ORDER BY entity_id;
CREATE UNIQUE INDEX library_entities_entity_id ON library_entities (entity_id);
