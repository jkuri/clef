use log::info;
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub upstream_registry: String,
    pub port: u16,
    pub host: String,
    pub scheme: String,
    pub cache_enabled: bool,
    pub cache_dir: String,
    pub cache_max_size_mb: u64,
    pub cache_ttl_hours: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            upstream_registry: "https://registry.npmjs.org".to_string(),
            port: 8000,
            host: "127.0.0.1".to_string(),
            scheme: "http".to_string(),
            cache_enabled: true,
            cache_dir: "./data".to_string(),
            cache_max_size_mb: 1024, // 1GB default
            cache_ttl_hours: 24, // 24 hours default
        }
    }
}

impl AppConfig {
    pub fn get_scheme(&self) -> &str {
        &self.scheme
    }

    pub fn from_env() -> Self {
        let upstream_registry = env::var("PNRS_UPSTREAM_REGISTRY")
            .unwrap_or_else(|_| "https://registry.npmjs.org".to_string());

        let port = env::var("PNRS_PORT")
            .unwrap_or_else(|_| "8000".to_string())
            .parse::<u16>()
            .unwrap_or(8000);

        let host = env::var("PNRS_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        // Auto-detect scheme based on port or explicit configuration
        let scheme = env::var("PNRS_SCHEME")
            .unwrap_or_else(|_| {
                if port == 443 {
                    "https".to_string()
                } else {
                    "http".to_string()
                }
            });

        let cache_enabled = env::var("PNRS_CACHE_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let cache_dir = env::var("PNRS_CACHE_DIR")
            .unwrap_or_else(|_| "./data".to_string());

        let cache_max_size_mb = env::var("PNRS_CACHE_MAX_SIZE_MB")
            .unwrap_or_else(|_| "1024".to_string())
            .parse::<u64>()
            .unwrap_or(1024);

        let cache_ttl_hours = env::var("PNRS_CACHE_TTL_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse::<u64>()
            .unwrap_or(24);

        info!("Configuration loaded:");
        info!("  Upstream Registry: {}", upstream_registry);
        info!("  Host: {}", host);
        info!("  Port: {}", port);
        info!("  Scheme: {}", scheme);
        info!("  Cache Enabled: {}", cache_enabled);
        info!("  Cache Directory: {}", cache_dir);
        info!("  Cache Max Size: {} MB", cache_max_size_mb);
        info!("  Cache TTL: {} hours", cache_ttl_hours);

        Self {
            upstream_registry,
            port,
            host,
            scheme,
            cache_enabled,
            cache_dir,
            cache_max_size_mb,
            cache_ttl_hours,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.upstream_registry, "https://registry.npmjs.org");
        assert_eq!(config.port, 8000);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.cache_enabled, true);
        assert_eq!(config.cache_dir, "./cache");
        assert_eq!(config.cache_max_size_mb, 1024);
        assert_eq!(config.cache_ttl_hours, 24);
    }

    #[test]
    fn test_config_parsing() {
        // Test port parsing
        assert_eq!("8080".parse::<u16>().unwrap_or(8000), 8080);
        assert_eq!("invalid".parse::<u16>().unwrap_or(8000), 8000);
    }
}
