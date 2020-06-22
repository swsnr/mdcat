// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Rendering tests.
//!
//! Test a set of custom samples as well as all samples from the official common mark spec against
//! known golden files to catch rendering regressions.

// #![deny(warnings, missing_docs, clippy::all)]

use std::path::Path;

use pulldown_cmark::{Options, Parser};
use syntect::parsing::SyntaxSet;
use test_generator::test_resources;

use anyhow::{Context, Result};
use lazy_static::lazy_static;
use std::io::Write;
use url::Url;

lazy_static! {
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref SETTINGS_ANSI_ONLY: mdcat::Settings = mdcat::Settings {
        terminal_capabilities: mdcat::TerminalCapabilities::ansi(),
        terminal_size: mdcat::TerminalSize::default(),
        resource_access: mdcat::ResourceAccess::LocalOnly,
        syntax_set: (*SYNTAX_SET).clone(),
    };
    static ref SETTINGS_VTE50: mdcat::Settings = mdcat::Settings {
        terminal_capabilities: mdcat::TerminalCapabilities::vte50(),
        terminal_size: mdcat::TerminalSize::default(),
        resource_access: mdcat::ResourceAccess::LocalOnly,
        syntax_set: (*SYNTAX_SET).clone(),
    };
    static ref SETTINGS_ITERM2: mdcat::Settings = mdcat::Settings {
        terminal_capabilities: mdcat::TerminalCapabilities::iterm2(),
        terminal_size: mdcat::TerminalSize::default(),
        resource_access: mdcat::ResourceAccess::LocalOnly,
        syntax_set: (*SYNTAX_SET).clone(),
    };
}

fn render<P: AsRef<Path>>(markdown_file: P, settings: &mdcat::Settings) -> Result<String> {
    let markdown = std::fs::read_to_string(&markdown_file).with_context(|| {
        format!(
            "Failed to read markdown from {}",
            markdown_file.as_ref().display()
        )
    })?;
    let parser = Parser::new_ext(
        &markdown,
        Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH,
    );
    let abs_path = std::fs::canonicalize(&markdown_file).with_context(|| {
        format!(
            "Failed to convert {} to an absolute path",
            markdown_file.as_ref().display()
        )
    })?;
    let base_dir = abs_path
        .parent()
        .expect("Absolute file name must have a parent!");
    let mut sink = Vec::new();
    mdcat::push_tty(settings, &mut sink, base_dir, parser).with_context(|| {
        format!(
            "Failed to render contents of {}",
            markdown_file.as_ref().display()
        )
    })?;
    String::from_utf8(sink).with_context(|| "Failed to convert rendered result to string")
}

fn replace_system_specific_urls(s: &str) -> String {
    let hostname = gethostname::gethostname().to_string_lossy().to_string();
    let cwd = std::env::current_dir().expect("Require working directory");

    let mut cwd_url = Url::from_directory_path(cwd).expect("Working directory URL");
    cwd_url
        .set_host(Some(hostname.as_ref()))
        .expect("gethostname as URL host");
    let mut host_url = Url::parse("file://").expect("file://");
    host_url
        .set_host(Some(hostname.as_ref()))
        .expect("gethostname as URL host");
    s.replace(cwd_url.as_str(), "file://HOSTNAME/WORKING_DIRECTORY/")
        .replace(host_url.as_str(), "file://HOSTNAME/")
}

fn render_golden_file<P: AsRef<Path>>(
    golden_dir: P,
    markdown_file: &str,
    settings: &mdcat::Settings,
) {
    // Replace environment specific facts in
    let rendered = replace_system_specific_urls(&render(markdown_file, settings).unwrap());

    let prefix = "tests/render/md";
    let golden_path = Path::new(markdown_file)
        .strip_prefix(prefix)
        .with_context(|| format!("Failed to strip {} from {}", prefix, markdown_file))
        .unwrap();

    let mut mint = goldenfile::Mint::new(
        golden_dir.as_ref().join(
            golden_path
                .parent()
                .with_context(|| format!("Failed to get parent of {}", golden_path.display()))
                .unwrap(),
        ),
    );
    let mut golden = mint
        .new_goldenfile(
            golden_path
                .file_stem()
                .with_context(|| format!("Failed to get stem of {}", golden_path.display()))
                .unwrap(),
        )
        .with_context(|| format!("Failed to open golden file at {}", golden_path.display()))
        .unwrap();
    golden
        .write_all(rendered.as_bytes())
        .with_context(|| format!("Failed to write to {}", golden_path.display()))
        .unwrap();
}

/// Test basic rendering.
#[test_resources("tests/render/md/*/*.md")]
fn ansi_only(markdown_file: &str) {
    render_golden_file(
        "tests/render/golden/ansi-only",
        markdown_file,
        &*SETTINGS_ANSI_ONLY,
    )
}

/// Test basic rendering plus inline links.
#[test_resources("tests/render/md/*/*.md")]
fn vte50(markdown_file: &str) {
    render_golden_file("tests/render/golden/vte50", markdown_file, &*SETTINGS_VTE50)
}

/// Test the full shebang, but not on Windows, since the iTerm2 backend has some unimplemented stuff on Windows.
#[test_resources("tests/render/md/*/*.md")]
#[cfg(not(windows))]
fn iterm2(markdown_file: &str) {
    render_golden_file(
        "tests/render/golden/iterm2",
        markdown_file,
        &*SETTINGS_ITERM2,
    )
}
