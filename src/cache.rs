use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::api::{Client, GetUsageError, UsageProvider};
use crate::core::UsageStats;

fn default_cache_path() -> PathBuf {
    let config_dir = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_default()
                .join(".config")
        });
    config_dir.join("claude-genmon/cache.json")
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Lower bound on how long a 429 backs off for, applied even when the
/// server's `Retry-After` is missing or `0`. Anthropic's usage endpoint has
/// been observed to report `Retry-After: 0` while still rejecting requests,
/// which by itself would let the cache hit the API again on every call.
const MIN_BACKOFF_SECS: u64 = 30;

/// Resolves a `Retry-After` header value (delta-seconds or an HTTP-date) to
/// an absolute Unix timestamp, at least `MIN_BACKOFF_SECS` in the future.
fn parse_retry_after(value: Option<&str>, now: u64) -> u64 {
    let requested = value.and_then(|v| {
        if let Ok(secs) = v.trim().parse::<u64>() {
            return Some(now + secs);
        }
        let at = httpdate::parse_http_date(v.trim()).ok()?;
        Some(at.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(now))
    });
    requested.unwrap_or(now).max(now + MIN_BACKOFF_SECS)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct CacheData {
    stats: Option<UsageStats>,
    retry_after_unix: Option<u64>,
    #[serde(default)]
    written_at_unix: u64,
}

/// Wraps a [`UsageProvider`], caching the last valid result on disk and
/// backing off for the duration of a `Retry-After` window on 429 responses
/// instead of re-hitting the API.
pub struct Cache<C: UsageProvider = Client> {
    client: C,
    path: PathBuf,
}

impl Default for Cache<Client> {
    fn default() -> Self {
        Self {
            client: Client::default(),
            path: default_cache_path(),
        }
    }
}

impl<C: UsageProvider> Cache<C> {
    /// Only used directly in tests, to inject a mock provider and a
    /// tempdir-scoped path; `Cache::<Client>::default()` covers real usage.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn new(client: C, path: PathBuf) -> Self {
        Self { client, path }
    }

    pub fn get_usage(&self) -> Result<UsageStats> {
        let mut cache = self.load();
        let now = unix_now();

        if let Some(retry_at) = cache.retry_after_unix
            && now < retry_at
        {
            return cache.stats.clone().ok_or_else(|| {
                anyhow!("rate limited until {retry_at}, no cached usage available")
            });
        }

        match self.client.get_usage() {
            Ok(stats) => {
                cache.stats = Some(stats.clone());
                cache.retry_after_unix = None;
                self.save(&mut cache)?;
                Ok(stats)
            }
            Err(GetUsageError::RateLimited { retry_after }) => {
                cache.retry_after_unix = Some(parse_retry_after(retry_after.as_deref(), now));
                self.save(&mut cache)?;
                cache
                    .stats
                    .clone()
                    .ok_or_else(|| anyhow!("rate limited, no cached usage available"))
            }
            Err(GetUsageError::Other(e)) => Err(e),
        }
    }

    fn load(&self) -> CacheData {
        std::fs::read_to_string(&self.path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    fn save(&self, data: &mut CacheData) -> Result<()> {
        data.written_at_unix = unix_now();
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        let content = serde_json::to_string(data)?;
        let tmp_path = self.path.with_extension("json.tmp");
        std::fs::write(&tmp_path, &content)
            .with_context(|| format!("writing {}", tmp_path.display()))?;
        std::fs::rename(&tmp_path, &self.path).with_context(|| {
            format!("renaming {} to {}", tmp_path.display(), self.path.display())
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use super::*;

    struct MockProvider {
        calls: Cell<u32>,
        responses: RefCell<Vec<Result<UsageStats, GetUsageError>>>,
    }

    impl MockProvider {
        fn new(responses: Vec<Result<UsageStats, GetUsageError>>) -> Self {
            Self {
                calls: Cell::new(0),
                responses: RefCell::new(responses),
            }
        }
    }

    impl UsageProvider for MockProvider {
        fn get_usage(&self) -> Result<UsageStats, GetUsageError> {
            self.calls.set(self.calls.get() + 1);
            self.responses.borrow_mut().remove(0)
        }
    }

    fn sample_stats(utilization: f64) -> UsageStats {
        let mut stats = UsageStats::default();
        stats.five_hours.utilization = utilization;
        stats
    }

    fn temp_cache_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "claude-genmon-cache-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        path.push("cache.json");
        path
    }

    #[test]
    fn calls_client_and_persists_on_empty_cache() {
        let path = temp_cache_path();
        let provider = MockProvider::new(vec![Ok(sample_stats(10.0))]);
        let cache = Cache::new(provider, path.clone());

        let before = unix_now();
        let stats = cache.get_usage().unwrap();
        assert_eq!(stats.five_hours.utilization, 10.0);
        assert_eq!(cache.client.calls.get(), 1);
        assert!(path.exists());
        assert!(cache.load().written_at_unix >= before);

        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn serves_cached_value_within_retry_window() {
        let path = temp_cache_path();
        let provider = MockProvider::new(vec![
            Ok(sample_stats(10.0)),
            Err(GetUsageError::RateLimited {
                retry_after: Some("120".to_string()),
            }),
        ]);
        let cache = Cache::new(provider, path.clone());

        cache.get_usage().unwrap(); // populates the cache
        cache.get_usage().unwrap(); // hits the 429, records retry_after, still serves cached stats
        let stats = cache.get_usage().unwrap(); // within the retry window, served from cache
        assert_eq!(stats.five_hours.utilization, 10.0);
        assert_eq!(cache.client.calls.get(), 2);

        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn zero_retry_after_still_applies_minimum_backoff() {
        let path = temp_cache_path();
        let provider = MockProvider::new(vec![
            Ok(sample_stats(10.0)),
            Err(GetUsageError::RateLimited { retry_after: Some("0".to_string()) }),
        ]);
        let cache = Cache::new(provider, path.clone());

        cache.get_usage().unwrap(); // populates the cache
        cache.get_usage().unwrap(); // hits the 429 with Retry-After: 0

        let retry_at = cache.load().retry_after_unix.unwrap();
        assert!(retry_at >= unix_now() + MIN_BACKOFF_SECS);

        // Still within the (floor-enforced) backoff window: no further call.
        let stats = cache.get_usage().unwrap();
        assert_eq!(stats.five_hours.utilization, 10.0);
        assert_eq!(cache.client.calls.get(), 2);

        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn calls_client_again_after_retry_window_expires() {
        let path = temp_cache_path();
        let provider = MockProvider::new(vec![Ok(sample_stats(10.0)), Ok(sample_stats(20.0))]);
        let cache = Cache::new(provider, path.clone());

        cache.get_usage().unwrap();

        let mut cache_data = cache.load();
        cache_data.retry_after_unix = Some(unix_now().saturating_sub(1));
        cache.save(&mut cache_data).unwrap();

        let stats = cache.get_usage().unwrap();
        assert_eq!(stats.five_hours.utilization, 20.0);
        assert_eq!(cache.client.calls.get(), 2);

        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn rate_limited_with_no_cached_stats_errors_without_looping() {
        let path = temp_cache_path();
        let provider = MockProvider::new(vec![Err(GetUsageError::RateLimited {
            retry_after: Some("120".to_string()),
        })]);
        let cache = Cache::new(provider, path.clone());

        assert!(cache.get_usage().is_err());
        assert!(cache.get_usage().is_err());
        assert_eq!(cache.client.calls.get(), 1);

        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn missing_cache_file_does_not_panic() {
        let path = temp_cache_path();
        let cache: Cache<MockProvider> = Cache::new(MockProvider::new(vec![]), path);
        let data = cache.load();
        assert!(data.stats.is_none());
        assert!(data.retry_after_unix.is_none());
    }
}
