// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Access to resources referenced from markdown documents.

use fehler::{throw, throws};
use std::fs::File;
use std::io::prelude::*;
use thiserror::Error;
use url::Url;

/// What kind of resources mdcat may access when rendering.
///
/// This struct denotes whether mdcat shows inline images from remote URLs or
/// just from local files.
#[derive(PartialEq, Debug, Copy, Clone)]
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

#[derive(Debug, Error)]
pub enum Error {
    #[cfg(feature = "reqwest")]
    #[error("HTTP request failed: {source}")]
    HttpRequestError {
        #[from]
        source: reqwest::Error,
    },
    #[cfg(feature = "reqwest")]
    #[error("HTTP GET {url} returned unexpected status code: {status}")]
    UnexpectedHttpStatus {
        url: Url,
        status: reqwest::StatusCode,
    },

    #[cfg(not(feature = "reqwest"))]
    #[error("Failed to invoke 'curl': {source}")]
    CurlInvocationFailed { source: std::io::Error },
    #[cfg(not(feature = "reqwest"))]
    #[error("'curl {url}' failed with code {status}: {stderr:?}")]
    CurlFailed {
        url: Url,
        status: std::process::ExitStatus,
        stderr: String,
    },

    #[error("Failed to read resource: {source}")]
    ReadError {
        #[from]
        source: std::io::Error,
    },

    #[error("Access to {url} denied by policy {access:?}")]
    AccessDenied { url: Url, access: ResourceAccess },

    #[error("Cannot convert local {url} to file path")]
    NoFilePath { url: Url },
    #[error("Scheme of {url} not supported")]
    UnsupportedScheme { url: Url },
}

/// Whether `url` is readable as local file:.
fn is_local(url: &Url) -> bool {
    url.scheme() == "file" && url.to_file_path().is_ok()
}

#[cfg(feature = "reqwest")]
#[throws]
fn fetch_http(url: &Url) -> Vec<u8> {
    let mut response = reqwest::blocking::get(url.clone())?;
    if response.status().is_success() {
        let mut buffer = Vec::new();
        response.read_to_end(&mut buffer)?;
        buffer
    } else {
        throw!(Error::UnexpectedHttpStatus {
            url: url.clone(),
            status: response.status()
        });
    }
}

#[cfg(not(feature = "reqwest"))]
#[throws]
fn fetch_http(url: &Url) -> Vec<u8> {
    let output = std::process::Command::new("curl")
        .arg("-fsSL")
        .arg(url.to_string())
        .output()
        .map_err(|e| Error::CurlInvocationFailed { source: e })?;

    if output.status.success() {
        output.stdout
    } else {
        throw!(Error::CurlFailed {
            url: url.clone(),
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).into()
        });
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
pub fn read_url(url: &Url, access: ResourceAccess) -> Result<Vec<u8>, Error> {
    if !access.permits(url) {
        throw!(Error::AccessDenied {
            url: url.clone(),
            access,
        })
    }
    match url.scheme() {
        "file" => match url.to_file_path() {
            Ok(path) => {
                let mut buffer = Vec::new();
                File::open(path)?.read_to_end(&mut buffer)?;
                Ok(buffer)
            }
            Err(()) => Err(Error::NoFilePath { url: url.clone() }),
        },
        "http" | "https" => fetch_http(url),
        _ => Err(Error::UnsupportedScheme { url: url.clone() }),
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
    fn read_url_with_http_url_fails_if_local_only_access() {
        let url = "https://eu.httpbin.org/status/404"
            .parse::<url::Url>()
            .unwrap();
        let error = read_url(&url, ResourceAccess::LocalOnly).unwrap_err();
        match error {
            Error::AccessDenied {
                url: denied_url,
                access: denied_access,
            } => {
                assert_eq!(denied_url, url);
                assert_eq!(denied_access, ResourceAccess::LocalOnly);
            }
            e => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn read_url_with_http_url_fails_when_status_404() {
        let url = "https://eu.httpbin.org/status/404"
            .parse::<url::Url>()
            .unwrap();
        let result = read_url(&url, ResourceAccess::RemoteAllowed);
        assert!(result.is_err(), "Unexpected success: {:?}", result);
        let error = result.unwrap_err().to_string();
        assert!(
            error.contains("https://eu.httpbin.org/status/404"),
            "Error did not contain URL: {}",
            error
        );
    }

    #[test]
    fn read_url_with_http_url_returns_content_when_status_200() {
        let url = "https://eu.httpbin.org/bytes/100"
            .parse::<url::Url>()
            .unwrap();
        let result = read_url(&url, ResourceAccess::RemoteAllowed);
        assert!(result.is_ok(), "Unexpected error: {:?}", result);
        assert_eq!(result.unwrap().len(), 100);
    }
}
