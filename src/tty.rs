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
use pulldown_cmark::{Event, Tag};
use pulldown_cmark::Event::*;
use pulldown_cmark::Tag::*;
use termion::{color, style};

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

/// Write markdown to a TTY.
///
/// Iterate over Markdown AST `events`, format each event for TTY output and
/// write the result to a `writer`.
///
/// `push_tty` tries to limit output to the given number of TTY `columns` but
/// does not guarantee that output stays within the column limit.
pub fn push_tty<'a, W, I>(writer: &mut W, columns: u16, events: I) -> Result<()>
where
    I: Iterator<Item = Event<'a>>,
    W: Write,
{
    let mut context = Context::new(writer, columns);
    for event in events {
        write_event(&mut context, event)?;
    }
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

/// Context for TTY rendering.
struct Context<'b, W: Write + 'b> {
    /// The writer to write to.
    writer: &'b mut W,
    /// The maximum number of columns to write.
    columns: u16,
    /// All styles applied to the current text.
    active_styles: Vec<String>,
    /// What level of emphasis we are currently at.
    ///
    /// We use this information to switch between italic and upright text for
    /// emphasis.
    emphasis_level: usize,
    /// The number of spaces to indent with.
    indent_level: usize,
    /// Whether we are at block-level or inline in a block.
    block_level: BlockLevel,
    /// The kind of the current list item.
    ///
    /// A stack of kinds to address nested lists.
    list_item_kind: Vec<ListItemKind>,
}

impl<'b, W: Write + 'b> Context<'b, W> {
    fn new(writer: &'b mut W, columns: u16) -> Context<'b, W> {
        Context {
            writer,
            columns,
            active_styles: Vec::new(),
            emphasis_level: 0,
            indent_level: 0,
            // We start inline; blocks must be started explicitly
            block_level: BlockLevel::Inline,
            list_item_kind: Vec::new(),
        }
    }

    /// Start a new block.
    ///
    /// Set `block_context` accordingly, and separate this block from the
    /// previous.
    fn start_inline_text(&mut self) -> Result<()> {
        match self.block_level {
            BlockLevel::Block => self.newline_and_indent()?,
            _ => (),
        }
        // We are inline now
        self.block_level = BlockLevel::Inline;
        Ok(())
    }

    /// End a block.
    ///
    /// Set `block_context` accordingly and end inline context—if present—with
    /// a line break.
    fn end_inline_text_with_margin(&mut self) -> Result<()> {
        match self.block_level {
            BlockLevel::Inline => self.newline()?,
            _ => (),
        };
        // We are back at blocks now
        self.block_level = BlockLevel::Block;
        Ok(())
    }

    /// Set all active styles on the underlying writer.
    fn flush_styles(&mut self) -> Result<()> {
        write!(self.writer, "{}", self.active_styles.join(""))
    }

    /// Write a newline.
    ///
    /// Restart all current styles after the newline.
    fn newline(&mut self) -> Result<()> {
        write!(self.writer, "{}\n", style::Reset)?;
        self.flush_styles()
    }

    /// Write a newline and indent.
    ///
    /// Reset formatting before the line break, and set all active styles again
    /// after the line break.
    fn newline_and_indent(&mut self) -> Result<()> {
        self.newline()?;
        self.indent()
    }

    /// Indent according to the current indentation level.
    fn indent(&mut self) -> Result<()> {
        write!(self.writer, "{}", " ".repeat(self.indent_level))
    }

    /// Enable a style.
    ///
    /// Add the style to the stack of active styles and write it to the writer.
    ///
    /// To undo a style call `active_styles.pop()`, followed by `set_styles()`
    /// or `newline()`.
    fn enable_style<S: Display>(&mut self, style: S) -> Result<()> {
        self.active_styles.push(format!("{}", style).to_owned());
        write!(self.writer, "{}", style)
    }

    /// Remove the last style and flush styles on the TTY.
    fn reset_last_style(&mut self) -> Result<()> {
        self.active_styles.pop();
        write!(self.writer, "{}", style::Reset)?;
        self.flush_styles()
    }

    /// Enable emphasis.
    ///
    /// Enable italic or upright text according to the current emphasis level.
    fn enable_emphasis(&mut self) -> Result<()> {
        self.emphasis_level += 1;
        if self.emphasis_level % 2 == 1 {
            self.enable_style(style::Italic)
        } else {
            self.enable_style(style::NoItalic)
        }
    }
}

/// Write a single `event` in the given context.
fn write_event<'a, W: Write>(ctx: &mut Context<W>, event: Event<'a>) -> Result<()> {
    match event {
        SoftBreak | HardBreak => ctx.newline_and_indent()?,
        Text(text) => write!(ctx.writer, "{}", text)?,
        Start(tag) => start_tag(ctx, tag)?,
        End(tag) => end_tag(ctx, tag)?,
        event => eprintln!("Unknown event: {:?}", event),
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
            write!(ctx.writer, "{}", "\u{2500}".repeat(ctx.columns as usize))?
        }
        Header(level) => {
            ctx.start_inline_text()?;
            let level_indicator = "  ".repeat((level - 1) as usize);
            ctx.enable_style(style::Bold)?;
            ctx.enable_style(color::Fg(color::Blue))?;
            write!(ctx.writer, "{}", level_indicator)?
        }
        BlockQuote => {
            ctx.indent_level += 4;
            ctx.start_inline_text()?;
            ctx.enable_style(color::Fg(color::LightBlack))?;
            ctx.enable_emphasis()?
        }
        CodeBlock(_) => {
            ctx.start_inline_text()?;
            ctx.enable_style(color::Fg(color::Yellow))?
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
            ctx.block_level = BlockLevel::Inline;
            match ctx.list_item_kind.pop() {
                Some(ListItemKind::Unordered) => {
                    write!(ctx.writer, "\u{2022} ")?;
                    ctx.indent_level += 2;
                    ctx.list_item_kind.push(ListItemKind::Unordered);
                }
                Some(ListItemKind::Ordered(number)) => {
                    write!(ctx.writer, "{:>2}. ", number)?;
                    ctx.indent_level += 4;
                    ctx.list_item_kind.push(ListItemKind::Ordered(number + 1));
                }
                None => panic!("List item without list item kind"),
            }
        }
        FootnoteDefinition(_) => panic!("mdless does not support footnotes"),
        Table(_alignment) => panic!("mdless does not support tables"),
        TableHead => (),
        TableRow => (),
        TableCell => (),
        Emphasis => ctx.enable_emphasis()?,
        Strong => ctx.enable_style(style::Bold)?,
        Code => ctx.enable_style(color::Fg(color::Yellow))?,
        Link(_, _) => (),
        Image(_, _) => (),
    };
    Ok(())
}

