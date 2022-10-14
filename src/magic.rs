// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Magic util functions for detecting image types.

use anyhow::Result;
use anyhow::{anyhow, Context};
use mime::Mime;
use tracing::{event, Level};

/// Whether the given data is SVG data.
pub fn is_svg(buffer: &[u8]) -> bool {
    is_mimetype(buffer, &mime::IMAGE_SVG)
}

/// Whether the given data is a PNG image.
pub fn is_png(buffer: &[u8]) -> bool {
    is_mimetype(buffer, &mime::IMAGE_PNG)
}

fn is_mimetype(buffer: &[u8], mime: &Mime) -> bool {
    get_mimetype_for_buffer(buffer).map_or_else(
        |error| {
            event!(
                Level::WARN,
                ?error,
                "checking for mime type {} failed: {}",
                mime,
                error
            );
            false
        },
        |detected| detected == *mime,
    )
}

#[cfg(not(windows))]
fn get_mimetype_for_buffer(buffer: &[u8]) -> Result<Mime> {
    use std::io::prelude::*;
    use std::io::ErrorKind;
    use std::process::*;

    let mut process = Command::new("file")
        .arg("--brief")
        .arg("--mime-type")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| "Failed to spawn mime --brief --mime-type")?;

    process
        .stdin
        .as_mut()
        .expect("Forgot to pipe stdin?")
        .write_all(buffer)
        .or_else(|error| match error.kind() {
            ErrorKind::BrokenPipe => Ok(()),
            _ => Err(error),
        })?;

    let output = process
        .wait_with_output()
        .with_context(|| "Failed to read output from mime --brief --mime-type")?;
    if output.status.success() {
        let stdout = std::str::from_utf8(&output.stdout)
            .with_context(|| {
                format!(
                    "mime --brief --mime-type returned non-utf8: {:?}",
                    output.stdout
                )
            })?
            .trim();
        let detected_type = stdout
            .parse::<Mime>()
            .with_context(|| format!("Failed to parse mime type from output: {}", stdout))?;
        Ok(detected_type)
    } else {
        Err(anyhow!(
            "file --brief --mime-type failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

#[cfg(windows)]
fn get_mimetype_for_buffer(buffer: &[u8]) -> Result<Mime> {
    use std::ffi::c_void;
    use windows::core::PWSTR;

    let mut ret = PWSTR::null();
    unsafe {
        // SAFETY: We pass a proper null pointer as return value, and an initialized
        // buffer with corresponding length as data buffer.
        windows::Win32::System::Com::Urlmon::FindMimeFromData(
            None,
            None,
            Some(buffer.as_ptr() as *const c_void),
            buffer.len().try_into().unwrap(),
            None,
            0,
            &mut ret,
            0,
        )
        .with_context(|| {
            "Failed to determine mime type of buffer through windows API".to_string()
        })?;
    }

    let mime_raw = if !ret.is_null() {
        unsafe {
            // SAFETY: We explicitly check that ret is not null, and thus can safely convert its contents now.
            ret.to_string()
                .with_context(|| "Response from windows API contained invalid UTF-16".to_string())
        }
    } else {
        Err(anyhow!("Windows API did not return a mime type!"))
    }?;
    let detected_type = mime_raw
        .parse::<Mime>()
        .with_context(|| format!("Failed to parse mime type from windows API: {}", mime_raw))?;
    Ok(detected_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_mimetype_of_png_image() {
        let data = include_bytes!("../sample/rust-logo-128x128.png");
        assert!(is_png(data));
    }

    #[test]
    fn detect_mimetype_of_svg_image() {
        let data = include_bytes!("../sample/rust-logo.svg");
        assert!(is_svg(data));
    }

    #[test]
    fn detect_mimetype_of_magic_param_bytes_max_length() {
        let data = std::iter::repeat(b'\0')
            .take(1_048_576)
            .collect::<Vec<u8>>();
        assert!(!is_svg(&data));
    }

    #[test]
    fn detect_mimetype_of_larger_than_magic_param_bytes_max_length() {
        let data = std::iter::repeat(b'\0')
            .take(1_048_576 * 2)
            .collect::<Vec<u8>>();
        assert!(!is_svg(&data));
    }
}
