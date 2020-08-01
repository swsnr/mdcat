// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use anyhow::{bail, Context, Result};
use std::io::Write;
use std::process::*;

/// The output for mdcat
pub enum Output {
    /// Standard output
    Stdout(std::io::Stdout),
    /// A pager
    Pager(Child),
}

impl Drop for Output {
    /// Drop the output.
    ///
    /// When outputting to a pager wait for the pager to exit.
    fn drop(&mut self) {
        if let Output::Pager(ref mut child) = *self {
            let _ = child.wait();
        }
    }
}

fn parse_env_var(name: &str) -> Result<Option<Vec<String>>> {
    use std::env::VarError;
    match std::env::var(name) {
        Ok(value) => shell_words::split(&value)
            .with_context(|| format!("Failed to parse value {} of {}", &value, &name))
            .map(Some),
        Err(VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(value)) => bail!("Value of {} not unicode: {:?}", name, value),
    }
}

fn pager_from_env() -> Result<Vec<String>> {
    match parse_env_var("MDCAT_PAGER")? {
        Some(command) => Ok(command),
        None => Ok(parse_env_var("PAGER")?.unwrap_or_else(|| vec!["less".into(), "-R".into()])),
    }
}

impl Output {
    /// Get the writer to write to the output.
    ///
    /// When outputting to a pager returns the stdin handle to the pager.
    pub fn writer(&mut self) -> &mut dyn Write {
        match self {
            Output::Stdout(handle) => handle,
            Output::Pager(child) => child.stdin.as_mut().unwrap(),
        }
    }

    /// Create a new output.
    ///
    /// If `try_paginate` is `true` try to output to a pager.  If stdout is not a TTY, that is, if
    /// there's no terminal to paginate on, print to stdout nonetheless.
    ///
    /// Take the pager command from `$MDCAT_PAGER` or `$PAGER`, and default to `less -R` if both are
    /// unset.  If any of the variables is empty use stdout (assuming that the user
    /// wanted to disabled paging explicitly).
    pub fn new(try_paginate: bool) -> Result<Output> {
        if try_paginate {
            match pager_from_env()?.split_first() {
                None => Ok(Output::Stdout(std::io::stdout())),
                Some((command, args)) => Command::new(command)
                    .args(args)
                    .stdin(Stdio::piped())
                    .spawn()
                    .with_context(|| {
                        format!("Failed to spawn pager {} with args {:?}", command, args)
                    })
                    .map(Output::Pager),
            }
        } else {
            Ok(Output::Stdout(std::io::stdout()))
        }
    }
}
