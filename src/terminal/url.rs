// Copyright 2018-2019 Sebastian Wiesner <sebastian@swsnr.de>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

// 	http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Url util functions for downloading content from url.

use failure::Error;
use url::Url;

/// Read the contents of the given `url` if supported.
///
/// Fail if we don’t know how to read from `url`, or if we fail to read from
/// URL.
///
/// We currently support `file:` URLs which the underlying operation system can
/// read (local on UNIX, UNC paths on Windows), and HTTP(S) URLs if enabled at
/// build system.
pub fn read_url(url: &Url) -> Result<Vec<u8>, Error> {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::{Error, ErrorKind};

    match url.scheme() {
        "file" => match url.to_file_path() {
            Ok(path) => {
                let mut buffer = Vec::new();
                File::open(path)?.read_to_end(&mut buffer)?;
                Ok(buffer)
            }
            Err(_) => Err(Error::new(
                ErrorKind::InvalidInput,
                format!("Remote file: URL {} not supported", url),
            )
            .into()),
        },
        #[cfg(feature = "remote_resources")]
        "http" | "https" => {
            let mut response = reqwest::get(url.clone())?;
            if response.status().is_success() {
                let mut buffer = Vec::new();
                response.read_to_end(&mut buffer)?;
                Ok(buffer)
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    format!("HTTP error status {} by GET {}", response.status(), url),
                )
                .into())
            }
        }
        _ => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Protocol of URL {} not supported", url),
        )
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn read_url_with_http_url_fails_when_status_404() {
        let url = "https://eu.httpbin.org/status/404"
            .parse::<url::Url>()
            .unwrap();
        let result = read_url(&url);
        assert!(result.is_err(), "Unexpected success: {:?}", result);
        let error = result.unwrap_err().to_string();
        assert_eq!(
            error,
            "HTTP error status 404 Not Found by GET https://eu.httpbin.org/status/404"
        )
    }

    #[test]
    fn read_url_with_http_url_returns_content_when_status_200() {
        let url = "https://eu.httpbin.org/bytes/100"
            .parse::<url::Url>()
            .unwrap();
        let result = read_url(&url);
        assert!(result.is_ok(), "Unexpected error: {:?}", result);
        assert_eq!(result.unwrap().len(), 100);
    }
}
