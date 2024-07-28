// Copyright Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{borrow::Cow, iter::{repeat, zip}};

use textwrap::core::{display_width, Fragment, Word};

/// A link on a terminal.
#[derive(Debug, PartialEq, Eq)]
pub struct Link<'a> {
    /// The ID of the link.
    ///
    /// The ID identifies adjacent link fragments belonging to the same link.
    /// Fragments with the same `url` and `id` should be joined into a single
    /// link before rendering to a TTY.
    id: String,
    /// The URL to link to.
    url: Cow<'a, str>,
}

/// A single fragment of wrappable and renderable text.
///
/// Wraps a [`textwrap::core::Word`] with an optional style and a link.
#[derive(Debug)]
pub struct RenderFragment<'a> {
    word: Word<'a>,
    style: Option<anstyle::Style>,
    link: Option<Link<'a>>,
}

impl<'a> RenderFragment<'a> {
    /// Add a style to this fragment.
    pub fn with_style(self, style: anstyle::Style) -> Self {
        Self {
            style: Some(style),
            ..self
        }
    }

    /// Add a link to this fragment.
    pub fn with_link(self, link: Link<'a>) -> Self {
        Self {
            link: Some(link),
            ..self
        }
    }
}

impl<'a> From<Word<'a>> for RenderFragment<'a> {
    /// Create a render fragment from a word.
    fn from(word: Word<'a>) -> Self {
        RenderFragment {
            word,
            style: None,
            link: None,
        }
    }
}

impl<'a> Fragment for RenderFragment<'a> {
    fn width(&self) -> f64 {
        self.word.width()
    }

    fn whitespace_width(&self) -> f64 {
        self.word.whitespace_width()
    }

    fn penalty_width(&self) -> f64 {
        self.word.penalty_width()
    }
}

/// A block of wrappable fragments.
#[derive(Debug, Default)]
pub struct FragmentsBlock<'a> {
    /// Fragments in this block.
    fragments: Vec<RenderFragment<'a>>,
    /// Indentation for the first line.
    first_line_indent: &'a str,
    /// Indentation for subsequent lines.
    subsequent_line_indent: &'a str,
}

impl<'a> FragmentsBlock<'a> {
    pub fn with_first_line_indent(self, first_line_indent: &'a str) -> Self {
        Self {
            first_line_indent,
            ..self
        }
    }

    pub fn with_subsequent_line_indent(self, subsequent_line_indent: &'a str) -> Self {
        Self {
            subsequent_line_indent,
            ..self
        }
    }

    pub fn add_fragment(mut self, fragment: RenderFragment<'a>) -> Self {
        self.fragments.push(fragment);
        self
    }

    pub fn wrap(&'a self, line_width: f64) -> FragmentLinesBLock<'a> {
        let line_widths = [
            line_width - display_width(self.first_line_indent) as f64,
            line_width - display_width(self.subsequent_line_indent) as f64,
        ];
        // TODO: Use optimal wrap
        let lines = textwrap::wrap_algorithms::wrap_first_fit(&self.fragments, &line_widths);
        FragmentLinesBLock {
            lines,
            first_line_indent: self.first_line_indent,
            subsequent_line_indent: self.subsequent_line_indent,
        }
    }
}

/// Any kind of block that we can render.
#[derive(Debug)]
pub enum RenderBlock<'a> {
    /// A block
    Fragments(FragmentsBlock<'a>),
}

/// A block of fragments in multiple lines.
#[derive(Debug)]
pub struct FragmentLinesBLock<'a> {
    /// Lines in this block.
    lines: Vec<&'a [RenderFragment<'a>]>,
    /// Indentation for the first line.
    first_line_indent: &'a str,
    /// Indentation for subsequent lines.
    subsequent_line_indent: &'a str,
}

impl<'a> FragmentLinesBLock<'a> {
    pub fn render(&self) -> impl Iterator<Item = RenderToken<'a>> {
        let indentation = [self.first_line_indent]
            .into_iter()
            .chain(repeat(self.subsequent_line_indent));
        zip(indentation, lines).flat_map(|(indentation, line)).
    }
}

pub enum RenderToken<'a> {
    Text(&'a str),
    Style(anstyle::Style),
    Reset(anstyle::Reset),
    SetLink(Link<'a>),
    ClearLink,
}
