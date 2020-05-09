// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Rendering algorithm.

use std::error::Error;
use std::io::prelude::*;
use std::path::Path;

use ansi_term::{Colour, Style};
use pulldown_cmark::Event::*;
use pulldown_cmark::Tag::*;
use pulldown_cmark::{Event, LinkType};
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
pub fn write_event<'a, W: Write>(
    writer: &mut W,
    settings: &Settings,
    base_dir: &Path,
    theme: &Theme,
    state: State,
    data: StateData<'a>,
    event: Event<'a>,
) -> Result<(State, StateData<'a>), Box<dyn Error>> {
    use self::InlineState::*;
    use self::ListItemState::*;
    use self::NestedState::*;
    use State::*;
    match (state, event) {
        // Top level items
        (TopLevel(attrs), Start(Paragraph)) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            Ok((
                NestedState(
                    Box::new(TopLevelAttrs::margin_before().into()),
                    Inline(InlineText, InlineAttrs::default()),
                ),
                data,
            ))
        }
        (TopLevel(attrs), Start(Heading(level))) => {
            let (data, links) = data.take_links();
            write_link_refs(writer, &settings.terminal_capabilities, links)?;
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            write_mark(writer, &settings.terminal_capabilities)?;

            Ok((
                write_start_heading(
                    writer,
                    &settings.terminal_capabilities,
                    TopLevelAttrs::margin_before().into(),
                    Style::new(),
                    level,
                )?,
                data,
            ))
        }
        (TopLevel(attrs), Start(BlockQuote)) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            Ok((
                NestedState(
                    Box::new(TopLevelAttrs::margin_before().into()),
                    // We've written a block-level margin already, so the first
                    // block inside the styled block should add another margin.
                    StyledBlockAttrs::default()
                        .block_quote()
                        .without_margin_before()
                        .into(),
                ),
                data,
            ))
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
            Ok((TopLevelAttrs::margin_before().into(), data))
        }
        (TopLevel(attrs), Start(CodeBlock(kind))) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }

            Ok((
                write_start_code_block(
                    writer,
                    settings,
                    TopLevel(TopLevelAttrs::margin_before()),
                    0,
                    Style::new(),
                    kind,
                    theme,
                )?,
                data,
            ))
        }
        (TopLevel(attrs), Start(List(start))) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            let kind = start.map_or(ListItemKind::Unordered, |start| {
                ListItemKind::Ordered(start)
            });
            Ok((
                NestedState(
                    Box::new(TopLevelAttrs::margin_before().into()),
                    Inline(ListItem(kind, StartItem), InlineAttrs::default()),
                ),
                data,
            ))
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
            Ok((TopLevel(TopLevelAttrs::no_margin_for_html_only()), data))
        }

        // Nested blocks with style, e.g. paragraphs in quotes, etc.
        (NestedState(return_to, StyledBlock(attrs)), Start(Paragraph)) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            write_indent(writer, attrs.indent)?;
            let inline = InlineAttrs::from(&attrs);
            Ok((
                NestedState(
                    Box::new(NestedState(return_to, attrs.with_margin_before().into())),
                    Inline(InlineText, inline),
                ),
                data,
            ))
        }
        (NestedState(return_to, StyledBlock(attrs)), Start(BlockQuote)) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            Ok((
                NestedState(
                    Box::new(NestedState(
                        return_to,
                        attrs.clone().with_margin_before().into(),
                    )),
                    attrs.without_margin_before().block_quote().into(),
                ),
                data,
            ))
        }
        (NestedState(return_to, StyledBlock(attrs)), Rule) => {
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
            Ok((
                NestedState(return_to, attrs.with_margin_before().into()),
                data,
            ))
        }
        (NestedState(return_to, StyledBlock(attrs)), Start(Heading(level))) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            write_indent(writer, attrs.indent)?;

            // We deliberately don't mark headings which aren't top-level.
            let style = attrs.style;
            Ok((
                write_start_heading(
                    writer,
                    &settings.terminal_capabilities,
                    NestedState(return_to, attrs.with_margin_before().into()),
                    style,
                    level,
                )?,
                data,
            ))
        }
        (NestedState(return_to, StyledBlock(attrs)), Start(List(start))) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            let kind = start.map_or(ListItemKind::Unordered, |start| {
                ListItemKind::Ordered(start)
            });
            let inline = InlineAttrs::from(&attrs);
            Ok((
                NestedState(
                    Box::new(NestedState(return_to, attrs.with_margin_before().into())),
                    Inline(ListItem(kind, StartItem), inline),
                ),
                data,
            ))
        }
        (NestedState(return_to, StyledBlock(attrs)), Start(CodeBlock(kind))) => {
            if attrs.margin_before != NoMargin {
                writeln!(writer)?;
            }
            let StyledBlockAttrs { indent, style, .. } = attrs;
            Ok((
                write_start_code_block(
                    writer,
                    settings,
                    NestedState(return_to, attrs.into()),
                    indent,
                    style,
                    kind,
                    theme,
                )?,
                data,
            ))
        }
        (NestedState(return_to, StyledBlock(attrs)), Html(html)) => {
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
            Ok((
                NestedState(return_to, attrs.without_margin_for_html_only().into()),
                data,
            ))
        }

        // Lists
        (NestedState(return_to, Inline(ListItem(kind, state), attrs)), Start(Item)) => {
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
            Ok((
                NestedState(
                    return_to,
                    Inline(ListItem(kind, StartItem), InlineAttrs { indent, style }),
                ),
                data,
            ))
        }
        (NestedState(return_to, Inline(ListItem(kind, state), attrs)), Start(Paragraph)) => {
            if state != StartItem {
                // Write margin, unless we're at the start of the list item in which case the first line of the
                // paragraph should go right beside the item bullet.
                writeln!(writer)?;
                write_indent(writer, attrs.indent)?;
            }
            Ok((
                NestedState(
                    Box::new(NestedState(
                        return_to,
                        Inline(ListItem(kind, ItemBlock), attrs.clone()),
                    )),
                    Inline(InlineText, attrs),
                ),
                data,
            ))
        }
        (NestedState(return_to, Inline(ListItem(kind, _), attrs)), Start(CodeBlock(ck))) => {
            writeln!(writer)?;
            let InlineAttrs { indent, style } = attrs;
            Ok((
                write_start_code_block(
                    writer,
                    settings,
                    NestedState(return_to, Inline(ListItem(kind, ItemBlock), attrs)),
                    indent,
                    style,
                    ck,
                    theme,
                )?,
                data,
            ))
        }
        (NestedState(return_to, Inline(ListItem(kind, _), attrs)), Rule) => {
            writeln!(writer)?;
            write_indent(writer, attrs.indent)?;
            write_rule(
                writer,
                &settings.terminal_capabilities,
                settings.terminal_size.width - (attrs.indent as usize),
            )?;
            writeln!(writer)?;
            Ok((
                NestedState(return_to, Inline(ListItem(kind, ItemBlock), attrs)),
                data,
            ))
        }
        (NestedState(return_to, Inline(ListItem(kind, state), attrs)), Start(Heading(level))) => {
            if state != StartItem {
                writeln!(writer)?;
                write_indent(writer, attrs.indent)?;
            }
            // We deliberately don't mark headings which aren't top-level.
            let style = attrs.style;
            Ok((
                write_start_heading(
                    writer,
                    &settings.terminal_capabilities,
                    NestedState(return_to, Inline(ListItem(kind, ItemBlock), attrs)),
                    style,
                    level,
                )?,
                data,
            ))
        }
        (NestedState(return_to, Inline(ListItem(kind, _), attrs)), Start(List(start))) => {
            writeln!(writer)?;
            let nested_kind = start.map_or(ListItemKind::Unordered, |start| {
                ListItemKind::Ordered(start)
            });
            Ok((
                NestedState(
                    Box::new(NestedState(
                        return_to,
                        Inline(ListItem(kind, ItemBlock), attrs.clone()),
                    )),
                    Inline(ListItem(nested_kind, StartItem), attrs),
                ),
                data,
            ))
        }
        (NestedState(return_to, Inline(ListItem(kind, _), attrs)), Start(BlockQuote)) => {
            writeln!(writer)?;
            let block_quote = StyledBlockAttrs::from(&attrs)
                .without_margin_before()
                .block_quote();
            Ok((
                NestedState(
                    Box::new(NestedState(
                        return_to,
                        Inline(ListItem(kind, ItemBlock), attrs),
                    )),
                    block_quote.into(),
                ),
                data,
            ))
        }
        (NestedState(return_to, Inline(ListItem(kind, state), attrs)), End(Item)) => {
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
            Ok((
                NestedState(
                    return_to,
                    Inline(ListItem(kind, state), InlineAttrs { indent, style }),
                ),
                data,
            ))
        }

        // Literal blocks without highlighting
        (NestedState(return_to, LiteralBlock(attrs)), Text(text)) => {
            let LiteralBlockAttrs { indent, style } = attrs;
            for line in LinesWithEndings::from(&text) {
                write_styled(writer, &settings.terminal_capabilities, &style, line)?;
                if line.ends_with('\n') {
                    write_indent(writer, indent)?;
                }
            }
            Ok((NestedState(return_to, attrs.into()), data))
        }
        (NestedState(return_to, LiteralBlock(_)), End(CodeBlock(_))) => {
            write_border(
                writer,
                &settings.terminal_capabilities,
                &settings.terminal_size,
            )?;
            Ok((*return_to, data))
        }

        // Highlighted code blocks
        (NestedState(return_to, HighlightBlock(mut attrs)), Text(text)) => {
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
            Ok((NestedState(return_to, attrs.into()), data))
        }
        (NestedState(return_to, HighlightBlock(_)), End(CodeBlock(_))) => {
            write_border(
                writer,
                &settings.terminal_capabilities,
                &settings.terminal_size,
            )?;
            Ok((*return_to, data))
        }

        // Inline markup
        (NestedState(return_to, Inline(state, attrs)), Start(Emphasis)) => {
            let indent = attrs.indent;
            let style = Style {
                is_italic: !attrs.style.is_italic,
                ..attrs.style
            };
            Ok((
                NestedState(
                    Box::new(NestedState(return_to, Inline(state, attrs))),
                    Inline(InlineText, InlineAttrs { style, indent }),
                ),
                data,
            ))
        }
        (NestedState(return_to, Inline(_, _)), End(Emphasis)) => Ok((*return_to, data)),
        (NestedState(return_to, Inline(state, attrs)), Start(Strong)) => {
            let indent = attrs.indent;
            let style = attrs.style.bold();
            Ok((
                NestedState(
                    Box::new(NestedState(return_to, Inline(state, attrs))),
                    Inline(InlineText, InlineAttrs { style, indent }),
                ),
                data,
            ))
        }
        (NestedState(return_to, Inline(_, _)), End(Strong)) => Ok((*return_to, data)),
        (NestedState(return_to, Inline(state, attrs)), Start(Strikethrough)) => {
            let style = attrs.style.strikethrough();
            let indent = attrs.indent;
            Ok((
                NestedState(
                    Box::new(NestedState(return_to, Inline(state, attrs))),
                    Inline(InlineText, InlineAttrs { style, indent }),
                ),
                data,
            ))
        }
        (NestedState(return_to, Inline(_, _)), End(Strikethrough)) => Ok((*return_to, data)),
        (NestedState(return_to, Inline(state, attrs)), Code(code)) => {
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &attrs.style.fg(Colour::Yellow),
                code,
            )?;
            Ok((NestedState(return_to, Inline(state, attrs)), data))
        }
        (NestedState(return_to, Inline(ListItem(kind, state), attrs)), TaskListMarker(checked)) => {
            let marker = if checked { "\u{2611} " } else { "\u{2610} " };
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &attrs.style,
                marker,
            )?;
            Ok((
                NestedState(return_to, Inline(ListItem(kind, state), attrs)),
                data,
            ))
        }
        // Inline line breaks
        (NestedState(return_to, Inline(state, attrs)), SoftBreak) => {
            writeln!(writer)?;
            write_indent(writer, attrs.indent)?;
            Ok((NestedState(return_to, Inline(state, attrs)), data))
        }
        (NestedState(return_to, Inline(state, attrs)), HardBreak) => {
            writeln!(writer)?;
            write_indent(writer, attrs.indent)?;
            Ok((NestedState(return_to, Inline(state, attrs)), data))
        }
        // Inline text
        (NestedState(return_to, Inline(ListItem(kind, ItemBlock), attrs)), Text(text)) => {
            // Fresh text after a new block, so indent again.
            write_indent(writer, attrs.indent)?;
            write_styled(writer, &settings.terminal_capabilities, &attrs.style, text)?;
            Ok((
                NestedState(return_to, Inline(ListItem(kind, ItemText), attrs)),
                data,
            ))
        }
        (NestedState(return_to, Inline(state, attrs)), Text(text)) => {
            write_styled(writer, &settings.terminal_capabilities, &attrs.style, text)?;
            Ok((NestedState(return_to, Inline(state, attrs)), data))
        }
        // Inline HTML
        (NestedState(return_to, Inline(ListItem(kind, ItemBlock), attrs)), Html(html)) => {
            // Fresh text after a new block, so indent again.
            write_indent(writer, attrs.indent)?;
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &attrs.style.fg(Colour::Green),
                html,
            )?;
            Ok((
                NestedState(return_to, Inline(ListItem(kind, ItemText), attrs)),
                data,
            ))
        }
        (NestedState(return_to, Inline(state, attrs)), Html(html)) => {
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &attrs.style.fg(Colour::Green),
                html,
            )?;
            Ok((NestedState(return_to, Inline(state, attrs)), data))
        }
        // Ending inline text
        (NestedState(return_to, Inline(_, _)), End(Paragraph)) => {
            writeln!(writer)?;
            Ok((*return_to, data))
        }
        (NestedState(return_to, Inline(_, _)), End(Heading(_))) => {
            writeln!(writer)?;
            Ok((*return_to, data))
        }

        // Links.
        //
        // Links need a bit more work than standard inline markup because we
        // need to keep track of link references if we can't write inline links.
        (NestedState(return_to, Inline(state, attrs)), Start(Link(_, target, _))) => {
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
            Ok((
                NestedState(
                    Box::new(NestedState(return_to, Inline(state, attrs))),
                    Inline(
                        link_state,
                        InlineAttrs {
                            indent,
                            style: style.fg(Colour::Blue),
                        },
                    ),
                ),
                data,
            ))
        }
        (NestedState(return_to, Inline(InlineLink, _)), End(Link(_, _, _))) => {
            match settings.terminal_capabilities.links {
                LinkCapability::OSC8(ref osc8) => {
                    osc8.clear_link(writer)?;
                }
                LinkCapability::NoLinks => {
                    panic!("Unreachable code: We opened an inline link but can't close it now?")
                }
            }
            Ok((*return_to, data))
        }
        // When closing email or autolinks in inline text just return because link, being identical
        // to the link text, was already written.
        (NestedState(return_to, Inline(InlineText, _)), End(Link(LinkType::Autolink, _, _))) => {
            Ok((*return_to, data))
        }
        (NestedState(return_to, Inline(InlineText, _)), End(Link(LinkType::Email, _, _))) => {
            Ok((*return_to, data))
        }
        (NestedState(return_to, Inline(InlineText, attrs)), End(Link(_, target, title))) => {
            let (data, index) = data.add_link(target, title);
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &attrs.style.fg(Colour::Blue),
                format!("[{}]", index),
            )?;
            Ok((*return_to, data))
        }

        // Images
        (NestedState(return_to, Inline(state, attrs)), Start(Image(_, link, _))) => {
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
                    .read_and_render(url)
                    .and_then(|contents| {
                        iterm2.write_inline_image(writer, url.as_str(), &contents)?;
                        Ok(RenderedImage)
                    })
                    .ok(),
                (Kitty(ref kitty), Some(ref url)) => kitty
                    .read_and_render(url)
                    .and_then(|contents| {
                        kitty.write_inline_image(writer, contents)?;
                        Ok(RenderedImage)
                    })
                    .ok(),
                (ImageCapability::NoImages, _) => None,
                (_, None) => None,
            }
            .unwrap_or_else(|| Inline(InlineText, InlineAttrs { indent, style }));
            Ok((
                NestedState(
                    Box::new(NestedState(return_to, Inline(state, attrs))),
                    image_state,
                ),
                data,
            ))
        }
        (NestedState(return_to, RenderedImage), End(Image(_, _, _))) => Ok((*return_to, data)),
        (NestedState(return_to, Inline(_, attrs)), End(Image(_, target, title))) => {
            let (data, index) = data.add_link(target, title);
            write_styled(
                writer,
                &settings.terminal_capabilities,
                &attrs.style,
                format!("[{}]", index),
            )?;
            Ok((*return_to, data))
        }

        // Unconditional returns to previous states
        (NestedState(return_to, _), End(BlockQuote)) => Ok((*return_to, data)),
        (NestedState(return_to, _), End(List(_))) => Ok((*return_to, data)),

        // Impossible events
        (s @ TopLevel(_), e @ Code(_)) => impossible(s, e),
        (s @ TopLevel(_), e @ Text(_)) => impossible(s, e),

        // TODO: Remove and cover all impossible cases when finishing this branch.
        (s, e) => panic!("Unexpected event in state {:?}: {:?}", s, e),
    }
}

#[inline]
fn impossible(state: State, event: Event) -> ! {
    panic!(
        "Event {:?} impossible in state {:?}

Please do report an issue at <https://github.com/lunaryorn/mdcat/issues/new> including

* a copy of this message, and
* the markdown document which caused this error.",
        state, event
    )
}

pub fn finish<'a, W: Write>(
    writer: &mut W,
    settings: &Settings,
    state: State,
    data: StateData<'a>,
) -> Result<(), Box<dyn Error>> {
    match state {
        State::TopLevel(_) => {
            write_link_refs(writer, &settings.terminal_capabilities, data.pending_links)?;
            Ok(())
        }
        _ => {
            panic!("Must finish in state TopLevel but got: {:?}", state);
        }
    }
}
