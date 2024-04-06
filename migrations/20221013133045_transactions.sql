DO $$ 
BEGIN CREATE TYPE tx_status AS ENUM ('pending', 'indexed');
EXCEPTION
WHEN duplicate_object THEN null;
END $$;

CREATE TABLE IF NOT EXISTS transactions (
    id SERIAL,
    hash VARCHAR NOT NULL PRIMARY KEY,
    blocktime BIGINT NOT NULL,
    indexing_status tx_status NOT NULL,
    indexing_timestamp BIGINT NOT NULL
);