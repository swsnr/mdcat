// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::process::Command;

use syntect::highlighting::ThemeSet;

mod mdcat {
    include!("src/bin/mdcat/args.rs");
}

fn gen_completions<P: AsRef<Path>>(out_dir: P) -> Result<()> {
    use clap::CommandFactory;
    use clap_complete::*;

    let completions = out_dir.as_ref().join("completions");
    std::fs::create_dir_all(&completions).expect("Failed to create $OUT_DIR/completions");
    for program in ["mdcat", "mdless"] {
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
            let mut command = mdcat::Args::command();
            let subcommand = command.find_subcommand_mut(program).unwrap();
            generate_to(shell, subcommand, program, &completions)?;
        }
    }

    Ok(())
}

fn generate_theme_dump<P: AsRef<Path>>(out_dir: P) {
    let source_file = "src/render/Solarized (dark).tmTheme";
    println!("cargo:rerun-if-changed={source_file}");
    let theme = ThemeSet::get_theme(source_file).unwrap();
    syntect::dumps::dump_to_file(&theme, out_dir.as_ref().join("theme.dump")).unwrap();
}

fn build_manpage<P: AsRef<Path>>(out_dir: P) -> Result<()> {
    let target_file = out_dir.as_ref().join("mdcat.1");

    let mut command = Command::new("asciidoctor");
    command
        .args(["-b", "manpage", "-a", "reproducible"])
        .arg("-o")
        .arg(target_file)
        .arg("mdcat.1.adoc");
    let mut process = command.spawn().map_err(|err| match err.kind() {
        ErrorKind::NotFound => Error::new(
            ErrorKind::NotFound,
            "asciidoctor not found; please install asciidoctor to build the manpage!",
        ),
        _ => err,
    })?;
    let result = process.wait()?;

    if result.success() {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Other,
            format!("{:?} failed with exit code: {:?}", command, result.code()),
        ))
    }
}

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").expect("OUT_DIR not set");
    generate_theme_dump(&out_dir);

    println!("cargo:rerun-if-changed=src/bin/mdcat/args.rs");
    if let Err(error) = gen_completions(&out_dir) {
        println!("cargo:warning=Failed to build completions: {error}");
    }

    println!("cargo:rerun-if-changed=mdcat.1.adoc");
    if let Err(error) = build_manpage(&out_dir) {
        println!("cargo:warning=Failed to build manpage: {error}");
    }
}
