// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Access to resources referenced from markdown documents.

use std::fmt::Debug;
use std::io::{Error, ErrorKind, Result};

use mime::Mime;
use url::Url;

mod file;
mod http;

pub use file::FileResourceHandler;
pub use http::HttpResourceHandler;

/// Data of a resource with associated mime type.
#[derive(Debug, Clone)]
pub struct MimeData {
    /// The mime type if known.
    pub mime_type: Option<Mime>,
    /// The data.
    pub data: Vec<u8>,
}

/// Handle resource URLs.
pub trait ResourceUrlHandler: Send + Sync + Debug {
    /// Read a resource.
    ///
    /// Read data from the given `url`, and return the data and its associated mime type if known,
    /// or any IO error which occurred while reading from the resource.
    ///
    /// Alternatively, return an IO error with [`ErrorKind::Unsupported`] to indicate that the
    /// given `url` is not supported by this resource handler.  In this case a higher level
    /// resource handler may try a different handler.
    fn read_resource(&self, url: &Url) -> Result<MimeData>;
}

impl<'a, R: ResourceUrlHandler + ?Sized> ResourceUrlHandler for &'a R {
    fn read_resource(&self, url: &Url) -> Result<MimeData> {
        (*self).read_resource(url)
    }
}

/// Filter by URL scheme.
///
/// Return `Ok(url)` if `url` has the given `scheme`, otherwise return an IO error with error kind
/// [`ErrorKind::Unsupported`].
pub fn filter_schemes<'a>(schemes: &[&str], url: &'a Url) -> Result<&'a Url> {
    if schemes.contains(&url.scheme()) {
        Ok(url)
    } else {
        Err(Error::new(
            ErrorKind::Unsupported,
            format!("Unsupported scheme in {url}, expected one of {schemes:?}"),
        ))
    }
}

/// A resource handler which dispatches reading among a list of inner handlers.
#[derive(Debug)]
pub struct DispatchingResourceHandler {
    /// Inner handlers.
    handlers: Vec<Box<dyn ResourceUrlHandler>>,
}

impl DispatchingResourceHandler {
    /// Create a new handler wrapping all given `handlers`.
    pub fn new(handlers: Vec<Box<dyn ResourceUrlHandler>>) -> Self {
        Self { handlers }
    }
}

impl ResourceUrlHandler for DispatchingResourceHandler {
    /// Read from the given resource `url`.
    ///
    /// Try every inner handler one after another, while handlers return an
    /// [`ErrorKind::Unsupported`] IO error.  For any other error abort and return the error.
    ///
    /// Return the first different result, i.e. either data read or another error.
    fn read_resource(&self, url: &Url) -> Result<MimeData> {
        for handler in &self.handlers {
            match handler.read_resource(url) {
                Ok(data) => return Ok(data),
                Err(error) if error.kind() == ErrorKind::Unsupported => continue,
                Err(error) => return Err(error),
            }
        }
        Err(Error::new(
            ErrorKind::Unsupported,
            format!("No handler supported reading from {url}"),
        ))
    }
}

/// A resource handler which doesn't read anything.
#[derive(Debug, Clone, Copy)]
pub struct NoopResourceHandler;

impl ResourceUrlHandler for NoopResourceHandler {
    /// Always return an [`ErrorKind::Unsupported`] error.
    fn read_resource(&self, url: &Url) -> Result<MimeData> {
        Err(Error::new(
            ErrorKind::Unsupported,
            format!("Reading from resource {url} is not supported"),
        ))
    }
}
