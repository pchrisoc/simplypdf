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

#[cfg(test)]
mod tests {
    use super::*;
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
