// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Rendering algorithm.

use std::io::prelude::*;
use std::path::Path;

use ansi_term::{Colour, Style};
use fehler::throws;
use pulldown_cmark::Event::*;
use pulldown_cmark::Tag::*;
use pulldown_cmark::{Event, LinkType};
use std::io::Error;
use syntect::highlighting::{HighlightIterator, Highlighter, Theme};
use syntect::util::LinesWithEndings;
use url::Url;

use crate::terminal::*;
use crate::Settings;

mod data;
mod state;
mod write;

use state::*;
use write::*;

use crate::render::state::MarginControl::{Margin, NoMargin};
pub use data::StateData;
pub use state::State;

fn resolve_reference(base_dir: &Path, reference: &str) -> Option<Url> {
    Url::parse(reference)
        .or_else(|_| Url::from_file_path(base_dir.join(reference)))
        .ok()
}

#[allow(clippy::cognitive_complexity)]
#[throws]
pub fn write_event<'a, W: Write>(
    writer: &mut W,
    settings: &Settings,
    base_dir: &Path,
    theme: &Theme,
    state: State,
    data: StateData<'a>,
    event: Event<'a>,
) -> (State, StateData<'a>) {
    use self::InlineState::*;
    use self::ListItemState::*;
    use self::StackedState::*;
    use State::*;
    match (state, event) {
        // Top level items
        (TopLevel(attrs), Start(Paragraph)) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            State::stack_onto(TopLevelAttrs::margin_before())
                .current(Inline(InlineText, InlineAttrs::default()))
                .and_data(data)
        }
        (TopLevel(attrs), Start(Heading(level))) => {
            let (data, links) = data.take_links();
            write_link_refs(writer, &settings.terminal_capabilities, links)?;
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
        }
        (TopLevel(attrs), Rule) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            write_rule(
                writer,
                &settings.terminal_capabilities,
                settings.terminal_size.width,
            )?;
            writeln!(writer)?;
            TopLevel(TopLevelAttrs::margin_before()).and_data(data)
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
            TopLevel(TopLevelAttrs::no_margin_for_html_only()).and_data(data)
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
        }
        (Stacked(stack, StyledBlock(attrs)), Start(BlockQuote)) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            stack
                .push(attrs.clone().with_margin_before().into())
                .current(attrs.without_margin_before().block_quote().into())
                .and_data(data)
        }
        (Stacked(stack, StyledBlock(attrs)), Rule) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            write_indent(writer, attrs.indent)?;
            write_rule(
                writer,
                &settings.terminal_capabilities,
                settings.terminal_size.width - (attrs.indent as usize),
            )?;
            writeln!(writer)?;
            stack
                .current(attrs.with_margin_before().into())
                .and_data(data)
        }
        (Stacked(stack, StyledBlock(attrs)), Start(Heading(level))) => {
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
                    write!(writer, "{:>2}. ", no)?;
                    indent + 4
                }
            };
            stack
                .current(Inline(
                    ListItem(kind, StartItem),
                    InlineAttrs { indent, style },
                ))
                .and_data(data)
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
        }
        (Stacked(stack, Inline(ListItem(kind, _), attrs)), Rule) => {
            writeln!(writer)?;
            write_indent(writer, attrs.indent)?;
            write_rule(
                writer,
                &settings.terminal_capabilities,
                settings.terminal_size.width - (attrs.indent as usize),
            )?;
            writeln!(writer)?;
            stack
                .current(Inline(ListItem(kind, ItemBlock), attrs))
                .and_data(data)
        }
        (Stacked(stack, Inline(ListItem(kind, state), attrs)), Start(Heading(level))) => {
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
                .current(Inline(ListItem(kind, state), InlineAttrs { indent, style }))
                .and_data(data)
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
            stack.current(attrs.into()).and_data(data)
        }
        (Stacked(stack, LiteralBlock(_)), End(CodeBlock(_))) => {
            write_border(
                writer,
                &settings.terminal_capabilities,
                &settings.terminal_size,
            )?;
            stack.pop().and_data(data)
        }

        // Highlighted code blocks
        (Stacked(stack, HighlightBlock(mut attrs)), Text(text)) => {
            let highlighter = Highlighter::new(theme);
            for line in LinesWithEndings::from(&text) {
                let ops = attrs.parse_state.parse_line(line, &settings.syntax_set);
                highlighting::write_as_ansi(
                    writer,
                    attrs.ansi,
                    HighlightIterator::new(&mut attrs.highlight_state, &ops, line, &highlighter),
                )?;
                if text.ends_with('\n') {
                    write_indent(writer, attrs.indent)?;
                }
            }
            stack.current(attrs.into()).and_data(data)
        }
        (Stacked(stack, HighlightBlock(_)), End(CodeBlock(_))) => {
            write_border(
                writer,
                &settings.terminal_capabilities,
                &settings.terminal_size,
            )?;
            stack.pop().and_data(data)
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
        }
        (Stacked(stack, Inline(_, _)), End(Emphasis)) => (stack.pop(), data),
        (Stacked(stack, Inline(state, attrs)), Start(Strong)) => {
            let indent = attrs.indent;
            let style = attrs.style.bold();
            stack
                .push(Inline(state, attrs))
                .current(Inline(state, InlineAttrs { style, indent }))
                .and_data(data)
        }
        (Stacked(stack, Inline(_, _)), End(Strong)) => (stack.pop(), data),
        (Stacked(stack, Inline(state, attrs)), Start(Strikethrough)) => {
            let style = attrs.style.strikethrough();
            let indent = attrs.indent;
            stack
                .push(Inline(state, attrs))
                .current(Inline(state, InlineAttrs { style, indent }))
                .and_data(data)
        }
        (Stacked(stack, Inline(_, _)), End(Strikethrough)) => (stack.pop(), data),
        (Stacked(stack, Inline(state, attrs)), Code(code)) => {
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &attrs.style.fg(Colour::Yellow),
                code,
            )?;
            (stack.current(Inline(state, attrs)), data)
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
        }
        // Inline line breaks
        (Stacked(stack, Inline(state, attrs)), SoftBreak) => {
            writeln!(writer)?;
            write_indent(writer, attrs.indent)?;
            (stack.current(Inline(state, attrs)), data)
        }
        (Stacked(stack, Inline(state, attrs)), HardBreak) => {
            writeln!(writer)?;
            write_indent(writer, attrs.indent)?;
            (stack.current(Inline(state, attrs)), data)
        }
        // Inline text
        (Stacked(stack, Inline(ListItem(kind, ItemBlock), attrs)), Text(text)) => {
            // Fresh text after a new block, so indent again.
            write_indent(writer, attrs.indent)?;
            write_styled(writer, &settings.terminal_capabilities, &attrs.style, text)?;
            stack
                .current(Inline(ListItem(kind, ItemText), attrs))
                .and_data(data)
        }
        (Stacked(stack, Inline(state, attrs)), Text(text)) => {
            write_styled(writer, &settings.terminal_capabilities, &attrs.style, text)?;
            (stack.current(Inline(state, attrs)), data)
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
        }
        (Stacked(stack, Inline(state, attrs)), Html(html)) => {
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &attrs.style.fg(Colour::Green),
                html,
            )?;
            (stack.current(Inline(state, attrs)), data)
        }
        // Ending inline text
        (Stacked(stack, Inline(_, _)), End(Paragraph)) => {
            writeln!(writer)?;
            (stack.pop(), data)
        }
        (Stacked(stack, Inline(_, _)), End(Heading(_))) => {
            writeln!(writer)?;
            (stack.pop(), data)
        }

        // Links.
        //
        // Links need a bit more work than standard inline markup because we
        // need to keep track of link references if we can't write inline links.
        (Stacked(stack, Inline(state, attrs)), Start(Link(_, target, _))) => {
            let link_state = match settings.terminal_capabilities.links {
                LinkCapability::OSC8(ref osc8) => {
                    // TODO: Handle email links
                    resolve_reference(base_dir, &target)
                        .and_then(|url| osc8.set_link_url(writer, url).ok())
                        .and(Some(InlineLink))
                }
                LinkCapability::NoLinks => None,
            }
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
        }
        (Stacked(stack, Inline(InlineLink, _)), End(Link(_, _, _))) => {
            match settings.terminal_capabilities.links {
                LinkCapability::OSC8(ref osc8) => {
                    osc8.clear_link(writer)?;
                }
                LinkCapability::NoLinks => {
                    panic!("Unreachable code: We opened an inline link but can't close it now?")
                }
            }
            (stack.pop(), data)
        }
        // When closing email or autolinks in inline text just return because link, being identical
        // to the link text, was already written.
        (Stacked(stack, Inline(InlineText, _)), End(Link(LinkType::Autolink, _, _))) => {
            (stack.pop(), data)
        }
        (Stacked(stack, Inline(InlineText, _)), End(Link(LinkType::Email, _, _))) => {
            (stack.pop(), data)
        }
        (Stacked(stack, Inline(InlineText, attrs)), End(Link(_, target, title))) => {
            let (data, index) = data.add_link(target, title);
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &attrs.style.fg(Colour::Blue),
                format!("[{}]", index),
            )?;
            (stack.pop(), data)
        }

        // Images
        (Stacked(stack, Inline(state, attrs)), Start(Image(_, link, _))) => {
            let InlineAttrs { style, indent } = attrs;
            use ImageCapability::*;
            let image_state = match (
                &settings.terminal_capabilities.image,
                resolve_reference(base_dir, &link),
            ) {
                (Terminology(terminology), Some(ref url)) => {
                    terminology.write_inline_image(writer, settings.terminal_size, url)?;
                    Some(RenderedImage)
                }
                (ITerm2(iterm2), Some(ref url)) => iterm2
                    .read_and_render(url, settings.resource_access)
                    .and_then(|contents| {
                        iterm2.write_inline_image(writer, url.as_str(), &contents)?;
                        Ok(RenderedImage)
                    })
                    .map(|_| RenderedImage)
                    .ok(),
                (Kitty(ref kitty), Some(ref url)) => kitty
                    .read_and_render(url, settings.resource_access)
                    .and_then(|contents| {
                        kitty.write_inline_image(writer, contents)?;
                        Ok(RenderedImage)
                    })
                    .ok(),
                (ImageCapability::NoImages, _) => None,
                (_, None) => None,
            }
            .unwrap_or_else(|| Inline(InlineText, InlineAttrs { indent, style }));
            stack
                .push(Inline(state, attrs))
                .current(image_state)
                .and_data(data)
        }
        (Stacked(stack, RenderedImage), Text(_)) => (Stacked(stack, RenderedImage), data),
        (Stacked(stack, RenderedImage), End(Image(_, _, _))) => (stack.pop(), data),
        (Stacked(stack, Inline(_, attrs)), End(Image(_, target, title))) => {
            let (data, index) = data.add_link(target, title);
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &attrs.style,
                format!("[{}]", index),
            )?;
            (stack.pop(), data)
        }

        // Unconditional returns to previous states
        (Stacked(stack, _), End(BlockQuote)) => (stack.pop(), data),
        (Stacked(stack, _), End(List(_))) => (stack.pop(), data),

        // Impossible events
        (s, e) => panic!(
            "Event {:?} impossible in state {:?}

Please do report an issue at <https://github.com/lunaryorn/mdcat/issues/new> including

* a copy of this message, and
* the markdown document which caused this error.",
            s, e
        ),
    }
}

