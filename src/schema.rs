// @generated automatically by Diesel CLI.

diesel::table! {
    package_files (id) {
        id -> Integer,
        package_version_id -> Integer,
        filename -> Text,
        size_bytes -> BigInt,
        content_type -> Nullable<Text>,
        etag -> Nullable<Text>,
        upstream_url -> Text,
        file_path -> Text,
        created_at -> Timestamp,
        last_accessed -> Timestamp,
        access_count -> Integer,
    }
}

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
    package_versions (id) {
        id -> Integer,
        package_id -> Integer,
        version -> Text,
        description -> Nullable<Text>,
        main_file -> Nullable<Text>,
        scripts -> Nullable<Text>,
        dependencies -> Nullable<Text>,
        dev_dependencies -> Nullable<Text>,
        peer_dependencies -> Nullable<Text>,
        engines -> Nullable<Text>,
        shasum -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    packages (id) {
        id -> Integer,
        name -> Text,
        description -> Nullable<Text>,
        author_id -> Nullable<Integer>,
        homepage -> Nullable<Text>,
        repository_url -> Nullable<Text>,
        license -> Nullable<Text>,
        keywords -> Nullable<Text>,
        is_private -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
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

diesel::joinable!(package_files -> package_versions (package_version_id));
diesel::joinable!(package_owners -> users (user_id));
diesel::joinable!(package_versions -> packages (package_id));
diesel::joinable!(packages -> users (author_id));
diesel::joinable!(user_tokens -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    package_files,
    package_owners,
    package_versions,
    packages,
    user_tokens,
    users,
);
