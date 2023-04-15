// Copyright Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Rendering algorithm.

use std::io::prelude::*;
use std::io::Result;

use anstyle::{Effects, Style};
use pulldown_cmark::Event::*;
use pulldown_cmark::Tag::*;
use pulldown_cmark::{Event, LinkType};
use syntect::highlighting::HighlightIterator;
use syntect::util::LinesWithEndings;
use textwrap::core::display_width;
use tracing::{event, instrument, Level};
use url::Url;

use crate::render::highlighting::HIGHLIGHTER;
use crate::resources::ResourceUrlHandler;
use crate::theme::CombineStyle;
use crate::{Environment, Settings};

mod data;
mod highlighting;
mod state;
mod write;

use crate::references::*;
use state::*;
use write::*;

use crate::render::data::CurrentLine;
use crate::render::state::MarginControl::{Margin, NoMargin};
use crate::terminal::capabilities::LinkCapability;
pub use data::StateData;
pub use state::State;
pub use state::StateAndData;

#[allow(clippy::cognitive_complexity)]
#[instrument(level = "trace", skip(writer, settings, environment, resource_handler))]
pub fn write_event<'a, W: Write>(
    writer: &mut W,
    settings: &Settings,
    environment: &Environment,
    resource_handler: &dyn ResourceUrlHandler,
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
                    settings.theme.heading_style,
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
                &settings.theme,
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
                &settings.theme.html_block_style,
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
                &settings.theme,
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
                    settings.theme.heading_style.on_top_of(&style),
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
                    writer, settings, indent, style, kind,
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
                &settings.theme.html_block_style.on_top_of(&attrs.style),
                html,
            )?;
            stack
                .current(attrs.without_margin_for_html_only().into())
                .and_data(data)
                .ok()
        }

        // Lists
        (Stacked(stack, Inline(ListItem(kind, state), attrs)), Start(Item)) => {
            let InlineAttrs { indent, style, .. } = attrs;
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
                .and_data(data.current_line(CurrentLine {
                    length: indent as usize,
                    trailing_space: None,
                }))
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
            let InlineAttrs { indent, style, .. } = attrs;
            stack
                .push(Inline(ListItem(kind, ItemBlock), attrs))
                .current(write_start_code_block(writer, settings, indent, style, ck)?)
                .and_data(data)
                .ok()
        }
        (Stacked(stack, Inline(ListItem(kind, _), attrs)), Rule) => {
            writeln!(writer)?;
            write_indent(writer, attrs.indent)?;
            write_rule(
                writer,
                &settings.terminal_capabilities,
                &settings.theme,
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
                    settings.theme.heading_style.on_top_of(&style),
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
            let InlineAttrs { indent, style, .. } = attrs;
            let data = if state != ItemBlock {
                // End the inline text of this item
                writeln!(writer)?;
                data.current_line(CurrentLine::empty())
            } else {
                data
            };
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
            write_code_block_border(
                writer,
                &settings.theme,
                &settings.terminal_capabilities,
                &settings.terminal_size,
            )?;
            stack.pop().and_data(data).ok()
        }

        // Highlighted code blocks
        (Stacked(stack, HighlightBlock(mut attrs)), Text(text)) => {
            for line in LinesWithEndings::from(&text) {
                let ops = attrs
                    .parse_state
                    .parse_line(line, settings.syntax_set)
                    .expect("syntect parsing shouldn't fail in mdcat");
                highlighting::write_as_ansi(
                    writer,
                    attrs.ansi,
                    HighlightIterator::new(&mut attrs.highlight_state, &ops, line, &HIGHLIGHTER),
                )?;
                if text.ends_with('\n') {
                    write_indent(writer, attrs.indent)?;
                }
            }
            stack.current(attrs.into()).and_data(data).ok()
        }
        (Stacked(stack, HighlightBlock(_)), End(CodeBlock(_))) => {
            write_code_block_border(
                writer,
                &settings.theme,
                &settings.terminal_capabilities,
                &settings.terminal_size,
            )?;
            stack.pop().and_data(data).ok()
        }

        // Inline markup
        (Stacked(stack, Inline(state, attrs)), Start(Emphasis)) => {
            let InlineAttrs { style, indent } = attrs;
            let effects = style.get_effects();
            let style =
                style.effects(effects.set(Effects::ITALIC, !effects.contains(Effects::ITALIC)));
            stack
                .push(Inline(state, attrs))
                .current(Inline(state, InlineAttrs { style, indent }))
                .and_data(data)
                .ok()
        }
        (Stacked(stack, Inline(_, _)), End(Emphasis)) => stack.pop().and_data(data).ok(),
        (Stacked(stack, Inline(state, attrs)), Start(Strong)) => {
            let InlineAttrs { indent, .. } = attrs;
            let style = attrs.style.bold();
            stack
                .push(Inline(state, attrs))
                .current(Inline(state, InlineAttrs { style, indent }))
                .and_data(data)
                .ok()
        }
        (Stacked(stack, Inline(_, _)), End(Strong)) => stack.pop().and_data(data).ok(),
        (Stacked(stack, Inline(state, attrs)), Start(Strikethrough)) => {
            let InlineAttrs { indent, .. } = attrs;
            let style = attrs.style.strikethrough();
            stack
                .push(Inline(state, attrs))
                .current(Inline(state, InlineAttrs { style, indent }))
                .and_data(data)
                .ok()
        }
        (Stacked(stack, Inline(_, _)), End(Strikethrough)) => stack.pop().and_data(data).ok(),
        (Stacked(stack, Inline(state, attrs)), Code(code)) => {
            let current_line = write_styled_and_wrapped(
                writer,
                &settings.terminal_capabilities,
                &settings.theme.code_style.on_top_of(&attrs.style),
                settings.terminal_size.columns,
                attrs.indent as usize,
                data.current_line,
                code,
            )?;
            let data = StateData {
                current_line,
                ..data
            };
            Ok(stack.current(Inline(state, attrs)).and_data(data))
        }
        (Stacked(stack, Inline(ListItem(kind, state), attrs)), TaskListMarker(checked)) => {
            let marker = if checked { "\u{2611}" } else { "\u{2610}" };
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &attrs.style,
                marker,
            )?;
            let length = data.current_line.length + display_width(marker);
            Ok(stack
                .current(Inline(ListItem(kind, state), attrs))
                .and_data(data.current_line(CurrentLine {
                    length,
                    trailing_space: Some(" ".to_owned()),
                })))
        }
        // Inline line breaks
        (Stacked(stack, Inline(state, attrs)), SoftBreak) => {
            let length = data.current_line.length;

            Ok(stack
                .current(Inline(state, attrs))
                .and_data(data.current_line(CurrentLine {
                    length,
                    trailing_space: Some(" ".to_owned()),
                })))
        }
        (Stacked(stack, Inline(state, attrs)), HardBreak) => {
            writeln!(writer)?;
            write_indent(writer, attrs.indent)?;

            Ok(stack
                .current(Inline(state, attrs))
                .and_data(data.current_line(CurrentLine::empty())))
        }
        // Inline text
        (Stacked(stack, Inline(ListItem(kind, ItemBlock), attrs)), Text(text)) => {
            // Fresh text after a new block, so indent again.
            write_indent(writer, attrs.indent)?;
            let current_line = write_styled_and_wrapped(
                writer,
                &settings.terminal_capabilities,
                &attrs.style,
                settings.terminal_size.columns,
                attrs.indent as usize,
                data.current_line,
                text,
            )?;
            Ok(stack
                .current(Inline(ListItem(kind, ItemText), attrs))
                .and_data(StateData {
                    current_line,
                    ..data
                }))
        }
        // Inline blocks don't wrap
        (Stacked(stack, Inline(InlineBlock, attrs)), Text(text)) => {
            write_styled(writer, &settings.terminal_capabilities, &attrs.style, text)?;
            Ok(stack.current(Inline(InlineBlock, attrs)).and_data(data))
        }
        (Stacked(stack, Inline(state, attrs)), Text(text)) => {
            let current_line = write_styled_and_wrapped(
                writer,
                &settings.terminal_capabilities,
                &attrs.style,
                settings.terminal_size.columns,
                attrs.indent as usize,
                data.current_line,
                text,
            )?;
            Ok(stack.current(Inline(state, attrs)).and_data(StateData {
                current_line,
                ..data
            }))
        }
        // Inline HTML
        (Stacked(stack, Inline(ListItem(kind, ItemBlock), attrs)), Html(html)) => {
            // Fresh text after a new block, so indent again.
            write_indent(writer, attrs.indent)?;
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &settings.theme.inline_html_style.on_top_of(&attrs.style),
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
                &settings.theme.inline_html_style.on_top_of(&attrs.style),
                html,
            )?;
            stack.current(Inline(state, attrs)).and_data(data).ok()
        }
        // Ending inline text
        (Stacked(stack, Inline(_, _)), End(Paragraph)) => {
            writeln!(writer)?;
            Ok(stack
                .pop()
                .and_data(data.current_line(CurrentLine::empty())))
        }
        (Stacked(stack, Inline(_, _)), End(Heading(_, _, _))) => {
            writeln!(writer)?;
            Ok(stack
                .pop()
                .and_data(data.current_line(CurrentLine::empty())))
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
                        style: settings.theme.link_style.on_top_of(&style),
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
            let (data, index) = data.add_link(target, title, settings.theme.link_style);
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &settings.theme.link_style.on_top_of(&attrs.style),
                format!("[{index}]"),
            )?;
            stack.pop().and_data(data).ok()
        }

        // Images
        (Stacked(stack, Inline(state, attrs)), Start(Image(_, link, _))) => {
            let InlineAttrs { style, indent } = attrs;
            let resolved_link = environment.resolve_reference(&link);
            let image_state = match (settings.terminal_capabilities.image, resolved_link) {
                (Some(capability), Some(ref url)) => capability
                    .image_protocol()
                    .write_inline_image(writer, &resource_handler, url, &settings.terminal_size)
                    .map_err(|error| {
                        event!(Level::ERROR, ?error, %url, "failed to render image with capability {:?}: {:#}", capability, error);
                        error
                    })
                    .map(|_| RenderedImage)
                    .ok(),
                (None, Some(url)) =>
                    if let InlineLink(_) = state {
                        event!(Level::WARN, url = %url, "Terminal does not support images, want to render image as link but cannot: Already inside a link");
                        None
                    } else {
                        event!(Level::INFO, url = %url, "Terminal does not support images, rendering image as link");
                        match settings.terminal_capabilities.links {
                            Some(capability) => match capability {
                                LinkCapability::Osc8(osc8) => {
                                    osc8.set_link_url(writer, url, &environment.hostname)?;
                                    Some(Inline(
                                        InlineLink(capability),
                                        InlineAttrs {
                                            indent,
                                            style: settings.theme.image_link_style.on_top_of(&style),
                                        },
                                    ))
                                }
                            },
                            None => None,
                        }
                    },
                (_, None) => None,
            }
            .unwrap_or_else(|| {
                event!(Level::WARN, "Rendering image {} as inline text, without link", link);
                // Inside an inline link keep the link style; we cannot nest links so we
                // should clarify that clicking the link follows the link target and not the image.
                let style = if let InlineLink(_) = state {
                    style
                } else {
                    settings.theme.image_link_style.on_top_of(&style)
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
                let (data, index) = data.add_link(target, title, settings.theme.image_link_style);
                write_styled(
                    writer,
                    &settings.terminal_capabilities,
                    // Regardless of text style always colour the reference to make clear it points to
                    // an image
                    &settings.theme.image_link_style.on_top_of(&attrs.style),
                    format!("[{index}]"),
                )?;
                stack.pop().and_data(data).ok()
            }
        }

        // Unconditional returns to previous states
        (Stacked(stack, _), End(BlockQuote)) => stack.pop().and_data(data).ok(),
        (Stacked(stack, _), End(List(_))) => stack.pop().and_data(data).ok(),

        // Impossible events
        (s, e) => panic!("Event {e:?} impossible in state {s:?}"),
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
