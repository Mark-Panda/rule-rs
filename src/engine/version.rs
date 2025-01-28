use crate::types::RuleChain;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct VersionManager {
    current_version: AtomicU64,
}

pub struct Version {
    pub version: u64,
    pub timestamp: i64,
}

impl VersionManager {
    pub fn new() -> Self {
        Self {
            current_version: AtomicU64::new(1),
        }
    }

    pub fn create_version(&self, _chain: &RuleChain) -> Version {
        let version = self.current_version.fetch_add(1, Ordering::SeqCst);
        Version {
            version,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn get_current_version(&self) -> u64 {
        self.current_version.load(Ordering::SeqCst)
    }
}
