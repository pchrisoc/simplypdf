//! Unit-aware material and labor rate calculations for takeoff estimates.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RateLine {
    pub quantity: f64,
    pub unit: String,
    pub material_rate: f64,
    pub labor_rate: f64,
    pub waste_percent: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RateTotal {
    pub billable_quantity: f64,
    pub material_cost: f64,
    pub labor_cost: f64,
    pub total_cost: f64,
    pub unit: String,
}

impl RateLine {
    pub fn calculate(&self) -> Result<RateTotal, String> {
        if self.unit.trim().is_empty() {
            return Err("unit is required".into());
        }
        if ![
            self.quantity,
            self.material_rate,
            self.labor_rate,
            self.waste_percent,
        ]
        .iter()
        .all(|v| v.is_finite() && *v >= 0.0)
        {
            return Err("quantity, rates, and waste must be finite and nonnegative".into());
        }
        let billable_quantity = self.quantity * (1.0 + self.waste_percent / 100.0);
        let material_cost = billable_quantity * self.material_rate;
        let labor_cost = self.quantity * self.labor_rate;
        let total_cost = material_cost + labor_cost;
        if ![billable_quantity, material_cost, labor_cost, total_cost]
            .iter()
            .all(|v| v.is_finite())
        {
            return Err("rate calculation overflow".into());
        }
        Ok(RateTotal {
            billable_quantity,
            material_cost,
            labor_cost,
            total_cost,
            unit: self.unit.clone(),
        })
    }
}

/// Sum an estimate only when every line has the same physical unit.
/// Currency totals from unlike units may be combined separately, but quantities may not.
pub fn aggregate_rates(lines: &[RateLine]) -> Result<Option<RateTotal>, String> {
    let Some(first) = lines.first() else {
        return Ok(None);
    };
    let unit = first.unit.trim();
    let mut total = RateTotal {
        billable_quantity: 0.0,
        material_cost: 0.0,
        labor_cost: 0.0,
        total_cost: 0.0,
        unit: unit.to_owned(),
    };
    for line in lines {
        if line.unit.trim() != unit {
            return Err("cannot combine quantities with different units".into());
        }
        let next = line.calculate()?;
        total.billable_quantity += next.billable_quantity;
        total.material_cost += next.material_cost;
        total.labor_cost += next.labor_cost;
        total.total_cost += next.total_cost;
        if ![
            total.billable_quantity,
            total.material_cost,
            total.labor_cost,
            total.total_cost,
        ]
        .iter()
        .all(|value| value.is_finite())
        {
            return Err("rate aggregation overflow".into());
        }
    }
    Ok(Some(total))
}


/// Aggregate mixed-unit takeoffs into separate quantity totals for each unit.
/// Material and labor costs remain attached to the corresponding physical unit.
pub fn aggregate_rates_by_unit(lines: &[RateLine]) -> Result<Vec<RateTotal>, String> {
    let mut groups = std::collections::BTreeMap::<String, RateTotal>::new();
    for line in lines {
        let next = line.calculate()?;
        let unit = line.unit.trim();
        let total = groups.entry(unit.to_owned()).or_insert_with(|| RateTotal {
            billable_quantity: 0.0,
            material_cost: 0.0,
            labor_cost: 0.0,
            total_cost: 0.0,
            unit: unit.to_owned(),
        });
        total.billable_quantity += next.billable_quantity;
        total.material_cost += next.material_cost;
        total.labor_cost += next.labor_cost;
        total.total_cost += next.total_cost;
        if ![
            total.billable_quantity,
            total.material_cost,
            total.labor_cost,
            total.total_cost,
        ]
        .iter()
        .all(|value| value.is_finite())
        {
            return Err("rate aggregation overflow".into());
        }
    }
    Ok(groups.into_values().collect())
}

/// Currency-only totals can combine rates for different physical units.
/// The caller must ensure every rate is denominated in the same currency.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EstimateCost {
    pub material_cost: f64,
    pub labor_cost: f64,
    pub total_cost: f64,
}

