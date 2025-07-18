use crate::config::AppConfig;
use crate::services::{CacheService, DatabaseService};
use std::sync::Arc;

#[derive(Debug)]
pub struct AppState {
    pub config: AppConfig,
    pub client: reqwest::Client,
    pub cache: Arc<CacheService>,
    pub database: Arc<DatabaseService>,
}