/// Write the end of a `tag` in the given context.
fn end_tag<'a, W: Write>(ctx: &mut Context<W>, tag: Tag<'a>) -> Result<()> {
    match tag {
        Paragraph => ctx.end_inline_text_with_margin()?,
        Rule => {
            ctx.active_styles.pop();
            ctx.end_inline_text_with_margin()?
        }
        Header(_) => {
            ctx.active_styles.pop();
            ctx.active_styles.pop();
            ctx.end_inline_text_with_margin()?
        }
        BlockQuote => {
            ctx.indent_level -= 4;
            ctx.emphasis_level -= 1;
            ctx.active_styles.pop();
            ctx.reset_last_style()?;
            ctx.end_inline_text_with_margin()?
        }
        CodeBlock(_) => {
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
                Some(&ListItemKind::Ordered(_)) => ctx.indent_level -= 4,
                Some(&ListItemKind::Unordered) => ctx.indent_level -= 2,
                None => (),
            }
            ctx.end_inline_text_with_margin()?
        }
        FootnoteDefinition(_) => (),
        Table(_) => (),
        TableHead => (),
        TableRow => (),
        TableCell => (),
        Emphasis => {
            ctx.reset_last_style()?;
            ctx.emphasis_level -= 1;
            ()
        }
        Strong => ctx.reset_last_style()?,
        Code => ctx.reset_last_style()?,
        Link(_, _) => (),
        Image(_, _) => (),
    };
    Ok(())
}
