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

#![deny(warnings)]
#![deny(missing_docs)]
// Warn about deprecated trait object syntax
#![deny(bare_trait_objects)]
#![cfg_attr(feature = "cargo-clippy", feature(tool_lints))]
#![cfg_attr(feature = "cargo-clippy", deny(clippy::all))]

//! Write markdown to TTYs.

// Used by iTerm support on macos
#[cfg(feature = "iterm2")]
extern crate base64;
#[cfg(feature = "iterm2")]
extern crate mime;

// Used by Terminology support
#[cfg(feature = "terminology")]
extern crate immeta;

extern crate atty;
#[macro_use]
extern crate failure;
extern crate pulldown_cmark;
extern crate reqwest;
extern crate syntect;
extern crate term_size;
extern crate url;

use failure::Error;
use pulldown_cmark::Event::*;
use pulldown_cmark::Tag::*;
use pulldown_cmark::{Event, Tag};
use std::borrow::Cow;
use std::collections::VecDeque;
use std::io::Write;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

// These modules support iterm2; we do not need them if iterm2 is off.
#[cfg(feature = "iterm2")]
mod magic;
#[cfg(feature = "iterm2")]
mod process;
#[cfg(feature = "iterm2")]
mod svg;

mod resources;
mod terminal;

use resources::Resource;
use terminal::*;

// Expose some select things for use in main
pub use resources::ResourceAccess;
pub use terminal::Size as TerminalSize;
pub use terminal::{detect_terminal, AnsiTerminal, DumbTerminal, Terminal};

