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

mod state;
pub use state::State;

pub fn render_event<W: Write>(
    writer: &mut W,
    settings: &Settings,
    _environment: &Environment,
    _resource_handler: &dyn ResourceUrlHandler,
    mut state: State,
    event: Event,
) -> Result<State> {
    match event {
        Event::Start(Tag::Paragraph) => Ok(state.initialize_fresh_paragraph()),
        Event::Start(Tag::Heading(_, _, _)) => {
            todo!()
        }
        Event::Start(Tag::BlockQuote) => {
            todo!()
        }
        Event::Start(Tag::CodeBlock(_)) => {
            todo!()
        }
        Event::Start(Tag::List(_)) => {
            todo!()
        }
        Event::Start(Tag::Item) => {
            todo!()
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
        Event::Start(Tag::Strong) => Ok(state.push_inline_style(&Style::new().bold())),
        Event::Start(Tag::Strikethrough) => {
            Ok(state.push_inline_style(&Style::new().strikethrough()))
        }
        Event::Start(Tag::Link(_, _, _)) => {
            todo!()
        }
        Event::Start(Tag::Image(_, _, _)) => {
            todo!()
        }
        Event::End(Tag::Paragraph) => {
            // We've written a paragraph so the paragraph which comes next needs to have a margin.
            Ok(state.flush_paragraph(writer)?.with_margin_before())
        }
        Event::End(Tag::Heading(_, _, _)) => {
            todo!()
        }
        Event::End(Tag::BlockQuote) => {
            todo!()
        }
        Event::End(Tag::CodeBlock(_)) => {
            todo!()
        }
        Event::End(Tag::List(_)) => {
            todo!()
        }
        Event::End(Tag::Item) => {
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
        Event::End(Tag::Emphasis) => Ok(state.toggle_italic()),
        Event::End(Tag::Strong) => Ok(state.pop_inline_style()),
        Event::End(Tag::Strikethrough) => Ok(state.pop_inline_style()),
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
            todo!()
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
