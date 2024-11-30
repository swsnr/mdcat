// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use anstyle::Style;
use pulldown_cmark::{Alignment, CowStr, LinkType};

/// A pending link.
#[derive(Debug, PartialEq)]
pub struct PendingLink<'a> {
    /// The type of this link.
    pub(crate) link_type: LinkType,
    /// The destination URL of this link.
    pub(crate) dest_url: CowStr<'a>,
    /// The link title as it appeared in Markdown.
    pub(crate) title: CowStr<'a>,
}

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

/// A cell in the table.
#[derive(Debug)]
pub struct TableCell<'a> {
    // TODO: Support styles of fragments.
    /// Renderable fragments in a table cell.
    pub(super) fragments: Vec<CowStr<'a>>,
}

impl TableCell<'_> {
    /// A new empty table cell.
    pub(super) fn empty() -> Self {
        Self {
            fragments: Vec::new(),
        }
    }
}

/// A row in the table.
#[derive(Debug)]
pub struct TableRow<'a> {
    /// Completed cells of the table row.
    pub(super) cells: Vec<TableCell<'a>>,
    /// Current incomplete cell of the table row.
    pub(super) current_cell: TableCell<'a>,
}

impl TableRow<'_> {
    /// A new empty table row.
    pub(super) fn empty() -> Self {
        Self {
            cells: Vec::new(),
            current_cell: TableCell::empty(),
        }
    }
}

/// The state of the current table.
#[derive(Debug)]
pub struct CurrentTable<'a> {
    /// Head row of the table.
    pub(super) head: Option<TableRow<'a>>,
    /// Complete rows of the table.
    pub(super) rows: Vec<TableRow<'a>>,
    /// Current incomplete row of the table.
    pub(super) current_row: TableRow<'a>,
    /// Alignments of columns.
    pub(super) alignments: Vec<Alignment>,
}

impl<'a> CurrentTable<'a> {
    /// A new empty table.
    pub(super) fn empty() -> Self {
        Self {
            head: None,
            rows: Vec::new(),
            current_row: TableRow::empty(),
            alignments: Vec::new(),
        }
    }

    /// Push a fragment to the current cell of the current row.
    pub(super) fn push_fragment(mut self, fragment: CowStr<'a>) -> Self {
        self.current_row.current_cell.fragments.push(fragment);
        self
    }

    /// Complete the current cell and start a new cell in the current row.
    pub(super) fn end_cell(mut self) -> Self {
        self.current_row.cells.push(self.current_row.current_cell);
        self.current_row.current_cell = TableCell::empty();
        self
    }

    /// Complete the head row and start a new row.
    pub(super) fn end_head(mut self) -> Self {
        self.head = Some(self.current_row);
        self.current_row = TableRow::empty();
        self
    }

    /// Complete the current row and start a new row.
    pub(super) fn end_row(mut self) -> Self {
        self.rows.push(self.current_row);
        self.current_row = TableRow::empty();
        self
    }
}

/// Data associated with rendering state.
///
/// Unlike state attributes state data represents cross-cutting
/// concerns which are manipulated across all states.
#[derive(Debug)]
pub struct StateData<'a> {
    /// A list of pending links.
    ///
    /// These are links which we still need to create a reference number for.
    pub(super) pending_links: Vec<PendingLink<'a>>,
    /// A list of pending reference link definitions.
    ///
    /// These are links which mdcat already created a reference number for
    /// but didn't yet write out.
    pub(super) pending_link_definitions: Vec<LinkReferenceDefinition<'a>>,
    /// The reference number for the next link.
    pub(super) next_link: u16,
    /// The state of the current line for render.md.wrapping.
    pub(super) current_line: CurrentLine,
    /// The state of the current table.
    pub(super) current_table: CurrentTable<'a>,
}

impl<'a> StateData<'a> {
    pub(crate) fn current_line(self, current_line: CurrentLine) -> Self {
        Self {
            current_line,
            ..self
        }
    }

    /// Push a pending link.
    pub(crate) fn push_pending_link(
        mut self,
        link_type: LinkType,
        dest_url: CowStr<'a>,
        title: CowStr<'a>,
    ) -> Self {
        self.pending_links.push(PendingLink {
            link_type,
            dest_url,
            title,
        });
        self
    }

    /// Pop a pending link.
    ///
    /// Panics if there is no pending link.
    pub(crate) fn pop_pending_link(mut self) -> (Self, PendingLink<'a>) {
        let link = self.pending_links.pop().unwrap();
        (self, link)
    }

    /// Add a pending link to the state data.
    ///
    /// `target` is the link target, and `title` the link title to show after the URL.
    /// `colour` is the colour to use for foreground text to differentiate between
    /// different types of links.
    pub(crate) fn add_link_reference(
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

    pub(crate) fn take_link_references(self) -> (Self, Vec<LinkReferenceDefinition<'a>>) {
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

impl Default for StateData<'_> {
    fn default() -> Self {
        StateData {
            pending_links: Vec::new(),
            pending_link_definitions: Vec::new(),
            next_link: 1,
            current_line: CurrentLine::empty(),
            current_table: CurrentTable::empty(),
        }
    }
}
