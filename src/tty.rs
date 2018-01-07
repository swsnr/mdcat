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

struct Block {
    indent_level: usize,
}

struct Inline {
    block: Block,
}

enum State {
    Initial,
    Block(Block),
    Inline(Inline),
}

/// Context for TTY rendering.
struct Context<'b, W: Write + 'b> {
    /// The writer to write to
    writer: &'b mut W,
    /// The block/inline state
    state: State,
    /// The maximum number of columns to write
    columns: u16,
    /// All styles applied to the current text
    active_styles: Vec<String>,
    /// Whether emphasis is active.
    emphasis_level: usize,
}

impl<'b, W: Write + 'b> Context<'b, W> {
    fn new(writer: &'b mut W, columns: u16) -> Context<'b, W> {
        Context {
            writer,
            columns,
            state: State::Initial,
            active_styles: Vec::new(),
            emphasis_level: 0,
        }
    }

    fn start_block(&mut self) -> Result<()> {
        let next_state: Result<State> = match self.state {
            State::Block(block) => {
                self.newline()?;
                Ok(State::Block(block))
            }
            State::Inline(ref inline) => Ok(State::Block(inline.block)),
            State::Initial => Ok(State::Block(Block { indent_level: 0 })),
        };
        self.state = next_state?;
        Ok(())
    }

    fn end_block(&mut self) -> Result<()> {
        match self.state {
            State::Inline(inline) => {
                self.state = State::Block(inline.block);
                self.newline()?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Set all active styles on the writer.
    fn set_styles(&mut self) -> Result<()> {
        write!(self.writer, "{}", self.active_styles.join(""))
    }

    /// Write a newline.
    ///
    /// Reset formatting before the line break, and set all active styles again
    /// after the line break.
    fn newline(&mut self) -> Result<()> {
        write!(self.writer, "{}\n", style::Reset)?;
        self.indent()?;
        self.set_styles()
    }

    fn indent(&mut self) -> Result<()> {
        match self.state {
            State::Block(block) => write!(self.writer, "{}", " ".repeat(block.indent_level)),
            _ => Ok(()),
        }
    }

    // fn start_block(&mut self) -> Result<()> {
    //     match self.state {
    //         Initi
    //     }
    // }

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

    /// Remove the last style.
    fn reset_last_style(&mut self) -> Result<()> {
        self.active_styles.pop();
        write!(self.writer, "{}", style::Reset)?;
        self.set_styles()
    }

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
        SoftBreak | HardBreak => ctx.newline()?,
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
        Paragraph => ctx.start_block()?,
        Rule => {
            ctx.start_block()?;
            ctx.enable_style(color::Fg(color::LightBlack))?;
            write!(ctx.writer, "{}", "\u{2500}".repeat(ctx.columns as usize))?
        }
        Header(level) => {
            ctx.start_block()?;
            let level_indicator = "  ".repeat((level - 1) as usize);
            ctx.enable_style(style::Bold)?;
            ctx.enable_style(color::Fg(color::Blue))?;
            write!(ctx.writer, "{}", level_indicator)?
        }
        BlockQuote => {
            // ctx. += 4;
            ctx.start_block()?;
            ctx.enable_style(color::Fg(color::White))?;
            ctx.enable_emphasis()?
        }
        CodeBlock(_) => {
            ctx.start_block()?;
            ctx.enable_style(color::Fg(color::Yellow))?
        }
        List(_) => ctx.start_block()?,
        Item => {
            write!(ctx.writer, "\u{2022} ")?;
            // ctx.indent_level += 2;

            // ctx.is_first = true;
        }
        FootnoteDefinition(_) => (),
        Table(_alignment) => (),
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
        Paragraph => ctx.end_block()?,
        Rule => {
            ctx.active_styles.pop();
            ctx.newline()?
        }
        Header(_) => {
            ctx.active_styles.pop();
            ctx.active_styles.pop();
            ctx.newline()?
        }
        BlockQuote => {
            // ctx.indent_level -= 4;
            ctx.active_styles.pop();
            ctx.emphasis_level -= 1;
            ctx.reset_last_style()?;
            ctx.end_block()?
        }
        CodeBlock(_) => ctx.reset_last_style()?,
        List(_) => ctx.end_block()?,
        Item => {
            // ctx.indent_level -= 2;
            ctx.newline()?
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
