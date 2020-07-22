// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![deny(warnings, clippy::all)]

//! Show CommonMark documents on TTYs.

use clap::{value_t, values_t};
use fehler::throws;
use mdcat::{Environment, Settings};
use std::fs::File;
use std::io::prelude::*;
use std::io::{stdin, stdout};
use std::io::{Error, Result};
use std::path::PathBuf;
use syntect::parsing::SyntaxSet;

use mdcat::{ResourceAccess, TerminalCapabilities, TerminalSize};

mod cli;

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

fn process_file(filename: &str, settings: &Settings, dump_events: bool) -> Result<()> {
    let (base_dir, input) = read_input(filename)?;
    let parser = mdcat::parse_ext_supported(&input);
    let env = Environment::for_local_directory(&base_dir)?;

    if dump_events {
        mdcat::dump_states(settings, &env, &mut stdout(), parser)
    } else {
        mdcat::push_tty(settings, &env, &mut stdout(), parser)
    }
    .or_else(|error| {
        if error.kind() == std::io::ErrorKind::BrokenPipe {
            Ok(())
        } else {
            Err(error)
        }
    })
}

/// Represent command line arguments.
struct Arguments {
    filenames: Vec<String>,
    terminal_capabilities: TerminalCapabilities,
    resource_access: ResourceAccess,
    columns: usize,
    dump_events: bool,
    detect_only: bool,
    fail_fast: bool,
}

impl Arguments {
    /// Create command line arguments from matches.
    fn from_matches(matches: &clap::ArgMatches<'_>) -> clap::Result<Self> {
        let terminal_capabilities = if matches.is_present("no_colour") {
            // If the user disabled colours assume a dumb terminal
            TerminalCapabilities::none()
        } else if matches.is_present("ansi_only") {
            TerminalCapabilities::ansi()
        } else {
            TerminalCapabilities::detect()
        };

        // On Windows 10 we need to enable ANSI term explicitly.
        #[cfg(windows)]
        {
            ansi_term::enable_ansi_support().ok();
        }

        let filenames = values_t!(matches, "filenames", String)?;
        let dump_events = matches.is_present("dump_events");
        let detect_only = matches.is_present("detect_only");
        let fail_fast = matches.is_present("fail_fast");
        let columns = value_t!(matches, "columns", usize)?;
        let resource_access = if matches.is_present("local_only") {
            ResourceAccess::LocalOnly
        } else {
            ResourceAccess::RemoteAllowed
        };

        Ok(Arguments {
            filenames,
            columns,
            resource_access,
            dump_events,
            detect_only,
            fail_fast,
            terminal_capabilities,
        })
    }
}

fn main() {
    let size = TerminalSize::detect().unwrap_or_default();
    let columns = size.width.to_string();

    let matches = cli::app(&columns).get_matches();
    let arguments = Arguments::from_matches(&matches).unwrap_or_else(|e| e.exit());

    if arguments.detect_only {
        println!("Terminal: {}", arguments.terminal_capabilities.name);
    } else {
        let Arguments {
            filenames,
            dump_events,
            fail_fast,
            terminal_capabilities,
            columns,
            resource_access,
            ..
        } = arguments;

        let settings = Settings {
            terminal_capabilities,
            terminal_size: TerminalSize {
                width: columns,
                ..size
            },
            resource_access,
            syntax_set: SyntaxSet::load_defaults_newlines(),
        };
        let exit_code = filenames
            .iter()
            .try_fold(0, |code, filename| {
                process_file(filename, &settings, dump_events)
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
            .unwrap_or(1);
        std::process::exit(exit_code);
    }
}
