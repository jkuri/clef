// @generated automatically by Diesel CLI.

diesel::table! {
    cache_stats (id) {
        id -> Integer,
        hit_count -> BigInt,
        miss_count -> BigInt,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    metadata_cache (id) {
        id -> Integer,
        package_name -> Text,
        size_bytes -> BigInt,
        file_path -> Text,
        etag -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        last_accessed -> Timestamp,
        access_count -> Integer,
    }
}

diesel::table! {
    organization_members (id) {
        id -> Integer,
        user_id -> Integer,
        organization_id -> Integer,
        role -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    organizations (id) {
        id -> Integer,
        name -> Text,
        display_name -> Nullable<Text>,
        description -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

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
    package_tags (id) {
        id -> Integer,
        package_name -> Text,
        tag_name -> Text,
        version -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
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
        readme -> Nullable<Text>,
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
        created_at -> Timestamp,
        updated_at -> Timestamp,
        organization_id -> Nullable<Integer>,
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

diesel::joinable!(organization_members -> organizations (organization_id));
diesel::joinable!(organization_members -> users (user_id));
diesel::joinable!(package_files -> package_versions (package_version_id));
diesel::joinable!(package_owners -> users (user_id));
diesel::joinable!(package_versions -> packages (package_id));
diesel::joinable!(packages -> organizations (organization_id));
diesel::joinable!(packages -> users (author_id));
diesel::joinable!(user_tokens -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    cache_stats,
    metadata_cache,
    organization_members,
    organizations,
    package_files,
    package_owners,
    package_tags,
    package_versions,
    packages,
    user_tokens,
    users,
);
