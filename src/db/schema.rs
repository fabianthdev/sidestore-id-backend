// @generated automatically by Diesel CLI.

diesel::table! {
    app_review_signatures (id) {
        #[max_length = 255]
        id -> Varchar,
        #[max_length = 255]
        user_id -> Varchar,
        #[max_length = 255]
        status -> Varchar,
        sequence_number -> Int4,
        #[max_length = 255]
        source_id -> Varchar,
        #[max_length = 255]
        app_bundle_id -> Varchar,
        #[max_length = 255]
        app_version -> Nullable<Varchar>,
        review_rating -> Nullable<Int4>,
        #[max_length = 255]
        signature -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    oauth_authorizations (user_id, client_id) {
        #[max_length = 255]
        user_id -> Varchar,
        #[max_length = 255]
        client_id -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        #[max_length = 255]
        id -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        username -> Nullable<Varchar>,
        #[max_length = 255]
        password_hash -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(app_review_signatures -> users (user_id));
diesel::joinable!(oauth_authorizations -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    app_review_signatures,
    oauth_authorizations,
    users,
);
