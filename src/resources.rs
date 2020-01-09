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

//! Access to resources referenced from markdown documents.

use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use url::Url;

/// What kind of resources mdcat may access when rendering.
///
/// This struct denotes whether mdcat shows inline images from remote URLs or
/// just from local files.
#[derive(Debug, Copy, Clone)]
pub enum ResourceAccess {
    /// Use only local files and prohibit remote resources.
    LocalOnly,
    /// Use local and remote resources alike.
    RemoteAllowed,
}

impl ResourceAccess {
    /// Whether the resource access permits access to the given `url`.
    pub fn permits(self, url: &Url) -> bool {
        match self {
            ResourceAccess::LocalOnly if is_local(url) => true,
            ResourceAccess::RemoteAllowed => true,
            _ => false,
        }
    }
}

/// Whether `url` is readable as local file:.
fn is_local(url: &Url) -> bool {
    url.scheme() == "file" && url.to_file_path().is_ok()
}

#[cfg(feature = "reqwest")]
fn fetch_http(url: &Url) -> Result<Vec<u8>, failure::Error> {
    let mut response = reqwest::blocking::get(url.clone())?;
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

#[cfg(not(feature = "reqwest"))]
fn fetch_http(url: &Url) -> Result<Vec<u8>, failure::Error> {
    let output = std::process::Command::new("curl")
        .arg("-fsSL")
        .arg(url.to_string())
        .output()?;

    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(Error::new(
            ErrorKind::Other,
            format!(
                "curl {} failed: {}",
                url,
                String::from_utf8_lossy(&output.stderr)
            ),
        )
        .into())
    }
}

/// Read the contents of the given `url` if supported.
///
/// Fail if we don’t know how to read from `url`, or if we fail to read from
/// URL.
///
/// We currently support `file:` URLs which the underlying operation system can
/// read (local on UNIX, UNC paths on Windows), and HTTP(S) URLs if enabled at
/// build system.
pub fn read_url(url: &Url) -> Result<Vec<u8>, failure::Error> {
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
        "http" | "https" => fetch_http(url),
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
    #[cfg(unix)]
    fn resource_access_permits_local_resource() {
        let resource = Url::parse("file:///foo/bar").unwrap();
        assert!(ResourceAccess::LocalOnly.permits(&resource));
        assert!(ResourceAccess::RemoteAllowed.permits(&resource));
    }

    #[test]
    #[cfg(unix)]
    fn resource_access_permits_remote_file_url() {
        let resource = Url::parse("file://example.com/foo/bar").unwrap();
        assert!(!ResourceAccess::LocalOnly.permits(&resource));
        assert!(ResourceAccess::RemoteAllowed.permits(&resource));
    }

    #[test]
    fn resource_access_permits_https_url() {
        let resource = Url::parse("https:///foo/bar").unwrap();
        assert!(!ResourceAccess::LocalOnly.permits(&resource));
        assert!(ResourceAccess::RemoteAllowed.permits(&resource));
    }

    #[test]
    fn read_url_with_http_url_fails_when_status_404() {
        let url = "https://eu.httpbin.org/status/404"
            .parse::<url::Url>()
            .unwrap();
        let result = read_url(&url);
        assert!(result.is_err(), "Unexpected success: {:?}", result);
        let error = result.unwrap_err().to_string();
        if cfg!(feature = "reqwest") {
            assert_eq!(
                error,
                "HTTP error status 404 Not Found by GET https://eu.httpbin.org/status/404"
            )
        } else {
            assert!(
                error.contains("curl https://eu.httpbin.org/status/404 failed:"),
                "Error did not contain expected string: {}",
                error
            );
            assert!(
                error.contains("404 NOT FOUND"),
                "Error did not contain expected string: {}",
                error
            );
        }
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
