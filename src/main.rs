// Copyright 2018 Sebastian Wiesner <sebastian@swsnr.de>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

// 	http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate pulldown_cmark;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate syntect;
extern crate termion;

use std::io::prelude::*;
use std::io::stdin;
use std::fs::File;
use std::error::Error;
use structopt::StructOpt;
use pulldown_cmark::Parser;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;

mod tty;

#[derive(StructOpt, Debug)]
struct Arguments {
    #[structopt(long = "dump-events", help = "Dump events and exit .")] dump_events: bool,
    #[structopt(long = "columns", help = "Maximum number of columns.  Defaults to terminal width")]
    columns: Option<u16>,
    #[structopt(short = "l", long = "light",
                help = "Use Solarized Light for syntax highlighting (default dark).")]
    light: bool,
    #[structopt(help = "Input file.  If absent or - read from standard input")]
    filename: Option<String>,
}

fn read_input(filename: Option<String>) -> std::io::Result<String> {
    let mut buffer = String::new();
    match filename {
        None => stdin().read_to_string(&mut buffer)?,
        Some(ref filename) if filename == "-" => stdin().read_to_string(&mut buffer)?,
        Some(ref filename) => {
            let mut source = File::open(filename)?;
            source.read_to_string(&mut buffer)?
        }
    };
    Ok(buffer)
}

/// Get the number of columns for the terminal from `$COLUMNS`.
///
/// Return `None` if the variable is not set or does not contain a valid number.
fn terminal_columns_from_env() -> Option<u16> {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
}

/// Get the number of columns from the TTY device.
///
/// Return `None` if TTY access fails.
fn terminal_columns_from_tty() -> Option<u16> {
    termion::terminal_size().map(|size| size.0).ok()
}

/// Make a best effort to get the number of columns for the terminal.
///
/// Try to get the terminal size from the TTY device, or from the `$COLUMNS`
/// environment variable, and eventually assume a default of 80 for safety.
fn terminal_columns() -> u16 {
    terminal_columns_from_tty()
        .or_else(terminal_columns_from_env)
        .unwrap_or(80)
}

fn process_arguments(args: Arguments) -> Result<(), Box<Error>> {
    let input = read_input(args.filename)?;
    let parser = Parser::new(&input);

    if args.dump_events {
        tty::dump_events(&mut std::io::stdout(), parser)?;
        Ok(())
    } else {
        let columns = args.columns.unwrap_or_else(terminal_columns);
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let themes = ThemeSet::load_defaults().themes;
        let theme = themes
            .get(if args.light {
                "Solarized (light)"
            } else {
                "Solarized (dark)"
            })
            .unwrap();
        tty::push_tty(&mut std::io::stdout(), columns, parser, syntax_set, theme)?;
        Ok(())
    }
}

fn main() {
    match process_arguments(Arguments::from_args()) {
        Ok(_) => std::process::exit(0),
        Err(error) => {
            eprintln!("Error: {}", error);
            std::process::exit(1);
        }
    }
}
