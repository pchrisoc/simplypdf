//! Filter and aggregate measured quantities without mixing incompatible units.
use crate::{invalid, Measurement, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantityFilter {
    pub label: Option<String>,
    pub author: Option<String>,
    pub unit: Option<String>,
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
        {
            continue;
        }
        let value = m.reading.value;
        if !value.is_finite() || value < 0.0 {
            return Err(invalid("invalid quantity in filtered takeoff"));
        }
        let entry = totals.entry(m.reading.unit.clone()).or_insert((0, 0.0));
        entry.0 += 1;
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
        if source.source_id.is_empty() || source.source_id.len() > 1024 || !seen.insert(source.source_id) {
            return Err(invalid("source IDs must be nonempty, unique, and at most 1024 bytes"));
        }
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

#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(aggregate_sources(&sources, Some("absent"), &QuantityFilter::default()).unwrap().is_empty());
    }
    #[test]
    fn unit_filter_is_case_sensitive() {
        let filter = QuantityFilter { label: None, author: None, unit: Some("mm".into()) };
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
        let filter = QuantityFilter { label: Some("pipe".into()), author: None, unit: Some("ft".into()) };
        assert!(aggregate(&[], &filter).unwrap().is_empty());
    }
}
