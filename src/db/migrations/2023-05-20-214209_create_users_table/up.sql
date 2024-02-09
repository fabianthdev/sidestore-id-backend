CREATE TABLE users
(
    id              VARCHAR(255)    PRIMARY KEY,
    email           VARCHAR(255)    UNIQUE NOT NULL,
    username        VARCHAR(255)    UNIQUE,
    password_hash   VARCHAR(255)    NOT NULL,
    created_at      TIMESTAMP       NOT NULL,
    updated_at      TIMESTAMP       NOT NULL
)