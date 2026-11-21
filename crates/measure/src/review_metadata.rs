//! Typed review metadata for annotation registers and discipline workflows.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewMetadata {
    pub discipline: Option<String>,
    pub assignee: Option<String>,
    pub priority: Option<String>,
    pub specification: Option<String>,
    pub package: Option<String>,
    pub tags: Vec<String>,
}

impl ReviewMetadata {
    pub fn validate(&self) -> Result<(), String> {
        let fields = [&self.discipline, &self.assignee, &self.priority, &self.specification, &self.package];
        if fields.into_iter().flatten().any(|s| s.trim().is_empty() || s.len() > 256) {
            return Err("review metadata fields must be nonempty and at most 256 characters".into());
        }
        if self.tags.len() > 64 {
            return Err("review metadata supports at most 64 tags".into());
        }
        let mut seen = BTreeSet::new();
        for tag in &self.tags {
            let normalized = tag.trim().to_lowercase();
            if normalized.is_empty() || tag.len() > 128 || !seen.insert(normalized) {
                return Err("review tags must be unique, nonempty and at most 128 characters".into());
            }
        }
        Ok(())
    }

    pub fn add_tag(&mut self, tag: String) -> Result<(), String> {
        let mut next = self.clone();
        next.tags.push(tag);
        next.validate()?;
        *self = next;
        Ok(())
    }

    /// Remove a tag without changing the stored spelling of other tags.
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        let before = self.tags.len();
        self.tags.retain(|value| !value.trim().eq_ignore_ascii_case(tag.trim()));
        self.tags.len() != before
    }

    /// Update a tag atomically; duplicate or invalid replacements leave metadata untouched.
    pub fn rename_tag(&mut self, old: &str, new: String) -> Result<bool, String> {
        let Some(index) = self.tags.iter().position(|tag| tag.trim().eq_ignore_ascii_case(old.trim())) else {
            return Ok(false);
        };
        let mut next = self.clone();
        if let Some(tag) = next.tags.get_mut(index) {
            *tag = new;
        }
        next.validate()?;
        *self = next;
        Ok(true)
    }

    /// Replace an optional field after validating the entire record.
    pub fn set_assignee(&mut self, assignee: Option<String>) -> Result<(), String> {
        let mut next = self.clone();
        next.assignee = assignee;
        next.validate()?;
        *self = next;
        Ok(())
    }

    /// Change review priority without losing existing metadata on invalid input.
    pub fn set_priority(&mut self, priority: Option<String>) -> Result<(), String> {
        let mut next = self.clone();
        next.priority = priority;
        next.validate()?;
        *self = next;
        Ok(())
    }

    /// Match a review package without treating missing packages as matches.
    pub fn matches_package(&self, package: &str) -> bool {
        self.package.as_ref().is_some_and(|value| value.trim().eq_ignore_ascii_case(package.trim()))
    }

    pub fn matches_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|value| value.trim().eq_ignore_ascii_case(tag.trim()))
    }

    /// Filter review records by assignee, including whitespace-tolerant comparisons.
    pub fn matches_assignee(&self, assignee: &str) -> bool {
        self.assignee.as_ref().is_some_and(|value| value.trim().eq_ignore_ascii_case(assignee.trim()))
    }

    /// Filter review records by priority without changing stored values.
    pub fn matches_priority(&self, priority: &str) -> bool {
        self.priority.as_ref().is_some_and(|value| value.trim().eq_ignore_ascii_case(priority.trim()))
    }

    pub fn matches_discipline(&self, discipline: &str) -> bool {
        self.discipline.as_ref().is_some_and(|value| value.trim().eq_ignore_ascii_case(discipline.trim()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_roundtrip_and_case_insensitive_matching() {
        let mut metadata = ReviewMetadata { discipline: Some("Mechanical".into()), ..ReviewMetadata::default() };
        metadata.add_tag("HVAC".into()).unwrap();
        assert!(metadata.matches_tag("hvac"));
        assert!(metadata.matches_discipline("mechanical"));
        let encoded = serde_json::to_string(&metadata).unwrap();
        let restored: ReviewMetadata = serde_json::from_str(&encoded).unwrap();
        assert_eq!(restored, metadata);
    }

    #[test]
    fn package_filter_handles_missing_and_case_insensitive_values() {
        let mut metadata = ReviewMetadata::default();
        assert!(!metadata.matches_package("Package A"));
        metadata.package = Some(" Package A ".into());
        assert!(metadata.matches_package("package a"));
        assert!(!metadata.matches_package("package b"));
    }

    #[test]
    fn assignee_and_priority_filters_handle_case_whitespace_and_missing_values() {
        let mut metadata = ReviewMetadata::default();
        assert!(!metadata.matches_assignee("Engineer"));
        assert!(!metadata.matches_priority("High"));
        metadata.set_assignee(Some("Engineer".into())).unwrap();
        metadata.set_priority(Some("High".into())).unwrap();
        assert!(metadata.matches_assignee(" engineer "));
        assert!(metadata.matches_priority(" high "));
        assert!(!metadata.matches_assignee("Other"));
        assert!(!metadata.matches_priority("Low"));
    }

    #[test]
    fn assignee_and_priority_updates_are_atomic() {
        let mut metadata = ReviewMetadata::default();
        metadata.set_assignee(Some("Engineer".into())).unwrap();
        metadata.set_priority(Some("High".into())).unwrap();
        assert!(metadata.set_assignee(Some(" ".into())).is_err());
        assert!(metadata.set_priority(Some(" ".into())).is_err());
        assert_eq!(metadata.assignee.as_deref(), Some("Engineer"));
        assert_eq!(metadata.priority.as_deref(), Some("High"));
        metadata.set_assignee(None).unwrap();
        assert_eq!(metadata.assignee, None);
    }

    #[test]
    fn tag_rename_is_atomic_and_rejects_duplicates() {
        let mut metadata = ReviewMetadata::default();
        metadata.add_tag("Pipe".into()).unwrap();
        metadata.add_tag("Duct".into()).unwrap();
        assert!(metadata.rename_tag(" pipe ", "Valve".into()).unwrap());
        assert_eq!(metadata.tags, vec!["Valve", "Duct"]);
        assert!(metadata.rename_tag("Valve", " duct ".into()).is_err());
        assert_eq!(metadata.tags, vec!["Valve", "Duct"]);
        assert!(!metadata.rename_tag("missing", "Other".into()).unwrap());
    }

    #[test]
    fn tag_removal_is_case_insensitive_and_preserves_other_tags() {
        let mut metadata = ReviewMetadata::default();
        metadata.add_tag("Pipe".into()).unwrap();
        metadata.add_tag("Duct".into()).unwrap();
        assert!(metadata.remove_tag(" pipe "));
        assert!(!metadata.remove_tag("pipe"));
        assert_eq!(metadata.tags, vec!["Duct"]);
    }

    #[test]
    fn tag_limit_rejects_overflow_without_mutation() {
        let mut metadata = ReviewMetadata::default();
        for i in 0..64 {
            metadata.add_tag(format!("tag-{i}")).unwrap();
        }
        assert!(metadata.add_tag("overflow".into()).is_err());
        assert_eq!(metadata.tags.len(), 64);
    }

    #[test]
    fn duplicate_tags_and_blank_fields_are_rejected_without_mutation() {
        let mut metadata = ReviewMetadata::default();
        metadata.add_tag("Pipe".into()).unwrap();
        assert!(metadata.add_tag(" pipe ".into()).is_err());
        assert_eq!(metadata.tags, vec!["Pipe"]);
        metadata.assignee = Some(" ".into());
        assert!(metadata.validate().is_err());
    }
}
