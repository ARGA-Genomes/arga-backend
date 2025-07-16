-- Modify "users" table
ALTER TABLE "public"."users" ALTER COLUMN "name" TYPE character varying, ALTER COLUMN "email" TYPE character varying, ALTER COLUMN "user_role" TYPE character varying, ALTER COLUMN "password_hash" TYPE character varying, DROP COLUMN "password_salt", ADD COLUMN "session_id" character varying NULL;
