-- Create "agent_logs" table
CREATE TABLE "public"."agent_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "agent_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "agent_logs_dataset_version_id" to table: "agent_logs"
CREATE INDEX "agent_logs_dataset_version_id" ON "public"."agent_logs" ("dataset_version_id");
-- Create index "agent_logs_entity_id" to table: "agent_logs"
CREATE INDEX "agent_logs_entity_id" ON "public"."agent_logs" ("entity_id");
-- Create index "agent_logs_parent_id" to table: "agent_logs"
CREATE INDEX "agent_logs_parent_id" ON "public"."agent_logs" ("parent_id");


-- manually added agent view
CREATE MATERIALIZED VIEW agent_entities AS
SELECT entity_id FROM agent_logs GROUP BY entity_id ORDER BY entity_id;
CREATE UNIQUE INDEX agent_entities_entity_id ON agent_entities (entity_id);
