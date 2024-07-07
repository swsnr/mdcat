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
    #[instrument(level = "debug", skip(self), fields(url = %url))]
    fn read_resource(&self, url: &Url) -> Result<MimeData> {
        filter_schemes(&["http", "https"], url).and_then(|url| {
            event!(Level::DEBUG, "Requesting remote HTTP resource {}", url);
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
    use std::sync::OnceLock;
    use std::time::Duration;
    use std::{convert::Infallible, net::SocketAddr};

    use futures::StreamExt;
    use http_body_util::combinators::BoxBody;
    use http_body_util::{BodyExt, Empty, Full, StreamBody};
    use hyper::body::{Bytes, Frame, Incoming};
    use hyper::service::service_fn;
    use hyper::{Request, Response};
    use hyper_util::rt::TokioIo;
    use pulldown_cmark_mdcat::resources::ResourceUrlHandler;
    use reqwest::Url;
    use tokio::net::TcpListener;
    use tokio::runtime::Runtime;
    use tokio::sync::oneshot;
    use tokio::task::JoinHandle;
    use tokio_stream::wrappers::IntervalStream;

    use super::HttpResourceHandler;

    async fn mock_service(
        req: Request<Incoming>,
    ) -> Result<Response<BoxBody<Bytes, Infallible>>, Infallible> {
        let response = match req.uri().path() {
            "/png" => Response::builder()
                .status(200)
                .header("content-type", "image/png")
                .body(Full::new(Bytes::from("would-be-a-png-image")).boxed())
                .unwrap(),
            "/empty-response" => Response::builder()
                .status(201)
                .body(Empty::new().boxed())
                .unwrap(),
            "/drip-very-slow" => {
                let chunk_count = 30_000;
                const CHUNK_SIZE: usize = 1000;
                let data_stream =
                    IntervalStream::new(tokio::time::interval(Duration::from_secs(5)))
                        .map(|_| Bytes::copy_from_slice(&[b'x'; CHUNK_SIZE]))
                        .map(|chunk| Ok(Frame::data(chunk)))
                        .take(chunk_count);
                Response::builder()
                    .status(200)
                    .header("content-length", chunk_count * CHUNK_SIZE)
                    .header("content-type", "application/octet-stream")
                    .body(BoxBody::new(StreamBody::new(data_stream)))
                    .unwrap()
            }
            // Drip-feed a very very large response with a 1kb chunk per 250ms, with content-length
            // set appropriately.
            "/drip-large" => {
                let chunk_count = 150_000;
                const CHUNK_SIZE: usize = 1000;
                let data_stream =
                    IntervalStream::new(tokio::time::interval(Duration::from_millis(250)))
                        .map(|_| Bytes::copy_from_slice(&[b'x'; CHUNK_SIZE]))
                        .map(|chunk| Ok(Frame::data(chunk)))
                        .take(chunk_count);
                Response::builder()
                    .status(200)
                    .header("content-length", chunk_count * CHUNK_SIZE)
                    .header("content-type", "application/octet-stream")
                    .body(BoxBody::new(StreamBody::new(data_stream)))
                    .unwrap()
            }
            _ => Response::builder()
                .status(404)
                .body(Empty::new().boxed())
                .unwrap(),
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

            let (terminate_sender, mut terminate_receiver) = oneshot::channel();
            let (addr_sender, addr_receiver) = oneshot::channel();
            let join_handle = runtime.spawn(async move {
                let listener = TcpListener::bind(addr).await.unwrap();
                addr_sender.send(listener.local_addr().unwrap()).unwrap();
                loop {
                    tokio::select! {
                        Ok((stream, _)) = listener.accept() => {
                            let io = TokioIo::new(stream);
                            tokio::task::spawn(async move {
                                hyper::server::conn::http1::Builder::new()
                                    .serve_connection(io, service_fn(mock_service))
                                    .await
                                    .unwrap();
                            });
                        }
                        _ = (&mut terminate_receiver) => break,
                    };
                }
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

    static CLIENT: OnceLock<HttpResourceHandler> = OnceLock::new();

    fn client() -> &'static HttpResourceHandler {
        CLIENT.get_or_init(|| HttpResourceHandler::with_user_agent(52_428_800, "foo/0.0").unwrap())
    }

    #[test]
    fn read_url_with_http_url_fails_when_status_404() {
        let server = MockServer::start();
        let url = server.url().join("really-not-there").unwrap();
        let result = client().read_resource(&url);
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
        let result = client().read_resource(&url);
        assert!(result.is_ok(), "Unexpected error: {result:?}");
        let data = result.unwrap();
        assert_eq!(data.mime_type, None);
        assert!(data.data.is_empty(), "Data not empty: {:?}", data.data);
    }

    #[test]
    fn read_url_with_http_url_returns_content_type() {
        let server = MockServer::start();
        let url = server.url().join("/png").unwrap();
        let result = client().read_resource(&url);
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
        let result = client().read_resource(&url);
        assert!(result.is_err(), "Unexpected success: {result:?}");
        let error = format!("{:#}", result.unwrap_err());
        assert_eq!(
            error,
            format!("Failed to read from {url}: error decoding response body")
        );
    }

    #[test]
    fn read_url_with_http_url_fails_fast_when_size_limit_is_exceeded() {
        let server = MockServer::start();
        // Read from a large and slow response: The response would take eternal to complete, but
        // since we abort right after checking the size limit, this test fails fast instead of
        // trying to read the entire request.
        let url = server.url().join("/drip-large").unwrap();
        let result = client().read_resource(&url);
        assert!(result.is_err(), "Unexpected success: {result:?}");
        let error = format!("{:#}", result.unwrap_err());
        assert_eq!(
            error,
            format!("{url} reports size 150000000 which exceeds limit 52428800, refusing to read")
        );
    }
}
