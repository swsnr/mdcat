// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Rendering tests.
//!
//! Test a set of custom samples as well as all samples from the official common mark spec against
//! known golden files to catch rendering regressions.

#![deny(warnings, clippy::all)]

use std::path::Path;

use glob::glob;
use mdcat_http_reqwest::HttpResourceHandler;
use once_cell::sync::Lazy;
use pulldown_cmark::{Options, Parser};
use similar_asserts::assert_eq;
use syntect::parsing::SyntaxSet;
use url::Url;

use pulldown_cmark_mdcat::resources::*;
use pulldown_cmark_mdcat::terminal::{TerminalProgram, TerminalSize};
use pulldown_cmark_mdcat::Settings;
use pulldown_cmark_mdcat::{Environment, Theme};

static TEST_READ_LIMIT: u64 = 5_242_880;

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);

static RESOURCE_HANDLER: Lazy<DispatchingResourceHandler> = Lazy::new(|| {
    let handlers: Vec<Box<dyn ResourceUrlHandler>> = vec![
        Box::new(FileResourceHandler::new(TEST_READ_LIMIT)),
        Box::new(
            HttpResourceHandler::with_user_agent(
                TEST_READ_LIMIT,
                concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
            )
            .unwrap(),
        ),
    ];
    DispatchingResourceHandler::new(handlers)
});

fn render_to_string<P: AsRef<Path>>(markdown_file: P, settings: &Settings) -> String {
    let markdown = std::fs::read_to_string(&markdown_file).unwrap();
    let parser = Parser::new_ext(
        &markdown,
        Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH,
    );
    let abs_path = std::fs::canonicalize(&markdown_file).unwrap();
    let base_dir = abs_path
        .parent()
        .expect("Absolute file name must have a parent!");
    let mut sink = Vec::new();
    let env = Environment {
        hostname: "HOSTNAME".to_string(),
        ..Environment::for_local_directory(&base_dir).unwrap()
    };
    pulldown_cmark_mdcat::push_tty(settings, &env, &*RESOURCE_HANDLER, &mut sink, parser).unwrap();
    String::from_utf8(sink).unwrap()
}

fn replace_system_specific_urls(input: String) -> String {
    let cwd = std::env::current_dir().expect("Require working directory");

    let mut urls = [
        // Replace any URLs pointing to the current working directory
        (
            Url::from_directory_path(&cwd).expect("Working directory URL"),
            "file://HOSTNAME/WORKING_DIRECTORY/",
        ),
        // Replace any URLs pointing to the root URL, to account for windows drive letters.
        (
            Url::from_directory_path(&cwd)
                .expect("Working directory URL")
                .join("/")
                .expect("Root URL"),
            "file://HOSTNAME/ROOT/",
        ),
    ];

    urls.iter_mut().fold(input, |s, (url, replacement)| {
        url.set_host(Some("HOSTNAME"))
            .expect("gethostname as URL host");
        s.replace(url.as_str(), replacement)
    })
}

fn test_with_golden_file<S: AsRef<Path>, T: AsRef<Path>>(
    markdown_file: S,
    golden_file_directory: T,
    settings: &Settings,
) {
    // Replace environment specific facts in
    let actual = replace_system_specific_urls(render_to_string(markdown_file.as_ref(), settings));

    let basename = markdown_file
        .as_ref()
        .strip_prefix("tests/render/md")
        .unwrap()
        .with_extension("");
    let expected_file = golden_file_directory.as_ref().join(&basename);

    if std::env::var_os("MDCAT_UPDATE_GOLDEN_FILES").is_some() {
        std::fs::create_dir_all(expected_file.parent().unwrap()).unwrap();
        std::fs::write(&expected_file, &actual).unwrap()
    } else {
        let expected = std::fs::read_to_string(&expected_file).unwrap();
        assert_eq!(actual, expected, "Test case: {}", basename.display());
    }
}

/// Test rendering without any formatting, just to test the general layout
#[test]
fn dump() {
    let settings = Settings {
        terminal_capabilities: TerminalProgram::Dumb.capabilities(),
        terminal_size: TerminalSize::default(),
        theme: Theme::default(),
        syntax_set: &SYNTAX_SET,
    };
    for markdown_file in glob("tests/render/md/*/*.md").unwrap() {
        test_with_golden_file(
            markdown_file.unwrap(),
            "tests/render/golden/dump",
            &settings,
        )
    }
}

/// Test rendering without inline images.
#[test]
fn ansi_only() {
    let settings = Settings {
        terminal_capabilities: TerminalProgram::Ansi.capabilities(),
        terminal_size: TerminalSize::default(),
        theme: Theme::default(),
        syntax_set: &SYNTAX_SET,
    };
    for markdown_file in glob("tests/render/md/*/*.md").unwrap() {
        test_with_golden_file(
            markdown_file.unwrap(),
            "tests/render/golden/ansi-only",
            &settings,
        )
    }
}

/// Test rendering with inline images and jump marks.
#[test]
fn iterm2() {
    let settings = Settings {
        terminal_capabilities: TerminalProgram::ITerm2.capabilities(),
        terminal_size: TerminalSize::default(),
        theme: Theme::default(),
        syntax_set: &SYNTAX_SET,
    };
    for markdown_file in glob("tests/render/md/*/*.md").unwrap() {
        test_with_golden_file(
            markdown_file.unwrap(),
            "tests/render/golden/iterm2",
            &settings,
        )
    }
}
