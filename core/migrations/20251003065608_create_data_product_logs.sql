-- Create "data_product_logs" table
CREATE TABLE "public"."data_product_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "data_product_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "data_product_logs_dataset_version_id" to table: "data_product_logs"
CREATE INDEX "data_product_logs_dataset_version_id" ON "public"."data_product_logs" ("dataset_version_id");
-- Create index "data_product_logs_entity_id" to table: "data_product_logs"
CREATE INDEX "data_product_logs_entity_id" ON "public"."data_product_logs" ("entity_id");
-- Create index "data_product_logs_parent_id" to table: "data_product_logs"
CREATE INDEX "data_product_logs_parent_id" ON "public"."data_product_logs" ("parent_id");



-- manual view creation
CREATE MATERIALIZED VIEW data_product_entities AS
SELECT entity_id FROM data_product_logs GROUP BY entity_id ORDER BY entity_id;
CREATE UNIQUE INDEX data_product_entities_entity_id ON data_product_entities (entity_id);
