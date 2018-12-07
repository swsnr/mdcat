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

use mime::Mime;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::process::*;

pub fn detect_mime_type(buffer: &[u8]) -> Result<Mime, failure::Error> {
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

    #[test]
    fn detect_mimetype_of_png_image() {
        let data = include_bytes!("../../../sample/rust-logo-128x128.png");
        let result = detect_mime_type(data);
        assert!(result.is_ok(), "Unexpected error: {:?}", result);
        assert_eq!(result.unwrap(), mime::IMAGE_PNG);
    }

    #[test]
    fn detect_mimetype_of_svg_image() {
        let data = include_bytes!("../../../sample/rust-logo.svg");
        let result = detect_mime_type(data);
        assert!(result.is_ok(), "Unexpected error: {:?}", result);
        let mime = result.unwrap();
        assert_eq!(mime.type_(), mime::IMAGE);
        assert_eq!(mime.subtype().as_str(), "svg");
    }
}
