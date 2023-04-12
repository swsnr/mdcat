// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::path::Path;

use syntect::highlighting::ThemeSet;

fn generate_theme_dump<P: AsRef<Path>>(out_dir: P) {
    let source_file = "src/render/Solarized (dark).tmTheme";
    println!("cargo:rerun-if-changed={source_file}");
    let theme = ThemeSet::get_theme(source_file).unwrap();
    syntect::dumps::dump_to_file(&theme, out_dir.as_ref().join("theme.dump")).unwrap();
}

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").expect("OUT_DIR not set");
    generate_theme_dump(out_dir);
}
