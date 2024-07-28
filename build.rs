// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::process::Command;

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

    println!("cargo:rerun-if-changed=mdcat.1.adoc");
    if let Err(error) = build_manpage(&out_dir) {
        println!("cargo:warning=Failed to build manpage: {error}");
    }
}
