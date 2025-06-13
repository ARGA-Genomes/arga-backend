-- Create index "dataset_version_dataset_id_created_at" to table: "dataset_versions"
CREATE UNIQUE INDEX "dataset_version_dataset_id_created_at" ON "public"."dataset_versions" ("dataset_id", "created_at");
