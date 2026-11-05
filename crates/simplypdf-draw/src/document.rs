/*
 * Translated and adapted from LibreOffice Draw at
 * bce0998afefdbc355585ca324285661a2170ba77. Sources: sd/source/core/drawdoc.cxx,
 * sd/source/core/drawdoc2.cxx and svx/source/svdraw/svdlayer.cxx.
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

//! Standard Draw pages and the document's layer table.

pub const A4_PORTRAIT: [u32; 2] = [21_000, 29_700];
pub const DEFAULT_MARGIN: u32 = 1_000;

const DEFAULT_LAYERS: [&str; 5] = [
    "layout", "background", "backgroundobjects", "controls", "measurelines",
];

/// All page measurements are in hundredths of a millimeter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub size: [u32; 2],
    pub margins: [u32; 4],
    pub shapes: Vec<Shape>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    Rectangle,
    Ellipse,
}

/// A normalized bounding rectangle on a specific Draw layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shape {
    pub kind: ShapeKind,
    pub layer: u8,
    pub min: [u32; 2],
    pub max: [u32; 2],
}

impl Shape {
    pub fn new(kind: ShapeKind, layer: u8, first: [u32; 2], second: [u32; 2]) -> Option<Self> {
        let min = [0, 1].map(|axis| first[axis].min(second[axis]));
        let max = [0, 1].map(|axis| first[axis].max(second[axis]));
        if min[0] == max[0] || min[1] == max[1] {
            return None;
        }
        Some(Self { kind, layer, min, max })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer {
    pub id: u8,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pages: Vec<Page>,
    layers: Vec<Layer>,
}

impl Document {
    /// Standard drawing page from Draw's new-document path without a printer.
    /// The caller provides the platform's paper size. Master, notes, and
    /// handout pages are separate model types still to be ported.
    pub fn new(paper: [u32; 2]) -> Self {
        let mut document = Self {
            pages: vec![Page { size: paper, margins: [DEFAULT_MARGIN; 4], shapes: Vec::new() }],
            layers: Vec::new(),
        };
        for name in DEFAULT_LAYERS {
            document.add_layer(name.into()).expect("five default layers fit");
        }
        document
    }

    pub fn pages(&self) -> &[Page] {
        &self.pages
    }

    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    pub fn control_layer(&self) -> u8 {
        3
    }

    /// Append a document layer with the first free Draw layer ID.
    /// ID 255 is reserved. Exhaustion is explicit in the Rust API.
    pub fn add_layer(&mut self, name: String) -> Option<u8> {
        let mut used = [false; 255];
        for layer in &self.layers {
            used[usize::from(layer.id)] = true;
        }
        let id = used.iter().position(|used| !used)? as u8;
        self.layers.push(Layer { id, name });
        Some(id)
    }

    /// Add a blank standard page using the active page's format.
    pub fn add_page_after(&mut self, index: usize) -> Option<usize> {
        let page = self.pages.get(index)?;
        let next = index + 1;
        self.pages.insert(next, Page { size: page.size, margins: page.margins, shapes: Vec::new() });
        Some(next)
    }

    pub fn add_shape(&mut self, page: usize, shape: Shape) -> bool {
        if !self.layers.iter().any(|layer| layer.id == shape.layer) {
            return false;
        }
        let Some(page) = self.pages.get_mut(page) else { return false };
        if (0..2).any(|axis| shape.min[axis] >= shape.max[axis] || shape.max[axis] > page.size[axis]) {
            return false;
        }
        page.shapes.push(shape);
        true
    }

    pub fn pop_shape(&mut self, page: usize) -> Option<Shape> {
        self.pages.get_mut(page)?.shapes.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_draw_layers_in_order_and_uses_no_printer_margins() {
        let doc = Document::new(A4_PORTRAIT);
        assert_eq!(doc.layers().iter().map(|layer| layer.name.as_str()).collect::<Vec<_>>(), DEFAULT_LAYERS);
        assert_eq!(doc.layers()[doc.control_layer() as usize].name, "controls");
        assert_eq!(doc.pages()[0].size, A4_PORTRAIT);
        assert_eq!(doc.pages()[0].margins, [1_000; 4]);
    }

    #[test]
    fn document_layer_ids_do_not_use_the_reserved_value() {
        let mut doc = Document::new(A4_PORTRAIT);
        for id in 5..255 {
            assert_eq!(doc.add_layer(format!("Layer {id}")), Some(id));
        }
        assert_eq!(doc.add_layer("overflow".into()), None);
    }

    #[test]
    fn new_page_copies_format_and_shapes_remain_with_their_page() {
        let mut doc = Document::new(A4_PORTRAIT);
        let shape = Shape::new(ShapeKind::Rectangle, 0, [200, 700], [100, 300]).unwrap();
        assert_eq!(shape.min, [100, 300]);
        assert!(doc.add_shape(0, shape));
        assert_eq!(doc.add_page_after(0), Some(1));
        assert_eq!(doc.pages()[0].shapes, [shape]);
        assert!(doc.pages()[1].shapes.is_empty());
        assert_eq!(doc.pages()[1].size, A4_PORTRAIT);
        assert_eq!(doc.pop_shape(0), Some(shape));
        assert!(Shape::new(ShapeKind::Ellipse, 0, [0, 1], [0, 3]).is_none());
    }
}
