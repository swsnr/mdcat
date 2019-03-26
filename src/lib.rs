// Copyright 2018-2019 Sebastian Wiesner <sebastian@swsnr.de>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

// 	http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![deny(warnings, missing_docs, clippy::all)]

//! Write markdown to TTYs.

#[cfg(feature = "resources")]
use url;

use ansi_term::{Colour, Style};
use failure::Error;
use pulldown_cmark::Event::*;
use pulldown_cmark::Tag::*;
use pulldown_cmark::{CowStr, Event, Tag};
use std::collections::VecDeque;
use std::io;
use std::io::Write;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

mod resources;
mod terminal;

// Expose some select things for use in main
pub use crate::resources::ResourceAccess;
pub use crate::terminal::*;

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
pub fn push_tty<'a, 'e, W, I>(
    writer: &'a mut W,
    capabilities: TerminalCapabilities,
    size: TerminalSize,
    mut events: I,
    base_dir: &'a Path,
    resource_access: ResourceAccess,
    syntax_set: SyntaxSet,
) -> Result<(), Error>
where
    I: Iterator<Item = Event<'e>>,
    W: Write,
{
    let theme = &ThemeSet::load_defaults().themes["Solarized (dark)"];
    events
        .try_fold(
            Context::new(
                writer,
                capabilities,
                size,
                base_dir,
                resource_access,
                syntax_set,
                theme,
            ),
            write_event,
        )?
        .write_pending_links()?;
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
    destination: CowStr<'a>,
    /// The link title.
    title: CowStr<'a>,
}

/// Input context.
#[cfg(feature = "resources")]
struct ResourceContext<'a> {
    /// The base directory, to resolve relative paths.
    base_dir: &'a Path,
    /// What resources we may access when processing markdown.
    resource_access: ResourceAccess,
}

#[cfg(feature = "resources")]
impl ResourceContext<'_> {
    /// Resolve a reference in the input.
    ///
    /// If `reference` parses as URL return the parsed URL.  Otherwise assume
    /// `reference` is a file path, resolve it against `base_dir` and turn it
    /// into a file:// URL.  If this also fails return `None`.
    fn resolve_reference(&self, reference: &str) -> Option<url::Url> {
        use url::Url;
        Url::parse(reference)
            .or_else(|_| Url::from_file_path(self.base_dir.join(reference)))
            .ok()
    }
}

/// Context for TTY output.
struct OutputContext<'a, W: Write> {
    /// The terminal dimensions to limit output to.
    size: TerminalSize,
    /// A writer to the terminal.
    writer: &'a mut W,
    /// The capabilities of the terminal.
    capabilities: TerminalCapabilities,
}

#[derive(Debug)]
struct StyleContext {
    /// The current style
    current: Style,
    /// Previous styles.
    ///
    /// Holds previous styles; whenever we disable the current style we restore
    /// the last one from this list.
    previous: Vec<Style>,
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
    last_text: Option<CowStr<'a>>,
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
struct Context<'io, 'c, 'l, W: Write> {
    #[cfg(feature = "resources")]
    /// Context for input.
    resources: ResourceContext<'io>,
    /// Context for output.
    output: OutputContext<'io, W>,
    /// Context for styling
    style: StyleContext,
    /// Context for the current block.
    block: BlockContext,
    /// Context to keep track of links.
    links: LinkContext<'l>,
    /// Context for code blocks
    code: CodeContext<'c>,
    /// Context for images.
    image: ImageContext,
    /// The kind of the current list item.
    ///
    /// A stack of kinds to address nested lists.
    list_item_kind: Vec<ListItemKind>,
}

