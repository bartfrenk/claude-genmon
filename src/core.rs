use anyhow::Result;

/// Merged usage statistics from one or more [`UsageSource`]s.
#[derive(Debug, Default, Clone)]
pub struct UsageStats {
    pub five_hour_utilization: Option<f64>,
    pub five_hour_resets_at: Option<String>,
    pub seven_day_utilization: Option<f64>,
    pub seven_day_resets_at: Option<String>,
    pub total_sessions: Option<u64>,
    pub total_messages: Option<u64>,
    pub total_tokens: Option<u64>,
}

impl UsageStats {
    /// Left-biased merge: each field keeps `self`'s value, falling back to
    /// `other`'s when `self`'s is `None`.
    pub fn merge(self, other: UsageStats) -> UsageStats {
        UsageStats {
            five_hour_utilization: self.five_hour_utilization.or(other.five_hour_utilization),
            five_hour_resets_at: self.five_hour_resets_at.or(other.five_hour_resets_at),
            seven_day_utilization: self.seven_day_utilization.or(other.seven_day_utilization),
            seven_day_resets_at: self.seven_day_resets_at.or(other.seven_day_resets_at),
            total_sessions: self.total_sessions.or(other.total_sessions),
            total_messages: self.total_messages.or(other.total_messages),
            total_tokens: self.total_tokens.or(other.total_tokens),
        }
    }
}

/// A source of usage statistics.
pub trait UsageSource: Sync {
    fn fetch(&self) -> Result<UsageStats>;
}

/// Fetches from `left` and `right` concurrently and left-biased-merges the
/// results. If one side fails, the other's result is returned outright. If
/// both fail, `left`'s error is returned.
pub fn combine(left: &dyn UsageSource, right: &dyn UsageSource) -> Result<UsageStats> {
    let (left_result, right_result) = std::thread::scope(|scope| {
        let left_handle = scope.spawn(|| left.fetch());
        let right_handle = scope.spawn(|| right.fetch());
        (left_handle.join().unwrap(), right_handle.join().unwrap())
    });

    match (left_result, right_result) {
        (Ok(l), Ok(r)) => Ok(l.merge(r)),
        (Ok(l), Err(_)) => Ok(l),
        (Err(_), Ok(r)) => Ok(r),
        (Err(left_err), Err(_)) => Err(left_err),
    }
}
