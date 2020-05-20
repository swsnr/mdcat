// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![deny(warnings, missing_docs, clippy::all)]

///! Check every test case from the commonmark spec.
///
/// The commonmark-spec/update.py script generates a markdown file for each official test case;
/// this module generates a corresponding test case for each markdown file.
///
/// **Note:** These test cases are not intended to set an expection about how the "right" rendering
/// looks like; they exist to test that we _can_ render all sorts of commonmark constructs without
/// errors or panics, and to catch regressions and unintended changes in rendering format when
/// working on the rendering code.
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::path::Path;

use lazy_static::lazy_static;
use pulldown_cmark::{Options, Parser};
use syntect::parsing::SyntaxSet;
use test_generator::test_resources;

lazy_static! {
    // Re-use settings for every generated test; constructing a `SyntaxSet` is really expansive and
    // and doing it for every test again causes a nasty drop in execution speed.
    static ref SETTINGS: mdcat::Settings = mdcat::Settings {
        terminal_capabilities: mdcat::TerminalCapabilities::ansi(),
        terminal_size: mdcat::TerminalSize::default(),
        resource_access: mdcat::ResourceAccess::LocalOnly,
        syntax_set: SyntaxSet::load_defaults_newlines(),
    };
}

fn render<P: AsRef<Path>>(file: P) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let markdown = {
        let mut buffer = String::new();
        let mut source = File::open(file)?;
        source.read_to_string(&mut buffer)?;
        buffer
    };
    let parser = Parser::new_ext(&markdown, options);
    let mut render_buffer: Vec<u8> = Vec::new();
    mdcat::push_tty(
        &*SETTINGS,
        &mut render_buffer,
        &std::env::current_dir().expect("No working directory"),
        parser,
    )?;
    Ok(render_buffer)
}

/// Check whether the given markdown file renders as expected.
///
/// The expectation is set by an `.expected` file beneath the markdown file; if this file does not
/// exist write the rendered output to this file and panic, in order to update a test case in case
/// of a spec update and notify the developer about it.
///
/// If the expectation exists write the rendered output to an `.actual` file beneath it, for
/// inspection, and compare the output against the expectation.
#[test_resources("tests/commonmark-spec/*.md")]
fn renders_as_expected(resource: &str) {
    let expected_output = format!("{}.expected", resource);

    if Path::new(&expected_output).is_file() {
        let actual_output = format!("{}.actual", resource);
        let mut sink = File::create(&actual_output).expect("Opening actual output file failed");
        sink.write_all(&render(resource).expect("Rendering failed"))
            .expect("Writing actual output failed");

        let mut expected = Vec::new();
        let mut source = File::open(&expected_output).expect("Opening expected output failed");
        source
            .read_to_end(&mut expected)
            .expect("Reading expected output failed");
        pretty_assertions::assert_eq!(
            String::from_utf8_lossy(&render(resource).expect("Rendering failed")),
            String::from_utf8_lossy(&expected)
        );
    } else {
        let rendered = render(resource).expect("Rendering failed");
        let mut sink = File::create(&expected_output).expect("Opening expected output file failed");
        sink.write_all(&rendered)
            .expect("Writing expected output failed");
        panic!("No expected output found; creating at {}", expected_output);
    }
}
