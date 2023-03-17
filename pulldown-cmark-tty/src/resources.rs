// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Access to resources referenced from markdown documents.

use std::fs::File;
use std::io::prelude::*;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use mime::Mime;
use once_cell::sync::Lazy;
use reqwest::{
    blocking::{Client, ClientBuilder},
    header::CONTENT_TYPE,
};
use tracing::{event, Level};
use url::Url;

static CLIENT: Lazy<Option<Client>> = Lazy::new(|| {
    ClientBuilder::new()
        // Use env_proxy to extract proxy information from the environment; it's more flexible and
        // accurate than reqwest's built-in env proxy support.
        .proxy(reqwest::Proxy::custom(|url| {
            env_proxy::for_url(url).to_url()
        }))
        // Use somewhat aggressive timeouts to avoid blocking rendering for long; we have graceful
        // fallbacks since we have to support terminals without image capabilities anyways.
        .timeout(Some(Duration::from_millis(100)))
        .connect_timeout(Some(Duration::from_secs(1)))
        .referer(false)
        .user_agent(concat!("mdcat/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| {
            event!(
                Level::ERROR,
                ?error,
                "Failed to initialize HTTP client: {}",
                error
            );
            error
        })
        .ok()
});

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

fn fetch_http(url: &Url) -> Result<(Option<Mime>, Vec<u8>)> {
    let response = CLIENT
        .as_ref()
        .with_context(|| "HTTP client not available".to_owned())?
        .get(url.clone())
        .send()
        .with_context(|| format!("Failed to GET {url}"))?
        .error_for_status()?;

    let content_type = response.headers().get(CONTENT_TYPE);
    event!(
        Level::DEBUG,
        "Raw Content-Type of remote resource {}: {:?}",
        &url,
        &content_type
    );
    let mime_type = content_type
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<Mime>().ok());
    event!(
        Level::DEBUG,
        "Parsed Content-Type of remote resource {}: {:?}",
        &url,
        &mime_type
    );

    match response.content_length() {
        // The server gave us no content size so read until the end of the stream, but not more than our read limit.
        None => {
            // An educated guess for a good capacity,
            let mut buffer = Vec::with_capacity(1_048_576);
            // We read one byte more than our limit, so that we can differentiate between a regular EOF and one from hitting the limit.
            response
                .take(RESOURCE_READ_LIMIT + 1)
                .read_to_end(&mut buffer)
                .with_context(|| format!("Failed to read from {url}"))?;

            if RESOURCE_READ_LIMIT < buffer.len() as u64 {
                Err(anyhow!(
                    "Contents of {url} exceeded {RESOURCE_READ_LIMIT}, rejected",
                ))
            } else {
                Ok((mime_type, buffer))
            }
        }
        // If we've got a content-size use it to read exactly as many bytes as the server told us to do (within limits)
        Some(size) => {
            if RESOURCE_READ_LIMIT < size {
                Err(anyhow!(
                    "{url} reports size {size} which exceeds limit {RESOURCE_READ_LIMIT}, refusing to read",
                ))
            } else {
                let mut buffer = vec![0; size as usize];
                response
                    // Just to be on the safe side limit the read operation explicitly, just in case we got the above check wrong
                    .take(RESOURCE_READ_LIMIT)
                    .read_exact(buffer.as_mut_slice())
                    .with_context(|| format!("Failed to read from {url}"))?;

                Ok((mime_type, buffer))
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
pub fn read_url(url: &Url, access: ResourceAccess) -> Result<(Option<Mime>, Vec<u8>)> {
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
                File::open(&path)
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
                    let mime_type = mime_guess::from_path(&path).first();
                    if mime_type.is_none() {
                        event!(
                            Level::DEBUG,
                            "Failed to guess mime type from {}",
                            path.display()
                        );
                    }
                    Ok((mime_type, buffer))
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
    use std::{convert::Infallible, net::SocketAddr};

    use super::*;
    use hyper::{
        body::Bytes,
        service::{make_service_fn, service_fn},
        Body, Request, Response, Server,
    };
    use pretty_assertions::assert_eq;
    use tokio::{runtime::Runtime, sync::oneshot, task::JoinHandle};

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
    fn read_url_with_local_path_returns_content_type() {
        let cwd = Url::from_directory_path(std::env::current_dir().unwrap()).unwrap();

        let resource = cwd.join("../sample/rust-logo.svg").unwrap();
        let (mime_type, _) = read_url(&resource, ResourceAccess::LocalOnly).unwrap();
        assert_eq!(mime_type, Some(mime::IMAGE_SVG));

        let resource = cwd.join("../sample/rust-logo-128x128.png").unwrap();
        let (mime_type, _) = read_url(&resource, ResourceAccess::LocalOnly).unwrap();
        assert_eq!(mime_type, Some(mime::IMAGE_PNG));
    }

    #[test]
    fn read_url_with_http_url_fails_if_local_only_access() {
        let url = "https://github.com".parse::<url::Url>().unwrap();
        let error = read_url(&url, ResourceAccess::LocalOnly)
            .unwrap_err()
            .to_string();
        assert_eq!(
            error,
            "Access denied to URL https://github.com/ by policy LocalOnly"
        );
    }

    async fn mock_service(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let response = match req.uri().path() {
            "/png" => Response::builder()
                .status(200)
                .header("content-type", "image/png")
                .body(Body::from("would-be-a-png-image"))
                .unwrap(),
            "/empty-response" => Response::builder().status(201).body(Body::empty()).unwrap(),
            "/drip-very-slow" => {
                let (mut sender, body) = Body::channel();
                let size = 30_000;
                tokio::spawn(async move {
                    for chunk in std::iter::repeat(Bytes::copy_from_slice(&[b'x'; 1000])).take(size)
                    {
                        if sender
                            .send_data(Bytes::copy_from_slice(&chunk))
                            .await
                            .is_err()
                        {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                });
                Response::builder()
                    .status(200)
                    .header("content-length", size * 1000)
                    .header("content-type", "application/octet-stream")
                    .body(body)
                    .unwrap()
            }
            // Drip-feed a very very large response with a 1kb chunk per 250ms, with content-length
            // set appropriately.
            "/drip-large" => {
                let (mut sender, body) = Body::channel();
                let size = 150_000;
                tokio::spawn(async move {
                    for chunk in std::iter::repeat(Bytes::copy_from_slice(&[b'x'; 1000])).take(size)
                    {
                        if sender
                            .send_data(Bytes::copy_from_slice(&chunk))
                            .await
                            .is_err()
                        {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(250)).await;
                    }
                });
                Response::builder()
                    .status(200)
                    .header("content-length", size * 1000)
                    .header("content-type", "application/octet-stream")
                    .body(body)
                    .unwrap()
            }
            _ => Response::builder().status(404).body(Body::empty()).unwrap(),
        };
        Ok(response)
    }

    struct MockServer {
        runtime: Runtime,
        join_handle: Option<JoinHandle<()>>,
        terminate_server: Option<oneshot::Sender<()>>,
        local_addr: SocketAddr,
    }

    impl MockServer {
        fn start() -> Self {
            let addr: SocketAddr = "[::1]:0".parse().unwrap();
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap();
            let (terminate_sender, terminate_receiver) = oneshot::channel();
            let (addr_sender, addr_receiver) = oneshot::channel();
            let join_handle = runtime.spawn(async move {
                let make_service = make_service_fn(|_conn| async {
                    Ok::<_, Infallible>(service_fn(mock_service))
                });
                let server = Server::bind(&addr).serve(make_service);
                addr_sender.send(server.local_addr()).unwrap();
                let shutdown = server.with_graceful_shutdown(async {
                    terminate_receiver.await.ok().unwrap_or_default()
                });
                let _ = shutdown.await;
            });
            let local_addr = runtime.block_on(addr_receiver).unwrap();
            Self {
                join_handle: Some(join_handle),
                runtime,
                terminate_server: Some(terminate_sender),
                local_addr,
            }
        }

        fn url(&self) -> Url {
            let mut url = Url::parse("http://localhost").unwrap();
            url.set_port(Some(self.local_addr.port())).unwrap();
            url.set_ip_host(self.local_addr.ip()).unwrap();
            url
        }
    }

    impl Drop for MockServer {
        fn drop(&mut self) {
            if let Some(terminate) = self.terminate_server.take() {
                terminate.send(()).ok();
            }
            if let Some(handle) = self.join_handle.take() {
                self.runtime.block_on(handle).ok();
            }
        }
    }

    #[test]
    fn read_url_with_http_url_fails_when_status_404() {
        let server = MockServer::start();
        let url = server.url().join("really-not-there").unwrap();
        let result = read_url(&url, ResourceAccess::RemoteAllowed);
        assert!(result.is_err(), "Unexpected success: {result:?}");
        assert_eq!(
            format!("{:#}", result.unwrap_err()),
            format!("HTTP status client error (404 Not Found) for url ({url})")
        )
    }

    #[test]
    fn read_url_with_http_url_empty_response() {
        let server = MockServer::start();
        let url = server.url().join("/empty-response").unwrap();
        let result = read_url(&url, ResourceAccess::RemoteAllowed);
        assert!(result.is_ok(), "Unexpected error: {result:?}");
        let (mime_type, contents) = result.unwrap();
        assert_eq!(mime_type, None);
        assert!(contents.is_empty(), "Contents not empty: {contents:?}");
    }

    #[test]
    fn read_url_with_http_url_returns_content_type() {
        let server = MockServer::start();
        let url = server.url().join("/png").unwrap();
        let result = read_url(&url, ResourceAccess::RemoteAllowed);
        assert!(result.is_ok(), "Unexpected error: {result:?}");
        let (mime_type, contents) = result.unwrap();
        assert_eq!(mime_type, Some(mime::IMAGE_PNG));
        assert_eq!(
            std::str::from_utf8(&contents).unwrap(),
            "would-be-a-png-image"
        );
    }

    #[test]
    fn read_url_with_http_url_times_out_fast_on_slow_response() {
        let server = MockServer::start();
        // Read from a small but slow response: We wouldn't hit the size limit, but we should time
        // out aggressively.
        let url = server.url().join("/drip-very-slow").unwrap();
        let result = read_url(&url, ResourceAccess::RemoteAllowed);
        assert!(result.is_err(), "Unexpected success: {result:?}");
        let error = format!("{:#}", result.unwrap_err());
        assert_eq!(
            error,
            format!("Failed to read from {url}: error decoding response body: operation timed out: operation timed out")
        );
    }

    #[test]
    fn read_url_with_http_url_fails_fast_when_size_limit_is_exceeded() {
        let server = MockServer::start();
        // Read from a large and slow response: The response would take eternal to complete, but
        // since we abort right after checking the size limit, this test fails fast instead of
        // trying to read the entire request.
        let url = server.url().join("/drip-large").unwrap();
        let result = read_url(&url, ResourceAccess::RemoteAllowed);
        assert!(result.is_err(), "Unexpected success: {result:?}");
        let error = format!("{:#}", result.unwrap_err());
        assert_eq!(
            error,
            format!("{url} reports size 150000000 which exceeds limit 104857600, refusing to read")
        );
    }
}