#[throws]
pub fn finish<'a, W: Write>(
    writer: &mut W,
    settings: &Settings,
    state: State,
    data: StateData<'a>,
) -> () {
    match state {
        State::TopLevel(_) => {
            write_link_refs(writer, &settings.terminal_capabilities, data.pending_links)?;
        }
        _ => {
            panic!("Must finish in state TopLevel but got: {:?}", state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_reference;
    use pretty_assertions::assert_eq;
    use url::Url;

    #[test]
    fn resolve_reference_with_url() {
        let url = resolve_reference(
            &std::env::current_dir().expect("Current dir"),
            "http://www.example.com/reference",
        );
        assert_eq!(
            url.as_ref().map_or("", |u| u.as_str()),
            "http://www.example.com/reference"
        );
    }

    #[test]
    fn resolve_reference_with_relative_url() {
        let cwd = std::env::current_dir().expect("Current dir");
        let cwd_url = Url::from_directory_path(&cwd).expect("Current dir URL");
        let url = resolve_reference(&cwd, "./foo.md");

        assert!(
            url.as_ref()
                .map_or(false, |u| u.as_str().starts_with("file://")),
            "Url {:?} starts with file://",
            url
        );
        assert!(
            url.as_ref()
                .map_or(false, |u| u.as_str().starts_with(cwd_url.as_str())),
            "Url {:?} starts with {}",
            url,
            cwd_url
        );
        assert!(
            url.as_ref()
                .map_or(false, |u| u.as_str().ends_with("/foo.md")),
            "Url {:?} ends with foo.md",
            url
        );
    }
}
