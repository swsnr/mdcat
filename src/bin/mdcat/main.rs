// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![deny(warnings, clippy::all)]

//! Show CommonMark documents on TTYs.

use std::fs::File;
use std::io::prelude::*;
use std::io::Result;
use std::io::{stdin, BufWriter};
use std::path::PathBuf;

use pulldown_cmark::{Options, Parser};
use syntect::parsing::SyntaxSet;
use tracing::{event, instrument, Level};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::output::Output;
use mdcat::terminal::{TerminalProgram, TerminalSize};
#[cfg(feature = "render-image")]
use mdcat::ResourceAccess;
use mdcat::{Environment, Settings};

mod args;
mod output;

/// Read input for `filename`.
///
/// If `filename` is `-` read from standard input, otherwise try to open and
/// read the given file.
fn read_input<T: AsRef<str>>(filename: T) -> Result<(PathBuf, String)> {
    let cd = std::env::current_dir()?;
    let mut buffer = String::new();

    if filename.as_ref() == "-" {
        stdin().read_to_string(&mut buffer)?;
        Ok((cd, buffer))
    } else {
        let mut source = File::open(filename.as_ref())?;
        source.read_to_string(&mut buffer)?;
        let base_dir = cd
            .join(filename.as_ref())
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or(cd);
        Ok((base_dir, buffer))
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

fn main() {
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

    use crate::args::Args;
    use clap::Parser;

    let args = Args::parse().command;
    event!(target: "mdcat::main", Level::TRACE, ?args, "mdcat arguments");

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
        // On Windows 10 we need to enable ANSI term explicitly.
        #[cfg(windows)]
        {
            event!(target: "mdcat::main", Level::TRACE, "Enable ANSI support in windows terminal");
            ansi_term::enable_ansi_support().ok();
        }

        let size = TerminalSize::detect().unwrap_or_default();
        let columns = args.columns.unwrap_or(size.columns);

        let exit_code = match Output::new(args.paginate()) {
            Ok(mut output) => {
                let settings = Settings {
                    terminal_capabilities: terminal.capabilities(),
                    terminal_size: TerminalSize { columns, ..size },
                    #[cfg(feature = "render-image")]
                    resource_access: if args.local_only {
                        ResourceAccess::LocalOnly
                    } else {
                        ResourceAccess::RemoteAllowed
                    },
                    syntax_set: SyntaxSet::load_defaults_newlines(),
                };
                #[cfg(feature = "render-image")]
                event!(
                    target: "mdcat::main",
                    Level::TRACE,
                    ?settings.terminal_size,
                    ?settings.terminal_capabilities,
                    ?settings.resource_access,
                    "settings"
                );
                #[cfg(not(feature = "render-image"))]
                event!(
                    target: "mdcat::main",
                    Level::TRACE,
                    ?settings.terminal_size,
                    ?settings.terminal_capabilities,
                    "settings"
                );
                args.filenames
                    .iter()
                    .try_fold(0, |code, filename| {
                        process_file(filename, &settings, &mut output)
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

#[cfg(test)]
mod tests {
    use crate::args::Args;
    use clap::CommandFactory;

    #[test]
    fn verify_app() {
        Args::command().debug_assert();
    }
}
