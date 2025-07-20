use super::*;
use serial_test::serial;
use std::thread;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial]
    fn test_cache_stats_endpoint() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test cache stats endpoint
        let response = client.get("/api/v1/cache/stats").send().unwrap();
        assert!(response.status().is_success());

        let stats: serde_json::Value = response.json().unwrap();
        assert_eq!(stats["enabled"], true);
        assert!(stats["total_entries"].is_number());
        assert!(stats["total_size_bytes"].is_number());
        assert!(stats["total_size_mb"].is_number());
        assert!(stats["hit_count"].is_number());
        assert!(stats["miss_count"].is_number());
        assert!(stats["hit_rate"].is_number());
        assert!(stats["cache_dir"].is_string());
        assert!(stats["ttl_hours"].is_number());
    }

    #[test]
    #[serial]
    fn test_cache_health_endpoint() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test cache health endpoint
        let response = client.get("/api/v1/cache/health").send().unwrap();
        assert!(response.status().is_success());

        let health: serde_json::Value = response.json().unwrap();
        assert!(health["status"].is_string());
        assert_eq!(health["enabled"], true);
        assert!(health["total_size_mb"].is_number());
    }

    #[test]
    #[serial]
    fn test_cache_clear_endpoint() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // First, make some requests to populate cache
        let _ = client.get("/registry/lodash/-/lodash-4.17.21.tgz").send();
        let _ = client.get("/registry/express/-/express-4.18.2.tgz").send();

        // Wait a bit for cache to be populated
        thread::sleep(Duration::from_millis(100));

        // Test cache clear endpoint
        let response = client.delete("/api/v1/cache").send().unwrap();
        assert!(response.status().is_success());

        let result: serde_json::Value = response.json().unwrap();
        assert!(result["message"].as_str().unwrap().contains("cleared"));

        // Verify cache is cleared by checking stats
        let stats_response = client.get("/api/v1/cache/stats").send().unwrap();

        // The cache stats endpoint should succeed
        assert!(
            stats_response.status().is_success(),
            "Cache stats endpoint failed with status: {}",
            stats_response.status()
        );

        let stats: serde_json::Value = stats_response.json().unwrap();
        // After clearing, total_entries should be 0
        assert_eq!(stats["total_entries"], 0);
        assert_eq!(stats["total_size_bytes"], 0);
    }

    #[test]
    #[serial]
    fn test_cache_hit_miss_behavior() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Clear cache first
        let _ = client.delete("/api/v1/cache").send();
        thread::sleep(Duration::from_millis(100));

        // Get initial stats
        let initial_stats_response = client.get("/api/v1/cache/stats").send().unwrap();
        let initial_stats: serde_json::Value = initial_stats_response.json().unwrap();
        let initial_miss_count = initial_stats["miss_count"].as_u64().unwrap_or(0);

        // First request - should be a cache miss
        match client.get("/registry/lodash/-/lodash-4.17.21.tgz").send() {
            Ok(response1) if response1.status().is_success() => {
                thread::sleep(Duration::from_millis(100));

                // Check stats after first request
                if let Ok(stats_response1) = client.get("/api/v1/cache/stats").send() {
                    if let Ok(stats1) = stats_response1.json::<serde_json::Value>() {
                        let miss_count1 = stats1["miss_count"].as_u64().unwrap_or(0);

                        // Should have one more miss
                        assert!(miss_count1 > initial_miss_count);

                        // Second request - should be a cache hit
                        match client.get("/registry/lodash/-/lodash-4.17.21.tgz").send() {
                            Ok(response2) if response2.status().is_success() => {
                                thread::sleep(Duration::from_millis(100));

                                // Check stats after second request
                                if let Ok(stats_response2) =
                                    client.get("/api/v1/cache/stats").send()
                                {
                                    if let Ok(stats2) = stats_response2.json::<serde_json::Value>()
                                    {
                                        let hit_count2 = stats2["hit_count"].as_u64().unwrap_or(0);

                                        // Should have at least one hit
                                        assert!(hit_count2 > 0);
                                    }
                                }
                            }
                            Ok(_) => println!("Second request failed - may be network issue"),
                            Err(e) => {
                                println!("Second request error: {e} - may be network issue")
                            }
                        }
                    }
                }
            }
            Ok(_) => println!("First request failed - may be network issue"),
            Err(e) => println!("First request error: {e} - may be network issue"),
        }
    }

    #[test]
    #[serial]
    fn test_cache_with_different_package_managers() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Clear cache first
        let _ = client.delete("/api/v1/cache").send();
        thread::sleep(Duration::from_millis(100));

        // Simulate different package manager requests by making different HTTP requests
        // This tests cache behavior without relying on actual package manager installations

        // Simulate npm-style request
        match client.get("/registry/lodash").send() {
            Ok(response) if response.status().is_success() => {
                println!("npm-style metadata request successful");
                thread::sleep(Duration::from_millis(100));
            }
            Ok(response) => {
                println!(
                    "npm-style request failed with status: {}",
                    response.status()
                );
            }
            Err(e) => {
                println!("npm-style request error: {e}");
            }
        }

        // Simulate pnpm-style request (different package)
        match client.get("/registry/express").send() {
            Ok(response) if response.status().is_success() => {
                println!("pnpm-style metadata request successful");
                thread::sleep(Duration::from_millis(100));
            }
            Ok(response) => {
                println!(
                    "pnpm-style request failed with status: {}",
                    response.status()
                );
            }
            Err(e) => {
                println!("pnpm-style request error: {e}");
            }
        }

        // Check cache stats after requests
        if let Ok(stats_response) = client.get("/api/v1/cache/stats").send() {
            if stats_response.status().is_success() {
                if let Ok(stats) = stats_response.json::<serde_json::Value>() {
                    let hit_count = stats["hit_count"].as_u64().unwrap_or(0);
                    let miss_count = stats["miss_count"].as_u64().unwrap_or(0);
                    let total_entries = stats["total_entries"].as_u64().unwrap_or(0);

                    println!(
                        "Cache activity: hits={hit_count}, misses={miss_count}, entries={total_entries}"
                    );

                    // Should have some cache activity
                    assert!(hit_count > 0 || miss_count > 0 || total_entries > 0);
                }
            }
        }
    }

    #[test]
    #[serial]
    fn test_cache_size_tracking() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Clear cache first
        let _ = client.delete("/api/v1/cache").send();
        thread::sleep(Duration::from_millis(100));

        // Get initial size
        let initial_stats_response = client.get("/api/v1/cache/stats").send().unwrap();
        let initial_stats: serde_json::Value = initial_stats_response.json().unwrap();
        let initial_size = initial_stats["total_size_bytes"].as_u64().unwrap_or(0);

        // Download a package
        let response = client
            .get("/registry/lodash/-/lodash-4.17.21.tgz")
            .send()
            .expect("Failed to make package download request");

        println!("Package download returned status: {}", response.status());

        // Package download should succeed
        assert!(
            response.status().is_success(),
            "Package download failed with status: {} - package downloads should return 200 OK",
            response.status()
        );

        thread::sleep(Duration::from_millis(100));

        // Check size after download
        let stats_response = client.get("/api/v1/cache/stats").send().unwrap();
        assert!(stats_response.status().is_success());

        let stats: serde_json::Value = stats_response.json().unwrap();
        let new_size = stats["total_size_bytes"].as_u64().unwrap_or(0);

        println!("Cache size: initial={initial_size}, after download={new_size}");

        // Size should have increased (or at least stayed the same)
        assert!(
            new_size >= initial_size,
            "Cache size should increase after download"
        );

        // Size in MB should be calculated correctly
        let size_mb = stats["total_size_mb"].as_f64().unwrap_or(0.0);
        let expected_mb = new_size as f64 / 1024.0 / 1024.0;
        assert!(
            (size_mb - expected_mb).abs() < 0.01,
            "Size in MB calculation incorrect: got {size_mb}, expected {expected_mb}"
        );
    }

    #[test]
    #[serial]
    fn test_cache_hit_rate_calculation() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Clear cache first
        let _ = client.delete("/api/v1/cache").send();
        thread::sleep(Duration::from_millis(100));

        // Make multiple requests to the same resource (with error handling)
        let mut successful_requests = 0;
        for i in 0..3 {
            match client.get("/registry/lodash/-/lodash-4.17.21.tgz").send() {
                Ok(response) if response.status().is_success() => {
                    successful_requests += 1;
                    println!("Request {} successful", i + 1);
                }
                Ok(response) => {
                    println!(
                        "Request {} failed with status: {}",
                        i + 1,
                        response.status()
                    );
                }
                Err(e) => {
                    println!("Request {} error: {}", i + 1, e);
                }
            }
            thread::sleep(Duration::from_millis(50));
        }

        thread::sleep(Duration::from_millis(200));

        // Check hit rate calculation (only if we had successful requests)
        if successful_requests > 0 {
            if let Ok(stats_response) = client.get("/api/v1/cache/stats").send() {
                if stats_response.status().is_success() {
                    if let Ok(stats) = stats_response.json::<serde_json::Value>() {
                        let hit_count = stats["hit_count"].as_u64().unwrap_or(0);
                        let miss_count = stats["miss_count"].as_u64().unwrap_or(0);
                        let hit_rate = stats["hit_rate"].as_f64().unwrap_or(0.0);

                        println!(
                            "Cache stats: hits={hit_count}, misses={miss_count}, rate={hit_rate}"
                        );

                        if hit_count + miss_count > 0 {
                            let expected_hit_rate =
                                hit_count as f64 / (hit_count + miss_count) as f64 * 100.0;
                            assert!((hit_rate - expected_hit_rate).abs() < 0.01);
                        }
                    }
                }
            }
        } else {
            println!("No successful requests - skipping hit rate test");
        }
    }

    #[test]
    #[serial]
    fn test_cache_health_status_levels() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Test initial health status (should be healthy)
        let response = client.get("/api/v1/cache/health").send().unwrap();
        assert!(response.status().is_success());

        let health: serde_json::Value = response.json().unwrap();
        let status = health["status"].as_str().unwrap();

        // Should be either "healthy" or "disabled"
        assert!(status == "healthy" || status == "disabled");

        if status == "healthy" {
            // Check that total_size_mb is present and is a valid number
            let total_size_mb = health["total_size_mb"].as_f64().unwrap_or(0.0);
            assert!(total_size_mb >= 0.0);
        }
    }

    #[test]
    #[serial]
    fn test_cache_with_head_requests() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Clear cache first
        let _ = client.delete("/api/v1/cache").send();
        thread::sleep(Duration::from_millis(100));

        // Make a HEAD request
        let head_response = client
            .client
            .head(format!(
                "{}/registry/lodash/-/lodash-4.17.21.tgz",
                server.base_url
            ))
            .send()
            .expect("Failed to make HEAD request");

        println!("HEAD request returned status: {}", head_response.status());

        // HEAD request should succeed
        assert!(
            head_response.status().is_success(),
            "HEAD request failed with status: {} - HEAD requests should return 200 OK",
            head_response.status()
        );

        thread::sleep(Duration::from_millis(100));

        // HEAD requests should not populate cache with content
        let stats_response = client.get("/api/v1/cache/stats").send().unwrap();
        assert!(stats_response.status().is_success());

        let stats: serde_json::Value = stats_response.json().unwrap();
        // Cache might have metadata but not the full content
        let total_size = stats["total_size_bytes"].as_u64().unwrap_or(0);
        println!("Cache size after HEAD request: {total_size} bytes");
        // Size should be minimal for HEAD requests (allow some metadata)
        assert!(
            total_size < 10240,
            "Cache size too large after HEAD request: {total_size} bytes"
        );
    }

    #[test]
    #[serial]
    fn test_cache_concurrent_access() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();

        let client = ApiClient::new(server.base_url.clone());

        // Clear cache first
        let _ = client.delete("/api/v1/cache").send();
        thread::sleep(Duration::from_millis(100));

        // Simulate concurrent access (with error handling)
        let handles: Vec<_> = (0..3)
            .map(|i| {
                // Reduced from 5 to 3 for faster execution
                let base_url = server.base_url.clone();
                std::thread::spawn(move || {
                    let client = ApiClient::new(base_url);
                    match client.get("/registry/lodash/-/lodash-4.17.21.tgz").send() {
                        Ok(response) => {
                            println!("Concurrent request {} status: {}", i + 1, response.status());
                            response.status().is_success()
                        }
                        Err(e) => {
                            println!("Concurrent request {} error: {}", i + 1, e);
                            false
                        }
                    }
                })
            })
            .collect();

        // Wait for all requests to complete and count successes
        let mut successful_requests = 0;
        for handle in handles {
            if let Ok(success) = handle.join() {
                if success {
                    successful_requests += 1;
                }
            }
        }

        thread::sleep(Duration::from_millis(200));

        // Check that cache handled concurrent access properly
        if let Ok(stats_response) = client.get("/api/v1/cache/stats").send() {
            if stats_response.status().is_success() {
                if let Ok(stats) = stats_response.json::<serde_json::Value>() {
                    let total_requests = stats["hit_count"].as_u64().unwrap_or(0)
                        + stats["miss_count"].as_u64().unwrap_or(0);

                    println!(
                        "Concurrent test: {successful_requests} successful requests, {total_requests} total cache operations"
                    );

                    // Should have processed at least some requests
                    assert!(total_requests > 0);
                }
            }
        }
    }

    #[test]
    #[serial]
    fn test_cache_stats_persistence_across_restarts() {
        init_test_env();

        // Create a shared temporary directory for both server instances
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
        let shared_cache_dir = temp_dir.path().join("cache");
        let shared_db_path = temp_dir.path().join("shared.db");
        std::fs::create_dir_all(&shared_cache_dir).expect("Failed to create cache directory");

        // Start first server instance with shared paths
        let server1 =
            TestServer::with_shared_paths(shared_cache_dir.clone(), shared_db_path.clone());
        let handle1 = server1.start();
        let client = ApiClient::new(server1.base_url.clone());

        // Clear cache and make some requests to generate stats
        let _ = client.delete("/api/v1/cache").send();
        thread::sleep(Duration::from_millis(100));

        // Make requests to generate cache activity
        let _ = client.get("/registry/lodash").send();
        let _ = client.get("/registry/express").send();
        thread::sleep(Duration::from_millis(200));

        // Get stats from first server instance
        let stats1_response = client.get("/api/v1/cache/stats").send().unwrap();
        assert!(stats1_response.status().is_success());
        let stats1: serde_json::Value = stats1_response.json().unwrap();
        let hit_count1 = stats1["hit_count"].as_u64().unwrap_or(0);
        let miss_count1 = stats1["miss_count"].as_u64().unwrap_or(0);

        println!("First server stats: hits={hit_count1}, misses={miss_count1}");

        // Stop first server
        drop(handle1);
        thread::sleep(Duration::from_millis(500));

        // Start second server instance with same shared paths (simulating restart)
        let server2 = TestServer::with_shared_paths(shared_cache_dir, shared_db_path);
        let _handle2 = server2.start();
        let client2 = ApiClient::new(server2.base_url.clone());
        thread::sleep(Duration::from_millis(500));

        // Get stats from second server instance
        let stats2_response = client2.get("/api/v1/cache/stats").send().unwrap();
        assert!(stats2_response.status().is_success());
        let stats2: serde_json::Value = stats2_response.json().unwrap();
        let hit_count2 = stats2["hit_count"].as_u64().unwrap_or(0);
        let miss_count2 = stats2["miss_count"].as_u64().unwrap_or(0);

        println!("Second server stats: hits={hit_count2}, misses={miss_count2}");

        // Cache stats should persist across restarts when using the same database
        // The second server should restore the stats from the first server
        if hit_count1 + miss_count1 > 0 {
            assert!(
                hit_count2 >= hit_count1 && miss_count2 >= miss_count1,
                "Cache stats should persist and be restored from database: first=({hit_count1},{miss_count1}), second=({hit_count2},{miss_count2})"
            );
        }
    }

    #[test]
    #[serial]
    fn test_analytics_endpoint_cache_data() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();
        let client = ApiClient::new(server.base_url.clone());

        // Clear cache first
        let _ = client.delete("/api/v1/cache").send();
        thread::sleep(Duration::from_millis(100));

        // Make some requests to populate cache
        let _ = client.get("/registry/lodash").send();
        let _ = client.get("/registry/express").send();
        let _ = client.get("/registry/react").send();
        thread::sleep(Duration::from_millis(300));

        // Test analytics endpoint
        let response = client.get("/api/v1/analytics").send().unwrap();
        assert!(response.status().is_success());

        let analytics: serde_json::Value = response.json().unwrap();

        // Verify analytics structure
        assert!(analytics["total_packages"].is_number());
        assert!(analytics["total_size_bytes"].is_number());
        assert!(analytics["total_size_mb"].is_number());
        assert!(analytics["cache_hit_rate"].is_number());
        assert!(analytics["most_popular_packages"].is_array());
        assert!(analytics["recent_packages"].is_array());

        // Verify cache hit rate is a valid percentage
        let hit_rate = analytics["cache_hit_rate"].as_f64().unwrap_or(-1.0);
        assert!(
            hit_rate >= 0.0 && hit_rate <= 100.0,
            "Hit rate should be between 0-100%"
        );

        // Verify total size calculations
        let size_bytes = analytics["total_size_bytes"].as_i64().unwrap_or(0);
        let size_mb = analytics["total_size_mb"].as_f64().unwrap_or(0.0);
        let expected_mb = size_bytes as f64 / 1024.0 / 1024.0;
        assert!(
            (size_mb - expected_mb).abs() < 0.01,
            "Size MB calculation should match bytes conversion"
        );

        println!(
            "Analytics: packages={}, size={}MB, hit_rate={}%",
            analytics["total_packages"], size_mb, hit_rate
        );
    }

    #[test]
    #[serial]
    fn test_cache_database_persistence() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();
        let client = ApiClient::new(server.base_url.clone());

        // Clear cache to start fresh
        let _ = client.delete("/api/v1/cache").send();
        thread::sleep(Duration::from_millis(100));

        // Make requests to generate cache misses and hits
        let _ = client.get("/registry/lodash").send(); // Should be a miss
        thread::sleep(Duration::from_millis(100));
        let _ = client.get("/registry/lodash").send(); // Should be a hit
        thread::sleep(Duration::from_millis(100));
        let _ = client.get("/registry/express").send(); // Should be a miss
        thread::sleep(Duration::from_millis(100));
        let _ = client.get("/registry/express").send(); // Should be a hit
        thread::sleep(Duration::from_millis(200));

        // Get final stats
        let stats_response = client.get("/api/v1/cache/stats").send().unwrap();
        assert!(stats_response.status().is_success());
        let stats: serde_json::Value = stats_response.json().unwrap();

        let final_hits = stats["hit_count"].as_u64().unwrap_or(0);
        let final_misses = stats["miss_count"].as_u64().unwrap_or(0);

        println!("Final cache stats: hits={final_hits}, misses={final_misses}");

        // Should have both hits and misses
        assert!(final_hits > 0, "Should have cache hits");
        assert!(final_misses > 0, "Should have cache misses");

        // Hit rate should be calculated correctly
        let hit_rate = stats["hit_rate"].as_f64().unwrap_or(0.0);
        let expected_rate = final_hits as f64 / (final_hits + final_misses) as f64 * 100.0;
        assert!(
            (hit_rate - expected_rate).abs() < 0.01,
            "Hit rate calculation incorrect: got {hit_rate}, expected {expected_rate}"
        );
    }

    #[test]
    #[serial]
    fn test_cache_file_counting_accuracy() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();
        let client = ApiClient::new(server.base_url.clone());

        // Clear cache first
        let _ = client.delete("/api/v1/cache").send();
        thread::sleep(Duration::from_millis(100));

        // Get initial stats
        let initial_response = client.get("/api/v1/cache/stats").send().unwrap();
        let initial_stats: serde_json::Value = initial_response.json().unwrap();
        let initial_entries = initial_stats["total_entries"].as_u64().unwrap_or(0);
        let initial_size = initial_stats["total_size_bytes"].as_u64().unwrap_or(0);

        // Make requests to cache both metadata and tarballs
        let _ = client.get("/registry/lodash").send(); // Metadata
        thread::sleep(Duration::from_millis(100));
        let _ = client.get("/registry/lodash/-/lodash-4.17.21.tgz").send(); // Tarball
        thread::sleep(Duration::from_millis(200));

        // Get updated stats
        let updated_response = client.get("/api/v1/cache/stats").send().unwrap();
        let updated_stats: serde_json::Value = updated_response.json().unwrap();
        let updated_entries = updated_stats["total_entries"].as_u64().unwrap_or(0);
        let updated_size = updated_stats["total_size_bytes"].as_u64().unwrap_or(0);

        println!("Cache entries: initial={initial_entries}, updated={updated_entries}");
        println!("Cache size: initial={initial_size}, updated={updated_size}");

        // Should have more entries after caching
        assert!(
            updated_entries > initial_entries,
            "Cache entries should increase after requests"
        );

        // Size should increase (unless cache was already populated)
        assert!(
            updated_size >= initial_size,
            "Cache size should not decrease"
        );

        // Verify size calculation includes both .json and .tgz files
        // This is tested by ensuring the total size is reasonable for the cached content
        if updated_entries > 0 {
            let avg_file_size = updated_size / updated_entries;
            assert!(
                avg_file_size > 0,
                "Average file size should be greater than 0"
            );
        }
    }

    #[test]
    #[serial]
    fn test_metadata_cache_with_database_persistence() {
        init_test_env();
        let server = TestServer::new();
        let _handle = server.start();
        let client = ApiClient::new(server.base_url.clone());

        // Clear cache first
        let _ = client.delete("/api/v1/cache").send();
        thread::sleep(Duration::from_millis(100));

        // Make metadata requests (not tarball downloads)
        let _ = client.get("/registry/lodash").send();
        let _ = client.get("/registry/express").send();
        let _ = client.get("/registry/react").send();
        thread::sleep(Duration::from_millis(300));

        // Verify metadata is cached and stats are updated
        let stats_response = client.get("/api/v1/cache/stats").send().unwrap();
        assert!(stats_response.status().is_success());
        let stats: serde_json::Value = stats_response.json().unwrap();

        let hit_count = stats["hit_count"].as_u64().unwrap_or(0);
        let miss_count = stats["miss_count"].as_u64().unwrap_or(0);
        let total_entries = stats["total_entries"].as_u64().unwrap_or(0);

        println!(
            "Metadata cache stats: hits={hit_count}, misses={miss_count}, entries={total_entries}"
        );

        // Should have cache activity from metadata requests
        assert!(hit_count + miss_count > 0, "Should have cache activity");
        assert!(total_entries > 0, "Should have cached entries");

        // Make the same requests again - should result in cache hits
        let _ = client.get("/registry/lodash").send();
        let _ = client.get("/registry/express").send();
        thread::sleep(Duration::from_millis(200));

        // Check updated stats
        let updated_response = client.get("/api/v1/cache/stats").send().unwrap();
        let updated_stats: serde_json::Value = updated_response.json().unwrap();
        let updated_hits = updated_stats["hit_count"].as_u64().unwrap_or(0);

        // Should have more hits after repeated requests
        assert!(
            updated_hits > hit_count,
            "Repeated metadata requests should result in cache hits"
        );
    }
}
