-- Create "tissue_logs" table
CREATE TABLE "public"."tissue_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "tissue_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "tissue_logs_dataset_version_id" to table: "tissue_logs"
CREATE INDEX "tissue_logs_dataset_version_id" ON "public"."tissue_logs" ("dataset_version_id");
-- Create index "tissue_logs_entity_id" to table: "tissue_logs"
CREATE INDEX "tissue_logs_entity_id" ON "public"."tissue_logs" ("entity_id");
-- Create index "tissue_logs_parent_id" to table: "tissue_logs"
CREATE INDEX "tissue_logs_parent_id" ON "public"."tissue_logs" ("parent_id");
