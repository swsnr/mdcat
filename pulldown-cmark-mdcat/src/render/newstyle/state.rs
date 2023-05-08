// Copyright  Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt::{Display, Formatter, Write};
use std::io::Result;

use anstyle::{Effects, Style};
use pulldown_cmark::CowStr;
use textwrap::core::display_width;
use textwrap::{wrap, Options, WrapAlgorithm};

use crate::theme::CombineStyle;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Indent {
    /// The amount of whitespace to add before the initial line of this paragraph.
    pub initial_indent: u16,
    /// The amount of whitespace to add before any subsequent lines in this paragraph.
    pub subsequent_indent: u16,
    /// A prefix to prepend between the indent and the text of every line.
    pub prefix: Option<String>,
}

impl Indent {
    /// No indentation.
    fn none() -> Self {
        Self {
            initial_indent: 0,
            subsequent_indent: 0,
            prefix: None,
        }
    }

    /// Render the initial indent to a formatter.
    pub fn render_initial(&self) -> IndentDisplay<'_> {
        IndentDisplay {
            length: self.initial_indent,
            prefix: self.prefix.as_ref().map_or("", |s| s.as_str()),
        }
    }

    /// Render the subsequent indent to a formatter.
    pub fn render_subsequent(&self) -> IndentDisplay<'_> {
        IndentDisplay {
            length: self.initial_indent,
            prefix: self.prefix.as_ref().map_or("", |s| s.as_str()),
        }
    }
}

#[derive(Debug)]
pub struct IndentDisplay<'a> {
    prefix: &'a str,
    length: u16,
}

impl<'a> Display for IndentDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:indent$}{}",
            "",
            self.prefix,
            indent = self.length as usize
        )
    }
}

/// Whether to add a margin.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Margin {
    /// Always add a margin.
    MarginBefore,
    /// Always add no margin.
    NoMargin,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum List {
    /// An unordered list
    Unordered,
    /// An ordered list with the given number for the next item.
    Ordered(u64),
}

/// State for current paragraph.
#[derive(Debug, Clone)]
struct Paragraph {
    /// Whether to add margin before.
    margin: Margin,
    /// Indentation settings for this paragraph.
    indent: Indent,
    /// Contents of this paragraph.
    ///
    /// Already formatted, but not yet filled and wrapped.
    contents: String,
    /// The stack of inline styles.
    ///
    /// The top-most style is what's currently used for rendering, and lower styles denote what style
    /// will become active again when the current inline item ends.
    inline_styles: Vec<Style>,
    /// List this paragraph is contained in and any parent lists
    list_level: Vec<List>,
    /// Level of link nesting inside links.
    ///
    /// Required because we can't really nest inline links in a terminal, so we need to keep track
    /// of whether we already wrote an inline inline.
    link_nesting: u16,
}

impl Paragraph {
    /// Create an empty paragraph with no margin.
    fn empty_no_margin() -> Self {
        Self {
            margin: Margin::NoMargin,
            indent: Indent::none(),
            /// 200 words per paragraph, times five characters per word on average, plus spaces and
            /// bit of space for control characters and non-ASCII characters is about 2 Kibibyte, so
            /// let's allocate 4 Kibibyte and hope that this is enough to avoid too many
            /// reallocations throughout rendering when we reuse this string.
            contents: String::with_capacity(4096),
            // 10 levels of nested styles would be a lot already.
            inline_styles: Vec::with_capacity(10),
            // Likewise 10 nested lists would be a lot.
            list_level: Vec::with_capacity(10),
            link_nesting: 0,
        }
    }

    /// Whether this paragraph is empty.
    fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }
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

/// Rendering state
#[derive(Debug)]
pub struct State<'a> {
    /// The current paragraph.
    paragraph: Paragraph,
    /// The maximum text width.
    column_width: usize,
    /// Whether styling is enabled.
    styling_enabled: bool,
    /// Pending links
    pending_link_definitions: Vec<LinkReferenceDefinition<'a>>,
}

