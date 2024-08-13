// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::Write;

use super::block::FragmentsBlock;
use anstyle::Style;
use pulldown_cmark::CowStr;

/// The definition of a reference link, i.e. a numeric index for a link.
#[derive(Debug, PartialEq)]
pub struct LinkReferenceDefinition<'a> {
    /// The reference index of this link.
    pub(crate) index: u16,
    /// The link target as it appeared in Markdown.
    pub(crate) target: CowStr<'a>,
    /// The link title as it appeared in Markdown.
    pub(crate) title: CowStr<'a>,
    /// The style to use for the link.
    pub(crate) style: Style,
}

/// The state of the current line for render.md.wrapping.
#[derive(Debug)]
pub struct CurrentLine {
    /// The line length
    pub(super) length: u16,
    /// Trailing space to add before continuing this line.
    pub(super) trailing_space: Option<String>,
}

impl CurrentLine {
    /// An empty current line
    pub(super) fn empty() -> Self {
        Self {
            length: 0,
            trailing_space: None,
        }
    }
}

/// Data associated with rendering state.
///
/// Unlike state attributes state data represents cross-cutting
/// concerns which are manipulated across all states.
#[derive(Debug)]
pub struct StateData<'a> {
    /// A list of pending reference link definitions.
    ///
    /// These are links which mdcat already created a reference number for
    /// but didn't yet write out.
    pub(super) pending_link_definitions: Vec<LinkReferenceDefinition<'a>>,
    /// The reference number for the next link.
    pub(super) next_link: u16,
    /// The current block we write to.
    pub(super) current_block: FragmentsBlock<'a>,
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
        style: Style,
    ) -> (Self, u16) {
        let index = self.next_link;
        self.next_link += 1;
        self.pending_link_definitions.push(LinkReferenceDefinition {
            index,
            target,
            title,
            style,
        });
        (self, index)
    }

    pub(crate) fn write_and_flush_block(
        mut self,
        writer: &mut dyn Write,
        columns: f64,
    ) -> std::io::Result<Self> {
        let tokens = self.current_block.wrap(columns).render();
        super::block::write_tokens(writer, &tokens)?;
        self.current_block = FragmentsBlock::default();
        Ok(self)
    }

    pub(crate) fn take_links(self) -> (Self, Vec<LinkReferenceDefinition<'a>>) {
        let links = self.pending_link_definitions;
        (
            StateData {
                pending_link_definitions: Vec::new(),
                ..self
            },
            links,
        )
    }
}

impl<'a> Default for StateData<'a> {
    fn default() -> Self {
        StateData {
            pending_link_definitions: Vec::new(),
            next_link: 1,
            current_block: FragmentsBlock::default(),
        }
    }
}
