// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use clap::*;

pub(crate) fn app(default_columns: String) -> Command {
    command!()
        .term_width(80)
        .after_help(
            "\
See 'man 1 mdcat' for more information.

Report issues to <https://codeberg.org/flausch/mdcat>.",
        )
        .arg(
            Arg::new("paginate")
                .short('p')
                .long("paginate")
                .action(ArgAction::SetTrue)
                .help("Paginate the output of mdcat with a pager like less."),
        )
        .arg(
            Arg::new("no_pager")
                .short('P')
                .long("no-pager")
                .action(ArgAction::SetTrue)
                .help("Do not page output.  Default if invoked as mdcat")
                .overrides_with("paginate"),
        )
        .arg(
            Arg::new("filenames")
                .action(ArgAction::Append)
                .help("The file to read.  If - read from standard input instead")
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .value_hint(ValueHint::FilePath)
                .default_value("-"),
        )
        .arg(
            Arg::new("no_colour")
                .short('c')
                .long("no-colour")
                .aliases(["nocolour", "no-color", "nocolor"])
                .action(ArgAction::SetTrue)
                .help("Disable all colours and other styles."),
        )
        .arg(
            Arg::new("columns")
                .long("columns")
                .value_parser(clap::value_parser!(usize))
                .help("Maximum number of columns to use for output")
                .default_value(default_columns),
        )
        .arg(
            Arg::new("local_only")
                .short('l')
                .long("local")
                .action(ArgAction::SetTrue)
                .help("Do not load remote resources like images"),
        )
        .arg(
            Arg::new("fail_fast")
                .long("fail")
                .action(ArgAction::SetTrue)
                .help("Exit immediately if any error occurs processing an input file"),
        )
        .arg(
            Arg::new("detect_only")
                .long("detect-only")
                .action(ArgAction::SetTrue)
                .help("Only detect the terminal type and exit")
                .hide(true),
        )
        .arg(
            Arg::new("ansi_only")
                .long("ansi-only")
                .help("Limit to standard ANSI formatting")
                .action(ArgAction::SetTrue)
                .conflicts_with("no_colour")
                .hide(true),
        )
}

#[test]
fn verify_app() {
    app("80".to_owned()).debug_assert();
}
