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

//! Detect mime type with `file`.

use failure::Error;
use std::str;
use std::io::prelude::*;
use std::process::*;
use mime::Mime;

/// A process failed.
#[derive(Debug, Fail)]
#[fail(display = "Command {} failed with {}: {}", command, status, error)]
pub struct ProcessError {
    /// The command that failed.
    pub command: String,
    /// The exit code of the failed command.
    pub status: ExitStatus,
    /// The error output of the command.
    pub error: String,
}

pub fn detect_mime_type(buffer: &[u8]) -> Result<Mime, Error> {
    let mut process = Command::new("file")
        .arg("--brief")
        .arg("--mime-type")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    process
        .stdin
        .as_mut()
        .expect("Forgot to pipe stdin?")
        .write_all(buffer)?;

    let output = process.wait_with_output()?;
    if output.status.success() {
        str::from_utf8(&output.stdout)?
            .trim()
            .parse()
            .map_err(Into::into)
    } else {
        Err(ProcessError {
            command: "file --brief --mime-type".to_string(),
            status: output.status,
            error: String::from_utf8_lossy(&output.stderr).into_owned(),
        }.into())
    }
}
