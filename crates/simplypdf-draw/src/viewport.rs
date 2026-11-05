/*
 * Translated to Rust from LibreOffice Draw, revision
 * bce0998afefdbc355585ca324285661a2170ba77, sd/source/ui/view/sdwindow.cxx.
 * The VCL coordinate conversion and window effects belong to the caller.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * This file incorporates work covered by the following license notice:
 *
 *   Licensed to the Apache Software Foundation (ASF) under one or more
 *   contributor license agreements. See the NOTICE file distributed
 *   with this work for additional information regarding copyright
 *   ownership. The ASF licenses this file to you under the Apache
 *   License, Version 2.0 (the "License"); you may not use this file
 *   except in compliance with the License. You may obtain a copy of
 *   the License at http://www.apache.org/licenses/LICENSE-2.0 .
 */

//! Draw's rectangle zoom calculations, independent of a native window.

const ZOOM_MULTIPLIER: u64 = 10_000;
const MAX_ZOOM: u16 = 3_000;
const DEFAULT_ZOOM: u16 = 100;

/// Zoom state and the output size at the current zoom, in Draw logical units.
#[derive(Debug, Clone, Copy)]
pub struct ZoomWindow {
    output_size: [u32; 2],
    current_zoom: u16,
    minimum_zoom: u16,
}

/// Effects the caller applies to its window after fitting a rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitRect {
    /// The rectangle has a zero dimension. Center using integral zoom.
    IntegralZoom(u16),
    /// The fit factor is too small to represent at Draw's precision.
    Unchanged(u16),
    /// Set the origin and update the window's zoom.
    Centered { origin: [i64; 2], zoom: u16 },
}

impl ZoomWindow {
    pub fn new(output_size: [u32; 2], current_zoom: u16, minimum_zoom: u16) -> Option<Self> {
        if minimum_zoom == 0 || !(minimum_zoom..=MAX_ZOOM).contains(&current_zoom) {
            return None;
        }
        Some(Self { output_size, current_zoom, minimum_zoom })
    }

    /// Port of `Window::GetZoomForRect`. Both sizes use the same logical units.
    pub fn zoom_for_rect(self, size: [u32; 2]) -> u16 {
        if size.contains(&0) {
            return DEFAULT_ZOOM;
        }
        let factor = self.fit_factor(size);
        if factor == 0 {
            return self.current_zoom;
        }
        self.clamp_zoom(self.scaled_zoom(factor))
    }

    /// Port of the zoom and position calculation in `Window::SetZoomRect`.
    /// Origins use the same logical units as sizes. The result is a window
    /// position, with each axis constrained to be nonnegative.
    pub fn fit_rect(
        self,
        view_origin: [i64; 2],
        rect_origin: [i64; 2],
        size: [u32; 2],
    ) -> FitRect {
        if size.contains(&0) {
            return FitRect::IntegralZoom(DEFAULT_ZOOM);
        }

        let mut factor = self.fit_factor(size);
        if factor == 0 {
            return FitRect::Unchanged(self.current_zoom);
        }
        let zoom = self.scaled_zoom(factor);
        if zoom > u64::from(MAX_ZOOM) {
            factor = factor * u64::from(MAX_ZOOM) / zoom;
        }

        // Draw adjusts the centering factor for the upper bound. Its lower
        // bound is applied later when setting the zoom, after positioning.
        let origin = [0, 1].map(|axis| {
            let visible = (f64::from(self.output_size[axis]) * ZOOM_MULTIPLIER as f64
                / factor as f64) as i64;
            let adjustment = (i64::from(size[axis]) - visible) / 2;
            window_position(i128::from(view_origin[axis]) + i128::from(rect_origin[axis]) + i128::from(adjustment))
        });
        FitRect::Centered { origin, zoom: self.clamp_zoom(zoom) }
    }

    /// Port of `Window::SetZoomIntegral` without changing a native window.
    /// Returns the centered origin and clamped zoom to apply.
    pub fn integral_zoom(self, origin: [i64; 2], requested_zoom: i32) -> ([i64; 2], u16) {
        let zoom = requested_zoom.clamp(i32::from(self.minimum_zoom), i32::from(MAX_ZOOM)) as u16;
        let centered = [0, 1].map(|axis| {
            let old_size = i64::from(self.output_size[axis]);
            let new_size = old_size * i64::from(self.current_zoom) / i64::from(zoom);
            window_position(i128::from(origin[axis]) + i128::from((old_size - new_size) / 2))
        });
        (centered, zoom)
    }

    fn fit_factor(self, [width, height]: [u32; 2]) -> u64 {
        let x = (f64::from(self.output_size[0]) * ZOOM_MULTIPLIER as f64 / f64::from(width)) as u64;
        let y = (f64::from(self.output_size[1]) * ZOOM_MULTIPLIER as f64 / f64::from(height)) as u64;
        x.min(y)
    }

    fn scaled_zoom(self, factor: u64) -> u64 {
        factor * u64::from(self.current_zoom) / ZOOM_MULTIPLIER
    }

    fn clamp_zoom(self, zoom: u64) -> u16 {
        zoom.clamp(u64::from(self.minimum_zoom), u64::from(MAX_ZOOM)) as u16
    }
}

fn window_position(value: i128) -> i64 {
    value.clamp(0, i128::from(i64::MAX)) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_uses_the_tighter_axis_and_centers_the_rectangle() {
        let window = ZoomWindow::new([800, 600], 100, 5).unwrap();
        assert_eq!(window.zoom_for_rect([400, 400]), 150);
        assert_eq!(
            window.fit_rect([10, 20], [100, 100], [400, 400]),
            FitRect::Centered { origin: [44, 120], zoom: 150 },
        );
        assert_eq!(window.zoom_for_rect([300, 400]), 150);
        assert_eq!(window.zoom_for_rect([500, 100]), 160);
    }

    #[test]
    fn applies_zoom_bounds_and_nonnegative_window_origins() {
        let window = ZoomWindow::new([800, 600], 100, 5).unwrap();
        assert_eq!(window.zoom_for_rect([1, 1]), 3000);
        assert_eq!(
            window.fit_rect([0, 0], [100, 200], [1, 1]),
            FitRect::Centered { origin: [88, 191], zoom: 3000 },
        );
        assert_eq!(
            window.fit_rect([0, 0], [-100, -100], [100_000, 100_000]),
            FitRect::Centered { origin: [0, 0], zoom: 5 },
        );
    }

    #[test]
    fn integral_zoom_keeps_the_center_and_clamps_zoom_and_position() {
        let window = ZoomWindow::new([800, 600], 100, 5).unwrap();
        assert_eq!(window.integral_zoom([20, 30], 200), ([220, 180], 200));
        assert_eq!(window.integral_zoom([20, 30], 0), ([0, 0], 5));
        assert_eq!(window.integral_zoom([20, 30], 5000), ([407, 320], 3000));
    }

    #[test]
    fn preserves_draws_two_degenerate_cases() {
        let window = ZoomWindow::new([1, 1], 120, 5).unwrap();
        assert_eq!(window.zoom_for_rect([0, 200]), 100);
        assert_eq!(window.fit_rect([0, 0], [0, 0], [0, 200]), FitRect::IntegralZoom(100));
        assert_eq!(window.zoom_for_rect([20_000, 20_000]), 120);
        assert_eq!(window.fit_rect([0, 0], [0, 0], [20_000, 20_000]), FitRect::Unchanged(120));
        assert!(ZoomWindow::new([1, 1], 0, 5).is_none());
    }
}
