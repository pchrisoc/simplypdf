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
        assert!(aggregate_rates(&[huge.clone(), huge.clone(), huge.clone(), huge.clone(), huge]).is_err());
    }
}
