// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Rendering algorithm.

use std::io::prelude::*;
use std::io::Result;

use ansi_term::{Colour, Style};
use anyhow::anyhow;
use pulldown_cmark::Event::*;
use pulldown_cmark::Tag::*;
use pulldown_cmark::{Event, LinkType};
use syntect::highlighting::{HighlightIterator, Highlighter, Theme};
use syntect::util::LinesWithEndings;
use tracing::{event, instrument, Level};
use url::Url;

use crate::terminal::*;
use crate::{Environment, Settings};

mod data;
mod state;
mod write;

use crate::references::*;
use state::*;
use write::*;

use crate::render::state::MarginControl::{Margin, NoMargin};
use crate::terminal::capabilities::LinkCapability;
pub use data::StateData;
pub use state::State;
pub use state::StateAndData;

#[allow(clippy::cognitive_complexity)]
#[instrument(level = "trace", skip(writer, settings, environment, theme))]
pub fn write_event<'a, W: Write>(
    writer: &mut W,
    settings: &Settings,
    environment: &Environment,
    theme: &Theme,
    state: State,
    data: StateData<'a>,
    event: Event<'a>,
) -> Result<StateAndData<StateData<'a>>> {
    use self::InlineState::*;
    use self::ListItemState::*;
    use self::StackedState::*;
    use State::*;

    event!(Level::TRACE, event = ?event, "rendering");
    match (state, event) {
        // Top level items
        (TopLevel(attrs), Start(Paragraph)) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            State::stack_onto(TopLevelAttrs::margin_before())
                .current(Inline(InlineText, InlineAttrs::default()))
                .and_data(data)
                .ok()
        }
        (TopLevel(attrs), Start(Heading(level, _, _))) => {
            let (data, links) = data.take_links();
            write_link_refs(writer, environment, &settings.terminal_capabilities, links)?;
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            write_mark(writer, &settings.terminal_capabilities)?;

            State::stack_onto(TopLevelAttrs::margin_before())
                .current(write_start_heading(
                    writer,
                    &settings.terminal_capabilities,
                    Style::new(),
                    level,
                )?)
                .and_data(data)
                .ok()
        }
        (TopLevel(attrs), Start(BlockQuote)) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            State::stack_onto(TopLevelAttrs::margin_before())
                .current(
                    // We've written a block-level margin already, so the first
                    // block inside the styled block should add another margin.
                    StyledBlockAttrs::default()
                        .block_quote()
                        .without_margin_before()
                        .into(),
                )
                .and_data(data)
                .ok()
        }
        (TopLevel(attrs), Rule) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            write_rule(
                writer,
                &settings.terminal_capabilities,
                settings.terminal_size.columns,
            )?;
            writeln!(writer)?;
            TopLevel(TopLevelAttrs::margin_before()).and_data(data).ok()
        }
        (TopLevel(attrs), Start(CodeBlock(kind))) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }

            State::stack_onto(TopLevelAttrs::margin_before())
                .current(write_start_code_block(
                    writer,
                    settings,
                    0,
                    Style::new(),
                    kind,
                    theme,
                )?)
                .and_data(data)
                .ok()
        }
        (TopLevel(attrs), Start(List(start))) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            let kind = start.map_or(ListItemKind::Unordered, |start| {
                ListItemKind::Ordered(start)
            });

            State::stack_onto(TopLevelAttrs::margin_before())
                .current(Inline(ListItem(kind, StartItem), InlineAttrs::default()))
                .and_data(data)
                .ok()
        }
        (TopLevel(attrs), Html(html)) => {
            if attrs.margin_before == Margin {
                writeln!(writer)?;
            }
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &Style::new().fg(Colour::Green),
                html,
            )?;
            TopLevel(TopLevelAttrs::no_margin_for_html_only())
                .and_data(data)
                .ok()
        }

        // Nested blocks with style, e.g. paragraphs in quotes, etc.
        (Stacked(stack, StyledBlock(attrs)), Start(Paragraph)) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            write_indent(writer, attrs.indent)?;
            let inline = InlineAttrs::from(&attrs);
            stack
                .push(attrs.with_margin_before().into())
                .current(Inline(InlineText, inline))
                .and_data(data)
                .ok()
        }
        (Stacked(stack, StyledBlock(attrs)), Start(BlockQuote)) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            stack
                .push(attrs.clone().with_margin_before().into())
                .current(attrs.without_margin_before().block_quote().into())
                .and_data(data)
                .ok()
        }
        (Stacked(stack, StyledBlock(attrs)), Rule) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            write_indent(writer, attrs.indent)?;
            write_rule(
                writer,
                &settings.terminal_capabilities,
                settings.terminal_size.columns - (attrs.indent as usize),
            )?;
            writeln!(writer)?;
            stack
                .current(attrs.with_margin_before().into())
                .and_data(data)
                .ok()
        }
        (Stacked(stack, StyledBlock(attrs)), Start(Heading(level, _, _))) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            write_indent(writer, attrs.indent)?;

            // We deliberately don't mark headings which aren't top-level.
            let style = attrs.style;
            stack
                .push(attrs.with_margin_before().into())
                .current(write_start_heading(
                    writer,
                    &settings.terminal_capabilities,
                    style,
                    level,
                )?)
                .and_data(data)
                .ok()
        }
        (Stacked(stack, StyledBlock(attrs)), Start(List(start))) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            let kind = start.map_or(ListItemKind::Unordered, |start| {
                ListItemKind::Ordered(start)
            });
            let inline = InlineAttrs::from(&attrs);
            stack
                .push(attrs.with_margin_before().into())
                .current(Inline(ListItem(kind, StartItem), inline))
                .and_data(data)
                .ok()
        }
        (Stacked(stack, StyledBlock(attrs)), Start(CodeBlock(kind))) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            let StyledBlockAttrs { indent, style, .. } = attrs;
            stack
                .push(attrs.into())
                .current(write_start_code_block(
                    writer, settings, indent, style, kind, theme,
                )?)
                .and_data(data)
                .ok()
        }
        (Stacked(stack, StyledBlock(attrs)), Html(html)) => {
            if attrs.margin_before == Margin {
                writeln!(writer)?;
            }
            write_indent(writer, attrs.indent)?;
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &attrs.style.fg(Colour::Green),
                html,
            )?;
            stack
                .current(attrs.without_margin_for_html_only().into())
                .and_data(data)
                .ok()
        }

        // Lists
        (Stacked(stack, Inline(ListItem(kind, state), attrs)), Start(Item)) => {
            let InlineAttrs { indent, style } = attrs;
            if state == ItemBlock {
                // Add margin
                writeln!(writer)?;
            }
            write_indent(writer, indent)?;
            let indent = match kind {
                ListItemKind::Unordered => {
                    write!(writer, "\u{2022} ")?;
                    indent + 2
                }
                ListItemKind::Ordered(no) => {
                    write!(writer, "{no:>2}. ")?;
                    indent + 4
                }
            };
            stack
                .current(Inline(
                    ListItem(kind, StartItem),
                    InlineAttrs { style, indent },
                ))
                .and_data(data)
                .ok()
        }
        (Stacked(stack, Inline(ListItem(kind, state), attrs)), Start(Paragraph)) => {
            if state != StartItem {
                // Write margin, unless we're at the start of the list item in which case the first line of the
                // paragraph should go right beside the item bullet.
                writeln!(writer)?;
                write_indent(writer, attrs.indent)?;
            }
            stack
                .push(Inline(ListItem(kind, ItemBlock), attrs.clone()))
                .current(Inline(InlineText, attrs))
                .and_data(data)
                .ok()
        }
        (Stacked(stack, Inline(ListItem(kind, _), attrs)), Start(CodeBlock(ck))) => {
            writeln!(writer)?;
            let InlineAttrs { indent, style } = attrs;
            stack
                .push(Inline(ListItem(kind, ItemBlock), attrs))
                .current(write_start_code_block(
                    writer, settings, indent, style, ck, theme,
                )?)
                .and_data(data)
                .ok()
        }
        (Stacked(stack, Inline(ListItem(kind, _), attrs)), Rule) => {
            writeln!(writer)?;
            write_indent(writer, attrs.indent)?;
            write_rule(
                writer,
                &settings.terminal_capabilities,
                settings.terminal_size.columns - (attrs.indent as usize),
            )?;
            writeln!(writer)?;
            stack
                .current(Inline(ListItem(kind, ItemBlock), attrs))
                .and_data(data)
                .ok()
        }
        (Stacked(stack, Inline(ListItem(kind, state), attrs)), Start(Heading(level, _, _))) => {
            if state != StartItem {
                writeln!(writer)?;
                write_indent(writer, attrs.indent)?;
            }
            // We deliberately don't mark headings which aren't top-level.
            let style = attrs.style;
            stack
                .push(Inline(ListItem(kind, ItemBlock), attrs))
                .current(write_start_heading(
                    writer,
                    &settings.terminal_capabilities,
                    style,
                    level,
                )?)
                .and_data(data)
                .ok()
        }
        (Stacked(stack, Inline(ListItem(kind, _), attrs)), Start(List(start))) => {
            writeln!(writer)?;
            let nested_kind = start.map_or(ListItemKind::Unordered, |start| {
                ListItemKind::Ordered(start)
            });
            stack
                .push(Inline(ListItem(kind, ItemBlock), attrs.clone()))
                .current(Inline(ListItem(nested_kind, StartItem), attrs))
                .and_data(data)
                .ok()
        }
        (Stacked(stack, Inline(ListItem(kind, _), attrs)), Start(BlockQuote)) => {
            writeln!(writer)?;
            let block_quote = StyledBlockAttrs::from(&attrs)
                .without_margin_before()
                .block_quote();
            stack
                .push(Inline(ListItem(kind, ItemBlock), attrs))
                .current(block_quote.into())
                .and_data(data)
                .ok()
        }
        (Stacked(stack, Inline(ListItem(kind, state), attrs)), End(Item)) => {
            let InlineAttrs { indent, style } = attrs;
            if state != ItemBlock {
                // End the inline text of this item
                writeln!(writer)?;
            }
            // Decrease indent back to the level where we can write the next item bullet, and increment the list item number.
            let (indent, kind) = match kind {
                ListItemKind::Unordered => (indent - 2, ListItemKind::Unordered),
                ListItemKind::Ordered(no) => (indent - 4, ListItemKind::Ordered(no + 1)),
            };
            stack
                .current(Inline(ListItem(kind, state), InlineAttrs { style, indent }))
                .and_data(data)
                .ok()
        }

        // Literal blocks without highlighting
        (Stacked(stack, LiteralBlock(attrs)), Text(text)) => {
            let LiteralBlockAttrs { indent, style } = attrs;
            for line in LinesWithEndings::from(&text) {
                write_styled(writer, &settings.terminal_capabilities, &style, line)?;
                if line.ends_with('\n') {
                    write_indent(writer, indent)?;
                }
            }
            stack.current(attrs.into()).and_data(data).ok()
        }
        (Stacked(stack, LiteralBlock(_)), End(CodeBlock(_))) => {
            write_border(
                writer,
                &settings.terminal_capabilities,
                &settings.terminal_size,
            )?;
            stack.pop().and_data(data).ok()
        }

        // Highlighted code blocks
        (Stacked(stack, HighlightBlock(mut attrs)), Text(text)) => {
            let highlighter = Highlighter::new(theme);
            for line in LinesWithEndings::from(&text) {
                let ops = attrs
                    .parse_state
                    .parse_line(line, &settings.syntax_set)
                    .expect("syntect parsing shouldn't fail in mdcat");
                highlighting::write_as_ansi(
                    writer,
                    attrs.ansi,
                    HighlightIterator::new(&mut attrs.highlight_state, &ops, line, &highlighter),
                )?;
                if text.ends_with('\n') {
                    write_indent(writer, attrs.indent)?;
                }
            }
            stack.current(attrs.into()).and_data(data).ok()
        }
        (Stacked(stack, HighlightBlock(_)), End(CodeBlock(_))) => {
            write_border(
                writer,
                &settings.terminal_capabilities,
                &settings.terminal_size,
            )?;
            stack.pop().and_data(data).ok()
        }

        // Inline markup
        (Stacked(stack, Inline(state, attrs)), Start(Emphasis)) => {
            let indent = attrs.indent;
            let style = Style {
                is_italic: !attrs.style.is_italic,
                ..attrs.style
            };
            stack
                .push(Inline(state, attrs))
                .current(Inline(state, InlineAttrs { style, indent }))
                .and_data(data)
                .ok()
        }
        (Stacked(stack, Inline(_, _)), End(Emphasis)) => stack.pop().and_data(data).ok(),
        (Stacked(stack, Inline(state, attrs)), Start(Strong)) => {
            let indent = attrs.indent;
            let style = attrs.style.bold();
            stack
                .push(Inline(state, attrs))
                .current(Inline(state, InlineAttrs { style, indent }))
                .and_data(data)
                .ok()
        }
        (Stacked(stack, Inline(_, _)), End(Strong)) => stack.pop().and_data(data).ok(),
        (Stacked(stack, Inline(state, attrs)), Start(Strikethrough)) => {
            let style = attrs.style.strikethrough();
            let indent = attrs.indent;
            stack
                .push(Inline(state, attrs))
                .current(Inline(state, InlineAttrs { style, indent }))
                .and_data(data)
                .ok()
        }
        (Stacked(stack, Inline(_, _)), End(Strikethrough)) => stack.pop().and_data(data).ok(),
        (Stacked(stack, Inline(state, attrs)), Code(code)) => {
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &attrs.style.fg(Colour::Yellow),
                code,
            )?;
            stack.current(Inline(state, attrs)).and_data(data).ok()
        }
        (Stacked(stack, Inline(ListItem(kind, state), attrs)), TaskListMarker(checked)) => {
            let marker = if checked { "\u{2611} " } else { "\u{2610} " };
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &attrs.style,
                marker,
            )?;
            stack
                .current(Inline(ListItem(kind, state), attrs))
                .and_data(data)
                .ok()
        }
        // Inline line breaks
        (Stacked(stack, Inline(state, attrs)), SoftBreak) => {
            writeln!(writer)?;
            write_indent(writer, attrs.indent)?;
            stack.current(Inline(state, attrs)).and_data(data).ok()
        }
        (Stacked(stack, Inline(state, attrs)), HardBreak) => {
            writeln!(writer)?;
            write_indent(writer, attrs.indent)?;
            stack.current(Inline(state, attrs)).and_data(data).ok()
        }
        // Inline text
        (Stacked(stack, Inline(ListItem(kind, ItemBlock), attrs)), Text(text)) => {
            // Fresh text after a new block, so indent again.
            write_indent(writer, attrs.indent)?;
            write_styled(writer, &settings.terminal_capabilities, &attrs.style, text)?;
            stack
                .current(Inline(ListItem(kind, ItemText), attrs))
                .and_data(data)
                .ok()
        }
        (Stacked(stack, Inline(state, attrs)), Text(text)) => {
            write_styled(writer, &settings.terminal_capabilities, &attrs.style, text)?;
            stack.current(Inline(state, attrs)).and_data(data).ok()
        }
        // Inline HTML
        (Stacked(stack, Inline(ListItem(kind, ItemBlock), attrs)), Html(html)) => {
            // Fresh text after a new block, so indent again.
            write_indent(writer, attrs.indent)?;
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &attrs.style.fg(Colour::Green),
                html,
            )?;
            stack
                .current(Inline(ListItem(kind, ItemText), attrs))
                .and_data(data)
                .ok()
        }
        (Stacked(stack, Inline(state, attrs)), Html(html)) => {
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &attrs.style.fg(Colour::Green),
                html,
            )?;
            stack.current(Inline(state, attrs)).and_data(data).ok()
        }
        // Ending inline text
        (Stacked(stack, Inline(_, _)), End(Paragraph)) => {
            writeln!(writer)?;
            stack.pop().and_data(data).ok()
        }
        (Stacked(stack, Inline(_, _)), End(Heading(_, _, _))) => {
            writeln!(writer)?;
            stack.pop().and_data(data).ok()
        }

        // Links.
        //
        // Links need a bit more work than standard inline markup because we
        // need to keep track of link references if we can't write inline links.
        (Stacked(stack, Inline(state, attrs)), Start(Link(link_type, target, _))) => {
            let link_state = settings
                .terminal_capabilities
                .links
                .and_then(|link_capability| match link_capability {
                    LinkCapability::Osc8(ref osc8) => {
                        let url = if let LinkType::Email = link_type {
                            // Turn email autolinks (i.e. <foo@example.com>) into mailto inline links
                            Url::parse(&format!("mailto:{target}")).ok()
                        } else {
                            environment.resolve_reference(&target)
                        };
                        url.and_then(|url| {
                            osc8.set_link_url(writer, url, &environment.hostname).ok()
                        })
                        .and(Some(InlineLink(link_capability)))
                    }
                })
                .unwrap_or(InlineText);

            let InlineAttrs { style, indent } = attrs;
            stack
                .push(Inline(state, attrs))
                .current(Inline(
                    link_state,
                    InlineAttrs {
                        indent,
                        style: style.fg(Colour::Blue),
                    },
                ))
                .and_data(data)
                .ok()
        }
        (Stacked(stack, Inline(InlineLink(capability), _)), End(Link(_, _, _))) => {
            match capability {
                LinkCapability::Osc8(ref osc8) => {
                    osc8.clear_link(writer)?;
                }
            }
            stack.pop().and_data(data).ok()
        }
        // When closing email or autolinks in inline text just return because link, being identical
        // to the link text, was already written.
        (Stacked(stack, Inline(InlineText, _)), End(Link(LinkType::Autolink, _, _))) => {
            stack.pop().and_data(data).ok()
        }
        (Stacked(stack, Inline(InlineText, _)), End(Link(LinkType::Email, _, _))) => {
            stack.pop().and_data(data).ok()
        }
        (Stacked(stack, Inline(InlineText, attrs)), End(Link(_, target, title))) => {
            let (data, index) = data.add_link(target, title, Colour::Blue);
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &attrs.style.fg(Colour::Blue),
                format!("[{index}]"),
            )?;
            stack.pop().and_data(data).ok()
        }

        // Images
        (Stacked(stack, Inline(state, attrs)), Start(Image(_, link, _))) => {
            let InlineAttrs { style, indent } = attrs;
            use crate::terminal::capabilities::ImageCapability::*;
            let resolved_link = environment.resolve_reference(&link);
            let image_state = match (settings.terminal_capabilities.image, resolved_link) {
                (Some(Terminology(terminology)), Some(ref url)) => {
                    terminology.write_inline_image(writer, settings.terminal_size, url)?;
                    Some(RenderedImage)
                }
                (Some(ITerm2(iterm2)), Some(ref url)) => iterm2
                    .read_and_render(url, settings.resource_access)
                    .map_err(|error| {
                        event!(Level::ERROR, ?error, %url, ?settings.resource_access, "failed to render image in iterm2: {:#}", error);
                        error
                    })
                    .and_then(|contents| {
                        // Use the last segment as file name for iterm2.
                        let name = url.path_segments().and_then(|s| s.last());
                        iterm2.write_inline_image(writer, name, &contents).map_err(|error| {
                            event!(Level::ERROR, ?error, "failed to write iterm image: {:#}", error);
                            error
                        })?;
                        Ok(RenderedImage)
                    })
                    .map(|_| RenderedImage)
                    .ok(),
                (Some(Kitty(kitty)), Some(ref url)) => settings
                    .terminal_size
                    .pixels
                    .ok_or_else(|| {
                        event!(Level::ERROR, "Kitty surprisingly did not report pixel size, cannot render image");
                        anyhow!("Terminal pixel size not available")
                    })
                    .and_then(|size| {
                        let image = kitty.read_and_render(url, settings.resource_access, size).map_err(|error| {
                            event!(Level::ERROR, ?error, %url, ?settings.resource_access, "failed to render image in kitty: {:#}", error);
                            error
                        })?;
                        kitty.write_inline_image(writer, image).map_err(|error| {
                            event!(Level::ERROR, ?error, "failed to write iterm kitty: {:#}", error);
                            error
                        })?;
                        Ok(RenderedImage)
                    })
                    .ok(),
                (None, Some(url)) => {
                    if let InlineLink(_) = state {
                        event!(Level::WARN, url = %url, "Terminal did not support images, want to render image as link but cannot: Already inside a link");
                        None
                    } else {
                        event!(Level::INFO, url = %url, "Terminal did not support images, rendering image as link");
                        match settings.terminal_capabilities.links {
                            Some(capability) => match capability {
                                LinkCapability::Osc8(osc8) => {
                                    osc8.set_link_url(writer, url, &environment.hostname)?;
                                    Some(Inline(
                                        InlineLink(capability),
                                        InlineAttrs {
                                            indent,
                                            style: style.fg(Colour::Purple),
                                        },
                                    ))
                                }
                            },
                            None => None,
                        }
                    }
                }
                (_, None) => None,
            }
            .unwrap_or_else(|| {
                event!(Level::WARN, "Rendering image {} as inline text, without link", link);
                // Inside an inline link keep the blue foreground colour; we cannot nest links so we
                // should clarify that clicking the link follows the link target and not the image.
                let style = if let InlineLink(_) = state {
                    style
                } else {
                    style.fg(Colour::Purple)
                };
                Inline(InlineText, InlineAttrs { style, indent })
            });
            stack
                .push(Inline(state, attrs))
                .current(image_state)
                .and_data(data)
                .ok()
        }
        (Stacked(stack, RenderedImage), Text(_)) => {
            Stacked(stack, RenderedImage).and_data(data).ok()
        }
        (Stacked(stack, RenderedImage), End(Image(_, _, _))) => stack.pop().and_data(data).ok(),
        (Stacked(stack, Inline(state, attrs)), End(Image(_, target, title))) => {
            if let InlineLink(capability) = state {
                match capability {
                    LinkCapability::Osc8(ref osc8) => {
                        osc8.clear_link(writer)?;
                    }
                }
                stack.pop().and_data(data).ok()
            } else {
                let (data, index) = data.add_link(target, title, Colour::Purple);
                write_styled(
                    writer,
                    &settings.terminal_capabilities,
                    // Regardless of text style always colour the reference to make clear it points to
                    // an image
                    &attrs.style.fg(Colour::Purple),
                    format!("[{index}]"),
                )?;
                stack.pop().and_data(data).ok()
            }
        }

        // Unconditional returns to previous states
        (Stacked(stack, _), End(BlockQuote)) => stack.pop().and_data(data).ok(),
        (Stacked(stack, _), End(List(_))) => stack.pop().and_data(data).ok(),

        // Impossible events
        (s, e) => panic!(
            "Event {e:?} impossible in state {s:?}

Please do report an issue at <https://github.com/swsnr/mdcat/issues/new> including

* a copy of this message, and
* the markdown document which caused this error.",
        ),
    }
}

#[instrument(level = "trace", skip(writer, settings, environment))]
pub fn finish<'a, W: Write>(
    writer: &mut W,
    settings: &Settings,
    environment: &Environment,
    state: State,
    data: StateData<'a>,
) -> Result<()> {
    match state {
        State::TopLevel(_) => {
            event!(
                Level::TRACE,
                "Writing {} pending link definitions",
                data.pending_link_definitions.len()
            );
            write_link_refs(
                writer,
                environment,
                &settings.terminal_capabilities,
                data.pending_link_definitions,
            )?;
            Ok(())
        }
        _ => {
            panic!("Must finish in state TopLevel but got: {state:?}");
        }
    }
}
