//! Filter and aggregate measured quantities without mixing incompatible units.
use crate::Measurement;
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

pub fn aggregate(measurements: &[Measurement], filter: &QuantityFilter) -> Result<Vec<QuantityTotal>, String> {
    let mut totals: BTreeMap<String, (usize, f64)> = BTreeMap::new();
    for m in measurements {
        if filter.label.as_ref().is_some_and(|s| !m.label.eq_ignore_ascii_case(s)) ||
            filter.author.as_ref().is_some_and(|s| !m.author.eq_ignore_ascii_case(s)) ||
            filter.unit.as_ref().is_some_and(|s| !m.reading.unit.eq_ignore_ascii_case(s)) {
            continue;
        }
        let value = m.reading.value;
        if !value.is_finite() || value < 0.0 {
            return Err("invalid quantity in filtered takeoff".into());
        }
        let entry = totals.entry(m.reading.unit.clone()).or_insert((0, 0.0));
        entry.0 += 1;
        entry.1 += value;
        if !entry.1.is_finite() {
            return Err("filtered quantity overflow".into());
        }
    }
    Ok(totals.into_iter().map(|(unit, (count, quantity))| QuantityTotal { unit, count, quantity }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
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
