-- Create "annotation_logs" table
CREATE TABLE "public"."annotation_logs" (
 "operation_id" numeric NOT NULL,
 "parent_id" numeric NOT NULL,
 "entity_id" character varying NOT NULL,
 "dataset_version_id" uuid NOT NULL,
 "action" "public"."operation_action" NOT NULL,
 "atom" jsonb NOT NULL DEFAULT '{}',
 PRIMARY KEY ("operation_id"),
 CONSTRAINT "annotation_logs_dataset_version_id_fkey" FOREIGN KEY ("dataset_version_id") REFERENCES "public"."dataset_versions" ("id") ON UPDATE NO ACTION ON DELETE CASCADE
);
-- Create index "annotation_logs_dataset_version_id" to table: "annotation_logs"
CREATE INDEX "annotation_logs_dataset_version_id" ON "public"."annotation_logs" ("dataset_version_id");
-- Create index "annotation_logs_entity_id" to table: "annotation_logs"
CREATE INDEX "annotation_logs_entity_id" ON "public"."annotation_logs" ("entity_id");
-- Create index "annotation_logs_parent_id" to table: "annotation_logs"
CREATE INDEX "annotation_logs_parent_id" ON "public"."annotation_logs" ("parent_id");
-- Create "annotations" table
CREATE TABLE "public"."annotations" (
 "entity_id" character varying NOT NULL,
 "assembly_id" character varying NOT NULL,
 "name" character varying NULL,
 "provider" character varying NULL,
 "event_date" date NULL,
 "number_of_genes" integer NULL,
 "number_of_proteins" integer NULL,
 PRIMARY KEY ("entity_id"),
 CONSTRAINT "annotations_assembly_id_fkey" FOREIGN KEY ("assembly_id") REFERENCES "public"."assemblies" ("entity_id") ON UPDATE NO ACTION ON DELETE CASCADE
);


CREATE INDEX annotations_assembly_id ON annotations (assembly_id);


-- manually add entities view
CREATE MATERIALIZED VIEW annotation_entities AS
SELECT entity_id FROM annotation_logs GROUP BY entity_id ORDER BY entity_id;
CREATE UNIQUE INDEX annotation_entities_entity_id ON annotation_entities (entity_id);
