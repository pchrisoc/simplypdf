/*
 * Translated to Rust and adapted for SimplyPDF from the LibreOffice project.
 * Upstream bce0998afefdbc355585ca324285661a2170ba77, sd/source/ui/view/zoomlist.cxx
 * and sd/source/ui/inc/zoomlist.hxx. See docs/libreoffice-draw-port.md.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * This file incorporates work covered by the following license notice:
 *
 *   Licensed to the Apache Software Foundation (ASF) under one or more
 *   contributor license agreements. See the NOTICE file distributed
 *   with this work for additional information regarding copyright
 *   ownership. The ASF licenses this file to you under the Apache
 *   License, Version 2.0 (the "License"); you may not use this file
 *   except in compliance with the License. You may obtain a copy of
 *   the License at https://www.apache.org/licenses/LICENSE-2.0 .
 */

const MAX_ENTRIES: usize = 10;

/// Draw's bounded view history. The caller supplies its own view state and
/// refreshes command availability after modifying the history.
#[derive(Debug)]
pub struct ZoomHistory<T> {
    entries: Vec<T>,
    position: usize,
}

impl<T> Default for ZoomHistory<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            position: 0,
        }
    }
}

impl<T> ZoomHistory<T> {
    /// Insert at Draw's current history position. Forward entries are retained.
    /// At capacity, Draw first removes the oldest entry and reuses the index.
    pub fn insert(&mut self, view: T) {
        if self.entries.len() == MAX_ENTRIES {
            self.entries.remove(0);
        } else if !self.entries.is_empty() {
            self.position += 1;
        }
        self.entries.insert(self.position, view);
    }

    pub fn current(&self) -> Option<&T> {
        self.entries.get(self.position)
    }

    pub fn can_step_forward(&self) -> bool {
        self.position + 1 < self.entries.len()
    }

    pub fn can_step_back(&self) -> bool {
        self.position > 0
    }

    /// Advance when possible. At the boundary, return the current view.
    pub fn step_forward(&mut self) -> Option<&T> {
        if self.can_step_forward() {
            self.position += 1;
        }
        self.current()
    }

    /// Step back when possible. At the boundary, return the current view.
    pub fn step_back(&mut self) -> Option<&T> {
        self.position = self.position.saturating_sub(1);
        self.current()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_history_and_boundaries_are_safe() {
        let mut history = ZoomHistory::default();
        assert_eq!(history.step_back(), None);
        assert_eq!(history.step_forward(), None);
        history.insert("fit");
        history.insert("detail");
        assert_eq!(history.step_forward(), Some(&"detail"));
        assert!(history.can_step_back());
        assert_eq!(history.step_back(), Some(&"fit"));
        assert_eq!(history.step_back(), Some(&"fit"));
        assert!(history.can_step_forward());
        assert_eq!(history.step_forward(), Some(&"detail"));
    }

    #[test]
    fn insertion_after_stepping_back_retains_forward_views() {
        let mut history = ZoomHistory::default();
        for view in ["fit", "detail a", "detail b"] {
            history.insert(view);
        }
        assert_eq!(history.step_back(), Some(&"detail a"));
        history.insert("new detail");
        assert_eq!(history.current(), Some(&"new detail"));
        assert_eq!(history.step_forward(), Some(&"detail b"));
    }

    #[test]
    fn full_history_evicts_the_oldest_and_reuses_the_cursor_index() {
        let mut history = ZoomHistory::default();
        for view in 0..10 {
            history.insert(view);
        }
        assert_eq!(history.step_back(), Some(&8));
        history.insert(10);
        assert_eq!(history.current(), Some(&10));
        assert_eq!(history.step_back(), Some(&8));
        assert_eq!(history.step_forward(), Some(&10));
        assert_eq!(history.step_forward(), Some(&9));

        let mut oldest = 9;
        let mut entries = 1;
        while history.can_step_back() {
            oldest = *history.step_back().unwrap();
            entries += 1;
        }
        assert_eq!(oldest, 1);
        assert_eq!(entries, 10);
    }
}
