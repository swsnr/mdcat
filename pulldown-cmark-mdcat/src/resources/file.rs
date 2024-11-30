// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! File resources.

use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;

use mime::Mime;
use tracing::{event, instrument, Level};
use url::Url;

use super::{filter_schemes, MimeData, ResourceUrlHandler};

/// A resource handler for `file:` URLs.
#[derive(Debug, Clone)]
pub struct FileResourceHandler {
    read_limit: u64,
}

impl FileResourceHandler {
    /// Create a resource handler for `file:` URLs.
    ///
    /// The resource handler does not read beyond `read_limit`.
    pub fn new(read_limit: u64) -> Self {
        Self { read_limit }
    }
}

/// Guess a mimetype, in so far as mdcat makes use of the mime type.
///
/// This function recognizes
///
/// - SVG images because mdcat needs to render SVG images explicitly, and
/// - PNG images because kitty can pass through PNG images in some cases.
///
/// It checks mime types exclusively by looking at the lowercase extension.
///
/// It ignores all other extensions and mime types and returns `None` in these cases.
fn guess_mimetype<P: AsRef<Path>>(path: P) -> Option<Mime> {
    path.as_ref()
        .extension()
        .map(|s| s.to_ascii_lowercase())
        .and_then(|s| match s.to_str() {
            Some("png") => Some(mime::IMAGE_PNG),
            Some("svg") => Some(mime::IMAGE_SVG),
            _ => None,
        })
}

impl ResourceUrlHandler for FileResourceHandler {
    #[instrument(level = "debug", skip(self))]
    fn read_resource(&self, url: &Url) -> Result<MimeData> {
        filter_schemes(&["file"], url).and_then(|url| {
            match url.to_file_path() {
                Ok(path) => {
                    event!(
                        Level::DEBUG,
                        "Reading from resource file {}",
                        path.display()
                    );
                    let mut buffer = Vec::new();
                    File::open(&path)?
                        // Read a byte more than the limit differentiate an expected EOF from hitting the limit
                        .take(self.read_limit + 1)
                        .read_to_end(&mut buffer)?;

                    if self.read_limit < buffer.len() as u64 {
                        Err(Error::new(
                            ErrorKind::FileTooLarge,
                            format!("Contents of {url} exceeded {} bytes", self.read_limit),
                        ))
                    } else {
                        let mime_type = guess_mimetype(&path);
                        if mime_type.is_none() {
                            event!(
                                Level::DEBUG,
                                "Failed to guess mime type from {}",
                                path.display()
                            );
                        }
                        Ok(MimeData {
                            mime_type,
                            data: buffer,
                        })
                    }
                }
                Err(_) => Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("Cannot convert URL {url} to file path"),
                )),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::resources::*;
    use similar_asserts::assert_eq;
    use url::Url;

    #[test]
    fn read_resource_returns_content_type() {
        let cwd = Url::from_directory_path(std::env::current_dir().unwrap()).unwrap();
        let client = FileResourceHandler::new(5_000_000);

        let resource = cwd.join("../sample/rust-logo.svg").unwrap();
        let mime_type = client.read_resource(&resource).unwrap().mime_type;
        assert_eq!(mime_type, Some(mime::IMAGE_SVG));

        let resource = cwd.join("../sample/rust-logo-128x128.png").unwrap();
        let mime_type = client.read_resource(&resource).unwrap().mime_type;
        assert_eq!(mime_type, Some(mime::IMAGE_PNG));
    }

    #[test]
    fn read_resource_obeys_size_limit() {
        let cwd = Url::from_directory_path(std::env::current_dir().unwrap()).unwrap();
        let client = FileResourceHandler { read_limit: 10 };

        let resource = cwd.join("../sample/rust-logo.svg").unwrap();
        let error = client.read_resource(&resource).unwrap_err().to_string();
        assert_eq!(error, format!("Contents of {resource} exceeded 10 bytes"));
    }

    #[test]
    fn read_resource_ignores_http() {
        let url = Url::parse("https://example.com").unwrap();

        let client = FileResourceHandler { read_limit: 10 };
        let error = client.read_resource(&url).unwrap_err().to_string();
        assert_eq!(
            error,
            "Unsupported scheme in https://example.com/, expected one of [\"file\"]"
        );
    }
}
