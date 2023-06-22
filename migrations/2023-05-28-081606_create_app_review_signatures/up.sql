CREATE TABLE app_review_signatures
(
    id              VARCHAR(255)    PRIMARY KEY,
    user_id         VARCHAR(255)    NOT NULL REFERENCES users (id) ON DELETE NO ACTION,
    status          VARCHAR(255)    NOT NULL,
    sequence_number INTEGER         NOT NULL,
    source_id       VARCHAR(255)    NOT NULL,
    app_bundle_id   VARCHAR(255)    NOT NULL,
    app_version     VARCHAR(255)    ,
    review_rating   INTEGER         ,
    signature       VARCHAR(255)    ,
    created_at      TIMESTAMP       NOT NULL,
    updated_at      TIMESTAMP       NOT NULL
)