impl<'io, 'c, 'l, W: Write> Context<'io, 'c, 'l, W> {
    fn new(
        writer: &'io mut W,
        capabilities: TerminalCapabilities,
        size: TerminalSize,
        base_dir: &'io Path,
        resource_access: ResourceAccess,
        syntax_set: SyntaxSet,
        theme: &'c Theme,
    ) -> Context<'io, 'c, 'l, W> {
        #[cfg(not(feature = "resources"))]
        {
            // Mark variables as used if resources are disabled to keep public
            // interface stable but avoid compiler warnings
            let _ = base_dir;
            let _ = resource_access;
        }
        Context {
            #[cfg(feature = "resources")]
            resources: ResourceContext {
                base_dir,
                resource_access,
            },
            output: OutputContext {
                size,
                writer,
                capabilities,
            },
            style: StyleContext {
                current: Style::new(),
                previous: Vec::new(),
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
    fn start_inline_text(&mut self) -> io::Result<()> {
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
    fn end_inline_text_with_margin(&mut self) -> io::Result<()> {
        if let BlockLevel::Inline = self.block.level {
            self.newline()?;
        };
        // We are back at blocks now
        self.block.level = BlockLevel::Block;
        Ok(())
    }

    /// Write a newline.
    ///
    /// Restart all current styles after the newline.
    fn newline(&mut self) -> io::Result<()> {
        writeln!(self.output.writer)
    }

    /// Write a newline and indent.
    ///
    /// Reset format before the line break, and set all active styles again
    /// after the line break.
    fn newline_and_indent(&mut self) -> io::Result<()> {
        self.newline()?;
        self.indent()
    }

    /// Indent according to the current indentation level.
    fn indent(&mut self) -> io::Result<()> {
        write!(
            self.output.writer,
            "{}",
            " ".repeat(self.block.indent_level)
        )
        .map_err(Into::into)
    }

    /// Push a new style.
    ///
    /// Pass the current style to `f` and push the style it returns as the new
    /// current style.
    fn set_style(&mut self, style: Style) {
        self.style.previous.push(self.style.current);
        self.style.current = style;
    }

    /// Drop the current style, and restore the previous one.
    fn drop_style(&mut self) {
        match self.style.previous.pop() {
            Some(old) => self.style.current = old,
            None => self.style.current = Style::new(),
        };
    }

    /// Write `text` with the given `style`.
    fn write_styled<S: AsRef<str>>(&mut self, style: &Style, text: S) -> io::Result<()> {
        match self.output.capabilities.style {
            StyleCapability::None => write!(self.output.writer, "{}", text.as_ref())?,
            StyleCapability::Ansi(ref ansi) => {
                ansi.write_styled(self.output.writer, style, text)?
            }
        }
        Ok(())
    }

    /// Write `text` with current style.
    fn write_styled_current<S: AsRef<str>>(&mut self, text: S) -> io::Result<()> {
        let style = self.style.current;
        self.write_styled(&style, text)
    }

    /// Enable emphasis.
    ///
    /// Enable italic or upright text according to the current emphasis level.
    fn enable_emphasis(&mut self) {
        self.style.emphasis_level += 1;
        let is_italic = self.style.emphasis_level % 2 == 1;
        {
            let new_style = Style {
                is_italic,
                ..self.style.current
            };
            self.set_style(new_style);
        }
    }

    /// Add a link to the context.
    ///
    /// Return the index of the link.
    fn add_link(&mut self, destination: CowStr<'l>, title: CowStr<'l>) -> usize {
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
            let link_style = self.style.current.fg(Colour::Blue);
            while let Some(link) = self.links.pending_links.pop_front() {
                let link_text = format!("[{}]: {} {}", link.index, link.destination, link.title);
                self.write_styled(&link_style, link_text)?;
                self.newline()?
            }
        };
        Ok(())
    }

    /// Write a simple border.
    fn write_border(&mut self) -> io::Result<()> {
        let separator = "\u{2500}".repeat(self.output.size.width.min(20));
        let style = self.style.current.fg(Colour::Green);
        self.write_styled(&style, separator)?;
        self.newline()
    }

    /// Write highlighted `text`.
    ///
    /// If the code context has a highlighter, use it to highlight `text` and
    /// write it.  Otherwise write `text` without highlighting.
    fn write_highlighted(&mut self, text: CowStr<'l>) -> io::Result<()> {
        let mut wrote_highlighted: bool = false;
        if let Some(ref mut highlighter) = self.code.current_highlighter {
            if let StyleCapability::Ansi(ref ansi) = self.output.capabilities.style {
                let regions = highlighter.highlight(&text, &self.code.syntax_set);
                highlighting::write_as_ansi(self.output.writer, ansi, &regions)?;
                wrote_highlighted = true;
            }
        }
        if !wrote_highlighted {
            self.write_styled_current(&text)?;
            self.links.last_text = Some(text);
        }
        Ok(())
    }

    /// Set a mark on the current position of the terminal if supported,
    /// otherwise do nothing.
    fn set_mark_if_supported(&mut self) -> io::Result<()> {
        match self.output.capabilities.marks {
            #[cfg(feature = "iterm2")]
            MarkCapability::ITerm2(ref marks) => marks.set_mark(self.output.writer),
            MarkCapability::None => Ok(()),
        }
    }
}

/// Write a single `event` in the given context.
fn write_event<'io, 'c, 'l, W: Write>(
    mut ctx: Context<'io, 'c, 'l, W>,
    event: Event<'l>,
) -> Result<Context<'io, 'c, 'l, W>, Error> {
    match event {
        SoftBreak | HardBreak => {
            ctx.newline_and_indent()?;
            Ok(ctx)
        }
        Text(text) => {
            // When we wrote an inline image suppress the text output, ie, the
            // image title.  We do not need it if we can show the image on the
            // terminal.
            if !ctx.image.inline_image {
                ctx.write_highlighted(text)?;
            }
            Ok(ctx)
        }
        TaskListMarker(_) => panic!("mdcat does not support task lists"),
        Start(tag) => start_tag(ctx, tag),
        End(tag) => end_tag(ctx, tag),
        Html(content) => {
            ctx.newline()?;
            let html_style = ctx.style.current.fg(Colour::Green);
            for line in content.lines() {
                ctx.write_styled(&html_style, line)?;
                ctx.newline()?;
            }
            Ok(ctx)
        }
        InlineHtml(tag) => {
            let style = ctx.style.current.fg(Colour::Green);
            ctx.write_styled(&style, tag)?;
            Ok(ctx)
        }
        FootnoteReference(_) => panic!("mdcat does not support footnotes"),
    }
}

