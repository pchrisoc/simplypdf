//! Versioned, validated choice lists for takeoff classification.
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceList {
    pub name: String,
    pub version: u32,
    pub values: Vec<String>,
}

impl ChoiceList {
    pub fn validate(&self) -> Result<(), String> {
        if self.version == 0 || self.name.trim().is_empty() {
            return Err("choice list needs a name and nonzero version".into());
        }
        let mut seen = HashSet::new();
        for value in &self.values {
            let normalized = value.trim().to_lowercase();
            if normalized.is_empty() || !seen.insert(normalized) {
                return Err("choice list contains an empty or duplicate value".into());
            }
        }
        Ok(())
    }
    pub fn add(&mut self, value: String) -> Result<(), String> {
        let mut next = self.clone();
        next.values.push(value);
        next.validate()?;
        *self = next;
        Ok(())
    }
    pub fn contains(&self, value: &str) -> bool {
        self.values.iter().any(|item| item.eq_ignore_ascii_case(value.trim()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn choices_reject_case_insensitive_duplicates_without_mutation() {
        let mut list = ChoiceList { name: "Material".into(), version: 1, values: vec!["Copper".into()] };
        assert!(list.add(" copper ".into()).is_err());
        assert_eq!(list.values, vec!["Copper"]);
        assert!(list.contains("COPPER"));
        list.add("Steel".into()).unwrap();
        assert_eq!(list.values.len(), 2);
    }
    #[test]
    fn empty_names_and_versions_are_invalid() {
        let list = ChoiceList { name: "".into(), version: 0, values: vec![] };
        assert!(list.validate().is_err());
    }
}
