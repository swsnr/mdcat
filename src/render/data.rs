// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use ansi_term::Colour;
use pulldown_cmark::CowStr;

#[derive(Debug, PartialEq)]
pub struct Link<'a> {
    pub(crate) index: u16,
    pub(crate) target: CowStr<'a>,
    pub(crate) title: CowStr<'a>,
    pub(crate) colour: Colour,
}

/// Data associated with rendering state.
///
/// Unlike state attributes state data represents cross-cutting
/// concerns which are manipulated across all states.
#[derive(Debug)]
pub struct StateData<'a> {
    /// A list of pending reference links.
    ///
    /// These are links which mdcat already created a reference number for
    /// but didn't yet write out.
    pub(super) pending_links: Vec<Link<'a>>,
    /// The reference number for the next link.
    pub(super) next_link: u16,
}

impl<'a> StateData<'a> {
    /// Add a pending link to the state data.
    ///
    /// `target` is the link target, and `title` the link title to show after the URL.
    /// `colour` is the colour to use for foreground text to differentiate between
    /// different types of links.
    pub(crate) fn add_link(
        mut self,
        target: CowStr<'a>,
        title: CowStr<'a>,
        colour: Colour,
    ) -> (Self, u16) {
        let index = self.next_link;
        self.next_link += 1;
        self.pending_links.push(Link {
            index,
            target,
            title,
            colour,
        });
        (self, index)
    }

    pub(crate) fn take_links(self) -> (Self, Vec<Link<'a>>) {
        let links = self.pending_links;
        (
            StateData {
                pending_links: Vec::new(),
                ..self
            },
            links,
        )
    }
}

impl<'a> Default for StateData<'a> {
    fn default() -> Self {
        StateData {
            pending_links: Vec::new(),
            next_link: 1,
        }
    }
}
