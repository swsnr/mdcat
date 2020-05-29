// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Magic util functions for detecting image types.

use mime::{FromStrError, Mime};
use std::io::prelude::*;
use std::io::ErrorKind;
use std::process::*;
use std::str::Utf8Error;
use thiserror::Error;

/// Whether the given MIME type denotes an SVG image.
pub fn is_svg(mime: &Mime) -> bool {
    mime.type_() == mime::IMAGE && mime.subtype().as_str() == "svg"
}

/// Whether the given MIME type denotes a PNG image.
pub fn is_png(mime: &Mime) -> bool {
    *mime == mime::IMAGE_PNG
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to invoke 'file': {source}")]
    FileInvocationError {
        #[from]
        source: std::io::Error,
    },
    #[error("'file' failed with code {status}: {stderr:?}")]
    FileFailed { status: ExitStatus, stderr: String },
    #[error("'file' returned an invalid mime type: {source}")]
    InvalidMimeTypeError {
        #[from]
        source: FromStrError,
    },
    #[error("'file' returned invalid UTF-8: {source}")]
    InvalidOutputError {
        #[from]
        source: Utf8Error,
    },
}

/// Detect mime type with `file`.
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
        .write_all(buffer)
        .or_else(|error| match error.kind() {
            ErrorKind::BrokenPipe => Ok(()),
            _ => Err(error),
        })?;

    let output = process.wait_with_output()?;
    if output.status.success() {
        std::str::from_utf8(&output.stdout)?
            .trim()
            .parse()
            .map_err(Into::into)
    } else {
        Err(Error::FileFailed {
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn detect_mimetype_of_png_image() {
        let data = include_bytes!("../sample/rust-logo-128x128.png");
        let result = detect_mime_type(data);
        assert!(result.is_ok(), "Unexpected error: {:?}", result);
        assert_eq!(result.unwrap(), mime::IMAGE_PNG);
    }

    #[test]
    fn detect_mimetype_of_svg_image() {
        let data = include_bytes!("../sample/rust-logo.svg");
        let result = detect_mime_type(data);
        assert!(result.is_ok(), "Unexpected error: {:?}", result);
        let mime = result.unwrap();
        assert_eq!(mime.type_(), mime::IMAGE);
        assert_eq!(mime.subtype().as_str(), "svg");
    }

    #[test]
    fn detect_mimetype_of_magic_param_bytes_max_length() {
        let data = std::iter::repeat(b'\0')
            .take(1_048_576)
            .collect::<Vec<u8>>();
        let result = detect_mime_type(&data);
        assert!(result.is_ok(), "Unexpected error: {:?}", result);
    }

    #[test]
    fn detect_mimetype_of_larger_than_magic_param_bytes_max_length() {
        let data = std::iter::repeat(b'\0')
            .take(1_048_576 * 2)
            .collect::<Vec<u8>>();
        let result = detect_mime_type(&data);
        assert!(result.is_ok(), "Unexpected error: {:?}", result);
    }
}
