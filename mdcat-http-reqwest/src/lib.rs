// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! HTTP resource handler for [`pulldown_cmark_mdcat`].

#![deny(warnings, missing_docs, clippy::all)]
#![forbid(unsafe_code)]

use std::io::prelude::*;
use std::io::{Error, ErrorKind, Result};
use std::time::Duration;

use mime::Mime;
use pulldown_cmark_mdcat::resources::{filter_schemes, MimeData, ResourceUrlHandler};
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::header::CONTENT_TYPE;
use reqwest::Url;
use tracing::{event, instrument, Level};

/// A client for HTTP resources.
#[derive(Debug, Clone)]
pub struct HttpResourceHandler {
    read_limit: u64,
    http_client: Client,
}

/// Create a [`ClientBuilder`] preconfigured with default settings for mdcat.
pub fn build_default_client() -> ClientBuilder {
    ClientBuilder::new()
        // Use somewhat aggressive timeouts to avoid blocking rendering for long; we have graceful
        // fallbacks since we have to support terminals without image capabilities anyways.
        .timeout(Some(Duration::from_secs(1)))
        .connect_timeout(Some(Duration::from_secs(1)))
        .referer(false)
}

impl HttpResourceHandler {
    /// Create a new handler for HTTP resources.
    ///
    /// `read_limit` is the maximum amount of bytes to read from a HTTP resource before failing,
    /// and `http_client` is the underlying HTTP  client.
    pub fn new(read_limit: u64, http_client: Client) -> Self {
        Self {
            read_limit,
            http_client,
        }
    }

    /// Create a new HTTP resource handler..
    ///
    /// `read_limit` is the maximum amount of bytes to read from a HTTP resource, and and
    /// `user_agent` is the string to use as user agent for all requests.
    ///
    /// Create a HTTP client with some standard settings.
    pub fn with_user_agent(read_limit: u64, user_agent: &str) -> Result<Self> {
        build_default_client()
            .user_agent(user_agent)
            .build()
            .map_err(|err| Error::new(ErrorKind::Other, err))
            .map(|client| HttpResourceHandler::new(read_limit, client))
    }
}

impl ResourceUrlHandler for HttpResourceHandler {
    #[instrument(level = "debug", skip(self))]
    fn read_resource(&self, url: &Url) -> Result<MimeData> {
        filter_schemes(&["http", "https"], url).and_then(|url| {
            let response = self
                .http_client
                .get(url.clone())
                .send()
                .and_then(|r| r.error_for_status())
                .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;

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
                        .take(self.read_limit + 1)
                        .read_to_end(&mut buffer)
                        .map_err(|error| {
                            Error::new(error.kind(), format!("Failed to read from {url}: {error}"))
                        })?;

                    if self.read_limit < buffer.len() as u64 {
                        // TODO: Use ErrorKind::FileTooLarge once stabilized
                        Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("Contents of {url} exceeded {}, rejected", self.read_limit),
                        ))
                    } else {
                        Ok(MimeData {
                            mime_type,
                            data: buffer,
                        })
                    }
                }
                // If we've got a content-size use it to read exactly as many bytes as the server told us to do (within limits)
                Some(size) => {
                    if self.read_limit < size {
                        // TODO: Use ErrorKind::FileTooLarge once stabilized
                        Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("{url} reports size {size} which exceeds limit {}, refusing to read", self.read_limit)))
                    } else {
                        let mut buffer = vec![0; size as usize];
                        response
                            // Just to be on the safe side limit the read operation explicitly, just in case we got the above check wrong
                            .take(self.read_limit)
                            .read_exact(buffer.as_mut_slice())
                            .map_err(|error| {
                                Error::new(error.kind(), format!("Failed to read from {url}: {error}"))
                            })?;

                        Ok(MimeData {
                            mime_type,
                            data: buffer,
                        })
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::{convert::Infallible, net::SocketAddr};

    use hyper::body::Bytes;
    use hyper::service::{make_service_fn, service_fn};
    use hyper::{Body, Request, Response, Server};
    use once_cell::sync::Lazy;
    use pulldown_cmark_mdcat::resources::ResourceUrlHandler;
    use reqwest::Url;
    use tokio::runtime::Runtime;
    use tokio::sync::oneshot;
    use tokio::task::JoinHandle;

    use super::HttpResourceHandler;

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
                        tokio::time::sleep(Duration::from_secs(5)).await;
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

    static CLIENT: Lazy<HttpResourceHandler> =
        Lazy::new(|| HttpResourceHandler::with_user_agent(52_428_800, "foo/0.0").unwrap());

    #[test]
    fn read_url_with_http_url_fails_when_status_404() {
        let server = MockServer::start();
        let url = server.url().join("really-not-there").unwrap();
        let result = CLIENT.read_resource(&url);
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
        let result = CLIENT.read_resource(&url);
        assert!(result.is_ok(), "Unexpected error: {result:?}");
        let data = result.unwrap();
        assert_eq!(data.mime_type, None);
        assert!(data.data.is_empty(), "Data not empty: {:?}", data.data);
    }

    #[test]
    fn read_url_with_http_url_returns_content_type() {
        let server = MockServer::start();
        let url = server.url().join("/png").unwrap();
        let result = CLIENT.read_resource(&url);
        assert!(result.is_ok(), "Unexpected error: {result:?}");
        let data = result.unwrap();
        assert_eq!(data.mime_type, Some(mime::IMAGE_PNG));
        assert_eq!(
            std::str::from_utf8(&data.data).unwrap(),
            "would-be-a-png-image"
        );
    }

    #[test]
    fn read_url_with_http_url_times_out_fast_on_slow_response() {
        let server = MockServer::start();
        // Read from a small but slow response: We wouldn't hit the size limit, but we should time
        // out aggressively.
        let url = server.url().join("/drip-very-slow").unwrap();
        let result = CLIENT.read_resource(&url);
        assert!(result.is_err(), "Unexpected success: {result:?}");
        let error = format!("{:#}", result.unwrap_err());
        assert_eq!(
            error,
            format!("Failed to read from {url}: error decoding response body: operation timed out")
        );
    }

    #[test]
    fn read_url_with_http_url_fails_fast_when_size_limit_is_exceeded() {
        let server = MockServer::start();
        // Read from a large and slow response: The response would take eternal to complete, but
        // since we abort right after checking the size limit, this test fails fast instead of
        // trying to read the entire request.
        let url = server.url().join("/drip-large").unwrap();
        let result = CLIENT.read_resource(&url);
        assert!(result.is_err(), "Unexpected success: {result:?}");
        let error = format!("{:#}", result.unwrap_err());
        assert_eq!(
            error,
            format!("{url} reports size 150000000 which exceeds limit 52428800, refusing to read")
        );
    }
}
