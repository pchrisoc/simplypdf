//! Typed review metadata for annotation registers and discipline workflows.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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

    /// Set a specification reference without losing prior valid metadata on errors.
    pub fn set_specification(&mut self, specification: Option<String>) -> Result<(), String> {
        let mut next = self.clone();
        next.specification = specification;
        next.validate()?;
        *self = next;
        Ok(())
    }

    /// Match specification references for review-register filtering.
    pub fn matches_specification(&self, specification: &str) -> bool {
        self.specification.as_ref().is_some_and(|value| value.trim().eq_ignore_ascii_case(specification.trim()))
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

/// Optional review-register filters. Multiple specified fields are combined with AND.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewMetadataFilter {
    pub discipline: Option<String>,
    pub assignee: Option<String>,
    pub priority: Option<String>,
    pub specification: Option<String>,
    pub package: Option<String>,
    pub tag: Option<String>,
}

impl ReviewMetadataFilter {
    pub fn matches(&self, metadata: &ReviewMetadata) -> bool {
        self.discipline.as_ref().is_none_or(|v| metadata.matches_discipline(v))
            && self.assignee.as_ref().is_none_or(|v| metadata.matches_assignee(v))
            && self.priority.as_ref().is_none_or(|v| metadata.matches_priority(v))
            && self.specification.as_ref().is_none_or(|v| metadata.matches_specification(v))
            && self.package.as_ref().is_none_or(|v| metadata.matches_package(v))
            && self.tag.as_ref().is_none_or(|v| metadata.matches_tag(v))
    }
}

/// Review metadata keyed by stable annotation IDs, suitable for serialized document sidecars.
/// A missing key means no review metadata was assigned to that annotation.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewMetadataRegistry {
    entries: BTreeMap<String, ReviewMetadata>,
}

impl ReviewMetadataRegistry {
    pub fn get(&self, annotation_id: &str) -> Option<&ReviewMetadata> {
        self.entries.get(annotation_id)
    }

    /// Validate the complete update before replacing any stored value.
    pub fn set(&mut self, annotation_id: String, metadata: ReviewMetadata) -> Result<(), String> {
        if annotation_id.trim().is_empty() || annotation_id.len() > 256 {
            return Err("annotation ID must be nonempty and at most 256 characters".into());
        }
        metadata.validate()?;
        if !self.entries.contains_key(&annotation_id) && self.entries.len() >= 100_000 {
            return Err("review metadata registry is full".into());
        }
        self.entries.insert(annotation_id, metadata);
        Ok(())
    }

    /// Apply a batch of updates only when every ID and metadata record is valid.
    /// Duplicate IDs in the batch use the final value.
    pub fn set_many(&mut self, updates: Vec<(String, ReviewMetadata)>) -> Result<(), String> {
        if updates.len() > 100_000 {
            return Err("review metadata batch is too large".into());
        }
        let mut next = self.clone();
        for (id, metadata) in updates {
            next.set(id, metadata)?;
        }
        *self = next;
        Ok(())
    }

    /// Query registry entries by review metadata without mutating their stable IDs.
    /// Results are sorted by annotation ID for deterministic register navigation.
    pub fn filter<'a>(&'a self, query: &ReviewMetadataFilter) -> Vec<(&'a str, &'a ReviewMetadata)> {
        self.entries
            .iter()
            .filter(|(_, metadata)| query.matches(metadata))
            .map(|(id, metadata)| (id.as_str(), metadata))
            .collect()
    }

    pub fn remove(&mut self, annotation_id: &str) -> Option<ReviewMetadata> {
        self.entries.remove(annotation_id)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_filters_are_combined_and_results_sorted_by_stable_id() {
        let mut registry = ReviewMetadataRegistry::default();
        let hvac = ReviewMetadata {
            discipline: Some("Mechanical".into()),
            assignee: Some("Alex".into()),
            tags: vec!["HVAC".into()],
            ..ReviewMetadata::default()
        };
        registry.set("z".into(), hvac.clone()).unwrap();
        registry.set("a".into(), hvac).unwrap();
        registry.set("m".into(), ReviewMetadata::default()).unwrap();
        let filter = ReviewMetadataFilter {
            discipline: Some(" mechanical ".into()),
            tag: Some("hvac".into()),
            ..ReviewMetadataFilter::default()
        };
        assert_eq!(registry.filter(&filter).iter().map(|(id, _)| *id).collect::<Vec<_>>(), vec!["a", "z"]);
        let missing = ReviewMetadataFilter { assignee: Some("Nobody".into()), ..filter };
        assert!(registry.filter(&missing).is_empty());
    }


    #[test]
    fn registry_batch_is_atomic_and_last_duplicate_wins() {
        let mut registry = ReviewMetadataRegistry::default();
        let first = ReviewMetadata::default();
        let second = ReviewMetadata { priority: Some("High".into()), ..ReviewMetadata::default() };
        registry.set_many(vec![("a".into(), first.clone()), ("a".into(), second.clone())]).unwrap();
        assert_eq!(registry.get("a"), Some(&second));
        let invalid = ReviewMetadata { assignee: Some(" ".into()), ..ReviewMetadata::default() };
        assert!(registry.set_many(vec![("b".into(), first), ("c".into(), invalid)]).is_err());
        assert_eq!(registry.len(), 1);
        assert!(registry.get("b").is_none());
    }

    #[test]
    fn registry_roundtrip_and_atomic_replacement() {
        let mut registry = ReviewMetadataRegistry::default();
        let first = ReviewMetadata { discipline: Some("Mechanical".into()), ..ReviewMetadata::default() };
        registry.set("markup-1".into(), first.clone()).unwrap();
        let json = serde_json::to_string(&registry).unwrap();
        let mut restored: ReviewMetadataRegistry = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.get("markup-1"), Some(&first));
        let invalid = ReviewMetadata { priority: Some(" ".into()), ..ReviewMetadata::default() };
        assert!(restored.set("markup-1".into(), invalid).is_err());
        assert_eq!(restored.get("markup-1"), Some(&first));
        assert!(restored.set(" ".into(), first.clone()).is_err());
        assert_eq!(restored.len(), 1);
        assert_eq!(restored.remove("markup-1"), Some(first));
        assert!(restored.is_empty());
    }

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
    fn specification_updates_and_filtering_preserve_valid_values() {
        let mut metadata = ReviewMetadata::default();
        assert!(!metadata.matches_specification("23 05 00"));
        metadata.set_specification(Some("23 05 00".into())).unwrap();
        assert!(metadata.matches_specification(" 23 05 00 "));
        assert!(!metadata.matches_specification("23 07 00"));
        assert!(metadata.set_specification(Some(" ".into())).is_err());
        assert_eq!(metadata.specification.as_deref(), Some("23 05 00"));
        metadata.set_specification(None).unwrap();
        assert!(!metadata.matches_specification("23 05 00"));
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
