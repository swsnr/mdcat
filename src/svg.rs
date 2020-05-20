// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! SVG "rendering" for mdcat.

use std::io::prelude::*;
use std::io::{Error, ErrorKind, Result};
use std::process::{Command, Stdio};

/// Render an SVG image to a PNG pixel graphic for display.
pub fn render_svg(svg: &[u8]) -> Result<Vec<u8>> {
    render_svg_with_rsvg_convert(svg)
}

/// Render an SVG file with `rsvg-convert`.
fn render_svg_with_rsvg_convert(svg: &[u8]) -> Result<Vec<u8>> {
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
        Err(Error::new(
            ErrorKind::Other,
            format!(
                "rsvg-convert failed with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ),
        ))
    }
}
