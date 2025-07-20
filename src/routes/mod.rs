pub mod api;
pub mod auth;
pub mod packages;
pub mod security;

use rocket::routes;

pub fn get_routes() -> Vec<rocket::Route> {
    routes![
        // API routes with /api/v1/ prefix
        api::health_check,
        api::list_packages,
        api::get_package_versions,
        api::get_popular_packages,
        api::get_cache_analytics,
        api::get_cache_stats,
        api::clear_cache,
        api::cache_health,
        api::login,
        api::register,
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
        auth::npm_publish,
    ]
}
