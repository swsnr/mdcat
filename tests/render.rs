// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Rendering tests.
//!
//! Test a set of custom samples as well as all samples from the official common mark spec against
//! known golden files to catch rendering regressions.

// #![deny(warnings, missing_docs, clippy::all)]

use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use glob::glob;
use pulldown_cmark::{Options, Parser};
use syntect::parsing::SyntaxSet;

use lazy_static::lazy_static;

lazy_static! {
    // Re-use settings for every generated test; constructing a `SyntaxSet` is really expansive and
    // and doing it for every test again causes a nasty drop in execution speed.
    static ref SETTINGS_ANSI_ONLY: mdcat::Settings = mdcat::Settings {
        terminal_capabilities: mdcat::TerminalCapabilities::ansi(),
        terminal_size: mdcat::TerminalSize::default(),
        resource_access: mdcat::ResourceAccess::LocalOnly,
        syntax_set: SyntaxSet::load_defaults_newlines(),
    };
}

fn render<P: AsRef<Path>>(file: P, settings: &mdcat::Settings) -> Result<Vec<u8>, Box<dyn Error>> {
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
        settings,
        &mut render_buffer,
        &std::env::current_dir().expect("No working directory"),
        parser,
    )?;
    Ok(render_buffer)
}

#[test]
fn ansi_only_commonmark_spec() -> Result<(), Box<dyn Error>> {
    let mut mint = goldenfile::Mint::new("tests/render/golden/commonmark-spec");
    for md_file_res in glob("tests/render/md/commonmark-spec/*.md")? {
        let md_file = md_file_res?;
        mint.new_goldenfile(md_file.file_stem().unwrap())?
            .write_all(&render(md_file, &*SETTINGS_ANSI_ONLY)?)?;
    }
    Ok(())
}

#[test]
fn ansi_only_samples() -> Result<(), Box<dyn Error>> {
    let mut mint = goldenfile::Mint::new("tests/render/golden/samples");
    for md_file_res in glob("tests/render/md/samples/*.md")? {
        let md_file = md_file_res?;
        mint.new_goldenfile(md_file.file_stem().unwrap())?
            .write_all(&render(md_file, &*SETTINGS_ANSI_ONLY)?)?;
    }
    Ok(())
}
