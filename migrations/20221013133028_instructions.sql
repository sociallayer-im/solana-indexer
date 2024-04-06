CREATE TABLE IF NOT EXISTS instructions (
    id VARCHAR PRIMARY KEY,
    tx_hash VARCHAR NOT NULL,
    program_id VARCHAR NOT NULL,
    blocktime BIGINT NOT NULL,
    data VARCHAR NOT NULL
);