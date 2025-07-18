pub mod auth;
pub mod health;
pub mod packages;
pub mod cache;
pub mod analytics;
pub mod security;

use rocket::routes;

pub fn get_routes() -> Vec<rocket::Route> {
    routes![
        health::health_check,
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
        cache::get_cache_stats,
        cache::clear_cache,
        cache::cache_health,
        analytics::list_packages,
        analytics::get_package_versions,
        analytics::get_popular_packages,
        analytics::get_cache_analytics,
        security::security_advisories_bulk,
        security::security_audits_quick,
        auth::npm_login,
        auth::npm_whoami,
        auth::npm_publish,
        auth::login,
        auth::register
    ]
}


