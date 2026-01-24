-- Create classification_schemas table for NATO classification conversions
CREATE TABLE IF NOT EXISTS classification_schemas (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    creator_id UUID NOT NULL,
        FOREIGN KEY(creator_id)
        REFERENCES users(id) ON DELETE RESTRICT,
    nation_code VARCHAR(3) NOT NULL,
    -- Conversions to NATO
    to_nato_unclassified VARCHAR(128) NOT NULL,
    to_nato_restricted VARCHAR(128) NOT NULL,
    to_nato_confidential VARCHAR(128) NOT NULL,
    to_nato_secret VARCHAR(128) NOT NULL,
    to_nato_top_secret VARCHAR(128) NOT NULL,
    -- Conversions from NATO
    from_nato_unclassified VARCHAR(128) NOT NULL,
    from_nato_restricted VARCHAR(128) NOT NULL,
    from_nato_confidential VARCHAR(128) NOT NULL,
    from_nato_secret VARCHAR(128) NOT NULL,
    from_nato_top_secret VARCHAR(128) NOT NULL,
    -- Other details
    caveats TEXT NOT NULL DEFAULT '',
    version VARCHAR(32) NOT NULL,
    nation_id UUID NOT NULL,
        FOREIGN KEY(nation_id)
        REFERENCES nations(id) ON DELETE RESTRICT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP DEFAULT NULL
);

-- Indexes for efficient querying
CREATE INDEX classification_schemas__creator_id_idx ON classification_schemas(creator_id);
CREATE INDEX classification_schemas__nation_code_idx ON classification_schemas(nation_code);
CREATE INDEX classification_schemas__nation_id_idx ON classification_schemas(nation_id);
CREATE INDEX classification_schemas__version_idx ON classification_schemas(version);
CREATE UNIQUE INDEX classification_schemas__nation_version_idx ON classification_schemas(nation_code, version);