/// Aggregate estimate costs without adding incompatible physical quantities.
pub fn aggregate_estimate_cost(lines: &[RateLine]) -> Result<EstimateCost, String> {
    let mut costs = EstimateCost {
        material_cost: 0.0,
        labor_cost: 0.0,
        total_cost: 0.0,
    };
    for line in lines {
        let next = line.calculate()?;
        costs.material_cost += next.material_cost;
        costs.labor_cost += next.labor_cost;
        costs.total_cost += next.total_cost;
        if ![costs.material_cost, costs.labor_cost, costs.total_cost]
            .iter()
            .all(|value| value.is_finite())
        {
            return Err("estimate cost aggregation overflow".into());
        }
    }
    Ok(costs)
}


/// A named rate schedule entry, independent of the measurement's physical quantity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RateScheduleEntry {
    pub code: String,
    pub unit: String,
    pub material_rate: f64,
    pub labor_rate: f64,
    pub waste_percent: f64,
}

/// Resolve a takeoff against an editable schedule, rejecting ambiguous codes and
/// incompatible units instead of silently applying a rate for the wrong unit.
pub fn rate_line_from_schedule(
    quantity: f64,
    unit: &str,
    code: &str,
    schedule: &[RateScheduleEntry],
) -> Result<RateLine, String> {
    let code = code.trim();
    if code.is_empty() {
        return Err("rate code is required".into());
    }
    let mut match_entry = None;
    for entry in schedule {
        if entry.code.trim() == code {
            if match_entry.is_some() {
                return Err("duplicate rate code".into());
            }
            match_entry = Some(entry);
        }
    }
    let entry = match_entry.ok_or_else(|| "rate code not found".to_owned())?;
    if entry.unit.trim().is_empty() || entry.unit.trim() != unit.trim() {
        return Err("rate unit does not match measurement unit".into());
    }
    let line = RateLine {
        quantity,
        unit: unit.trim().to_owned(),
        material_rate: entry.material_rate,
        labor_rate: entry.labor_rate,
        waste_percent: entry.waste_percent,
    };
    line.calculate()?;
    Ok(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(quantity: f64, unit: &str) -> RateLine {
        RateLine {
            quantity,
            unit: unit.into(),
            material_rate: 2.0,
            labor_rate: 3.0,
            waste_percent: 10.0,
        }
    }


    #[test]
    fn schedule_resolves_named_rate_and_calculates_cost() {
        let schedule = [RateScheduleEntry {
            code: "P-100".into(),
            unit: "ft".into(),
            material_rate: 4.0,
            labor_rate: 6.0,
            waste_percent: 5.0,
        }];
        let line = rate_line_from_schedule(10.0, " ft ", " P-100 ", &schedule).unwrap();
        let total = line.calculate().unwrap();
        assert_eq!(total.billable_quantity, 10.5);
        assert_eq!(total.material_cost, 42.0);
        assert_eq!(total.labor_cost, 60.0);
        assert_eq!(total.total_cost, 102.0);
    }

    #[test]
    fn schedule_rejects_unknown_ambiguous_incompatible_and_invalid_rates() {
        let entry = RateScheduleEntry {
            code: "A".into(),
            unit: "ft".into(),
            material_rate: 2.0,
            labor_rate: 3.0,
            waste_percent: 0.0,
        };
        assert!(rate_line_from_schedule(1.0, "ft", "missing", &[entry.clone()]).is_err());
        assert!(rate_line_from_schedule(1.0, "m", "A", &[entry.clone()]).is_err());
        assert!(rate_line_from_schedule(1.0, "ft", "A", &[entry.clone(), entry.clone()]).is_err());
        assert!(rate_line_from_schedule(f64::NAN, "ft", "A", &[entry.clone()]).is_err());
        let mut invalid = entry;
        invalid.material_rate = -1.0;
        assert!(rate_line_from_schedule(1.0, "ft", "A", &[invalid]).is_err());
    }

    #[test]
    fn mixed_units_can_sum_costs_but_not_quantities() {
        let lines = [line(10.0, "ft"), line(2.0, "each")];
        assert!(aggregate_rates(&lines).is_err());
        let costs = aggregate_estimate_cost(&lines).unwrap();
        assert_eq!(costs.material_cost, 26.4);
        assert_eq!(costs.labor_cost, 36.0);
        assert!((costs.total_cost - 62.4).abs() < 1e-10);
        assert_eq!(
            aggregate_estimate_cost(&[]).unwrap(),
            EstimateCost {
                material_cost: 0.0,
                labor_cost: 0.0,
                total_cost: 0.0,
            }
        );
    }

    #[test]
    fn estimate_cost_rejects_invalid_lines_and_overflow() {
        let mut invalid = line(1.0, "ft");
        invalid.labor_rate = f64::INFINITY;
        assert!(aggregate_estimate_cost(&[invalid]).is_err());
        let mut huge = line(f64::MAX / 4.0, "ft");
        huge.material_rate = 1.0;
        huge.labor_rate = 1.0;
        huge.waste_percent = 0.0;
        assert!(aggregate_estimate_cost(&[
            huge.clone(),
            huge.clone(),
            huge.clone(),
            huge.clone(),
            huge,
        ])
        .is_err());
    }

    #[test]
    fn grouped_estimates_keep_units_separate_and_sorted() {
        let lines = [line(2.0, "each"), line(10.0, "ft"), line(5.0, "ft")];
        let groups = aggregate_rates_by_unit(&lines).unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].unit, "each");
        assert_eq!(groups[0].billable_quantity, 2.2);
        assert_eq!(groups[1].unit, "ft");
        assert_eq!(groups[1].billable_quantity, 16.5);
        assert_eq!(groups[1].total_cost, 78.0);
        assert!(aggregate_rates_by_unit(&[]).unwrap().is_empty());
        let mut invalid = line(1.0, "ft");
        invalid.quantity = f64::NAN;
        assert!(aggregate_rates_by_unit(&[invalid]).is_err());
    }

    #[test]
    fn grouped_estimates_trim_units_and_reject_group_overflow() {
        let lines = [line(2.0, " ft "), line(3.0, "ft")];
        let groups = aggregate_rates_by_unit(&lines).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].unit, "ft");
        assert_eq!(groups[0].billable_quantity, 5.5);
        let mut huge = line(f64::MAX / 4.0, "ft");
        huge.material_rate = 1.0;
        huge.labor_rate = 1.0;
        huge.waste_percent = 0.0;
        assert!(aggregate_rates_by_unit(&[
            huge.clone(),
            huge.clone(),
            huge.clone(),
            huge.clone(),
            huge,
        ])
        .is_err());
    }

    #[test]
    fn material_waste_does_not_inflate_labor_quantity() {
        let total = line(10.0, "ft").calculate().unwrap();
        assert_eq!(total.billable_quantity, 11.0);
        assert_eq!(total.material_cost, 22.0);
        assert_eq!(total.labor_cost, 30.0);
        assert_eq!(total.total_cost, 52.0);
    }

    #[test]
    fn invalid_and_overflowing_rates_are_rejected() {
        let mut value = line(1.0, "m");
        value.material_rate = f64::NAN;
        assert!(value.calculate().is_err());
        value.material_rate = f64::MAX;
        value.quantity = f64::MAX;
        assert!(value.calculate().is_err());
    }

    #[test]
    fn aggregates_same_unit_and_rejects_mixed_units() {
        assert_eq!(aggregate_rates(&[]).unwrap(), None);
        let total = aggregate_rates(&[line(10.0, "ft"), line(5.0, "ft")])
            .unwrap()
            .unwrap();
        assert_eq!(total.billable_quantity, 16.5);
        assert_eq!(total.material_cost, 33.0);
        assert_eq!(total.labor_cost, 45.0);
        assert_eq!(total.total_cost, 78.0);
        assert!(aggregate_rates(&[line(1.0, "ft"), line(1.0, "m")]).is_err());
    }

    #[test]
    fn aggregation_rejects_nonfinite_sum() {
        let mut huge = line(f64::MAX / 4.0, "ft");
        huge.material_rate = 1.0;
        huge.labor_rate = 1.0;
        huge.waste_percent = 0.0;
        assert!(huge.calculate().is_ok());
        assert!(aggregate_rates(&[
            huge.clone(),
            huge.clone(),
            huge.clone(),
            huge.clone(),
            huge,
        ])
        .is_err());
    }
}
