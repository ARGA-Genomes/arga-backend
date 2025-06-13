-- Create "accession_event_logs" table
CREATE TABLE "public"."accession_event_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "accession_event_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "accession_event_logs_dataset_version_id" to table: "accession_event_logs"
CREATE INDEX "accession_event_logs_dataset_version_id" ON "public"."accession_event_logs" ("dataset_version_id");
-- Create index "accession_event_logs_entity_id" to table: "accession_event_logs"
CREATE INDEX "accession_event_logs_entity_id" ON "public"."accession_event_logs" ("entity_id");
-- Create index "accession_event_logs_parent_id" to table: "accession_event_logs"
CREATE INDEX "accession_event_logs_parent_id" ON "public"."accession_event_logs" ("parent_id");
-- Create "organism_logs" table
CREATE TABLE "public"."organism_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "organism_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "organism_logs_dataset_version_id" to table: "organism_logs"
CREATE INDEX "organism_logs_dataset_version_id" ON "public"."organism_logs" ("dataset_version_id");
-- Create index "organism_logs_entity_id" to table: "organism_logs"
CREATE INDEX "organism_logs_entity_id" ON "public"."organism_logs" ("entity_id");
-- Create index "organism_logs_parent_id" to table: "organism_logs"
CREATE INDEX "organism_logs_parent_id" ON "public"."organism_logs" ("parent_id");
