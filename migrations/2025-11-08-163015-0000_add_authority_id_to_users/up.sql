-- Add authority_id column to users table
-- Users belong to one Authority (which belongs to one Nation)
-- Only ADMIN users can exist without an authority

ALTER TABLE users
ADD COLUMN authority_id UUID;

-- Add foreign key constraint (NULLABLE to allow admin without authority)
ALTER TABLE users
ADD CONSTRAINT fk_users_authority
FOREIGN KEY (authority_id)
REFERENCES authorities(id)
ON DELETE RESTRICT;

-- Add index for performance (users are frequently queried by authority)
CREATE INDEX users__authority_id_idx ON users(authority_id);

-- Add comment explaining the business logic
COMMENT ON COLUMN users.authority_id IS 'Foreign key to authorities table. Required for all non-ADMIN users. Users belong to one Authority, which belongs to one Nation, establishing the user->authority->nation hierarchy.';
