// Copyright Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::borrow::Cow;
use textwrap::core::{display_width, Fragment, Word};

use crate::terminal::osc::clear_link;

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
    /// Set indentation for the first line.
    pub fn with_first_line_indent(self, first_line_indent: &'a str) -> Self {
        Self {
            first_line_indent,
            ..self
        }
    }

    /// Set indentation for subsequent lines.
    pub fn with_subsequent_line_indent(self, subsequent_line_indent: &'a str) -> Self {
        Self {
            subsequent_line_indent,
            ..self
        }
    }

    /// Add a fragment to this block.
    pub fn add_fragment(mut self, fragment: RenderFragment<'a>) -> Self {
        self.fragments.push(fragment);
        self
    }

    /// Wrap this block into lines of the given width.
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

// TODO: Make rendering and wrapping generic to support more kinds of blocks

/// Any kind of block that we can render.
#[derive(Debug)]
pub enum RenderBlock<'a> {
    /// A block of fragments which can be wrapped to a given column width.
    Fragments(FragmentsBlock<'a>),
    /// A single rule in a document.
    Rule(Rule),
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

/// A token which we can render to a writer.
#[derive(Debug)]
pub enum RenderToken<'a> {
    /// Text to write literally.
    Text(&'a str),
    /// Enable the given style.
    SetStyle(anstyle::Style),
    /// Clear the current style.
    ResetStyle(anstyle::Reset),
    /// Set the given link for subsequent text.
    SetLink(&'a Link<'a>),
    /// Clear the current link.
    ClearLink,
}

/// Convert a line into render tokens and append these to the given buffer.
fn push_tokens<'a>(line: &'a [RenderFragment<'a>], token_buffer: &mut Vec<RenderToken<'a>>) {
    let mut current_link = None;
    let mut current_style = None;
    let mut pending_space = "";
    for fragment in line {
        if current_link != fragment.link.as_ref() {
            if current_link.is_some() {
                token_buffer.push(RenderToken::ClearLink);
            }
            if let Some(link) = fragment.link.as_ref() {
                token_buffer.push(RenderToken::SetLink(link))
            }
            current_link = fragment.link.as_ref();
        }
        if current_style != fragment.style {
            if current_style.is_some() {
                token_buffer.push(RenderToken::ResetStyle(anstyle::Reset));
            }
            if let Some(style) = fragment.style {
                token_buffer.push(RenderToken::SetStyle(style));
            }
            current_style = fragment.style;
        }
        if !pending_space.is_empty() {
            token_buffer.push(RenderToken::Text(pending_space));
        }
        token_buffer.push(RenderToken::Text(fragment.word.word));
        pending_space = fragment.word.whitespace;
    }
    // If we have an ongoing link/style, clear either before finishing the line.
    if current_style.is_some() {
        token_buffer.push(RenderToken::ResetStyle(anstyle::Reset));
    }
    if current_link.is_some() {
        token_buffer.push(RenderToken::ClearLink);
    }
    // Finish the current line
    token_buffer.push(RenderToken::Text("\n"));
}

impl<'a> FragmentLinesBLock<'a> {
    /// Convert this block into tokens we can render to a writer.
    ///
    /// Return a list of tokens which can be written to any writer using [`write_tokens`].
    pub fn render(&self) -> Vec<RenderToken<'a>> {
        match self.lines.split_first() {
            None => Vec::new(),
            Some((first_line, rest)) => {
                use RenderToken::*;
                // We assume an every amount of ten fragments per line, at a wild guess.
                let mut buffer = Vec::with_capacity(self.lines.len() * 10);
                buffer.push(Text(self.first_line_indent));

                push_tokens(first_line, &mut buffer);
                for line in rest {
                    buffer.push(Text(self.subsequent_line_indent));
                    push_tokens(line, &mut buffer);
                }
                buffer
            }
        }
    }
}

/// Write render tokens to a writer.
pub fn write_tokens(
    sink: &mut dyn std::io::Write,
    tokens: &[RenderToken<'_>],
) -> std::io::Result<()> {
    for token in tokens {
        match *token {
            RenderToken::SetStyle(style) => style.write_to(sink)?,
            RenderToken::Text(text) => write!(sink, "{}", text)?,
            RenderToken::ResetStyle(reset) => write!(sink, "{}", reset.render())?,
            RenderToken::SetLink(_) => todo!("Write link with hostname and ID"),
            RenderToken::ClearLink => clear_link(sink)?,
        }
    }
    Ok(())
}
