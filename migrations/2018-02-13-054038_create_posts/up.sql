-- Your SQL goes here
CREATE TABLE posts (
    uuid UUID PRIMARY KEY,
    title VARCHAR NOT NULL,
    body TEXT NOT NULL,
    author VARCHAR NOT NULL,
    datetime TIMESTAMP NOT NULL
)
