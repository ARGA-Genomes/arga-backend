-- Create "project_logs" table
CREATE TABLE "public"."project_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "project_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "project_logs_dataset_version_id" to table: "project_logs"
CREATE INDEX "project_logs_dataset_version_id" ON "public"."project_logs" ("dataset_version_id");
-- Create index "project_logs_entity_id" to table: "project_logs"
CREATE INDEX "project_logs_entity_id" ON "public"."project_logs" ("entity_id");
-- Create index "project_logs_parent_id" to table: "project_logs"
CREATE INDEX "project_logs_parent_id" ON "public"."project_logs" ("parent_id");



-- manually add entities view
CREATE MATERIALIZED VIEW project_entities AS
SELECT entity_id FROM project_logs GROUP BY entity_id ORDER BY entity_id;
CREATE UNIQUE INDEX project_entities_entity_id ON project_entities (entity_id);
