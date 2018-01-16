// Copyright 2018 Sebastian Wiesner <sebastian@swsnr.de>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

// 	http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Write markdown to TTYs.

use std::fmt::Display;
use std::io::{Result, Write};
use std::borrow::Cow;
use std::collections::VecDeque;
use pulldown_cmark::{Event, Tag};
use pulldown_cmark::Event::*;
use pulldown_cmark::Tag::*;
use termion::{color, style};
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::Theme;
use syntect::util::as_24_bit_terminal_escaped;

/// Dump markdown events to a writer.
pub fn dump_events<'a, W, I>(writer: &mut W, events: I) -> Result<()>
where
    I: Iterator<Item = Event<'a>>,
    W: Write,
{
    for event in events {
        write!(writer, "{:?}\n", event)?;
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
/// What kind of format to use.
pub enum Format {
    /// No colours and no styles.
    NoColours,
    /// Basic colours and styles.
    Colours,
    /// Colours and additional formatting for iTerm.
    ITermColours,
}

/// Write markdown to a TTY.
///
/// Iterate over Markdown AST `events`, format each event for TTY output and
/// write the result to a `writer`.
///
/// `push_tty` tries to limit output to the given number of TTY `columns` but
/// does not guarantee that output stays within the column limit.
pub fn push_tty<'a, 'b, W, I>(
    writer: &mut W,
    columns: u16,
    events: I,
    format: Format,
    syntax_set: SyntaxSet,
    theme: &'b Theme,
) -> Result<()>
where
    I: Iterator<Item = Event<'a>>,
    W: Write,
{
    let mut context = Context::new(writer, columns, format, syntax_set, theme);
    for event in events {
        write_event(&mut context, event)?;
    }
    // At the end, print any remaining links
    context.write_pending_links()?;
    Ok(())
}

/// The "level" the current event occurs at.
#[derive(Debug, PartialEq)]
enum BlockLevel {
    /// The event occurs at block-level.
    Block,
    /// The event occurs in inline text.
    Inline,
}

/// The kind of the current list item
#[derive(Debug)]
enum ListItemKind {
    /// An unordered list item
    Unordered,
    /// An ordered list item with its current number
    Ordered(usize),
}

/// A link.
#[derive(Debug)]
struct Link<'a> {
    /// The index of the link.
    index: usize,
    /// The link destination.
    destination: Cow<'a, str>,
    /// The link title.
    title: Cow<'a, str>,
}

/// Context for TTY output.
struct OutputContext<'a, W: Write + 'a> {
    /// The writer to write to.
    writer: &'a mut W,
    /// The maximum number of columns to write.
    columns: u16,
}

#[derive(Debug)]
struct StyleContext {
    /// The format to use.
    format: Format,
    /// All styles applied to the current text.
    active_styles: Vec<String>,
    /// What level of emphasis we are currently at.
    ///
    /// We use this information to switch between italic and upright text for
    /// emphasis.
    emphasis_level: usize,
}

#[derive(Debug)]
struct BlockContext {
    /// The number of spaces to indent with.
    indent_level: usize,
    /// Whether we are at block-level or inline in a block.
    level: BlockLevel,
}

/// Context to keep track of links.
#[derive(Debug)]
struct LinkContext<'a> {
    /// Pending links to be flushed.
    pending_links: VecDeque<Link<'a>>,
    /// The index the next link will get
    next_link_index: usize,
    /// The last text seen.
    ///
    /// We use this field to track the content of link tags, and omit a link
    /// reference if the link text equals the link destination, ie, if the link
    /// appears in text literally.
    last_text: Option<Cow<'a, str>>,
}

struct CodeContext<'a> {
    /// Available syntaxes
    syntax_set: SyntaxSet,
    /// The theme to use for highlighting
    theme: &'a Theme,
    /// The current highlighter.
    ///
    /// If set assume we are in a code block and highlight all text with this
    /// highlighter.
    ///
    /// Otherwise we are either outside of a code block or in a code block we
    /// cannot highlight.
    current_highlighter: Option<HighlightLines<'a>>,
}

/// Context for TTY rendering.
struct Context<'a, 'b, 'c, W: Write + 'b> {
    /// Context for output.
    output: OutputContext<'b, W>,
    /// Context for styling
    style: StyleContext,
    /// Context for the current block.
    block: BlockContext,
    /// Context to keep track of links.
    links: LinkContext<'a>,
    /// Context for code blocks
    code: CodeContext<'c>,
    /// The kind of the current list item.
    ///
    /// A stack of kinds to address nested lists.
    list_item_kind: Vec<ListItemKind>,
}

