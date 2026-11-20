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
        if ![self.quantity, self.material_rate, self.labor_rate, self.waste_percent].iter().all(|v| v.is_finite() && *v >= 0.0) {
            return Err("quantity, rates, and waste must be finite and nonnegative".into());
        }
        let billable_quantity = self.quantity * (1.0 + self.waste_percent / 100.0);
        let material_cost = billable_quantity * self.material_rate;
        let labor_cost = self.quantity * self.labor_rate;
        let total_cost = material_cost + labor_cost;
        if ![billable_quantity, material_cost, labor_cost, total_cost].iter().all(|v| v.is_finite()) {
            return Err("rate calculation overflow".into());
        }
        Ok(RateTotal { billable_quantity, material_cost, labor_cost, total_cost, unit: self.unit.clone() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn material_waste_does_not_inflate_labor_quantity() {
        let line = RateLine { quantity: 10.0, unit: "ft".into(), material_rate: 2.0, labor_rate: 3.0, waste_percent: 10.0 };
        let total = line.calculate().unwrap();
        assert_eq!(total.billable_quantity, 11.0);
        assert_eq!(total.material_cost, 22.0);
        assert_eq!(total.labor_cost, 30.0);
        assert_eq!(total.total_cost, 52.0);
    }
    #[test]
    fn invalid_and_overflowing_rates_are_rejected() {
        let mut line = RateLine { quantity: 1.0, unit: "m".into(), material_rate: 1.0, labor_rate: 0.0, waste_percent: 0.0 };
        line.material_rate = f64::NAN;
        assert!(line.calculate().is_err());
        line.material_rate = f64::MAX;
        line.quantity = f64::MAX;
        assert!(line.calculate().is_err());
    }
}
