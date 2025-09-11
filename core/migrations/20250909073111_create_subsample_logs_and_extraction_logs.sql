-- Create "extraction_logs" table
CREATE TABLE "public"."extraction_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "extraction_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "extraction_logs_dataset_version_id" to table: "extraction_logs"
CREATE INDEX "extraction_logs_dataset_version_id" ON "public"."extraction_logs" ("dataset_version_id");
-- Create index "extraction_logs_entity_id" to table: "extraction_logs"
CREATE INDEX "extraction_logs_entity_id" ON "public"."extraction_logs" ("entity_id");
-- Create index "extraction_logs_parent_id" to table: "extraction_logs"
CREATE INDEX "extraction_logs_parent_id" ON "public"."extraction_logs" ("parent_id");
-- Create "subsample_logs" table
CREATE TABLE "public"."subsample_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "subsample_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "subsample_logs_dataset_version_id" to table: "subsample_logs"
CREATE INDEX "subsample_logs_dataset_version_id" ON "public"."subsample_logs" ("dataset_version_id");
-- Create index "subsample_logs_entity_id" to table: "subsample_logs"
CREATE INDEX "subsample_logs_entity_id" ON "public"."subsample_logs" ("entity_id");
-- Create index "subsample_logs_parent_id" to table: "subsample_logs"
CREATE INDEX "subsample_logs_parent_id" ON "public"."subsample_logs" ("parent_id");


-- Added manually
CREATE MATERIALIZED VIEW subsample_entities AS
SELECT entity_id FROM subsample_logs GROUP BY entity_id ORDER BY entity_id;
CREATE UNIQUE INDEX subsample_entities_entity_id ON subsample_entities (entity_id);

CREATE MATERIALIZED VIEW extraction_entities AS
SELECT entity_id FROM extraction_logs GROUP BY entity_id ORDER BY entity_id;
CREATE UNIQUE INDEX extraction_entities_entity_id ON extraction_entities (entity_id);
