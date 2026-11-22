//! Filter and aggregate measured quantities without mixing incompatible units.
use crate::{invalid, Measurement, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantityFilter {
    pub label: Option<String>,
    pub author: Option<String>,
    /// Match an HVAC takeoff's material (case-insensitive).
    pub material: Option<String>,
    /// Match an HVAC takeoff's service/system (case-insensitive).
    pub service: Option<String>,
    /// Match an HVAC takeoff's profile (case-insensitive).
    pub profile: Option<String>,
    /// Match an HVAC takeoff's nominal size (case-insensitive).
    pub size: Option<String>,
    pub unit: Option<String>,
    /// Restrict quantities to a zero-based PDF page index.
    pub page: Option<usize>,
    /// Restrict quantities to visible or hidden measurement annotations.
    pub visible: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuantityTotal {
    pub unit: String,
    pub count: usize,
    pub quantity: f64,
}

pub fn aggregate(measurements: &[Measurement], filter: &QuantityFilter) -> Result<Vec<QuantityTotal>> {
    let mut totals: BTreeMap<String, (usize, f64)> = BTreeMap::new();
    for m in measurements {
        if filter.label.as_ref().is_some_and(|s| !m.label.eq_ignore_ascii_case(s))
            || filter.author.as_ref().is_some_and(|s| !m.author.eq_ignore_ascii_case(s))
            || filter.unit.as_ref().is_some_and(|s| m.reading.unit != *s)
            || filter.material.as_ref().is_some_and(|s| !m.hvac.as_ref().is_some_and(|h| h.material.eq_ignore_ascii_case(s)))
            || filter.service.as_ref().is_some_and(|s| !m.hvac.as_ref().is_some_and(|h| h.service.eq_ignore_ascii_case(s)))
            || filter.profile.as_ref().is_some_and(|s| !m.hvac.as_ref().is_some_and(|h| h.profile.eq_ignore_ascii_case(s)))
            || filter.size.as_ref().is_some_and(|s| !m.hvac.as_ref().is_some_and(|h| h.size.eq_ignore_ascii_case(s)))
            || filter.page.is_some_and(|page| m.page != page)
            || filter.visible.is_some_and(|visible| m.visible != visible)
        {
            continue;
        }
        let value = m.reading.value;
        if !value.is_finite() || value < 0.0 {
            return Err(invalid("invalid quantity in filtered takeoff"));
        }
        let entry = totals.entry(m.reading.unit.clone()).or_insert((0, 0.0));
        entry.0 = entry.0.checked_add(1).ok_or_else(|| invalid("filtered count overflow"))?;
        entry.1 += value;
        if !entry.1.is_finite() {
            return Err(invalid("filtered quantity overflow"));
        }
    }
    Ok(totals.into_iter().map(|(unit, (count, quantity))| QuantityTotal { unit, count, quantity }).collect())
}

/// A source PDF and its measurements. Source keys must be stable across refreshes.
#[derive(Clone, Debug)]
pub struct QuantitySource<'a> {
    pub source_id: &'a str,
    pub measurements: &'a [Measurement],
}

/// Filter across independent PDFs without merging their annotation identities.
/// The optional source selector is an exact, case-sensitive stable source ID.
pub fn aggregate_sources(sources: &[QuantitySource<'_>], source_id: Option<&str>, filter: &QuantityFilter) -> Result<Vec<QuantityTotal>> {
    let mut totals: BTreeMap<String, (usize, f64)> = BTreeMap::new();
    let mut seen = std::collections::BTreeSet::new();
    for source in sources {
        if source.source_id.trim().is_empty() || source.source_id.len() > 1024 || !seen.insert(source.source_id) {
            return Err(invalid("source IDs must be nonempty, unique, and at most 1024 bytes"));
        }
    }
    if source_id.is_some_and(|id| !seen.contains(id)) {
        return Err(invalid("selected source ID was not found"));
    }
    for source in sources {
        if source_id.is_some_and(|id| id != source.source_id) {
            continue;
        }
        for total in aggregate(source.measurements, filter)? {
            let entry = totals.entry(total.unit).or_insert((0usize, 0.0f64));
            entry.0 = entry.0.checked_add(total.count).ok_or_else(|| invalid("filtered count overflow"))?;
            entry.1 += total.quantity;
            if !entry.1.is_finite() {
                return Err(invalid("filtered quantity overflow across PDFs"));
            }
        }
    }
    Ok(totals.into_iter().map(|(unit, (count, quantity))| QuantityTotal { unit, count, quantity }).collect())
}

/// One source's independently aggregated quantities, preserving source provenance.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceQuantityTotals {
    pub source_id: String,
    pub totals: Vec<QuantityTotal>,
}

/// Produce a reviewable breakdown for every source, including sources with no matches.
/// Reject duplicate or malformed identifiers before returning any partial results.
pub fn aggregate_by_source(sources: &[QuantitySource<'_>], filter: &QuantityFilter) -> Result<Vec<SourceQuantityTotals>> {
    let mut seen = std::collections::BTreeSet::new();
    let mut results = Vec::new();
    for source in sources {
        let id = source.source_id;
        if id.trim().is_empty() || id.len() > 1024 || !seen.insert(id) {
            return Err(invalid("source IDs must be nonblank, unique, and at most 1024 bytes"));
        }
        results.push(SourceQuantityTotals { source_id: id.to_owned(), totals: aggregate(source.measurements, filter)? });
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn multi_pdf_aggregation_preserves_units_and_source_provenance() {
        let scale = crate::Scale::default();
        let points = vec![[0.0, 0.0], [72.0, 0.0]];
        let reading = crate::reading(crate::Kind::Distance, &points, &scale).unwrap();
        let make = |id: &str, label: &str| Measurement {
            visible: true, page: 0, index: 0, id: id.into(),
            author: "engineer".into(), label: label.into(),
            points: points.clone(), scale: scale.clone(), reading: reading.clone(), hvac: None,
        };
        let a = [make("same-annotation", "pipe"), make("other", "duct")];
        let b = [make("same-annotation", "pipe")];
        let sources = [
            QuantitySource { source_id: "first.pdf", measurements: &a },
            QuantitySource { source_id: "second.pdf", measurements: &b },
        ];
        let filter = QuantityFilter { label: Some("PIPE".into()), ..Default::default() };
        let combined = aggregate_sources(&sources, None, &filter).unwrap();
        assert_eq!(combined.len(), 1);
        assert_eq!(combined[0].unit, reading.unit);
        assert_eq!(combined[0].count, 2);
        assert!((combined[0].quantity - 2.0 * reading.value).abs() < 1e-9);
        let per_source = aggregate_by_source(&sources, &filter).unwrap();
        assert_eq!(per_source.len(), 2);
        assert_eq!(per_source[0].source_id, "first.pdf");
        assert_eq!(per_source[1].source_id, "second.pdf");
        assert_eq!(per_source[0].totals[0].count, 1);
        assert_eq!(per_source[1].totals[0].count, 1);
        let selected = aggregate_sources(&sources, Some("second.pdf"), &filter).unwrap();
        assert_eq!(selected[0].count, 1);
    }
    #[test]
    fn page_and_visibility_filters_apply_to_quantities() {
        let scale = crate::Scale::default();
        let points = vec![[0.0, 0.0], [72.0, 0.0]];
        let reading = crate::reading(crate::Kind::Distance, &points, &scale).unwrap();
        let make = |page, visible| Measurement {
            visible, page, index: 0, id: format!("id-{page}-{visible}"),
            author: "reviewer".into(), label: "pipe".into(),
            points: points.clone(), scale: scale.clone(), reading: reading.clone(), hvac: None,
        };
        let measurements = [make(0, true), make(1, true), make(1, false)];
        let filter = QuantityFilter { page: Some(1), visible: Some(true), ..Default::default() };
        let totals = aggregate(&measurements, &filter).unwrap();
        assert_eq!(totals.len(), 1);
        assert_eq!(totals[0].count, 1);
        assert_eq!(totals[0].quantity, reading.value);
        let filter = QuantityFilter { page: Some(1), visible: Some(false), ..Default::default() };
        assert_eq!(aggregate(&measurements, &filter).unwrap()[0].count, 1);
        let filter = QuantityFilter { page: Some(2), ..Default::default() };
        assert!(aggregate(&measurements, &filter).unwrap().is_empty());
    }
    #[test]
    fn per_source_breakdown_preserves_order_and_empty_sources() {
        let empty: &[Measurement] = &[];
        let sources = [
            QuantitySource { source_id: "sheet-b", measurements: empty },
            QuantitySource { source_id: "sheet-a", measurements: empty },
        ];
        let result = aggregate_by_source(&sources, &QuantityFilter::default()).unwrap();
        assert_eq!(result.iter().map(|row| row.source_id.as_str()).collect::<Vec<_>>(), vec!["sheet-b", "sheet-a"]);
        assert!(result.iter().all(|row| row.totals.is_empty()));
    }
    #[test]
    fn per_source_breakdown_rejects_duplicate_and_blank_ids() {
        let empty: &[Measurement] = &[];
        let sources = [QuantitySource { source_id: " ", measurements: empty }];
        assert!(aggregate_by_source(&sources, &QuantityFilter::default()).is_err());
        assert!(aggregate_sources(&sources, None, &QuantityFilter::default()).is_err());
        let duplicates = [
            QuantitySource { source_id: "a", measurements: empty },
            QuantitySource { source_id: "a", measurements: empty },
        ];
        assert!(aggregate_by_source(&duplicates, &QuantityFilter::default()).is_err());
    }
    #[test]
    fn multi_pdf_sources_reject_duplicate_and_missing_ids() {
        let empty: &[Measurement] = &[];
        let sources = [
            QuantitySource { source_id: "sheet-a", measurements: empty },
            QuantitySource { source_id: "sheet-a", measurements: empty },
        ];
        assert!(aggregate_sources(&sources, None, &QuantityFilter::default()).is_err());
        let missing = [QuantitySource { source_id: "", measurements: empty }];
        assert!(aggregate_sources(&missing, None, &QuantityFilter::default()).is_err());
    }
    #[test]
    fn multi_pdf_empty_sources_and_source_selection() {
        let empty: &[Measurement] = &[];
        let sources = [
            QuantitySource { source_id: "sheet-a", measurements: empty },
            QuantitySource { source_id: "sheet-b", measurements: empty },
        ];
        assert!(aggregate_sources(&sources, None, &QuantityFilter::default()).unwrap().is_empty());
        assert!(aggregate_sources(&sources, Some("sheet-b"), &QuantityFilter::default()).unwrap().is_empty());
        assert!(aggregate_sources(&sources, Some("absent"), &QuantityFilter::default()).is_err());
    }
    #[test]
    fn selected_source_validates_all_source_ids_before_aggregation() {
        let empty: &[Measurement] = &[];
        let sources = [
            QuantitySource { source_id: "selected", measurements: empty },
            QuantitySource { source_id: "selected", measurements: empty },
        ];
        assert!(aggregate_sources(&sources, Some("selected"), &QuantityFilter::default()).is_err());
        let sources = [
            QuantitySource { source_id: "selected", measurements: empty },
            QuantitySource { source_id: " ", measurements: empty },
        ];
        assert!(aggregate_sources(&sources, Some("selected"), &QuantityFilter::default()).is_err());
        let sources = [QuantitySource { source_id: "selected", measurements: empty }];
        assert!(aggregate_sources(&sources, Some("missing"), &QuantityFilter::default()).is_err());
        assert!(aggregate_sources(&sources, Some("selected"), &QuantityFilter::default()).unwrap().is_empty());
    }
    #[test]
    fn hvac_classification_filters_exclude_unclassified_measurements() {
        let scale = crate::Scale::default();
        let points = vec![[0.0, 0.0], [72.0, 0.0]];
        let reading = crate::reading(crate::Kind::Distance, &points, &scale).unwrap();
        let make = |hvac| Measurement {
            visible: true, page: 0, index: 0, id: "id".into(),
            author: "engineer".into(), label: "route".into(),
            points: points.clone(), scale: scale.clone(), reading: reading.clone(), hvac,
        };
        let measurements = [make(Some(crate::hvac::Takeoff::pipe())), make(Some(crate::hvac::Takeoff::duct())), make(None)];
        let filter = QuantityFilter {
            material: Some("COPPER".into()), service: Some("hhw supply".into()),
            profile: Some("PIPE".into()), size: Some("3 IN".into()), ..Default::default()
        };
        let totals = aggregate(&measurements, &filter).unwrap();
        assert_eq!(totals.len(), 1);
        assert_eq!(totals[0].count, 1);
        assert_eq!(totals[0].quantity, reading.value);
        let filter = QuantityFilter { material: Some("unavailable".into()), ..Default::default() };
        assert!(aggregate(&measurements, &filter).unwrap().is_empty());
        let filter = QuantityFilter { service: Some("SA".into()), ..Default::default() };
        assert_eq!(aggregate(&measurements, &filter).unwrap()[0].count, 1);
    }
    #[test]
    fn unit_filter_is_case_sensitive() {
        let filter = QuantityFilter { unit: Some("mm".into()), ..Default::default() };
        assert_eq!(filter.unit.as_deref(), Some("mm"));
        assert_ne!(filter.unit.as_deref(), Some("Mm"));
    }
    #[test]
    fn errors_use_measurement_error_contract() {
        let result: crate::Result<Vec<QuantityTotal>> = aggregate(&[], &QuantityFilter::default());
        assert_eq!(result.unwrap(), Vec::<QuantityTotal>::new());
    }
    #[test]
    fn empty_selection_has_no_totals() {
        assert!(aggregate(&[], &QuantityFilter::default()).unwrap().is_empty());
    }
    #[test]
    fn empty_selection_remains_empty_with_filters() {
        let filter = QuantityFilter { label: Some("pipe".into()), unit: Some("ft".into()), ..Default::default() };
        assert!(aggregate(&[], &filter).unwrap().is_empty());
    }
}
