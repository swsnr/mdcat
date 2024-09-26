// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Rendering tests.
//!
//! Test a set of custom samples as well as all samples from the official common
//! mark spec against fixed snapshots to catch rendering regressions.

#![deny(warnings, clippy::all)]

use std::path::Path;
use std::sync::OnceLock;

use insta::{assert_snapshot, glob};
use mdcat_http_reqwest::HttpResourceHandler;
use pulldown_cmark::{Options, Parser};
use syntect::parsing::SyntaxSet;
use url::Url;

use pulldown_cmark_mdcat::resources::*;
use pulldown_cmark_mdcat::terminal::{TerminalProgram, TerminalSize};
use pulldown_cmark_mdcat::Settings;
use pulldown_cmark_mdcat::{Environment, Theme};

static TEST_READ_LIMIT: u64 = 5_242_880;

static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();

fn syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

static RESOURCE_HANDLER: OnceLock<DispatchingResourceHandler> = OnceLock::new();

fn resource_handler() -> &'static DispatchingResourceHandler {
    RESOURCE_HANDLER.get_or_init(|| {
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
    })
}

fn render_to_string<P: AsRef<Path>>(markdown_file: P, settings: &Settings) -> String {
    let markdown = std::fs::read_to_string(&markdown_file).unwrap();
    let parser = Parser::new_ext(
        &markdown,
        Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES,
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
    pulldown_cmark_mdcat::push_tty(settings, &env, resource_handler(), &mut sink, parser).unwrap();
    String::from_utf8(sink).unwrap()
}

#[test]
fn test_render() {
    let cwd = std::env::current_dir().expect("Require working directory");
    let mut cwd_url = Url::from_directory_path(&cwd).expect("Working directory URL");
    let mut root_url = cwd_url.join("/").expect("Join root URL");
    root_url.set_host(Some("HOSTNAME")).unwrap();
    cwd_url.set_host(Some("HOSTNAME")).unwrap();

    let dumb_settings = Settings {
        terminal_capabilities: TerminalProgram::Dumb.capabilities(),
        terminal_size: TerminalSize::default(),
        theme: Theme::default(),
        syntax_set: syntax_set(),
    };
    let ansi_settings = Settings {
        terminal_capabilities: TerminalProgram::Ansi.capabilities(),
        terminal_size: TerminalSize::default(),
        theme: Theme::default(),
        syntax_set: syntax_set(),
    };
    let iterm2_settings = Settings {
        terminal_capabilities: TerminalProgram::ITerm2.capabilities(),
        terminal_size: TerminalSize::default(),
        theme: Theme::default(),
        syntax_set: syntax_set(),
    };

    glob!("markdown/**/*.md", |markdown_file| {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path("snapshots/render");
        settings.set_prepend_module_to_snapshot(false);
        settings.add_filter(
            regex::escape(cwd_url.as_str()).as_str(),
            "file://HOSTNAME/WORKING_DIRECTORY/",
        );
        settings.add_filter(
            regex::escape(root_url.as_str()).as_str(),
            "file://HOSTNAME/ROOT/",
        );
        let category = markdown_file
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let name = markdown_file.file_stem().unwrap().to_str().unwrap();
        settings.set_snapshot_suffix(format!("{category}-{name}"));
        let _guard = settings.bind_to_scope();
        assert_snapshot!("dumb", render_to_string(markdown_file, &dumb_settings));
        assert_snapshot!("ansi", render_to_string(markdown_file, &ansi_settings));
        assert_snapshot!("iterm2", render_to_string(markdown_file, &iterm2_settings));
        drop(_guard);
    });
}
