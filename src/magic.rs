// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

// 	http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Magic util functions for detecting image types.

use mime::Mime;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::process::*;

/// The value of MAGIC_PARAM_BYTES_MAX borrowed from libmagic.
/// See the man-page libmagic(3).
const MAGIC_PARAM_BYTES_MAX: usize = 1_048_576;

/// Whether the given MIME type denotes an SVG image.
pub fn is_svg(mime: &Mime) -> bool {
    mime.type_() == mime::IMAGE && mime.subtype().as_str() == "svg"
}

/// Detect mime type with `file`.
pub fn detect_mime_type(buffer: &[u8]) -> Result<Mime, Box<dyn std::error::Error>> {
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
        // extract only the first 1mb of interesting bytes; otherwise the
        // method `write_all` fails with the "Broken pipe (os error 32)"
        // error if the data being piped is large enough to be terminated
        // due to signal SIGPIPE as the command file reads maximum
        // 1048576 bytes from the input data.
        // Check for MAGIC_PARAM_BYTES_MAX in the man-page of libmagic(3).
        .write_all(&buffer[..std::cmp::min(buffer.len(), MAGIC_PARAM_BYTES_MAX)])?;

    let output = process.wait_with_output()?;
    if output.status.success() {
        std::str::from_utf8(&output.stdout)?
            .trim()
            .parse()
            .map_err(Into::into)
    } else {
        Err(Error::new(
            ErrorKind::Other,
            format!(
                "file --brief --mime-type failed with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ),
        )
        .into())
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
            .take(MAGIC_PARAM_BYTES_MAX)
            .collect::<Vec<u8>>();
        let result = detect_mime_type(&data);
        assert!(result.is_ok(), "Unexpected error: {:?}", result);
    }

    #[test]
    fn detect_mimetype_of_larger_than_magic_param_bytes_max_length() {
        let data = std::iter::repeat(b'\0')
            .take(MAGIC_PARAM_BYTES_MAX * 2)
            .collect::<Vec<u8>>();
        let result = detect_mime_type(&data);
        assert!(result.is_ok(), "Unexpected error: {:?}", result);
    }
}
