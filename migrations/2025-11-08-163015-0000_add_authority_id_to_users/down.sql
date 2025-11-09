-- Rollback: Remove authority_id from users table

DROP INDEX IF EXISTS users__authority_id_idx;
ALTER TABLE users DROP CONSTRAINT IF EXISTS fk_users_authority;
ALTER TABLE users DROP COLUMN IF EXISTS authority_id;
