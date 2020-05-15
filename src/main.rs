// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

// 	http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![deny(warnings, clippy::all)]

//! Show CommonMark documents on TTYs.

use clap::{value_t, values_t};
use mdcat::Settings;
use pulldown_cmark::{Options, Parser};
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::{stdin, stdout};
use std::path::PathBuf;
use syntect::parsing::SyntaxSet;

use mdcat::{ResourceAccess, TerminalCapabilities, TerminalSize};

/// Read input for `filename`.
///
/// If `filename` is `-` read from standard input, otherwise try to open and
/// read the given file.
fn read_input<T: AsRef<str>>(filename: T) -> std::io::Result<(PathBuf, String)> {
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

fn process_file(
    filename: &str,
    settings: &Settings,
    dump_events: bool,
) -> Result<(), Box<dyn Error>> {
    let (base_dir, input) = read_input(filename)?;
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(&input, options);

    if dump_events {
        mdcat::dump_events(&mut std::io::stdout(), parser)
    } else {
        mdcat::push_tty(settings, &mut stdout(), &base_dir, parser)
    }
    .or_else(|error| match error.downcast_ref::<std::io::Error>() {
        Some(error) if error.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
        _ => Err(error),
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
    use clap::*;
    let size = TerminalSize::detect().unwrap_or_default();
    let columns = size.width.to_string();
    let app = app_from_crate!()
        // Merge flags and options w/ arguments together, include args in usage
        // string and show options in the order of declaration.  And also:
        // COLOURS <3
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::DontCollapseArgsInUsage)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::ColoredHelp)
        .after_help(
            "mdcat uses the standardized CommonMark dialect.  It formats
markdown documents for viewing in text terminals:

• Colours for headings, block quotes, etc
• Syntax highlighting for code blocks
• In some terminals: Inline images and inline links
• In iTerm2: Jump marks for headings

Copyright (C) 2018–2020 Sebastian Wiesner and contributors
Licensed under the Apache License, Version 2.0
Report issues to <https://github.com/lunaryorn/mdcat>.",
        )
        .arg(
            Arg::with_name("filenames")
                .multiple(true)
                .help("The file to read.  If - read from standard input instead")
                .default_value("-"),
        )
        .arg(
            Arg::with_name("no_colour")
                .short("c")
                .long("--no-colour")
                .aliases(&["nocolour", "no-color", "nocolor"])
                .help("Disable all colours and other styles."),
        )
        .arg(
            Arg::with_name("columns")
                .long("columns")
                .help("Maximum number of columns to use for output")
                .default_value(&columns),
        )
        .arg(
            Arg::with_name("local_only")
                .short("l")
                .long("local")
                .help("Do not load remote resources like images"),
        )
        .arg(
            Arg::with_name("dump_events")
                .long("dump-events")
                .help("Dump Markdown parser events and exit")
                .hidden(true),
        )
        .arg(
            Arg::with_name("fail_fast")
                .long("fail")
                .help("Exit immediately if any error occurs processing an input file"),
        )
        .arg(
            Arg::with_name("detect_only")
                .long("detect-only")
                .help("Only detect the terminal type and exit")
                .hidden(true),
        )
        .arg(
            Arg::with_name("ansi_only")
                .long("ansi-only")
                .help("Limit to standard ANSI formatting")
                .conflicts_with("no_colour")
                .hidden(true),
        );

    let matches = app.get_matches();
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
