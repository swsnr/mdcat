// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::ffi::OsString;
use std::path::Path;

mod mdcat {
    include!("src/bin/mdcat/args.rs");
}

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").expect("OUT_DIR not set");

    println!("cargo:rerun-if-changed=src/bin/mdcat/args.rs");
    gen_completions(&out_dir);
}

fn gen_completions(out_dir: &OsString) {
    use clap::*;
    let mut a = mdcat::app("80");

    let completions = Path::new(out_dir).join("completions");
    std::fs::create_dir_all(&completions).expect("Failed to create $OUT_DIR/completions");

    for shell in &[Shell::Bash, Shell::Zsh, Shell::Fish] {
        a.gen_completions("mdcat", *shell, &completions);
    }
}
