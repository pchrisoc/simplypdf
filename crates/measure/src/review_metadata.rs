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
        let fields = [
            &self.discipline,
            &self.assignee,
            &self.priority,
            &self.specification,
            &self.package,
        ];
        if fields.into_iter().flatten().any(|s| s.trim().is_empty() || s.len() > 256) {
            return Err("review metadata fields must be nonempty and at most 256 characters".into());
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

    pub fn matches_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|value| value.trim().eq_ignore_ascii_case(tag.trim()))
    }

    pub fn matches_discipline(&self, discipline: &str) -> bool {
        self.discipline.as_ref().is_some_and(|value| value.eq_ignore_ascii_case(discipline.trim()))
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
    fn duplicate_tags_and_blank_fields_are_rejected_without_mutation() {
        let mut metadata = ReviewMetadata::default();
        metadata.add_tag("Pipe".into()).unwrap();
        assert!(metadata.add_tag(" pipe ".into()).is_err());
        assert_eq!(metadata.tags, vec!["Pipe"]);
        metadata.assignee = Some(" ".into());
        assert!(metadata.validate().is_err());
    }
}
