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

//! SVG "rendering" for mdcat.

use failure::Error;
use process::ProcessError;
use std::io::prelude::*;
use std::process::{Command, Stdio};

/// Render an SVG image to a PNG pixel graphic for display.
pub fn render_svg(svg: &[u8]) -> Result<Vec<u8>, Error> {
    render_svg_with_rsvg_convert(svg)
}

/// Render an SVG file with `rsvg-convert
fn render_svg_with_rsvg_convert(svg: &[u8]) -> Result<Vec<u8>, Error> {
    let mut process = Command::new("rsvg-convert")
        .arg("--dpi-x=72")
        .arg("--dpi-y=72")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    process
        .stdin
        .as_mut()
        .expect("Forgot to pipe stdin?")
        .write_all(svg)?;

    let output = process.wait_with_output()?;

    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(ProcessError {
            command: "file --brief --mime-type".to_string(),
            status: output.status,
            error: String::from_utf8_lossy(&output.stderr).into_owned(),
        }.into())
    }
}
