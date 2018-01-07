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
extern crate termion;

use structopt::StructOpt;
use std::io::prelude::*;
use std::io::stdin;
use std::fs::File;
use pulldown_cmark::Parser;

mod tty;

#[derive(StructOpt, Debug)]
struct Arguments {
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

fn process_arguments(args: Arguments) -> std::io::Result<()> {
    let input = read_input(args.filename)?;
    let parser = Parser::new(&input);
    tty::push_tty(&mut std::io::stdout(), parser)
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
