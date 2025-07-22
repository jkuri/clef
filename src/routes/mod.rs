pub mod api;
pub mod auth;
pub mod organizations;
pub mod packages;
pub mod publish;
pub mod security;
pub mod static_files;

use rocket::routes;

pub fn get_routes() -> Vec<rocket::Route> {
    let api_routes = routes![
        // API routes with /api/v1/ prefix
        api::health_check,
        api::list_packages,
        api::get_package_versions,
        api::get_popular_packages,
        api::get_cache_analytics,
        api::get_cache_stats,
        api::clear_cache,
        api::cache_health,
        api::reprocess_cache,
        api::login,
        api::register,
        // Organization routes
        organizations::create_organization,
        organizations::get_organization,
        organizations::update_organization,
        organizations::delete_organization,
        organizations::add_member,
        organizations::update_member_role,
        organizations::remove_member,
        // Registry routes (used by npm client - no prefix change)
        // Scoped package routes (higher priority)
        packages::handle_scoped_package_metadata,
        packages::handle_scoped_package_version,
        packages::handle_scoped_package_tarball,
        packages::handle_scoped_package_tarball_head,
        // Regular package routes (lower priority)
        packages::handle_regular_package_metadata,
        packages::handle_regular_package_version,
        packages::handle_regular_package_tarball,
        packages::handle_regular_package_tarball_head,
        // Catch-all route (lowest priority)
        packages::handle_package_request,
        packages::handle_package_head_request,
        // Security routes (used by npm client)
        security::security_advisories_bulk,
        security::security_audits,
        security::security_audits_quick,
        // NPM-specific auth routes (used by npm client)
        auth::npm_login,
        auth::npm_whoami,
        auth::npm_logout,
        // NPM publish routes
        publish::npm_publish_scoped,
        publish::npm_publish,
    ];

    // Add static file routes (lowest priority)
    let mut all_routes = api_routes;
    all_routes.extend(static_files::get_static_routes());
    all_routes
}
