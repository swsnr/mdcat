// Copyright 2018-2019 Sebastian Wiesner <sebastian@swsnr.de>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

// 	http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![deny(warnings, missing_docs, clippy::all)]
// Currently we only run formatting tests on Unix, because we rely on a Python
// tool here, and I failed to setup Python properly on Travis CI' Windows
// workers.
#![cfg(unix)]

use mdcat;

use pretty_assertions::assert_eq;
use pulldown_cmark::{Options, Parser};
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use syntect::parsing::SyntaxSet;

fn format_ansi_to_html(markdown: &str) -> String {
    let child = Command::new("ansi2html")
        .arg("--input-encoding")
        .arg("utf8")
        .arg("--output-encoding")
        .arg("utf8")
        .arg("--markup-lines")
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to start ansi2html");
    {
        let size = mdcat::TerminalSize::default();
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let wd = std::env::current_dir().expect("No working directory");
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TASKLISTS);
        let parser = Parser::new_ext(markdown, options);
        mdcat::push_tty(
            &mut child.stdin.unwrap(),
            mdcat::TerminalCapabilities::ansi(),
            size,
            parser,
            &wd,
            mdcat::ResourceAccess::LocalOnly,
            syntax_set,
        )
        .expect("Formatting failed")
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

fn test_directory() -> PathBuf {
    Path::new(file!())
        .parent()
        .expect("Failed to get parent directory")
        .join("formatting")
}

fn read_file(basename: &str, extension: &str) -> String {
    let mut contents = String::new();
    let path = test_directory().join(basename).with_extension(extension);
    File::open(path)
        .and_then(|mut source| source.read_to_string(&mut contents))
        .expect("Failed to read test file");
    contents
}

fn assert_formats_to_expected_html(basename: &str) {
    let markdown = read_file(basename, "md");
    let actual_html = format_ansi_to_html(&markdown);

    let target = test_directory()
        .join(basename)
        .with_extension("actual.html");
    File::create(target)
        .and_then(|mut f| f.write_all(actual_html.as_bytes()))
        .expect("Failed to write actual HTML");

    let expected_html = read_file(basename, "expected.html");
    assert_eq!(actual_html, expected_html, "Different format produced");
}

macro_rules! test_compare_html(
    ($testname:ident) => (
        #[test]
        fn $testname() {
            crate::assert_formats_to_expected_html(stringify!($testname));
        }
    )
);

mod formatting {
    mod html {
        test_compare_html!(block_quote_and_ruler);
        test_compare_html!(code_blocks);
        test_compare_html!(headers_and_paragraphs);
        test_compare_html!(inline_formatting);
        test_compare_html!(just_a_line);
        test_compare_html!(links);
        test_compare_html!(lists);
        test_compare_html!(tasklist);
    }
}