impl<'a, 'b, 'c, W: Write + 'b> Context<'a, 'b, 'c, W> {
    fn new(
        writer: &'b mut W,
        columns: u16,
        format: Format,
        syntax_set: SyntaxSet,
        theme: &'c Theme,
    ) -> Context<'a, 'b, 'c, W> {
        Context {
            output: OutputContext { writer, columns },
            style: StyleContext {
                active_styles: Vec::new(),
                emphasis_level: 0,
                format,
            },
            block: BlockContext {
                indent_level: 0,
                /// Whether we are at block-level or inline in a block.
                level: BlockLevel::Inline,
            },
            links: LinkContext {
                pending_links: VecDeque::new(),
                next_link_index: 1,
                last_text: None,
            },
            code: CodeContext {
                syntax_set,
                theme,
                current_highlighter: None,
            },
            list_item_kind: Vec::new(),
        }
    }

    /// Start a new block.
    ///
    /// Set `block_context` accordingly, and separate this block from the
    /// previous.
    fn start_inline_text(&mut self) -> Result<()> {
        match self.block.level {
            BlockLevel::Block => self.newline_and_indent()?,
            _ => (),
        }
        // We are inline now
        self.block.level = BlockLevel::Inline;
        Ok(())
    }

    /// End a block.
    ///
    /// Set `block_context` accordingly and end inline context—if present—with
    /// a line break.
    fn end_inline_text_with_margin(&mut self) -> Result<()> {
        match self.block.level {
            BlockLevel::Inline => self.newline()?,
            _ => (),
        };
        // We are back at blocks now
        self.block.level = BlockLevel::Block;
        Ok(())
    }

    /// Set all active styles on the underlying writer.
    fn flush_styles(&mut self) -> Result<()> {
        match self.style.format {
            Format::NoColours => Ok(()),
            _ => write!(self.output.writer, "{}", self.style.active_styles.join("")),
        }
    }

    /// Write a newline.
    ///
    /// Restart all current styles after the newline.
    fn newline(&mut self) -> Result<()> {
        match self.style.format {
            Format::NoColours => write!(self.output.writer, "\n"),
            _ => {
                write!(self.output.writer, "{}\n", style::Reset)?;
                self.flush_styles()
            }
        }
    }

    /// Write a newline and indent.
    ///
    /// Reset format before the line break, and set all active styles again
    /// after the line break.
    fn newline_and_indent(&mut self) -> Result<()> {
        self.newline()?;
        self.indent()
    }

    /// Indent according to the current indentation level.
    fn indent(&mut self) -> Result<()> {
        write!(
            self.output.writer,
            "{}",
            " ".repeat(self.block.indent_level)
        )
    }

    /// Enable a style.
    ///
    /// Add the style to the stack of active styles and write it to the writer.
    ///
    /// To undo a style call `active_styles.pop()`, followed by `set_styles()`
    /// or `newline()`.
    fn enable_style<S: Display>(&mut self, style: S) -> Result<()> {
        match self.style.format {
            Format::NoColours => Ok(()),
            _ => {
                self.style
                    .active_styles
                    .push(format!("{}", style).to_owned());
                write!(self.output.writer, "{}", style)
            }
        }
    }

    /// Remove the last style and flush styles on the TTY.
    fn reset_last_style(&mut self) -> Result<()> {
        match self.style.format {
            Format::NoColours => Ok(()),
            _ => {
                self.style.active_styles.pop();
                write!(self.output.writer, "{}", style::Reset)?;
                self.flush_styles()
            }
        }
    }

    /// Enable emphasis.
    ///
    /// Enable italic or upright text according to the current emphasis level.
    fn enable_emphasis(&mut self) -> Result<()> {
        self.style.emphasis_level += 1;
        if self.style.emphasis_level % 2 == 1 {
            self.enable_style(style::Italic)
        } else {
            self.enable_style(style::NoItalic)
        }
    }

    /// Add a link to the context.
    ///
    /// Return the index of the link.
    fn add_link(&mut self, destination: Cow<'a, str>, title: Cow<'a, str>) -> usize {
        let index = self.links.next_link_index;
        self.links.next_link_index += 1;
        self.links.pending_links.push_back(Link {
            index,
            destination,
            title,
        });
        index
    }

    /// Write all pending links.
    ///
    /// Empty all pending links afterwards.
    fn write_pending_links(&mut self) -> Result<()> {
        if !self.links.pending_links.is_empty() {
            self.newline()?;
            self.enable_style(color::Fg(color::Blue))?;
            while let Some(link) = self.links.pending_links.pop_front() {
                write!(
                    self.output.writer,
                    "[{}]: {} {}",
                    link.index, link.destination, link.title
                )?;
                self.newline()?;
            }
            self.reset_last_style()?;
        };
        Ok(())
    }

    /// Set a mark for iTerm2.
    fn set_iterm_mark(&mut self) -> Result<()> {
        match self.style.format {
            Format::ITermColours => write!(self.output.writer, "\x1B]1337;SetMark\x07"),
            _ => Ok(()),
        }
    }
}

