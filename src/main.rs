// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![deny(warnings, clippy::all)]
#![forbid(unsafe_code)]

//! Show CommonMark documents on TTYs.

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use mdcat::{create_resource_handler, process_file};
use pulldown_cmark_mdcat::terminal::{TerminalProgram, TerminalSize};
use pulldown_cmark_mdcat::{Settings, Theme};
use syntect::parsing::SyntaxSet;
use tracing::{event, Level};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;

use mdcat::args::Args;
use mdcat::output::Output;

fn main() {
    // Setup human-readable panic handling for release builds
    human_panic::setup_panic!(
        human_panic::Metadata::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .homepage(env!("CARGO_PKG_HOMEPAGE"))
        .support(
            "Please open an issue at https://github.com/swsnr/mdcat/issues/new?labels=bug\nPlease include the source document which caused this crash.",
        )
    );

    // Initialize curl for remote resources
    curl::init();

    // Setup tracing
    let filter = EnvFilter::builder()
        // Disable all logging by default, to avoid interfering with regular output at all cost.
        // tracing is a debugging tool here so we expect it to be enabled explicitly.
        .with_default_directive(LevelFilter::OFF.into())
        .with_env_var("MDCAT_LOG")
        .from_env_lossy();
    tracing_subscriber::fmt::Subscriber::builder()
        .pretty()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse().command;
    event!(target: "mdcat::main", Level::TRACE, ?args, "mdcat arguments");

    if let Some(shell) = args.completions {
        let binary = match args {
            mdcat::args::Command::Mdcat { .. } => "mdcat",
            mdcat::args::Command::Mdless { .. } => "mdless",
        };
        let mut command = Args::command();
        let subcommand = command.find_subcommand_mut(binary).unwrap();
        generate(shell, subcommand, binary, &mut std::io::stdout());
        std::process::exit(0);
    }

    let terminal = if args.no_colour {
        TerminalProgram::Dumb
    } else if args.paginate() || args.ansi_only {
        // A pager won't support any terminal-specific features
        TerminalProgram::Ansi
    } else {
        TerminalProgram::detect()
    };

    if args.detect_and_exit {
        println!("Terminal: {terminal}");
    } else {
        // Enable Ansi color processing on Windows
        #[cfg(windows)]
        anstyle_query::windows::enable_ansi_colors();

        let terminal_size = TerminalSize::detect().unwrap_or_default();
        let terminal_size = if let Some(max_columns) = args.columns {
            terminal_size.with_max_columns(max_columns)
        } else {
            terminal_size
        };

        let exit_code = match Output::new(args.paginate()) {
            Ok(mut output) => {
                let settings = Settings {
                    terminal_capabilities: terminal.capabilities(),
                    terminal_size,
                    syntax_set: &SyntaxSet::load_defaults_newlines(),
                    theme: Theme::default(),
                };
                event!(
                    target: "mdcat::main",
                    Level::TRACE,
                    ?settings.terminal_size,
                    ?settings.terminal_capabilities,
                    "settings"
                );
                // TODO: Handle this error properly
                let resource_handler = create_resource_handler(args.resource_access()).unwrap();
                args.filenames
                    .iter()
                    .try_fold(0, |code, filename| {
                        process_file(filename, &settings, &resource_handler, &mut output)
                            .map(|_| code)
                            .or_else(|error| {
                                eprintln!("Error: {filename}: {error}");
                                if args.fail_fast {
                                    Err(error)
                                } else {
                                    Ok(1)
                                }
                            })
                    })
                    .unwrap_or(1)
            }
            Err(error) => {
                eprintln!("Error: {error:#}");
                128
            }
        };
        event!(target: "mdcat::main", Level::TRACE, "Exiting with final exit code {}", exit_code);
        std::process::exit(exit_code);
    }
}
