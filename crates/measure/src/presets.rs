//! Construction ruler scales at the nominal, full-size sheet dimensions.

use crate::PageUnit;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Preset {
    pub label: &'static str,
    paper: f64,
    page_unit: PageUnit,
    pub real: f64,
    pub unit: &'static str,
}

impl Preset {
    /// The paper side shrinks with the sheet. The real-world side stays the same.
    pub fn paper(self, half_size: bool) -> (f64, PageUnit) {
        (self.paper * if half_size { 0.5 } else { 1.0 }, self.page_unit)
    }
}

const fn architectural(label: &'static str, inches: f64) -> Preset {
    Preset { label, paper: inches, page_unit: PageUnit::In, real: 1.0, unit: "ft" }
}

pub const ARCHITECTURAL: &[Preset] = &[
    architectural("1/32\" = 1'", 1.0 / 32.0),
    architectural("1/16\" = 1'", 1.0 / 16.0),
    architectural("3/32\" = 1'", 3.0 / 32.0),
    architectural("1/8\" = 1'", 1.0 / 8.0),
    architectural("3/16\" = 1'", 3.0 / 16.0),
    architectural("1/4\" = 1'", 1.0 / 4.0),
    architectural("3/8\" = 1'", 3.0 / 8.0),
    architectural("1/2\" = 1'", 1.0 / 2.0),
    architectural("3/4\" = 1'", 3.0 / 4.0),
    architectural("1\" = 1'", 1.0),
    architectural("1-1/2\" = 1'", 1.5),
    architectural("2\" = 1'", 2.0),
    architectural("3\" = 1'", 3.0),
    architectural("6\" = 1'", 6.0),
    architectural("12\" = 1'", 12.0),
];

const fn engineering(label: &'static str, feet: f64) -> Preset {
    Preset { label, paper: 1.0, page_unit: PageUnit::In, real: feet, unit: "ft" }
}

pub const ENGINEERING: &[Preset] = &[
    engineering("1\" = 10'", 10.0),
    engineering("1\" = 20'", 20.0),
    engineering("1\" = 30'", 30.0),
    engineering("1\" = 40'", 40.0),
    engineering("1\" = 50'", 50.0),
    engineering("1\" = 60'", 60.0),
    engineering("1\" = 100'", 100.0),
    engineering("1\" = 200'", 200.0),
];

const fn metric(label: &'static str, ratio: f64) -> Preset {
    Preset { label, paper: 1.0, page_unit: PageUnit::Mm, real: ratio / 1000.0, unit: "m" }
}

pub const METRIC: &[Preset] = &[
    metric("1:1", 1.0),
    metric("1:2", 2.0),
    metric("1:5", 5.0),
    metric("1:10", 10.0),
    metric("1:20", 20.0),
    metric("1:25", 25.0),
    metric("1:50", 50.0),
    metric("1:75", 75.0),
    metric("1:100", 100.0),
    metric("1:125", 125.0),
    metric("1:150", 150.0),
    metric("1:200", 200.0),
    metric("1:250", 250.0),
    metric("1:500", 500.0),
    metric("1:1000", 1000.0),
    metric("1:2000", 2000.0),
    metric("1:5000", 5000.0),
];

pub fn all() -> impl Iterator<Item = Preset> {
    ARCHITECTURAL.iter().chain(ENGINEERING).chain(METRIC).copied()
}