/// Write a single `event` in the given context.
fn write_event<'a, 'b, 'c, W: Write>(
    ctx: &mut Context<'a, 'b, 'c, W>,
    event: Event<'a>,
) -> Result<()> {
    match event {
        SoftBreak | HardBreak => ctx.newline_and_indent()?,
        Text(text) => match ctx.code.current_highlighter {
            Some(ref mut highlighter) => {
                let regions = highlighter.highlight(&text);
                write!(
                    ctx.output.writer,
                    "{}",
                    as_24_bit_terminal_escaped(&regions, true)
                )?;
            }
            None => {
                write!(ctx.output.writer, "{}", text)?;
                ctx.links.last_text = Some(text);
            }
        },
        Start(tag) => start_tag(ctx, tag)?,
        End(tag) => end_tag(ctx, tag)?,
        Html(content) => {
            ctx.newline()?;
            ctx.enable_style(color::Fg(color::LightBlack))?;
            for line in content.lines() {
                write!(ctx.output.writer, "{}", line)?;
                ctx.newline()?;
            }
            ctx.reset_last_style()?;
        }
        InlineHtml(tag) => {
            ctx.enable_style(color::Fg(color::LightBlack))?;
            write!(ctx.output.writer, "{}", tag)?;
            ctx.reset_last_style()?;
        }
        FootnoteReference(_) => panic!("mdless does not support footnotes"),
    };
    Ok(())
}

/// Write the start of a `tag` in the given context.
fn start_tag<'a, W: Write>(ctx: &mut Context<W>, tag: Tag<'a>) -> Result<()> {
    match tag {
        Paragraph => ctx.start_inline_text()?,
        Rule => {
            ctx.start_inline_text()?;
            ctx.enable_style(color::Fg(color::LightBlack))?;
            write!(
                ctx.output.writer,
                "{}",
                "\u{2550}".repeat(ctx.output.columns as usize)
            )?
        }
        Header(level) => {
            // Before we start a new header, write all pending links to keep
            // them close to the text where they appeared in
            ctx.write_pending_links()?;
            ctx.start_inline_text()?;
            ctx.set_iterm_mark()?;
            let level_indicator = "\u{2504}".repeat(level as usize);
            ctx.enable_style(style::Bold)?;
            ctx.enable_style(color::Fg(color::Blue))?;
            write!(ctx.output.writer, "{}", level_indicator)?
        }
        BlockQuote => {
            ctx.block.indent_level += 4;
            ctx.start_inline_text()?;
            ctx.enable_style(color::Fg(color::LightBlack))?;
            ctx.enable_emphasis()?
        }
        CodeBlock(name) => {
            ctx.start_inline_text()?;
            ctx.enable_style(color::Fg(color::LightBlack))?;
            write!(
                ctx.output.writer,
                "{}\n",
                "\u{2500}".repeat(ctx.output.columns.min(20) as usize)
            )?;
            ctx.reset_last_style()?;
            if ctx.style.format != Format::NoColours {
                if name.is_empty() {
                    ctx.enable_style(color::Fg(color::Yellow))?
                } else {
                    if let Some(syntax) = ctx.code.syntax_set.find_syntax_by_token(&name) {
                        ctx.code.current_highlighter =
                            Some(HighlightLines::new(syntax, ctx.code.theme));
                        // Give the highlighter a clear terminal with no prior
                        // styles.
                        write!(ctx.output.writer, "{}", style::Reset)?;
                    }
                    if ctx.code.current_highlighter.is_none() {
                        // If we have no highlighter for the current block, fall
                        // back to default style.
                        ctx.enable_style(color::Fg(color::Yellow))?
                    }
                }
            }
        }
        List(kind) => {
            ctx.list_item_kind.push(match kind {
                Some(start) => ListItemKind::Ordered(start),
                None => ListItemKind::Unordered,
            });
            ctx.newline()?;
        }
        Item => {
            ctx.indent()?;
            ctx.block.level = BlockLevel::Inline;
            match ctx.list_item_kind.pop() {
                Some(ListItemKind::Unordered) => {
                    write!(ctx.output.writer, "\u{2022} ")?;
                    ctx.block.indent_level += 2;
                    ctx.list_item_kind.push(ListItemKind::Unordered);
                }
                Some(ListItemKind::Ordered(number)) => {
                    write!(ctx.output.writer, "{:>2}. ", number)?;
                    ctx.block.indent_level += 4;
                    ctx.list_item_kind.push(ListItemKind::Ordered(number + 1));
                }
                None => panic!("List item without list item kind"),
            }
        }
        FootnoteDefinition(_) => panic!("mdless does not support footnotes"),
        Table(_alignment) => panic!("mdless does not support tables"),
        TableHead => panic!("mdless does not support tables"),
        TableRow => panic!("mdless does not support tables"),
        TableCell => panic!("mdless does not support tables"),
        Emphasis => ctx.enable_emphasis()?,
        Strong => ctx.enable_style(style::Bold)?,
        Code => ctx.enable_style(color::Fg(color::Yellow))?,
        Link(_, _) => {
            // We do not need to do anything when opening links; we render the
            // link reference when closing the link.
        }
        Image(_, _) => panic!("mdless does not support images"),
    };
    Ok(())
}

