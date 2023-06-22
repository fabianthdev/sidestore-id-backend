// @generated automatically by Diesel CLI.

diesel::table! {
    app_review_signatures (id) {
        id -> Varchar,
        user_id -> Varchar,
        status -> Varchar,
        sequence_number -> Int4,
        source_id -> Varchar,
        app_bundle_id -> Varchar,
        app_version -> Nullable<Varchar>,
        review_rating -> Nullable<Int4>,
        signature -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Varchar,
        email -> Varchar,
        username -> Nullable<Varchar>,
        password_hash -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(app_review_signatures -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    app_review_signatures,
    users,
);
