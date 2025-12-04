-- Create "deposition_logs" table
CREATE TABLE "public"."deposition_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "deposition_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "deposition_logs_dataset_version_id" to table: "deposition_logs"
CREATE INDEX "deposition_logs_dataset_version_id" ON "public"."deposition_logs" ("dataset_version_id");
-- Create index "deposition_logs_entity_id" to table: "deposition_logs"
CREATE INDEX "deposition_logs_entity_id" ON "public"."deposition_logs" ("entity_id");
-- Create index "deposition_logs_parent_id" to table: "deposition_logs"
CREATE INDEX "deposition_logs_parent_id" ON "public"."deposition_logs" ("parent_id");
-- Create "depositions" table
CREATE TABLE "public"."depositions" (
 "entity_id" character varying NOT NULL,
 "assembly_id" character varying NOT NULL,
 "event_date" date NULL,
 "url" character varying NULL,
 "institution" character varying NULL,
 PRIMARY KEY ("entity_id"),
 CONSTRAINT "depositions_assembly_id_fkey" FOREIGN KEY ("assembly_id") REFERENCES "public"."assemblies" ("entity_id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "depositions_assembly_id" to table: "depositions"
CREATE INDEX "depositions_assembly_id" ON "public"."depositions" ("assembly_id");



-- manually create entity materialised view
CREATE MATERIALIZED VIEW deposition_entities AS
SELECT entity_id FROM deposition_logs GROUP BY entity_id ORDER BY entity_id;
CREATE UNIQUE INDEX deposition_entities_entity_id ON deposition_entities (entity_id);
