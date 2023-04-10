// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::{Result, Write};

use ansi_term::{Colour, Style};
use pulldown_cmark::{CodeBlockKind, HeadingLevel};
use syntect::highlighting::{HighlightState, Highlighter, Theme};
use syntect::parsing::{ParseState, ScopeStack};
use textwrap::core::{display_width, Word};
use textwrap::WordSeparator;

use crate::references::*;
use crate::render::data::{CurrentLine, LinkReferenceDefinition};
use crate::render::state::*;
use crate::terminal::capabilities::{MarkCapability, StyleCapability, TerminalCapabilities};
use crate::terminal::TerminalSize;
use crate::{Environment, Settings};

#[inline]
pub fn write_indent<W: Write>(writer: &mut W, level: u16) -> Result<()> {
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

fn write_remaining_lines<W: Write>(
    writer: &mut W,
    capabilities: &TerminalCapabilities,
    style: &Style,
    indent: usize,
    mut buffer: String,
    next_lines: &[&[Word]],
    last_line: &[Word],
) -> Result<CurrentLine> {
    // Finish the previous line
    writeln!(writer)?;
    write_indent(writer, indent as u16)?;
    // Now write all lines up to the last
    for line in next_lines {
        match line.split_last() {
            None => {
                // The line was empty, so there's nothing to do anymore.
            }
            Some((last, heads)) => {
                for word in heads {
                    buffer.push_str(word.word);
                    buffer.push_str(word.whitespace);
                }
                buffer.push_str(last.word);
                write_styled(writer, capabilities, style, &buffer)?;
                writeln!(writer)?;
                write_indent(writer, indent as u16)?;
                buffer.clear();
            }
        };
    }

    // Now write the last line and keep track of its width
    match last_line.split_last() {
        None => {
            // The line was empty, so there's nothing to do anymore.
            Ok(CurrentLine::empty())
        }
        Some((last, heads)) => {
            for word in heads {
                buffer.push_str(word.word);
                buffer.push_str(word.whitespace);
            }
            buffer.push_str(last.word);
            write_styled(writer, capabilities, style, &buffer)?;
            Ok(CurrentLine {
                length: textwrap::core::display_width(&buffer),
                trailing_space: Some(last.whitespace.to_owned()),
            })
        }
    }
}

pub fn write_styled_and_wrapped<W: Write, S: AsRef<str>>(
    writer: &mut W,
    capabilities: &TerminalCapabilities,
    style: &Style,
    max_width: usize,
    indent: usize,
    current_line: CurrentLine,
    text: S,
) -> Result<CurrentLine> {
    let words = WordSeparator::UnicodeBreakProperties
        .find_words(text.as_ref())
        .collect::<Vec<_>>();
    match words.first() {
        // There were no words in the text so we just do nothing.
        None => Ok(current_line),
        Some(first_word) => {
            let current_width = current_line.length
                + indent
                + current_line
                    .trailing_space
                    .as_ref()
                    .map_or(0, |s| display_width(s.as_ref()));

            // If the current line is not empty and we can't even add the first first word of the text to it
            // then lets finish the line and start over.  If the current line is empty the word simply doesn't
            // fit into the terminal size so we must print it anyway.
            if 0 < current_line.length && max_width < current_width + display_width(first_word) {
                writeln!(writer)?;
                write_indent(writer, indent as u16)?;
                return write_styled_and_wrapped(
                    writer,
                    capabilities,
                    style,
                    max_width,
                    indent,
                    CurrentLine::empty(),
                    text,
                );
            }

            let widths = [
                // For the first line we need to subtract the length of the current line, and
                // the trailing space we need to add if we add more words to this line
                (max_width - current_width.min(max_width)) as f64,
                // For remaining lines we only need to account for the indent
                (max_width - indent) as f64,
            ];
            let lines = textwrap::wrap_algorithms::wrap_first_fit(&words, &widths);
            match lines.split_first() {
                None => {
                    // there was nothing to wrap so we continue as before
                    Ok(current_line)
                }
                Some((first_line, tails)) => {
                    let mut buffer = String::with_capacity(max_width);

                    // Finish the current line
                    let new_current_line = match first_line.split_last() {
                        None => {
                            // The first line was empty, so there's nothing to do anymore.
                            current_line
                        }
                        Some((last, heads)) => {
                            if let Some(s) = current_line.trailing_space {
                                buffer.push_str(&s);
                            }
                            for word in heads {
                                buffer.push_str(word.word);
                                buffer.push_str(word.whitespace);
                            }
                            buffer.push_str(last.word);
                            let length =
                                current_line.length + textwrap::core::display_width(&buffer);
                            write_styled(writer, capabilities, style, &buffer)?;
                            buffer.clear();
                            CurrentLine {
                                length,
                                trailing_space: Some(last.whitespace.to_owned()),
                            }
                        }
                    };

                    // Now write the rest of the lines
                    match tails.split_last() {
                        None => {
                            // There are no more lines and we're done here.
                            //
                            // We arive here when the text fragment we wrapped above was
                            // shorter than the max length of the current line, i.e. we're
                            // still continuing with the current line.
                            Ok(new_current_line)
                        }
                        Some((last_line, next_lines)) => write_remaining_lines(
                            writer,
                            capabilities,
                            style,
                            indent,
                            buffer,
                            next_lines,
                            last_line,
                        ),
                    }
                }
            }
        }
    }
}

#[inline]
pub fn write_mark<W: Write>(writer: &mut W, capabilities: &TerminalCapabilities) -> Result<()> {
    if let Some(mark) = capabilities.marks {
        match mark {
            MarkCapability::ITerm2(marks) => marks.set_mark(writer),
        }
    } else {
        Ok(())
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

pub fn write_link_refs<W: Write>(
    writer: &mut W,
    environment: &Environment,
    capabilities: &TerminalCapabilities,
    links: Vec<LinkReferenceDefinition>,
) -> Result<()> {
    if !links.is_empty() {
        writeln!(writer)?;
        for link in links {
            let style = Style::new().fg(link.colour);
            write_styled(writer, capabilities, &style, &format!("[{}]: ", link.index))?;

            // If we can resolve the link try to write it as inline link to make the URL
            // clickable.  This mostly helps images inside inline links which we had to write as
            // reference links because we can't nest inline links.
            if let Some(url) = environment.resolve_reference(&link.target) {
                use crate::terminal::capabilities::LinkCapability::*;
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
    };
    Ok(())
}

pub fn write_start_code_block<W: Write>(
    writer: &mut W,
    settings: &Settings,
    indent: u16,
    style: Style,
    block_kind: CodeBlockKind<'_>,
    theme: &Theme,
) -> Result<StackedState> {
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
                None => Ok(LiteralBlockAttrs {
                    indent,
                    style: style.fg(Colour::Yellow),
                }
                .into()),
                Some(syntax) => {
                    let parse_state = ParseState::new(syntax);
                    let highlight_state =
                        HighlightState::new(&Highlighter::new(theme), ScopeStack::new());
                    Ok(HighlightBlockAttrs {
                        ansi: *ansi,
                        indent,
                        highlight_state,
                        parse_state,
                    }
                    .into())
                }
            }
        }
        (_, _) => Ok(LiteralBlockAttrs {
            indent,
            style: style.fg(Colour::Yellow),
        }
        .into()),
    }
}

pub fn write_start_heading<W: Write>(
    writer: &mut W,
    capabilities: &TerminalCapabilities,
    style: Style,
    level: HeadingLevel,
) -> Result<StackedState> {
    let style = style.fg(Colour::Blue).bold();
    write_styled(
        writer,
        capabilities,
        &style,
        "\u{2504}".repeat(level as usize),
    )?;

    // Headlines never wrap, so indent doesn't matter
    Ok(StackedState::Inline(
        InlineState::InlineBlock,
        InlineAttrs { style, indent: 0 },
    ))
}
