// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use clap::{Parser, ValueHint};

fn after_help() -> &'static str {
    "See 'man 1 mdcat' for more information.

Report issues to <https://codeberg.org/flausch/mdcat>."
}

fn long_version() -> &'static str {
    concat!(
        env!("CARGO_PKG_VERSION"),
        "
Copyright (C) Sebastian Wiesner and contributors

This program is subject to the terms of the Mozilla Public License,
v. 2.0. If a copy of the MPL was not distributed with this file,
You can obtain one at http://mozilla.org/MPL/2.0/."
    )
}

#[derive(Debug, Parser)]
#[command(author, version, about, after_help = after_help(), long_version = long_version())]
pub struct Args {
    /// Files to read.  If - read from standard input instead.
    #[arg(default_value="-", value_hint = ValueHint::FilePath)]
    pub filenames: Vec<String>,
    /// Paginate the output of mdcat with a pager like less.  Default if invoked as mdless.
    #[arg(short, long)]
    pub paginate: bool,
    /// Do not paginate output. Default if invoked as mdcat.
    #[arg(short, long, overrides_with = "paginate")]
    pub no_pager: bool,
    /// Disable all colours and other styles.
    #[arg(short = 'c', long, aliases=["nocolour", "no-color", "nocolor"])]
    pub no_colour: bool,
    /// Maximum number of columns to use for output.
    #[arg(long)]
    pub columns: Option<usize>,
    /// Do not load remote resources like images.
    #[arg(short, long = "local")]
    pub local_only: bool,
    /// Exit immediately if any error occurs processing an input file.
    #[arg(long = "fail")]
    pub fail_fast: bool,
    /// Only detect the terminal type and exit.
    #[arg(long, hide = true)]
    pub detect_only: bool,
    /// Limit to standard ANSI formatting.
    #[arg(long, conflicts_with = "no_colour", hide = true)]
    pub ansi_only: bool,
}