impl State<'_> {
    /// Create the initial state.
    pub fn initial(column_width: usize, styling_enabled: bool) -> Self {
        // TODO: Don't use a boolean parameter here, but a proper enum
        Self {
            column_width,
            styling_enabled,
            paragraph: Paragraph::empty_no_margin(),
            pending_link_definitions: Vec::with_capacity(10),
        }
    }

    /// Whether the current paragraph is empty.
    pub fn paragraph_is_empty(&self) -> bool {
        self.paragraph.is_empty()
    }

    /// Get the margin settings for the current paragraph.
    pub fn paragraph_margin(&self) -> Margin {
        self.paragraph.margin
    }

    /// Get the contents of the current paragraph.
    pub fn paragraph_contents(&self) -> &str {
        &self.paragraph.contents
    }

    /// Clear all contents in the current paragraph.
    pub fn clear_paragraph(mut self) -> Self {
        self.paragraph.contents.clear();
        self
    }

    /// Flush the current paragraph to the given sink.
    ///
    /// Wrap and indent the paragraph and write it to `sink`.  If required, write a margin before.
    /// Then empty the paragraph data.
    pub fn flush_paragraph<W: std::io::Write>(self, sink: &mut W) -> Result<Self> {
        if self.paragraph_is_empty() {
            // If the paragraph is empty do nothing, because there's nothing to be written.
            return Ok(self);
        }
        // Write margin before if required
        if let Margin::MarginBefore = self.paragraph.margin {
            writeln!(sink)?;
        }
        let indent = &self.paragraph.indent;
        let initial_indent = format!("{}", indent.render_initial());
        let subsequent_indent = format!("{}", indent.render_subsequent());
        let options = Options::new(self.column_width)
            .initial_indent(&initial_indent)
            .subsequent_indent(&subsequent_indent)
            // TODO: Change to optimal fit once the new algorithm works in general
            .wrap_algorithm(WrapAlgorithm::FirstFit);
        for line in wrap(&self.paragraph.contents, options) {
            writeln!(sink, "{line}")?;
        }
        // The paragraph was written so erase everything.
        Ok(self.clear_paragraph())
    }

    /// A sink to write contents into the current paragraph.
    pub fn sink(&mut self) -> &mut dyn Write {
        &mut self.paragraph.contents
    }

    /// Get the text width of subsequent lines in this paragraph.
    ///
    /// The text width is the width of the actual text, i.e. column width minus indent and prefix
    /// length.
    pub fn subsequent_text_width(&self) -> usize {
        self.column_width
            - self.paragraph.indent.subsequent_indent as usize
            - self
                .paragraph
                .indent
                .prefix
                .as_ref()
                .map_or(0, |p| display_width(p))
    }

    /// Require no margin before the next paragraph.
    pub fn no_margin_before(mut self) -> Self {
        self.paragraph.margin = Margin::NoMargin;
        self
    }

    /// Require a margin before the next paragraph.
    pub fn with_margin_before(mut self) -> Self {
        self.paragraph.margin = Margin::MarginBefore;
        self
    }

    /// Set the given `prefix` for every wrapped line in the current paragraph.
    pub fn with_line_prefix(mut self, prefix: String) -> Self {
        self.paragraph.indent.prefix = Some(prefix);
        self
    }

    /// Clear the prefix for this paragraph.
    pub fn clear_line_prefix(mut self) -> Self {
        self.paragraph.indent.prefix = None;
        self
    }

    pub fn paragraph_indent(&self) -> &Indent {
        &self.paragraph.indent
    }

    /// Add the given amount to the overall indentation.
    pub fn add_indent(mut self, indent: u16) -> Self {
        self.paragraph.indent.initial_indent += indent;
        self.add_subsequent_indent(indent)
    }

    /// Add the given amount to the indentation of subsequent lines.
    pub fn add_subsequent_indent(mut self, indent: u16) -> Self {
        self.paragraph.indent.subsequent_indent += indent;
        self
    }

    /// Subtract the given amount to the overall indentation
    pub fn dedent(mut self, indent: u16) -> Self {
        self.paragraph.indent.initial_indent -= indent;
        self.dedent_subsequent(indent)
    }

    /// Subtract the given amount from the indentation of subsequent lines
    pub fn dedent_subsequent(mut self, indent: u16) -> Self {
        self.paragraph.indent.subsequent_indent -= indent;
        self
    }

    pub fn reset_initial_indent(mut self) -> Self {
        self.paragraph.indent.initial_indent = self.paragraph.indent.subsequent_indent;
        self
    }

    /// Set the given style and add it to the stack.
    ///
    /// Do nothing if styling is disabled.
    fn set_style(mut self, style: Style) -> Self {
        if self.styling_enabled {
            write!(self.sink(), "{}", style.render()).unwrap();
            self.paragraph.inline_styles.push(style);
        }
        self
    }

    /// Push a new inline style for the current paragraph.
    ///
    /// The style is added on top of the current style and immediately activated.
    pub fn push_inline_style(self, style: &Style) -> Self {
        let new_style = match self.paragraph.inline_styles.last() {
            None => *style,
            Some(current) => style.on_top_of(current),
        };
        self.set_style(new_style)
    }

    /// Push a new inline style which toggles italics.
    pub fn toggle_italic(self) -> Self {
        let new_style = match self.paragraph.inline_styles.last() {
            None => Style::new().italic(),
            Some(current) => {
                let new_effects = if current.get_effects().contains(Effects::ITALIC) {
                    current.get_effects() - Effects::ITALIC
                } else {
                    current.get_effects() | Effects::ITALIC
                };
                current.effects(new_effects)
            }
        };
        self.set_style(new_style)
    }

    /// Pop the current inline style and reactivate the former style.
    pub fn pop_inline_style(mut self) -> Self {
        if let Some(old_style) = self.paragraph.inline_styles.pop() {
            write!(self.sink(), "{}", old_style.render_reset()).unwrap();
            // Re-enable the previous style if any.
            if let Some(previous_style) = self.paragraph.inline_styles.pop() {
                return self.set_style(previous_style);
            }
        }
        self
    }

    /// Push an unordered list
    pub fn push_unordered_list(mut self) -> Self {
        self.paragraph.list_level.push(List::Unordered);
        self
    }

    /// Push an ordered list with the given number for the next item.
    pub fn push_ordered_list(mut self, number: u64) -> Self {
        self.paragraph.list_level.push(List::Ordered(number));
        self
    }

    /// Pop and return the current list, and the changed state.
    pub fn pop_current_list(mut self) -> (Self, List) {
        let list = self.paragraph.list_level.pop().expect("Not inside a list!");
        (self, list)
    }

    /// Attempt to push an inline link into this state.
    ///
    /// Return Ok(self) if the caller can write an inline link, or Err(self) if the caller cannot
    /// use an inline link and instead needs to use a reference link.
    pub fn try_push_inline_link(mut self) -> std::result::Result<Self, Self> {
        if self.styling_enabled {
            self.paragraph.link_nesting += 1;
            if self.paragraph.link_nesting == 1 {
                Ok(self)
            } else {
                Err(self)
            }
        } else {
            Err(self)
        }
    }

    /// Attempt ot pop an inline link from this state.
    ///
    /// Return Ok(self) if the caller wrote an inline link, or Err(self) otherwise.
    pub fn try_pop_inline_link(mut self) -> std::result::Result<Self, Self> {
        if self.styling_enabled {
            if self.paragraph.link_nesting == 1 {
                self.paragraph.link_nesting -= 1;
                Ok(self)
            } else {
                if 0 < self.paragraph.link_nesting {
                    self.paragraph.link_nesting -= 1;
                }
                Err(self)
            }
        } else {
            Err(self)
        }
    }

    pub fn push_link_reference(
        mut self,
        target: CowStr<'_>,
        title: CowStr<'_>,
        style: Style,
    ) -> (Self, u16) {
        todo!()
    }
}
