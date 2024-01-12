// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    bufferline::{render_buffer_lines, BufferLine, BufferLines},
    push_tty,
    reflow::{LineComposer, WordWrapper, WrappedLine},
    resources::FileResourceHandler,
    Environment, Settings, TerminalProgram, TerminalSize, Theme,
};
use pulldown_cmark::{Options, Parser};
use ratatui::{prelude::*, widgets::StatefulWidget};
use ratatui::{text::StyledGrapheme, widgets::Widget};
use std::io::Write;
use std::{fs, path::PathBuf};
use syntect::parsing::SyntaxSet;
use unicode_width::UnicodeWidthStr;

///
pub enum PathOrStr {
    ///
    MdPath(PathBuf, String),

    ///
    MdStr(String),

    /// render String as normal text (just like Paragraph)
    NormalStr(String),
}

impl PathOrStr {
    ///
    pub fn from_md_path(p: PathBuf) -> Self {
        let input = fs::read_to_string(&p).unwrap();
        Self::MdPath(p, input)
    }

    ///
    pub fn get_str(&self) -> &str {
        match self {
            PathOrStr::MdPath(_, input) => input,
            PathOrStr::MdStr(input) => input,
            PathOrStr::NormalStr(input) => input,
        }
    }

    fn get_parser<'input, 'callback>(&'input self) -> Parser<'input, 'callback> {
        Parser::new_ext(
            self.get_str(),
            Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH,
        )
    }

    fn get_environment(&self) -> Environment {
        match self {
            PathOrStr::MdPath(p, _) => {
                Environment::for_local_directory(&p.parent().unwrap()).unwrap()
            }
            // give a fake path
            PathOrStr::MdStr(_) => Environment::for_local_directory(&PathBuf::from("/")).unwrap(),
            PathOrStr::NormalStr(_) => unreachable!(),
        }
    }

    pub fn get_bufferlines(&self, area: Rect) -> Vec<BufferLine> {
        match self {
            PathOrStr::MdPath(_, _) | PathOrStr::MdStr(_) => {
                let terminal_capabilities = TerminalProgram::detect().capabilities();
                let settings = Settings {
                    terminal_capabilities,
                    terminal_size: TerminalSize::detect()
                        .unwrap_or_default()
                        .with_max_columns(area.width),
                    syntax_set: &SyntaxSet::load_defaults_newlines(),
                    theme: Theme::default(),
                };
                let mut writer = BufferLines::new();
                push_tty(
                    &settings,
                    &self.get_environment(),
                    &FileResourceHandler::new(104_857_600),
                    &mut writer,
                    self.get_parser(),
                )
                .unwrap();

                writer.finish()
            }
            PathOrStr::NormalStr(text) => {
                let t = Text::raw(text);
                let styled = t.lines.iter().map(|line| {
                    let graphemes = line
                        .spans
                        .iter()
                        .flat_map(|span| span.styled_graphemes(Style::default()));
                    let alignment = line.alignment.unwrap_or(Alignment::Left);
                    (graphemes, alignment)
                });
                let line_composer = WordWrapper::new(styled, area.width, true);
                render_text(line_composer, area)
            }
        }
    }
}

fn render_text<'a, C: LineComposer<'a>>(mut composer: C, _area: Rect) -> Vec<BufferLine> {
    let mut writer = BufferLines::new();

    while let Some(WrappedLine {
        line: current_line,
        width: _current_line_width,
        alignment: _current_line_alignment,
    }) = composer.next_line()
    {
        for StyledGrapheme {
            symbol,
            style: _style,
        } in current_line
        {
            let width = symbol.width();
            if width == 0 {
                continue;
            }
            // If the symbol is empty, the last char which rendered last time will
            // leave on the line. It's a quick fix.
            let symbol = if symbol.is_empty() { " " } else { symbol };
            write!(writer, "{symbol}").unwrap();
        }
        writer.writeln_buffer();
    }
    writer.finish()
}

/// similar to paragraph
/// Robust but slow, especially when scroll frequently
/// see example/slow.rs
pub struct MarkdownWidget<'a> {
    /// path of markdown file
    pub path_or_str: &'a Vec<PathOrStr>,
}

impl Widget for MarkdownWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        StatefulWidget::render(self, area, buf, &mut Offset { x: 0, y: 0 })
    }
}

impl StatefulWidget for MarkdownWidget<'_> {
    type State = Offset;

    fn render(self, area: Rect, _buf: &mut Buffer, state: &mut Self::State) {
        yazi_adaptor::init();
        let terminal_capabilities = TerminalProgram::detect().capabilities();
        terminal_capabilities
            .image
            .unwrap()
            .image_erase(area)
            .unwrap();

        let buffer_lines: Vec<BufferLine> = self
            .path_or_str
            .iter()
            .map(|x| {
                let mut y = x.get_bufferlines(area);
                y.push(BufferLine::Line(Vec::new()));
                y
            })
            .flatten()
            .collect();
        render_buffer_lines(&buffer_lines, area, state);
    }
}

/// Fast, but tricky to use
/// see example/faster.rs
pub struct FasterMarkdownWidget<T: AsRef<[BufferLine]>> {
    pub t: T,
}

impl<T: AsRef<[BufferLine]>> StatefulWidget for FasterMarkdownWidget<T> {
    type State = Offset;

    fn render(self, area: Rect, _buf: &mut Buffer, state: &mut Self::State) {
        yazi_adaptor::init();
        TerminalProgram::detect()
            .capabilities()
            .image
            .unwrap()
            .image_erase(area)
            .unwrap();

        render_buffer_lines(&self.t.as_ref(), area, state);
    }
}

///
pub struct Offset {
    /// uselesss now
    pub x: u16,
    ///
    pub y: u16,
}
