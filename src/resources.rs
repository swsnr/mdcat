// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Access to resources referenced from markdown documents.

use anyhow::{anyhow, Context, Result};
use std::fs::File;
use std::io::prelude::*;
use ureq::AgentBuilder;
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

/// Whether `url` is readable as local file.
fn is_local(url: &Url) -> bool {
    url.scheme() == "file" && url.to_file_path().is_ok()
}

/// Read size limit for resources.
static RESOURCE_READ_LIMIT: u64 = 104_857_600;

fn fetch_http(url: &Url) -> Result<Vec<u8>> {
    let proxy = match env_proxy::for_url(url).to_string() {
        None => None,
        Some(proxy_url) => {
            let proxy = ureq::Proxy::new(&proxy_url).with_context(|| {
                format!("Failed to set proxy for URL {} to {}", url, &proxy_url)
            })?;
            Some(proxy)
        }
    };

    let response = proxy
        .map_or(AgentBuilder::new(), |proxy| {
            AgentBuilder::new().proxy(proxy)
        })
        .build()
        .request_url("GET", url)
        .set("User-Agent", concat!("mdcat/", env!("CARGO_PKG_VERSION")))
        .call()
        .with_context(|| format!("Failed to GET {url}"))?;

    match response.header("Content-Length") {
        // The server gave us no content size so read until the end of the stream, but not more than our read limit.
        None => {
            // An educated guess for a good capacity,
            let mut buffer = Vec::with_capacity(1_048_576);
            // We read one byte more than our limit, so that we can differentiate between a regular EOF and one from hitting the limit.
            response
                .into_reader()
                .take(RESOURCE_READ_LIMIT + 1)
                .read_to_end(&mut buffer)
                .with_context(|| format!("Failed to read from {url}"))?;

            if RESOURCE_READ_LIMIT < buffer.len() as u64 {
                Err(anyhow!(
                    "Contents of {url} exceeded {RESOURCE_READ_LIMIT}, rejected",
                ))
            } else {
                Ok(buffer)
            }
        }
        // If we've got a content-size use it to read exactly as many bytes as the server told us to do (within limits)
        Some(value) => {
            let size = value
                .parse::<usize>()
                .with_context(|| format!("{url} reports invalid content size {value}"))?;
            if RESOURCE_READ_LIMIT < size as u64 {
                Err(anyhow!(
                    "{url} reports size {size} which exceeds limit {RESOURCE_READ_LIMIT}, refusing to read",
                ))
            } else {
                let mut buffer = vec![0; size];
                response
                    .into_reader()
                    // Just to be on the safe side limit the read operation explicitly, just in case we got the above check wrong
                    .take(RESOURCE_READ_LIMIT)
                    .read_exact(buffer.as_mut_slice())
                    .with_context(|| format!("Failed to read from {url}"))?;

                Ok(buffer)
            }
        }
    }
}

/// Read the contents of the given `url` if supported.
///
/// Fail if
///
/// - we don’t know how to read from `url`, i.e. the scheme's not supported,
/// - if we fail to read from URL, or
/// - if contents of the URL exceed an internal hard-coded size limit (currently 100 MiB).
///
/// We currently support `file:` URLs which the underlying operation system can
/// read (local on UNIX, UNC paths on Windows), and HTTP(S) URLs.
pub fn read_url(url: &Url, access: ResourceAccess) -> Result<Vec<u8>> {
    if !access.permits(url) {
        return Err(anyhow!(
            "Access denied to URL {} by policy {:?}",
            url,
            access
        ));
    }
    match url.scheme() {
        "file" => match url.to_file_path() {
            Ok(path) => {
                let mut buffer = Vec::new();
                File::open(path)
                    .with_context(|| format!("Failed to open file at {url}"))?
                    // Read a byte more than the limit differentiate an expected EOF from hitting the limit
                    .take(RESOURCE_READ_LIMIT + 1)
                    .read_to_end(&mut buffer)
                    .with_context(|| format!("Failed to read from file at {url}"))?;

                if RESOURCE_READ_LIMIT < buffer.len() as u64 {
                    Err(anyhow!(
                        "Contents of {url} exceeded {RESOURCE_READ_LIMIT}, rejected",
                    ))
                } else {
                    Ok(buffer)
                }
            }
            Err(_) => Err(anyhow!("Cannot convert URL {url} to file path")),
        },
        "http" | "https" => fetch_http(url),
        _ => Err(anyhow!(
            "Cannot read from URL {url}, protocol not supported",
        )),
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
        let error = read_url(&url, ResourceAccess::LocalOnly)
            .unwrap_err()
            .to_string();
        assert_eq!(
            error,
            "Access denied to URL https://eu.httpbin.org/status/404 by policy LocalOnly"
        );
    }

    #[test]
    fn read_url_with_http_url_fails_when_status_404() {
        let url = "https://eu.httpbin.org/status/404"
            .parse::<url::Url>()
            .unwrap();
        let result = read_url(&url, ResourceAccess::RemoteAllowed);
        assert!(result.is_err(), "Unexpected success: {result:?}");
        let error = format!("{:#}", result.unwrap_err());
        assert_eq!(error, "Failed to GET https://eu.httpbin.org/status/404: https://eu.httpbin.org/status/404: status code 404")
    }

    #[test]
    fn read_url_with_http_url_returns_content_when_status_200() {
        let url = "https://eu.httpbin.org/bytes/100"
            .parse::<url::Url>()
            .unwrap();
        let result = read_url(&url, ResourceAccess::RemoteAllowed);
        assert!(result.is_ok(), "Unexpected error: {result:?}");
        assert_eq!(result.unwrap().len(), 100);
    }

    #[test]
    fn read_url_with_http_url_fails_when_size_limit_is_exceeded() {
        let url = "https://eu.httpbin.org/response-headers?content-length=115343400"
            .parse::<url::Url>()
            .unwrap();
        let result = read_url(&url, ResourceAccess::RemoteAllowed);
        assert!(result.is_err(), "Unexpected success: {result:?}");
        let error = format!("{:#}", result.unwrap_err());
        assert_eq!(error, "https://eu.httpbin.org/response-headers?content-length=115343400 reports size 115343400 which exceeds limit 104857600, refusing to read")
    }
}
