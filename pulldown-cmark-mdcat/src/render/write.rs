// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::{Result, Write};

use anstyle::Style;
use pulldown_cmark::{CodeBlockKind, HeadingLevel};
use syntect::highlighting::HighlightState;
use syntect::parsing::{ParseState, ScopeStack};
use textwrap::core::{display_width, Word};
use textwrap::WordSeparator;

use crate::bufferline::BufferLines;
use crate::references::*;
use crate::render::data::{CurrentLine, LinkReferenceDefinition};
use crate::render::highlighting::highlighter;
use crate::render::state::*;
use crate::terminal::capabilities::{MarkCapability, StyleCapability, TerminalCapabilities};
use crate::terminal::osc::{clear_link, set_link_url};
use crate::terminal::TerminalSize;
use crate::theme::CombineStyle;
use crate::Theme;
use crate::{Environment, Settings};

pub fn write_indent<W: Write>(writer: &mut W, level: u16) -> Result<()> {
    write!(writer, "{}", " ".repeat(level as usize))
}

pub fn write_styled<W: Write, S: AsRef<str>>(
    writer: &mut W,
    capabilities: &TerminalCapabilities,
    style: &Style,
    text: S,
) -> Result<()> {
    match capabilities.style {
        None => write!(writer, "{}", text.as_ref()),
        Some(StyleCapability::Ansi) => write!(
            writer,
            "{}{}{}",
            style.render(),
            text.as_ref(),
            style.render_reset()
        ),
    }
}

fn write_remaining_lines(
    writer: &mut BufferLines,
    capabilities: &TerminalCapabilities,
    style: &Style,
    indent: u16,
    mut buffer: String,
    next_lines: &[&[Word]],
    last_line: &[Word],
) -> Result<CurrentLine> {
    // Finish the previous line
    writer.writeln_buffer();
    write_indent(writer, indent)?;
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
                writer.writeln_buffer();
                write_indent(writer, indent)?;
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
                length: textwrap::core::display_width(&buffer) as u16,
                trailing_space: Some(last.whitespace.to_owned()),
            })
        }
    }
}

pub fn write_styled_and_wrapped<S: AsRef<str>>(
    writer: &mut BufferLines,
    capabilities: &TerminalCapabilities,
    style: &Style,
    max_width: u16,
    indent: u16,
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
                    .map_or(0, |s| display_width(s.as_ref()) as u16);

            // If the current line is not empty and we can't even add the first first word of the text to it
            // then lets finish the line and start over.  If the current line is empty the word simply doesn't
            // fit into the terminal size so we must print it anyway.
            if 0 < current_line.length
                && max_width < current_width + display_width(first_word) as u16
            {
                writer.writeln_buffer();
                write_indent(writer, indent)?;
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
                    let mut buffer = String::with_capacity(max_width as usize);

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
                                current_line.length + textwrap::core::display_width(&buffer) as u16;
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
                            // We arrive here when the text fragment we wrapped above was
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

pub fn write_mark<W: Write>(writer: &mut W, capabilities: &TerminalCapabilities) -> Result<()> {
    if let Some(mark) = capabilities.marks {
        match mark {
            MarkCapability::ITerm2(marks) => marks.set_mark(writer),
        }
    } else {
        Ok(())
    }
}

pub fn write_rule<W: Write>(
    writer: &mut W,
    capabilities: &TerminalCapabilities,
    theme: &Theme,
    length: u16,
) -> std::io::Result<()> {
    let rule = "\u{2550}".repeat(length as usize);
    write_styled(
        writer,
        capabilities,
        &Style::new().fg_color(Some(theme.rule_color)),
        rule,
    )
}

pub fn write_code_block_border(
    writer: &mut BufferLines,
    theme: &Theme,
    capabilities: &TerminalCapabilities,
    terminal_size: &TerminalSize,
) -> std::io::Result<()> {
    let separator = "\u{2500}".repeat(terminal_size.columns.min(20) as usize);
    write_styled(
        writer,
        capabilities,
        &Style::new().fg_color(Some(theme.code_block_border_color)),
        separator,
    )?;
    writer.writeln_buffer();
    Ok(())
}

pub fn write_link_refs(
    writer: &mut BufferLines,
    environment: &Environment,
    capabilities: &TerminalCapabilities,
    links: Vec<LinkReferenceDefinition>,
) -> Result<()> {
    if !links.is_empty() {
        writer.writeln_buffer();
        for link in links {
            write_styled(
                writer,
                capabilities,
                &link.style,
                &format!("[{}]: ", link.index),
            )?;

            // If we can resolve the link try to write it as inline link to make the URL
            // clickable.  This mostly helps images inside inline links which we had to write as
            // reference links because we can't nest inline links.
            if let Some(url) = environment.resolve_reference(&link.target) {
                match &capabilities.style {
                    Some(StyleCapability::Ansi) => {
                        set_link_url(writer, url, &environment.hostname)?;
                        write_styled(writer, capabilities, &link.style, link.target)?;
                        clear_link(writer)?;
                    }
                    None => write_styled(writer, capabilities, &link.style, link.target)?,
                };
            } else {
                write_styled(writer, capabilities, &link.style, link.target)?;
            }

            if !link.title.is_empty() {
                write_styled(
                    writer,
                    capabilities,
                    &link.style,
                    format!(" {}", link.title),
                )?;
            }
            writer.writeln_buffer();
        }
    };
    Ok(())
}

pub fn write_start_code_block(
    writer: &mut BufferLines,
    settings: &Settings,
    indent: u16,
    style: Style,
    block_kind: CodeBlockKind<'_>,
) -> Result<StackedState> {
    write_indent(writer, indent)?;
    write_code_block_border(
        writer,
        &settings.theme,
        &settings.terminal_capabilities,
        &settings.terminal_size,
    )?;
    // And start the indent for the contents of the block
    write_indent(writer, indent)?;

    match (&settings.terminal_capabilities.style, block_kind) {
        (Some(StyleCapability::Ansi), CodeBlockKind::Fenced(name)) if !name.is_empty() => {
            match settings.syntax_set.find_syntax_by_token(&name) {
                None => Ok(LiteralBlockAttrs {
                    indent,
                    style: settings.theme.code_style.on_top_of(&style),
                }
                .into()),
                Some(syntax) => {
                    let parse_state = ParseState::new(syntax);
                    let highlight_state = HighlightState::new(highlighter(), ScopeStack::new());
                    Ok(HighlightBlockAttrs {
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
            style: settings.theme.code_style.on_top_of(&style),
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
