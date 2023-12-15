-- Add up migration script here

CREATE TABLE users (
    userid UUID NOT NULL PRIMARY KEY DEFAULT (gen_random_uuid()),
    name     VARCHAR(32)  NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    created_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW(),
        updated_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW()
);

CREATE INDEX user_id ON users (userid);