/// Write the start of a `tag` in the given context.
fn start_tag<'io, 'c, 'l, W: Write>(
    mut ctx: Context<'io, 'c, 'l, W>,
    tag: Tag<'l>,
) -> Result<Context<'io, 'c, 'l, W>, Error> {
    match tag {
        HtmlBlock => panic!("mdcat does not support HTML blocks"),
        Paragraph => ctx.start_inline_text()?,
        Rule => {
            ctx.start_inline_text()?;
            let rule = "\u{2550}".repeat(ctx.output.size.width as usize);
            let style = ctx.style.current.fg(Colour::Green);
            ctx.write_styled(&style, rule)?
        }
        Header(level) => {
            // Before we start a new header, write all pending links to keep
            // them close to the text where they appeared in
            ctx.write_pending_links()?;
            ctx.start_inline_text()?;
            ctx.set_mark_if_supported()?;
            ctx.set_style(Style::new().fg(Colour::Blue).bold());
            ctx.write_styled_current("\u{2504}".repeat(level as usize))?
        }
        BlockQuote => {
            ctx.block.indent_level += 4;
            ctx.start_inline_text()?;
            // Make emphasis style and add green colour.
            ctx.enable_emphasis();
            ctx.style.current = ctx.style.current.fg(Colour::Green);
        }
        CodeBlock(name) => {
            ctx.start_inline_text()?;
            ctx.write_border()?;
            // Try to get a highlighter for the current code.
            ctx.code.current_highlighter = if name.is_empty() {
                None
            } else {
                ctx.code
                    .syntax_set
                    .find_syntax_by_token(&name)
                    .map(|syntax| HighlightLines::new(syntax, ctx.code.theme))
            };
            if ctx.code.current_highlighter.is_none() {
                // If we found no highlighter (code block had no language or
                // a language synctex doesn't support) we set a style to
                // highlight the code as generic fixed block.
                //
                // If we have a highlighter we set no style at all because
                // we pass the entire block contents through the highlighter
                // and directly write the result as ANSI.
                let style = ctx.style.current.fg(Colour::Yellow);
                ctx.set_style(style);
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
        FootnoteDefinition(_) => panic!("mdcat does not support footnotes"),
        Table(_) | TableHead | TableRow | TableCell => panic!("mdcat does not support tables"),
        Strikethrough => panic!("mdcat does not support strikethrough"),
        Emphasis => ctx.enable_emphasis(),
        Strong => {
            let style = ctx.style.current.bold();
            ctx.set_style(style)
        }
        Code => {
            let style = ctx.style.current.fg(Colour::Yellow);
            ctx.set_style(style)
        }
        Link(_, destination, _) => {
            // Do nothing if the terminal doesn’t support inline links of if `destination` is no
            // valid URL:  We will write a reference link when closing the link tag.
            match ctx.output.capabilities.links {
                #[cfg(feature = "osc8_links")]
                LinkCapability::OSC8(ref osc8) => {
                    // TODO: check link type (first tuple element) to write proper mailto link for
                    // emails
                    if let Some(url) = ctx.resources.resolve_reference(&destination) {
                        osc8.set_link_url(ctx.output.writer, url)?;
                        ctx.links.inside_inline_link = true;
                    }
                }
                LinkCapability::None => {
                    // Just mark destination as used
                    let _ = destination;
                }
            }
        }
        Image(_, link, _title) => match ctx.output.capabilities.image {
            #[cfg(feature = "terminology")]
            ImageCapability::Terminology(ref terminology) => {
                let access = ctx.resources.resource_access;
                if let Some(url) = ctx
                    .resources
                    .resolve_reference(&link)
                    .filter(|url| access.permits(url))
                {
                    terminology.write_inline_image(
                        &mut ctx.output.writer,
                        ctx.output.size,
                        &url,
                    )?;
                    ctx.image.inline_image = true;
                }
            }
            #[cfg(feature = "iterm2")]
            ImageCapability::ITerm2(ref iterm2) => {
                let access = ctx.resources.resource_access;
                if let Some(url) = ctx
                    .resources
                    .resolve_reference(&link)
                    .filter(|url| access.permits(url))
                {
                    if let Ok(contents) = iterm2.read_and_render(&url) {
                        iterm2.write_inline_image(ctx.output.writer, url.as_str(), &contents)?;
                        ctx.image.inline_image = true;
                    }
                }
            }
            ImageCapability::None => {
                // Just to mark "link" as used
                let _ = link;
            }
        },
    };
    Ok(ctx)
}

/// Write the end of a `tag` in the given context.
fn end_tag<'io, 'c, 'l, W: Write>(
    mut ctx: Context<'io, 'c, 'l, W>,
    tag: Tag<'l>,
) -> Result<Context<'io, 'c, 'l, W>, Error> {
    match tag {
        HtmlBlock => panic!("mdcat does not support HTML blocks"),
        Paragraph => ctx.end_inline_text_with_margin()?,
        Rule => ctx.end_inline_text_with_margin()?,
        Header(_) => {
            ctx.drop_style();
            ctx.end_inline_text_with_margin()?
        }
        BlockQuote => {
            ctx.block.indent_level -= 4;
            // Drop emphasis and current style
            ctx.style.emphasis_level -= 1;
            ctx.drop_style();
            ctx.end_inline_text_with_margin()?
        }
        CodeBlock(_) => {
            match ctx.code.current_highlighter {
                None => ctx.drop_style(),
                Some(_) => {
                    // If we had a highlighter we used `write_ansi` to write the
                    // entire highlighted block and so don't need to reset the
                    // current style here
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
        Strikethrough => panic!("mdcat does not support strikethrough"),
        Emphasis => {
            ctx.drop_style();
            ctx.style.emphasis_level -= 1;
        }
        Strong | Code => ctx.drop_style(),
        Link(_, destination, title) => {
            if ctx.links.inside_inline_link {
                match ctx.output.capabilities.links {
                    #[cfg(feature = "osc8_links")]
                    LinkCapability::OSC8(ref osc8) => {
                        osc8.clear_link(ctx.output.writer)?;
                    }
                    LinkCapability::None => {}
                }
                ctx.links.inside_inline_link = false;
            } else {
                // When we did not write an inline link, create a normal reference
                // link instead.  Even if the terminal supports inline links this
                // can still happen for anything that's not a valid URL.
                match ctx.links.last_text {
                    Some(ref text) if *text == destination => {
                        // Do nothing if the last printed text matches the destination of the link.
                        // In this case we likely looked at an inline autolink and we should not
                        // repeat the link when it's already in text.
                    }
                    _ => {
                        // Reference link
                        let index = ctx.add_link(destination, title);
                        let style = ctx.style.current.fg(Colour::Blue);
                        ctx.write_styled(&style, format!("[{}]", index))?
                    }
                }
            }
        }
        Image(_, link, _) => {
            if !ctx.image.inline_image {
                // If we could not write an inline image, write the image link
                // after the image title.
                let style = ctx.style.current.fg(Colour::Blue);
                ctx.write_styled(&style, format!(" ({})", link))?
            }
            ctx.image.inline_image = false;
        }
    };
    Ok(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use pulldown_cmark::Parser;

    fn render_string(
        input: &str,
        base_dir: &Path,
        resource_access: ResourceAccess,
        syntax_set: SyntaxSet,
        capabilities: TerminalCapabilities,
        size: TerminalSize,
    ) -> Result<Vec<u8>, Error> {
        let source = Parser::new(input);
        let mut sink = Vec::new();
        push_tty(
            &mut sink,
            capabilities,
            size,
            source,
            base_dir,
            resource_access,
            syntax_set,
        )?;
        Ok(sink)
    }

    #[test]
    #[allow(non_snake_case)]
    fn GH_49_format_no_colour_simple() {
        let result = String::from_utf8(
            render_string(
                "_lorem_ **ipsum** dolor **sit** _amet_",
                Path::new("/"),
                ResourceAccess::LocalOnly,
                SyntaxSet::default(),
                TerminalCapabilities::none(),
                TerminalSize::default(),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result, "lorem ipsum dolor sit amet\n");
    }
}
