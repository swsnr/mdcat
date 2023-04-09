// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Various tests for render.md.wrapping output.

#![deny(warnings, missing_docs, clippy::all)]

use glob::glob;
use once_cell::sync::Lazy;
use pulldown_cmark::{Options, Parser};
use syntect::parsing::SyntaxSet;

use anyhow::{Context, Result};
use mdcat::{terminal::TerminalProgram, Environment};

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static SETTINGS_ANSI_ONLY: Lazy<mdcat::Settings> = Lazy::new(|| mdcat::Settings {
    terminal_capabilities: TerminalProgram::Ansi.capabilities(),
    terminal_size: mdcat::terminal::TerminalSize::default(),
    resource_access: mdcat::ResourceAccess::LocalOnly,
    syntax_set: &SYNTAX_SET,
});

fn render_to_string<S: AsRef<str>>(markdown: S, settings: &mdcat::Settings) -> Result<String> {
    let parser = Parser::new_ext(
        markdown.as_ref(),
        Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH,
    );
    let mut sink = Vec::new();
    let env = Environment {
        hostname: "HOSTNAME".to_string(),
        ..Environment::for_local_directory(&std::env::current_dir()?)?
    };
    mdcat::push_tty(settings, &env, &mut sink, parser)?;
    String::from_utf8(sink).with_context(|| "Failed to convert rendered result to string")
}

#[test]
fn lines_are_below_column_width_of_terminal() {
    for entry in glob("tests/render/mnd/wrapping/*.md").unwrap() {
        let markdown_file = entry.unwrap();
        let markdown = std::fs::read_to_string(&markdown_file).unwrap();
        let result = render_to_string(markdown, &SETTINGS_ANSI_ONLY).unwrap();
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
