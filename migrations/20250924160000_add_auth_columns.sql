ALTER TABLE users
    ADD COLUMN password_hash TEXT NOT NULL DEFAULT '',
    ADD COLUMN role TEXT NOT NULL DEFAULT 'viewer';

ALTER TABLE users
    ADD CONSTRAINT users_role_check CHECK (role IN ('admin', 'viewer'));

ALTER TABLE users
    ALTER COLUMN password_hash DROP DEFAULT,
    ALTER COLUMN role DROP DEFAULT;