/// Dump markdown events to a writer.
pub fn dump_events<'a, W, I>(writer: &mut W, events: I) -> Result<(), Error>
where
    I: Iterator<Item = Event<'a>>,
    W: Write,
{
    for event in events {
        writeln!(writer, "{:?}", event)?;
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
pub fn push_tty<'a, W, I>(
    terminal: Box<dyn Terminal<TerminalWrite = W>>,
    size: TerminalSize,
    events: I,
    base_dir: &'a Path,
    resource_access: ResourceAccess,
    syntax_set: SyntaxSet,
) -> Result<(), Error>
where
    I: Iterator<Item = Event<'a>>,
    W: Write,
{
    let theme = &ThemeSet::load_defaults().themes["Solarized (dark)"];
    let mut context = Context::new(terminal, size, base_dir, resource_access, syntax_set, theme);
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

/// Input context.
struct InputContext<'a> {
    /// The base directory, to resolve relative paths.
    base_dir: &'a Path,
    /// What resources we may access when processing markdown.
    resource_access: ResourceAccess,
}

impl<'a> InputContext<'a> {
    /// Resolve a reference in the input.
    fn resolve_reference(&self, reference: &'a str) -> Resource {
        Resource::from_reference(self.base_dir, reference)
    }
}

/// Context for TTY output.
struct OutputContext<W: Write> {
    /// The terminal dimensions to limit output to.
    size: Size,
    /// The target terminal.
    terminal: Box<dyn Terminal<TerminalWrite = W>>,
}

#[derive(Debug)]
struct StyleContext {
    /// All styles applied to the current text.
    active_styles: Vec<AnsiStyle>,
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
    /// Whether we are inside an inline link currently.
    inside_inline_link: bool,
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

/// Context for images.
#[derive(Debug)]
struct ImageContext {
    /// Whether we currently write an inline image.
    ///
    /// Suppresses all text output.
    inline_image: bool,
}

/// Context for TTY rendering.
struct Context<'a, W: Write + 'a> {
    /// Context for input.
    input: InputContext<'a>,
    /// Context for output.
    output: OutputContext<W>,
    /// Context for styling
    style: StyleContext,
    /// Context for the current block.
    block: BlockContext,
    /// Context to keep track of links.
    links: LinkContext<'a>,
    /// Context for code blocks
    code: CodeContext<'a>,
    /// Context for images.
    image: ImageContext,
    /// The kind of the current list item.
    ///
    /// A stack of kinds to address nested lists.
    list_item_kind: Vec<ListItemKind>,
}

impl<'a, W: Write> Context<'a, W> {
    fn new(
        terminal: Box<dyn Terminal<TerminalWrite = W>>,
        size: Size,
        base_dir: &'a Path,
        resource_access: ResourceAccess,
        syntax_set: SyntaxSet,
        theme: &'a Theme,
    ) -> Context<'a, W> {
        Context {
            input: InputContext {
                base_dir,
                resource_access,
            },
            output: OutputContext { size, terminal },
            style: StyleContext {
                active_styles: Vec::new(),
                emphasis_level: 0,
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
                inside_inline_link: false,
            },
            code: CodeContext {
                syntax_set,
                theme,
                current_highlighter: None,
            },
            image: ImageContext {
                inline_image: false,
            },
            list_item_kind: Vec::new(),
        }
    }

    /// Start a new block.
    ///
    /// Set `block_context` accordingly, and separate this block from the
    /// previous.
    fn start_inline_text(&mut self) -> Result<(), Error> {
        if let BlockLevel::Block = self.block.level {
            self.newline_and_indent()?
        };
        // We are inline now
        self.block.level = BlockLevel::Inline;
        Ok(())
    }

    /// End a block.
    ///
    /// Set `block_context` accordingly and end inline context—if present—with
    /// a line break.
    fn end_inline_text_with_margin(&mut self) -> Result<(), Error> {
        if let BlockLevel::Inline = self.block.level {
            self.newline()?
        };
        // We are back at blocks now
        self.block.level = BlockLevel::Block;
        Ok(())
    }

    /// Set all active styles on the underlying writer.
    fn flush_styles(&mut self) -> Result<(), Error> {
        for style in &self.style.active_styles {
            self.output
                .terminal
                .set_style(*style)
                .ignore_not_supported()?;
        }
        Ok(())
    }

    /// Write a newline.
    ///
    /// Restart all current styles after the newline.
    fn newline(&mut self) -> Result<(), Error> {
        self.output
            .terminal
            .set_style(AnsiStyle::Reset)
            .ignore_not_supported()?;
        writeln!(self.output.terminal.write())?;
        self.flush_styles()
    }

    /// Write a newline and indent.
    ///
    /// Reset format before the line break, and set all active styles again
    /// after the line break.
    fn newline_and_indent(&mut self) -> Result<(), Error> {
        self.newline()?;
        self.indent()
    }

    /// Indent according to the current indentation level.
    fn indent(&mut self) -> Result<(), Error> {
        write!(
            self.output.terminal.write(),
            "{}",
            " ".repeat(self.block.indent_level)
        ).map_err(Into::into)
    }

    /// Enable a style.
    ///
    /// Add the style to the stack of active styles and write it to the writer.
    ///
    /// To undo a style call `active_styles.pop()`, followed by `set_styles()`
    /// or `newline()`.
    fn enable_style(&mut self, style: AnsiStyle) -> Result<(), Error> {
        self.style.active_styles.push(style);
        self.output.terminal.set_style(style).ignore_not_supported()
    }

    /// Remove the last style and flush styles on the TTY.
    fn reset_last_style(&mut self) -> Result<(), Error> {
        self.style.active_styles.pop();
        self.output
            .terminal
            .set_style(AnsiStyle::Reset)
            .ignore_not_supported()?;
        self.flush_styles()
    }

    /// Enable emphasis.
    ///
    /// Enable italic or upright text according to the current emphasis level.
    fn enable_emphasis(&mut self) -> Result<(), Error> {
        self.style.emphasis_level += 1;
        if self.style.emphasis_level % 2 == 1 {
            self.enable_style(AnsiStyle::Italic)
        } else {
            self.enable_style(AnsiStyle::NoItalic)
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
    fn write_pending_links(&mut self) -> Result<(), Error> {
        if !self.links.pending_links.is_empty() {
            self.newline()?;
            self.enable_style(AnsiStyle::Foreground(AnsiColour::Blue))?;
            while let Some(link) = self.links.pending_links.pop_front() {
                write!(
                    self.output.terminal.write(),
                    "[{}]: {} {}",
                    link.index,
                    link.destination,
                    link.title
                )?;
                self.newline()?;
            }
            self.reset_last_style()?;
        };
        Ok(())
    }

    /// Write a simple border.
    fn write_border(&mut self) -> Result<(), Error> {
        self.enable_style(AnsiStyle::Foreground(AnsiColour::Green))?;
        writeln!(
            self.output.terminal.write(),
            "{}",
            "\u{2500}".repeat(self.output.size.width.min(20))
        )?;
        self.reset_last_style()
    }

    /// Write highlighted `text`.
    ///
    /// If the code context has a highlighter, use it to highlight `text` and
    /// write it.  Otherwise write `text` without highlighting.
    fn write_highlighted(&mut self, text: Cow<'a, str>) -> Result<(), Error> {
        match self.code.current_highlighter {
            Some(ref mut highlighter) => {
                let regions = highlighter.highlight(&text);
                write_as_ansi(&mut *self.output.terminal, &regions)?;
            }
            None => {
                write!(self.output.terminal.write(), "{}", text)?;
                self.links.last_text = Some(text);
            }
        }
        Ok(())
    }
}

/// Write a single `event` in the given context.
fn write_event<'a, W: Write>(ctx: &mut Context<'a, W>, event: Event<'a>) -> Result<(), Error> {
    match event {
        SoftBreak | HardBreak => ctx.newline_and_indent()?,
        Text(text) => {
            // When we wrote an inline image suppress the text output, ie, the
            // image title.  We do not need it if we can show the image on the
            // terminal.
            if !ctx.image.inline_image {
                ctx.write_highlighted(text)?;
            }
        }
        Start(tag) => start_tag(ctx, tag)?,
        End(tag) => end_tag(ctx, tag)?,
        Html(content) => {
            ctx.newline()?;
            ctx.enable_style(AnsiStyle::Foreground(AnsiColour::Green))?;
            for line in content.lines() {
                write!(ctx.output.terminal.write(), "{}", line)?;
                ctx.newline()?;
            }
            ctx.reset_last_style()?;
        }
        InlineHtml(tag) => {
            ctx.enable_style(AnsiStyle::Foreground(AnsiColour::Green))?;
            write!(ctx.output.terminal.write(), "{}", tag)?;
            ctx.reset_last_style()?;
        }
        FootnoteReference(_) => panic!("mdcat does not support footnotes"),
    };
    Ok(())
}

/// Write the start of a `tag` in the given context.
fn start_tag<'a, W: Write>(ctx: &mut Context<W>, tag: Tag<'a>) -> Result<(), Error> {
    match tag {
        Paragraph => ctx.start_inline_text()?,
        Rule => {
            ctx.start_inline_text()?;
            ctx.enable_style(AnsiStyle::Foreground(AnsiColour::Green))?;
            write!(
                ctx.output.terminal.write(),
                "{}",
                "\u{2550}".repeat(ctx.output.size.width as usize)
            )?
        }
        Header(level) => {
            // Before we start a new header, write all pending links to keep
            // them close to the text where they appeared in
            ctx.write_pending_links()?;
            ctx.start_inline_text()?;
            ctx.output.terminal.set_mark().ignore_not_supported()?;
            let level_indicator = "\u{2504}".repeat(level as usize);
            ctx.enable_style(AnsiStyle::Bold)?;
            ctx.enable_style(AnsiStyle::Foreground(AnsiColour::Blue))?;
            write!(ctx.output.terminal.write(), "{}", level_indicator)?
        }
        BlockQuote => {
            ctx.block.indent_level += 4;
            ctx.start_inline_text()?;
            ctx.enable_style(AnsiStyle::Foreground(AnsiColour::Green))?;
            ctx.enable_emphasis()?
        }
        CodeBlock(name) => {
            ctx.start_inline_text()?;
            ctx.write_border()?;
            if ctx.output.terminal.supports_styles() {
                if name.is_empty() {
                    ctx.enable_style(AnsiStyle::Foreground(AnsiColour::Yellow))?
                } else {
                    if let Some(syntax) = ctx.code.syntax_set.find_syntax_by_token(&name) {
                        ctx.code.current_highlighter =
                            Some(HighlightLines::new(syntax, ctx.code.theme));
                        // Give the highlighter a clear terminal with no prior
                        // styles.
                        ctx.output.terminal.set_style(AnsiStyle::Reset)?;
                    }
                    if ctx.code.current_highlighter.is_none() {
                        // If we have no highlighter for the current block, fall
                        // back to default style.
                        ctx.enable_style(AnsiStyle::Foreground(AnsiColour::Yellow))?
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
                    write!(ctx.output.terminal.write(), "\u{2022} ")?;
                    ctx.block.indent_level += 2;
                    ctx.list_item_kind.push(ListItemKind::Unordered);
                }
                Some(ListItemKind::Ordered(number)) => {
                    write!(ctx.output.terminal.write(), "{:>2}. ", number)?;
                    ctx.block.indent_level += 4;
                    ctx.list_item_kind.push(ListItemKind::Ordered(number + 1));
                }
                None => panic!("List item without list item kind"),
            }
        }
        FootnoteDefinition(_) => panic!("mdcat does not support footnotes"),
        Table(_) | TableHead | TableRow | TableCell => panic!("mdcat does not support tables"),
        Emphasis => ctx.enable_emphasis()?,
        Strong => ctx.enable_style(AnsiStyle::Bold)?,
        Code => ctx.enable_style(AnsiStyle::Foreground(AnsiColour::Yellow))?,
        Link(destination, _) => {
            // Try to create an inline link, provided that the format supports
            // those and we can parse the destination as valid URL.  If we can't
            // or if the format doesn't support inline links, don't do anything
            // here; we will write a reference link when closing the link tag.
            let url = ctx.input.resolve_reference(&destination).into_url();
            if ctx.output.terminal.set_link(url.as_str()).is_ok() {
                ctx.links.inside_inline_link = true;
            }
        }
        Image(link, _title) => {
            let resource = ctx.input.resolve_reference(&link);
            if ctx
                .output
                .terminal
                .write_inline_image(ctx.output.size, &resource, ctx.input.resource_access)
                .is_ok()
            {
                // If we could write an inline image, disable text output to
                // suppress the image title.
                ctx.image.inline_image = true;
            }
        }
    };
    Ok(())
}

/// Write the end of a `tag` in the given context.
fn end_tag<'a, W: Write>(ctx: &mut Context<'a, W>, tag: Tag<'a>) -> Result<(), Error> {
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
                    ctx.output.terminal.set_style(AnsiStyle::Reset)?;
                    ctx.flush_styles()?;
                    ctx.code.current_highlighter = None;
                }
            }
            ctx.write_border()?;
            // Move back to block context, but do not add a dedicated margin
            // because the bottom border we printed above already acts as
            // margin.
            ctx.block.level = BlockLevel::Block;
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
        FootnoteDefinition(_) | Table(_) | TableHead | TableRow | TableCell => {}
        Emphasis => {
            ctx.reset_last_style()?;
            ctx.style.emphasis_level -= 1;
            ()
        }
        Strong | Code => ctx.reset_last_style()?,
        Link(destination, title) => if ctx.links.inside_inline_link {
            ctx.output.terminal.set_link("")?;
            ctx.links.inside_inline_link = false;
        } else {
            // When we did not write an inline link, create a normal reference
            // link instead.  Even if the terminal supports inline links this
            // can still happen for anything that's not a valid URL.
            match ctx.links.last_text {
                Some(ref text) if *text == destination => {
                    // Do nothing if the last printed text matches the
                    // destination of the link.  In this we likely looked at an
                    // inline autolink and we should not repeat the link when
                    // it's already in text.
                }
                _ => {
                    let index = ctx.add_link(destination, title);
                    // Reference link
                    ctx.enable_style(AnsiStyle::Foreground(AnsiColour::Blue))?;
                    write!(ctx.output.terminal.write(), "[{}]", index)?;
                    ctx.reset_last_style()?;
                }
            }
        },
        Image(link, _title) => {
            if !ctx.image.inline_image {
                // If we could not write an inline image, write the image link
                // after the image title.
                ctx.enable_style(AnsiStyle::Foreground(AnsiColour::Blue))?;
                write!(ctx.output.terminal.write(), " ({})", link)?;
                ctx.reset_last_style()?;
            }
            ctx.image.inline_image = false;
        }
    };
    Ok(())
}
