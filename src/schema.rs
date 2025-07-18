// @generated automatically by Diesel CLI.

diesel::table! {
    package_owners (id) {
        id -> Integer,
        package_name -> Text,
        user_id -> Integer,
        permission_level -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    packages (id) {
        id -> Integer,
        name -> Text,
        version -> Text,
        filename -> Text,
        size_bytes -> BigInt,
        etag -> Nullable<Text>,
        content_type -> Nullable<Text>,
        upstream_url -> Text,
        file_path -> Text,
        created_at -> Timestamp,
        last_accessed -> Timestamp,
        access_count -> Integer,
        author_id -> Nullable<Integer>,
        description -> Nullable<Text>,
        package_json -> Nullable<Text>,
        is_private -> Bool,
    }
}

diesel::table! {
    published_packages (id) {
        id -> Integer,
        name -> Text,
        version -> Text,
        description -> Nullable<Text>,
        author_id -> Integer,
        tarball_path -> Text,
        package_json -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        is_active -> Bool,
    }
}

diesel::table! {
    user_tokens (id) {
        id -> Integer,
        user_id -> Integer,
        token -> Text,
        token_type -> Text,
        created_at -> Timestamp,
        expires_at -> Nullable<Timestamp>,
        is_active -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        email -> Text,
        password_hash -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        is_active -> Bool,
    }
}

diesel::joinable!(package_owners -> users (user_id));
diesel::joinable!(published_packages -> users (author_id));
diesel::joinable!(user_tokens -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    package_owners,
    packages,
    published_packages,
    user_tokens,
    users,
);
