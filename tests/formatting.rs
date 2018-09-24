// Copyright 2018 Sebastian Wiesner <sebastian@swsnr.de>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

// 	http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate mdcat;
extern crate pulldown_cmark;
extern crate syntect;
#[macro_use]
extern crate pretty_assertions;

use pulldown_cmark::Parser;
use std::io::prelude::*;
use std::process::{Command, Stdio};
use syntect::parsing::SyntaxSet;

fn format_ansi_to_html(markdown: &str) -> String {
    let child = Command::new("ansi2html")
        .arg("--input-encoding")
        .arg("utf8")
        .arg("--output-encoding")
        .arg("utf8")
        .arg("--markup-lines")
        .arg("--partial")
        .arg("--inline")
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to start ansi2html");
    {
        let size = mdcat::TerminalSize::default();
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let wd = std::env::current_dir().expect("No working directory");
        let parser = Parser::new(markdown);
        mdcat::push_tty(
            Box::new(mdcat::AnsiTerminal::new(child.stdin.unwrap())),
            size,
            parser,
            &wd,
            mdcat::ResourceAccess::LocalOnly,
            syntax_set,
        ).expect("Formatting failed")
    }
    let mut buffer = Vec::new();
    child
        .stdout
        .unwrap()
        .read_to_end(&mut buffer)
        .expect("Failed to read");

    String::from_utf8(buffer)
        .expect("Failed to convert from bytes")
        // Normalize line endings
        .replace("\r\n", "\n")
}

macro_rules! test_compare_html(
    ($testname:ident) => (
        #[test]
        fn $testname() {
            assert_eq!(
                format_ansi_to_html(include_str!(concat!("formatting/", stringify!($testname), ".md"))),
                include_str!(concat!("formatting/", stringify!($testname), ".html"))
            );
        }
    )
);

test_compare_html!(just_a_line);
test_compare_html!(headers_and_paragraphs);
test_compare_html!(inline_formatting);
test_compare_html!(links);

// TODO: Lists
// TODO: Block quotes
// TODO: Code