/// Write the end of a `tag` in the given context.
fn end_tag<'a, 'b, 'c, W: Write>(ctx: &mut Context<'a, 'b, 'c, W>, tag: Tag<'a>) -> Result<()> {
    match tag {
        Paragraph => ctx.end_inline_text_with_margin()?,
        Rule => {
            ctx.style.active_styles.pop();
            ctx.end_inline_text_with_margin()?
        }
        Header(_) => {
            ctx.style.active_styles.pop();
            ctx.style.active_styles.pop();
            ctx.end_inline_text_with_margin()?
        }
        BlockQuote => {
            ctx.block.indent_level -= 4;
            ctx.style.emphasis_level -= 1;
            ctx.style.active_styles.pop();
            ctx.reset_last_style()?;
            ctx.end_inline_text_with_margin()?
        }
        CodeBlock(_) => {
            match ctx.code.current_highlighter {
                None => ctx.reset_last_style()?,
                Some(_) => {
                    // Reset anything left over from the highlighter and
                    // re-enable all current styles.
                    write!(ctx.output.writer, "{}\n", style::Reset)?;
                    ctx.flush_styles()?;
                    ctx.code.current_highlighter = None;
                }
            }
            ctx.enable_style(color::Fg(color::LightBlack))?;
            write!(
                ctx.output.writer,
                "{}\n",
                "\u{2500}".repeat(ctx.output.columns.min(20) as usize)
            )?;
            ctx.reset_last_style()?;
            ctx.end_inline_text_with_margin()?
        }
        List(_) => {
            // End the current list
            ctx.list_item_kind.pop();
            ctx.end_inline_text_with_margin()?;
        }
        Item => {
            // Reset indent level according to list item kind
            match ctx.list_item_kind.last() {
                Some(&ListItemKind::Ordered(_)) => ctx.block.indent_level -= 4,
                Some(&ListItemKind::Unordered) => ctx.block.indent_level -= 2,
                None => (),
            }
            ctx.end_inline_text_with_margin()?
        }
        FootnoteDefinition(_) => {}
        Table(_) => {}
        TableHead => {}
        TableRow => {}
        TableCell => {}
        Emphasis => {
            ctx.reset_last_style()?;
            ctx.style.emphasis_level -= 1;
            ()
        }
        Strong => ctx.reset_last_style()?,
        Code => ctx.reset_last_style()?,
        Link(destination, title) => match ctx.links.last_text {
            Some(ref text) if *text == destination => {
                // Do nothing if the last printed text matches the destination
                // of the link.  In this we likely looked at an inline autolink
                // and we should not repeat the link when it's already in text.
            }
            _ => {
                let index = ctx.add_link(destination, title);
                // Reference link
                ctx.enable_style(color::Fg(color::Blue))?;
                write!(ctx.output.writer, "[{}]", index)?;
                ctx.reset_last_style()?;
            }
        },
        Image(_, _) => {}
    };
    Ok(())
}
