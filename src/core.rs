use serde::{Deserialize, Serialize};

fn default_resets_at() -> String {
    "unknown".to_string()
}

/// Utilization and reset time for a single rate-limit window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitWindow {
    #[serde(default)]
    pub utilization: f64,
    #[serde(default = "default_resets_at")]
    pub resets_at: String,
}

impl Default for RateLimitWindow {
    fn default() -> Self {
        Self {
            utilization: 0.0,
            resets_at: default_resets_at(),
        }
    }
}

/// Usage statistics for the current 5-hour and 7-day rate-limit windows, as
/// returned by the Anthropic API's `/api/oauth/usage` endpoint.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageStats {
    #[serde(rename = "five_hour", default)]
    pub five_hours: RateLimitWindow,
    #[serde(rename = "seven_day", default)]
    pub seven_days: RateLimitWindow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_api_response() {
        let json = r#"{
            "five_hour": {"utilization": 42.5, "resets_at": "2026-09-05T00:00:00Z"},
            "seven_day": {"utilization": 10.0, "resets_at": "2026-09-10T00:00:00Z"}
        }"#;
        let stats: UsageStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.five_hours.utilization, 42.5);
        assert_eq!(stats.five_hours.resets_at, "2026-09-05T00:00:00Z");
        assert_eq!(stats.seven_days.utilization, 10.0);
    }

    #[test]
    fn defaults_missing_fields() {
        let stats: UsageStats = serde_json::from_str("{}").unwrap();
        assert_eq!(stats.five_hours.utilization, 0.0);
        assert_eq!(stats.five_hours.resets_at, "unknown");
    }
}
