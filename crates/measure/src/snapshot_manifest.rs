//! Versioned publication manifests for detached takeoff snapshots.
use crate::{Listing, SnapshotStatus, snapshot_status};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotManifest {
    pub source: String,
    pub published_at: String,
    pub status: SnapshotStatus,
}

impl SnapshotManifest {
    pub fn create(source: &str, published_at: &str, listing: &Listing) -> Result<Self, String> {
        if source.trim().is_empty() || published_at.trim().is_empty() {
            return Err("snapshot source and publication time are required".into());
        }
        let status = snapshot_status(listing, None).map_err(|e| e.to_string())?;
        Ok(Self { source: source.into(), published_at: published_at.into(), status })
    }
    pub fn needs_refresh(&self, listing: &Listing) -> Result<bool, String> {
        snapshot_status(listing, Some(&self.status.fingerprint)).map(|s| s.needs_refresh).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn manifest_detects_initial_publication_and_unchanged_snapshot() {
        let listing = Listing::default();
        let manifest = SnapshotManifest::create("drawing.pdf", "2026-11-20T00:00:00Z", &listing).unwrap();
        assert!(!manifest.needs_refresh(&listing).unwrap());
        let encoded = serde_json::to_string(&manifest).unwrap();
        let decoded: SnapshotManifest = serde_json::from_str(&encoded).unwrap();
        assert_eq!(manifest, decoded);
    }
    #[test]
    fn manifest_rejects_incomplete_or_unidentified_publications() {
        let mut listing = Listing::default();
        assert!(SnapshotManifest::create("", "now", &listing).is_err());
        listing.truncated = true;
        assert!(SnapshotManifest::create("drawing.pdf", "now", &listing).is_err());
    }
}
