CREATE TABLE users (
    id UUID PRIMARY KEY,
    name VARCHAR NOT NULL UNIQUE,
    privileges SMALLINT NOT NULL DEFAULT 1,
    password VARCHAR NOT NULL
)