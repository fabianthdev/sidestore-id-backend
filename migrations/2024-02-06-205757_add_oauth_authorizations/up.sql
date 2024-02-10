CREATE TABLE oauth_authorizations
(
    user_id    VARCHAR(255) NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    client_id  VARCHAR(255) NOT NULL,

    created_at TIMESTAMP    NOT NULL,
    updated_at TIMESTAMP    NOT NULL,

    PRIMARY KEY (user_id, client_id)
)