/*
 * Translated to Rust from LibreOffice Draw, revision
 * bce0998afefdbc355585ca324285661a2170ba77, sd/source/ui/func/fuzoom.cxx.
 * Native event capture, cursor updates and painting belong to the caller.
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

//! Drawing-window decisions for a Draw zoom or pan gesture.

/// A zoom rectangle in Draw logical coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZoomRect {
    pub origin: [i64; 2],
    pub size: [u32; 2],
}

/// The zoom tool's mode when it receives a pointer release.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Zoom,
    Pan,
}

/// The drawing or scrolling action for one move of a captured gesture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragEffect {
    None,
    Outline(ZoomRect),
    /// Apply the scroll and advance the drag anchor to the pointer position.
    Scroll([i64; 2]),
}

/// Logical coordinates supplied after the window processes edge scrolling.
#[derive(Clone, Copy, Debug)]
pub struct DragMove {
    pub mode: Mode,
    pub shift: bool,
    pub outline_visible: bool,
    pub begin: [i32; 2],
    pub end: [i32; 2],
    pub work_size: [u32; 2],
    pub page_size: [u32; 2],
}

impl DragMove {
    /// Port of the geometry decision in `FuZoom::MouseMove`.
    /// Shift switches to pan while a zoom outline has yet to be drawn.
    pub fn effect(self) -> DragEffect {
        if self.mode == Mode::Pan || (self.shift && !self.outline_visible) {
            if self.begin == self.end || self.page_size.contains(&0) {
                return DragEffect::None;
            }
            let divisors = [0, 1].map(|axis| self.work_size[axis] / self.page_size[axis]);
            if divisors.contains(&0) {
                return DragEffect::None;
            }
            return DragEffect::Scroll([0, 1].map(|axis| {
                (i64::from(self.begin[axis]) - i64::from(self.end[axis]))
                    / i64::from(divisors[axis])
            }));
        }

        DragEffect::Outline(ZoomRect {
            origin: [0, 1].map(|axis| i64::from(self.begin[axis].min(self.end[axis]))),
            // tools::Rectangle stores a closed interval between the two points.
            size: [0, 1].map(|axis| self.begin[axis].abs_diff(self.end[axis]).saturating_add(1)),
        })
    }
}

/// Coordinates and modifiers sampled when the button is released.
/// The caller converts from device pixels using its current viewport.
#[derive(Clone, Copy, Debug)]
pub struct Release {
    pub mode: Mode,
    pub shift: bool,
    /// Draw's primary shortcut modifier, which forces a zoom-out at the pointer.
    pub modifier_one: bool,
    pub point: [i32; 2],
    pub output_size: [u32; 2],
    pub selection: ZoomRect,
    pub selection_pixels: [u32; 2],
    pub drag_threshold_pixels: u32,
}

impl Release {
    /// Port of the zoom decision in `FuZoom::MouseButtonUp`.
    /// A pan release has no zoom rectangle. The caller records the final
    /// visible rectangle in history after applying any returned zoom.
    pub fn zoom_rect(self) -> Option<ZoomRect> {
        if self.mode == Mode::Pan || self.shift {
            return None;
        }

        let tolerance = 2 * u64::from(self.drag_threshold_pixels);
        let small_selection = self.selection_pixels.map(u64::from).into_iter()
            .all(|dimension| dimension < tolerance);
        if !small_selection && !self.modifier_one {
            return Some(self.selection);
        }

        let size = self.output_size.map(|dimension| {
            if self.modifier_one {
                dimension.saturating_mul(2)
            } else {
                dimension / 2
            }
        });
        let origin = [0, 1].map(|axis| {
            i64::from(self.point[axis]) - i64::from(size[axis] / 2)
        });
        Some(ZoomRect { origin, size })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release() -> Release {
        Release {
            mode: Mode::Zoom,
            shift: false,
            modifier_one: false,
            point: [500, 400],
            output_size: [801, 601],
            selection: ZoomRect { origin: [10, 20], size: [200, 300] },
            selection_pixels: [3, 4],
            drag_threshold_pixels: 3,
        }
    }

    #[test]
    fn drag_normalizes_the_outline_and_shift_only_switches_before_it_is_visible() {
        let movement = DragMove {
            mode: Mode::Zoom,
            shift: false,
            outline_visible: false,
            begin: [100, 100],
            end: [40, 125],
            work_size: [1200, 1800],
            page_size: [400, 600],
        };
        let outline = DragEffect::Outline(ZoomRect { origin: [40, 100], size: [61, 26] });
        assert_eq!(movement.effect(), outline);
        assert_eq!(DragMove { shift: true, ..movement }.effect(), DragEffect::Scroll([20, -8]));
        assert_eq!(DragMove { shift: true, outline_visible: true, ..movement }.effect(), outline);
        assert_eq!(
            DragMove { mode: Mode::Pan, work_size: [100, 1800], ..movement }.effect(),
            DragEffect::None,
        );
    }

    #[test]
    fn click_doubles_the_zoom_around_the_pointer() {
        assert_eq!(
            release().zoom_rect(),
            Some(ZoomRect { origin: [300, 250], size: [400, 300] }),
        );
    }

    #[test]
    fn reaching_the_threshold_on_either_axis_keeps_the_selection() {
        for selection_pixels in [[6, 0], [0, 6]] {
            let action = Release { selection_pixels, ..release() };
            assert_eq!(action.zoom_rect(), Some(action.selection));
        }
    }

    #[test]
    fn modifier_forces_zoom_out_and_panning_skips_zoom() {
        let action = Release { modifier_one: true, selection_pixels: [200, 300], ..release() };
        assert_eq!(
            action.zoom_rect(),
            Some(ZoomRect { origin: [-301, -201], size: [1602, 1202] }),
        );
        assert_eq!(Release { shift: true, ..action }.zoom_rect(), None);
        assert_eq!(Release { mode: Mode::Pan, ..action }.zoom_rect(), None);
    }
}
