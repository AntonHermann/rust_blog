CREATE TABLE posts (
    id UUID PRIMARY KEY,
    title VARCHAR NOT NULL,
    text TEXT NOT NULL,
    author UUID NOT NULL REFERENCES users (id),
    datetime TIMESTAMP NOT NULL DEFAULT now()
)
