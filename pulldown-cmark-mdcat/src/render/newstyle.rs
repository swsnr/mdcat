// Copyright  Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! New-style rendering algorithm.

// TODO: Rename to render.rs once stable

use crate::{Environment, ResourceUrlHandler, Settings};
use anstyle::Style;
use pulldown_cmark::{Event, Tag};
use std::io::{Result, Write};
use tracing::{event, instrument, Level};

mod state;
use crate::render::newstyle::state::List;
pub use state::State;

#[instrument(
    level = "trace",
    skip(writer, settings, _environment, _resource_handler, event)
)]
pub fn render_event<W: Write>(
    writer: &mut W,
    settings: &Settings,
    _environment: &Environment,
    _resource_handler: &dyn ResourceUrlHandler,
    mut state: State,
    event: Event,
) -> Result<State> {
    event!(Level::TRACE, "Rendering event {:?}", event);
    match event {
        // We don't need to do anything when starting a new paragraph, because indentation and styling
        // are already set up by previous tags.
        Event::Start(Tag::Paragraph) => Ok(state),
        Event::End(Tag::Paragraph) => {
            // We've written a paragraph so the paragraph which comes next needs to have a margin.
            // We also reset the initial indentation to subsequent indentation, because the next
            // paragraph should be indented like subsequent text, unless there's another special
            // block again, which changes the initial indentation.
            Ok(state
                .flush_paragraph(writer)?
                .with_margin_before()
                .reset_initial_indent())
        }
        Event::Start(Tag::Heading(level, _, _)) => Ok(state
            .push_inline_style(&settings.theme.heading_style)
            .with_line_prefix("\u{2504}".repeat(level as usize))),
        Event::End(Tag::Heading(_, _, _)) => Ok(state
            .pop_inline_style()
            .flush_paragraph(writer)?
            .clear_line_prefix()
            .with_margin_before()),
        Event::Start(Tag::BlockQuote) => Ok(state.toggle_italic().indent(4)),
        Event::End(Tag::BlockQuote) => Ok(state.toggle_italic().dedent(4)),
        Event::Start(Tag::CodeBlock(_)) => {
            todo!()
        }
        Event::End(Tag::CodeBlock(_)) => {
            todo!()
        }
        Event::Start(Tag::List(first_item)) => {
            let state = if state.paragraph_is_empty() {
                state
            } else {
                // The current paragraph is not empty, i.e. was not yet flushed.  This means we're
                // in a list without paragraphs and started a new nested list.  In this case the
                // list start occurs right in inline text, meaning there was no event to flush the
                // paragraph yet.
                //
                // So let's flush out the paragraph, reset initial indentation to subsequent
                // indentation (so that the next list item starts indented), and then also remove
                // the margin before the next paragraph, because lists without paragraphs render
                // condensed.
                state
                    .flush_paragraph(writer)?
                    .reset_initial_indent()
                    .no_margin_before()
            };
            Ok(match first_item {
                None => state.push_unordered_list(),
                Some(number) => state.push_ordered_list(number),
            })
        }
        Event::End(Tag::List(_)) => {
            // We don't need to flush anything here because we already flush paragraphs at the end
            // of each list item.
            let (state, _) = state.pop_current_list();
            Ok(state.with_margin_before())
        }
        Event::Start(Tag::Item) => {
            let (state, list) = state.pop_current_list();
            Ok(match list {
                List::Unordered => {
                    let mut state = state.indent_subsequent(2);
                    write!(state.sink(), "\u{2022} ").unwrap();
                    state.push_unordered_list()
                }
                List::Ordered(item_no) => {
                    let mut state = state.indent_subsequent(4);
                    write!(state.sink(), "{item_no:>2}. ").unwrap();
                    state.push_ordered_list(item_no)
                }
            })
        }
        Event::End(Tag::Item) => {
            let state = if state.paragraph_is_empty() {
                // If the current paragraph is already empty the entire list item was already
                // flushed out; this means there were paragraphs inside the list item, and we need
                // to ensure there's a margin before the next item.
                state.with_margin_before()
            } else {
                // Otherwise there's the list item text is still in the paragraph, meaning this item
                // only contains inline text.  Flush it to the terminal, and make sure that there's
                // _no_ margin before the next item.
                state.flush_paragraph(writer)?.no_margin_before()
            };
            let (state, list) = state.pop_current_list();
            Ok(match list {
                List::Unordered => state.dedent_subsequent(2).push_unordered_list(),
                List::Ordered(no) => state.dedent_subsequent(4).push_ordered_list(no + 1),
            }
            .reset_initial_indent())
        }
        Event::Start(Tag::FootnoteDefinition(_)) => {
            panic!("Footnotes are not supported yet, see https://github.com/swsnr/mdcat/issues/1")
        }
        Event::End(Tag::FootnoteDefinition(_)) => {
            panic!("Footnotes are not supported yet, see https://github.com/swsnr/mdcat/issues/1")
        }
        Event::Start(Tag::Table(_)) => {
            todo!()
        }
        Event::Start(Tag::TableHead) => {
            todo!()
        }
        Event::Start(Tag::TableRow) => {
            todo!()
        }
        Event::Start(Tag::TableCell) => {
            todo!()
        }
        Event::Start(Tag::Emphasis) => Ok(state.toggle_italic()),
        Event::End(Tag::Emphasis) => Ok(state.toggle_italic()),
        Event::Start(Tag::Strong) => Ok(state.push_inline_style(&Style::new().bold())),
        Event::End(Tag::Strong) => Ok(state.pop_inline_style()),
        Event::Start(Tag::Strikethrough) => {
            Ok(state.push_inline_style(&Style::new().strikethrough()))
        }
        Event::End(Tag::Strikethrough) => Ok(state.pop_inline_style()),
        Event::Start(Tag::Link(_, _, _)) => {
            todo!()
        }
        Event::Start(Tag::Image(_, _, _)) => {
            todo!()
        }
        Event::End(Tag::Table(_)) => {
            todo!()
        }
        Event::End(Tag::TableHead) => {
            todo!()
        }
        Event::End(Tag::TableRow) => {
            todo!()
        }
        Event::End(Tag::TableCell) => {
            todo!()
        }

        Event::End(Tag::Link(_, _, _)) => {
            todo!()
        }
        Event::End(Tag::Image(_, _, _)) => {
            todo!()
        }
        Event::Text(text) => {
            write!(state.sink(), "{}", text).unwrap();
            Ok(state)
        }
        Event::Code(code) => {
            let mut state = state.push_inline_style(&settings.theme.code_style);
            write!(state.sink(), "{}", code).unwrap();
            Ok(state.pop_inline_style())
        }
        Event::Html(_) => {
            todo!()
        }
        // We just ignore soft breaks because we wrap and fill
        Event::SoftBreak => Ok(state),
        Event::HardBreak => {
            // Idea: Let's treat this as a paragraph break, flush the current one and render the
            // next one, but without any margin.
            todo!()
        }
        Event::Rule => {
            // A rule is effectively a paragraph on its own, so let's check that the previous
            // paragraph is flushed.
            assert!(
                state.paragraph_is_empty(),
                "Previous paragraph not flushed, this is a rendering bug!"
            );
            let rule_style = Style::new().fg_color(Some(settings.theme.rule_color));
            let rule_length = state.subsequent_text_width();
            let mut state = state.push_inline_style(&rule_style);
            write!(state.sink(), "{}", "\u{2550}".repeat(rule_length)).unwrap();
            state.pop_inline_style().flush_paragraph(writer)
        }
        Event::TaskListMarker(_) => {
            todo!()
        }
        Event::FootnoteReference(_) => {
            panic!("Footnotes are not supported yet, see https://github.com/swsnr/mdcat/issues/1")
        }
    }
}

pub fn finish<W: Write>(
    _writer: &mut W,
    _settings: &Settings,
    _environment: &Environment,
    _resource_handler: &dyn ResourceUrlHandler,
    _state: State,
) -> Result<()> {
    Ok(())
}
