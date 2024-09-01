// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Various tests for render.md.wrapping output.

#![deny(warnings, clippy::all)]

use glob::glob;
use pulldown_cmark::{Options, Parser};
use syntect::parsing::SyntaxSet;

use pulldown_cmark_mdcat::resources::NoopResourceHandler;
use pulldown_cmark_mdcat::terminal::{TerminalProgram, TerminalSize};
use pulldown_cmark_mdcat::{Environment, Settings, Theme};

fn render_to_string<S: AsRef<str>>(markdown: S, settings: &Settings) -> String {
    let parser = Parser::new_ext(
        markdown.as_ref(),
        Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES,
    );
    let mut sink = Vec::new();
    let env = Environment {
        hostname: "HOSTNAME".to_string(),
        ..Environment::for_local_directory(&std::env::current_dir().unwrap()).unwrap()
    };
    pulldown_cmark_mdcat::push_tty(settings, &env, &NoopResourceHandler, &mut sink, parser)
        .unwrap();
    String::from_utf8(sink).unwrap()
}

#[test]
fn lines_are_below_column_width_of_terminal() {
    for entry in glob("tests/render/mnd/wrapping/*.md").unwrap() {
        let markdown_file = entry.unwrap();
        let markdown = std::fs::read_to_string(&markdown_file).unwrap();
        let settings = Settings {
            terminal_capabilities: TerminalProgram::Ansi.capabilities(),
            terminal_size: TerminalSize::default(),
            theme: Theme::default(),
            syntax_set: &SyntaxSet::load_defaults_newlines(),
        };
        let result = render_to_string(markdown, &settings);
        for line in result.lines() {
            let width = textwrap::core::display_width(line);
            assert!(
                width <= 80,
                "line {} has length {} in test case {}",
                line,
                width,
                markdown_file
                    .strip_prefix("tests/render/md")
                    .unwrap()
                    .with_extension("")
                    .display()
            );
        }
    }
}
