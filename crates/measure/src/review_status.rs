//! Configurable annotation review states with auditable transition history.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusEvent {
    pub status: String,
    pub author: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewStatus {
    pub allowed: Vec<String>,
    pub history: Vec<StatusEvent>,
}

impl ReviewStatus {
    pub fn new(allowed: Vec<String>) -> Result<Self, String> {
        let mut seen = BTreeSet::new();
        if allowed.is_empty() || allowed.iter().any(|s| s.trim().is_empty() || !seen.insert(s.trim().to_lowercase())) {
            return Err("review statuses must be nonempty and unique".into());
        }
        Ok(Self { allowed, history: Vec::new() })
    }

    /// Returns the status active at or before the supplied sortable timestamp.
    /// Timestamps must use one consistent lexicographically sortable format.
    pub fn status_at(&self, timestamp: &str) -> Option<&str> {
        self.history.iter().rfind(|event| event.timestamp.as_str() <= timestamp).map(|event| event.status.as_str())
    }

    /// Returns a chronological snapshot of transitions by the specified author.
    pub fn transitions_by<'a>(&'a self, author: &str) -> Vec<&'a StatusEvent> {
        self.history.iter().filter(|event| event.author.eq_ignore_ascii_case(author)).collect()
    }

    /// Returns the number of actual status changes made by each author.
    pub fn transition_counts(&self) -> std::collections::BTreeMap<String, usize> {
        let mut counts = std::collections::BTreeMap::new();
        for event in &self.history {
            *counts.entry(event.author.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Rebuilds the current state from a stored audit trail, validating each event.
    pub fn from_history(allowed: Vec<String>, history: Vec<StatusEvent>) -> Result<Self, String> {
        let mut review = Self::new(allowed)?;
        for event in history {
            if !review.transition(&event.status, &event.author, &event.timestamp)? {
                return Err("duplicate consecutive review status".into());
            }
        }
        Ok(review)
    }

    pub fn current(&self) -> Option<&str> {
        self.history.last().map(|event| event.status.as_str())
    }

    pub fn transition(&mut self, status: &str, author: &str, timestamp: &str) -> Result<bool, String> {
        let target = self.allowed.iter().find(|s| s.eq_ignore_ascii_case(status.trim())).ok_or_else(|| "unknown review status".to_string())?;
        if author.trim().is_empty() || timestamp.trim().is_empty() {
            return Err("review transition requires author and timestamp".into());
        }
        if self.history.last().is_some_and(|event| event.timestamp.as_str() > timestamp) {
            return Err("review timestamps must be chronological".into());
        }
        if self.current().is_some_and(|current| current == target) {
            return Ok(false);
        }
        self.history.push(StatusEvent { status: target.clone(), author: author.into(), timestamp: timestamp.into() });
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn records_transitions_without_duplicate_events() {
        let mut review = ReviewStatus::new(vec!["Open".into(), "Resolved".into()]).unwrap();
        assert!(review.transition("open", "alice", "t1").unwrap());
        assert!(!review.transition("Open", "alice", "t2").unwrap());
        assert!(review.transition("Resolved", "bob", "t3").unwrap());
        assert_eq!(review.current(), Some("Resolved"));
        assert_eq!(review.history.len(), 2);
        assert_eq!(review.history[0].author, "alice");
    }
    #[test]
    fn history_supports_temporal_queries_and_author_audits() {
        let mut review = ReviewStatus::new(vec!["Open".into(), "Resolved".into()]).unwrap();
        review.transition("Open", "alice", "2026-11-20T10:00:00Z").unwrap();
        review.transition("Resolved", "bob", "2026-11-20T11:00:00Z").unwrap();
        assert_eq!(review.status_at("2026-11-20T09:00:00Z"), None);
        assert_eq!(review.status_at("2026-11-20T10:30:00Z"), Some("Open"));
        assert_eq!(review.status_at("2026-11-20T12:00:00Z"), Some("Resolved"));
        assert_eq!(review.transitions_by("BOB").len(), 1);
    }
    #[test]
    fn rejects_out_of_order_transitions_without_mutating_history() {
        let mut review = ReviewStatus::new(vec!["Open".into(), "Resolved".into()]).unwrap();
        review.transition("Open", "alice", "2026-11-20T11:00:00Z").unwrap();
        assert!(review.transition("Resolved", "bob", "2026-11-20T10:00:00Z").is_err());
        assert_eq!(review.current(), Some("Open"));
        assert_eq!(review.history.len(), 1);
    }
    #[test]
    fn rehydration_validates_audit_trail_and_counts_authors() {
        let allowed = vec!["Open".into(), "Resolved".into()];
        let events = vec![
            StatusEvent { status: "Open".into(), author: "alice".into(), timestamp: "t1".into() },
            StatusEvent { status: "Resolved".into(), author: "bob".into(), timestamp: "t2".into() },
            StatusEvent { status: "Open".into(), author: "alice".into(), timestamp: "t3".into() },
        ];
        let review = ReviewStatus::from_history(allowed.clone(), events.clone()).unwrap();
        assert_eq!(review.current(), Some("Open"));
        assert_eq!(review.transition_counts().get("alice"), Some(&2));
        assert_eq!(review.transition_counts().get("bob"), Some(&1));
        let mut invalid = events;
        invalid[2].timestamp = "t0".into();
        assert!(ReviewStatus::from_history(allowed, invalid).is_err());
    }
    #[test]
    fn invalid_status_and_missing_authorship_do_not_change_history() {
        let mut review = ReviewStatus::new(vec!["Open".into()]).unwrap();
        assert!(review.transition("Closed", "alice", "t1").is_err());
        assert!(review.transition("Open", "", "t1").is_err());
        assert!(review.history.is_empty());
        assert!(ReviewStatus::new(vec!["Open".into(), "open".into()]).is_err());
    }
}
