// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![deny(warnings, clippy::all)]

//! Show CommonMark documents on TTYs.

use std::fs::File;
use std::io::prelude::*;
use std::io::{stdin, BufWriter};
use std::io::{Error, Result};
use std::path::PathBuf;

use fehler::throws;
use pulldown_cmark::{Options, Parser};
use syntect::parsing::SyntaxSet;
use tracing::{event, instrument, Level};

use crate::output::Output;
use mdcat::{Environment, Settings};
use mdcat::{ResourceAccess, TerminalCapabilities, TerminalSize};

mod args;
mod output;

/// Read input for `filename`.
///
/// If `filename` is `-` read from standard input, otherwise try to open and
/// read the given file.
#[throws]
fn read_input<T: AsRef<str>>(filename: T) -> (PathBuf, String) {
    let cd = std::env::current_dir()?;
    let mut buffer = String::new();

    if filename.as_ref() == "-" {
        stdin().read_to_string(&mut buffer)?;
        (cd, buffer)
    } else {
        let mut source = File::open(filename.as_ref())?;
        source.read_to_string(&mut buffer)?;
        let base_dir = cd
            .join(filename.as_ref())
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or(cd);
        (base_dir, buffer)
    }
}

#[instrument(skip(output, settings), level = "debug")]
fn process_file(filename: &str, settings: &Settings, output: &mut Output) -> Result<()> {
    let (base_dir, input) = read_input(filename)?;
    event!(
        Level::TRACE,
        "Read input, using {} as base directory",
        base_dir.display()
    );
    let parser = Parser::new_ext(
        &input,
        Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH,
    );
    let env = Environment::for_local_directory(&base_dir)?;

    let mut sink = BufWriter::new(output.writer());
    // mdcat::push_tty(settings, &env, &mut sink, parser).expect("FUCK");
    mdcat::push_tty(settings, &env, &mut sink, parser)
        .and_then(|_| {
            event!(Level::TRACE, "Finished rendering, flushing output");
            sink.flush()
        })
        .or_else(|error| {
            if error.kind() == std::io::ErrorKind::BrokenPipe {
                event!(Level::TRACE, "Ignoring broken pipe");
                Ok(())
            } else {
                event!(Level::ERROR, ?error, "Failed to process file: {:#}", error);
                Err(error)
            }
        })
}

/// Represent command line arguments.
#[derive(Debug)]
struct Arguments {
    filenames: Vec<String>,
    terminal_capabilities: TerminalCapabilities,
    resource_access: ResourceAccess,
    columns: usize,
    detect_only: bool,
    fail_fast: bool,
    paginate: bool,
}

fn is_mdless() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.file_stem().map(|stem| stem == "mdless"))
        .unwrap_or(false)
}

impl Arguments {
    /// Create command line arguments from matches.
    fn from_matches(matches: &clap::ArgMatches) -> clap::Result<Self> {
        // On Windows 10 we need to enable ANSI term explicitly.
        #[cfg(windows)]
        {
            trace!("Enable ANSI support in windows terminal");
            ansi_term::enable_ansi_support().ok();
        }

        let filenames = matches.values_of_t("filenames")?;
        let detect_only = matches.is_present("detect_only");
        let fail_fast = matches.is_present("fail_fast");
        let paginate =
            (is_mdless() || matches.is_present("paginate")) && !matches.is_present("no_pager");

        let columns = matches.value_of_t("columns")?;
        let resource_access = if matches.is_present("local_only") {
            ResourceAccess::LocalOnly
        } else {
            ResourceAccess::RemoteAllowed
        };

        let terminal_capabilities = if matches.is_present("no_colour") {
            // If the user disabled colours assume a dumb terminal
            TerminalCapabilities::none()
        } else if paginate || matches.is_present("ansi_only") {
            // A pager won't support any terminal-specific features
            TerminalCapabilities::ansi()
        } else {
            TerminalCapabilities::detect()
        };

        Ok(Arguments {
            filenames,
            terminal_capabilities,
            resource_access,
            columns,
            detect_only,
            fail_fast,
            paginate,
        })
    }
}

fn long_version() -> &'static str {
    concat!(
        clap::crate_version!(),
        "
Copyright (C) Sebastian Wiesner and contributors

This program is subject to the terms of the Mozilla Public License,
v. 2.0. If a copy of the MPL was not distributed with this file,
You can obtain one at http://mozilla.org/MPL/2.0/."
    )
}

fn main() {
    // Setup tracing
    tracing_subscriber::fmt::Subscriber::builder()
        .pretty()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_env("MDCAT_LOG"))
        .with_writer(std::io::stderr)
        .init();

    let size = TerminalSize::detect().unwrap_or_default();
    let columns = size.columns.to_string();

    let matches = args::app(&columns)
        .long_version(long_version())
        .get_matches();
    event!(Level::TRACE, "clap parsed: {:?}", matches);
    let arguments = Arguments::from_matches(&matches).unwrap_or_else(|e| e.exit());
    event!(Level::TRACE, "mdcat arguments: {:?}", arguments);

    if arguments.detect_only {
        println!("Terminal: {}", arguments.terminal_capabilities.name);
    } else {
        let Arguments {
            filenames,
            fail_fast,
            terminal_capabilities,
            columns,
            resource_access,
            paginate,
            ..
        } = arguments;

        let exit_code = match Output::new(paginate) {
            Ok(mut output) => {
                let settings = Settings {
                    terminal_capabilities,
                    terminal_size: TerminalSize { columns, ..size },
                    resource_access,
                    syntax_set: SyntaxSet::load_defaults_newlines(),
                };
                event!(
                    Level::TRACE,
                    ?settings.terminal_size,
                    ?settings.terminal_capabilities,
                    ?settings.resource_access,
                    "settings"
                );
                filenames
                    .iter()
                    .try_fold(0, |code, filename| {
                        process_file(filename, &settings, &mut output)
                            .map(|_| code)
                            .or_else(|error| {
                                eprintln!("Error: {}: {}", filename, error);
                                if fail_fast {
                                    Err(error)
                                } else {
                                    Ok(1)
                                }
                            })
                    })
                    .unwrap_or(1)
            }
            Err(error) => {
                eprintln!("Error: {:#}", error);
                128
            }
        };
        event!(Level::TRACE, "Exiting with final exit code {}", exit_code);
        std::process::exit(exit_code);
    }
}
