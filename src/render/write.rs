// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::{Error, Result, Write};

use ansi_term::{Colour, Style};
use fehler::throws;
use pulldown_cmark::CodeBlockKind;
use syntect::highlighting::{HighlightState, Highlighter, Theme};
use syntect::parsing::{ParseState, ScopeStack};

use crate::references::*;
use crate::render::data::LinkReferenceDefinition;
use crate::render::state::*;
use crate::{
    Environment, MarkCapability, Settings, StyleCapability, TerminalCapabilities, TerminalSize,
};

#[inline]
pub fn write_indent<W: Write>(writer: &mut W, level: u16) -> std::io::Result<()> {
    write!(writer, "{}", " ".repeat(level as usize))
}

#[inline]
pub fn write_styled<W: Write, S: AsRef<str>>(
    writer: &mut W,
    capabilities: &TerminalCapabilities,
    style: &Style,
    text: S,
) -> Result<()> {
    match capabilities.style {
        None => write!(writer, "{}", text.as_ref()),
        Some(StyleCapability::Ansi(ansi)) => ansi.write_styled(writer, style, text),
    }
}

#[inline]
#[throws]
pub fn write_mark<W: Write>(writer: &mut W, capabilities: &TerminalCapabilities) -> () {
    if let Some(mark) = capabilities.marks {
        match mark {
            MarkCapability::ITerm2(marks) => marks.set_mark(writer)?,
        }
    }
}

#[inline]
pub fn write_rule<W: Write>(
    writer: &mut W,
    capabilities: &TerminalCapabilities,
    length: usize,
) -> std::io::Result<()> {
    let rule = "\u{2550}".repeat(length);
    let style = Style::new().fg(Colour::Green);
    write_styled(writer, capabilities, &style, rule)
}

#[inline]
pub fn write_border<W: Write>(
    writer: &mut W,
    capabilities: &TerminalCapabilities,
    terminal_size: &TerminalSize,
) -> std::io::Result<()> {
    let separator = "\u{2500}".repeat(terminal_size.columns.min(20));
    let style = Style::new().fg(Colour::Green);
    write_styled(writer, capabilities, &style, separator)?;
    writeln!(writer)
}

#[throws]
pub fn write_link_refs<W: Write>(
    writer: &mut W,
    environment: &Environment,
    capabilities: &TerminalCapabilities,
    links: Vec<LinkReferenceDefinition>,
) -> () {
    if !links.is_empty() {
        writeln!(writer)?;
        for link in links {
            let style = Style::new().fg(link.colour);
            write_styled(writer, capabilities, &style, &format!("[{}]: ", link.index))?;

            // If we can resolve the link try to write it as inline link to make the URL
            // clickable.  This mostly helps images inside inline links which we had to write as
            // reference links because we can't nest inline links.
            if let Some(url) = environment.resolve_reference(&link.target) {
                use crate::LinkCapability::*;
                match &capabilities.links {
                    Some(Osc8(links)) => {
                        links.set_link_url(writer, url, &environment.hostname)?;
                        write_styled(writer, capabilities, &style, link.target)?;
                        links.clear_link(writer)?;
                    }
                    None => write_styled(writer, capabilities, &style, link.target)?,
                };
            } else {
                write_styled(writer, capabilities, &style, link.target)?;
            }

            if !link.title.is_empty() {
                write_styled(writer, capabilities, &style, format!(" {}", link.title))?;
            }
            writeln!(writer)?;
        }
    }
}

#[throws]
pub fn write_start_code_block<'a, W: Write>(
    writer: &mut W,
    settings: &Settings,
    indent: u16,
    style: Style,
    block_kind: CodeBlockKind<'a>,
    theme: &Theme,
) -> StackedState {
    write_indent(writer, indent)?;
    write_border(
        writer,
        &settings.terminal_capabilities,
        &settings.terminal_size,
    )?;
    // And start the indent for the contents of the block
    write_indent(writer, indent)?;

    match (&settings.terminal_capabilities.style, block_kind) {
        (Some(StyleCapability::Ansi(ansi)), CodeBlockKind::Fenced(name)) if !name.is_empty() => {
            match settings.syntax_set.find_syntax_by_token(&name) {
                None => LiteralBlockAttrs {
                    indent,
                    style: style.fg(Colour::Yellow),
                }
                .into(),
                Some(syntax) => {
                    let parse_state = ParseState::new(syntax);
                    let highlight_state =
                        HighlightState::new(&Highlighter::new(theme), ScopeStack::new());
                    HighlightBlockAttrs {
                        ansi: *ansi,
                        indent,
                        highlight_state,
                        parse_state,
                    }
                    .into()
                }
            }
        }
        (_, _) => LiteralBlockAttrs {
            indent,
            style: style.fg(Colour::Yellow),
        }
        .into(),
    }
}

#[throws]
pub fn write_start_heading<W: Write>(
    writer: &mut W,
    capabilities: &TerminalCapabilities,
    style: Style,
    level: u32,
) -> StackedState {
    let style = style.fg(Colour::Blue).bold();
    write_styled(
        writer,
        &capabilities,
        &style,
        "\u{2504}".repeat(level as usize),
    )?;

    // Headlines never wrap, so indent doesn't matter
    StackedState::Inline(InlineState::InlineText, InlineAttrs { style, indent: 0 })
}
