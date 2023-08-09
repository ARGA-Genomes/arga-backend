-- Reverting an enum value requires creating a new enum type, reassign all values to the new type,
-- then deleting the old type.
-- Because we want to strictly keep the migrations as schema-only migrations we opt not to revert
-- anything here.
select 1;
