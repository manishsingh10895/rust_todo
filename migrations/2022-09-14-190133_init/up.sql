-- Your SQL goes here

CREATE TABLE users (
    id UUID NOT NULL PRIMARY KEY,
    email VARCHAR(100) NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    password VARCHAR(200) NOT NULL,
    name VARCHAR(200) NOT NULL
);

CREATE TABLE todos (
    id UUID NOT NULL PRIMARY KEY,

    completed BOOLEAN NOT NULL,

    title VARCHAR(200) NOT NULL,

    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
