// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{cell::RefCell, time::Duration};

use curl::easy::{Easy2, Handler, WriteError};
use mime::Mime;
use pulldown_cmark_mdcat::{
    resources::{filter_schemes, MimeData},
    ResourceUrlHandler,
};
use tracing::{event, instrument, Level};
use url::Url;

/// Handle curl data by writing into a buffer.
#[derive(Debug, Clone, Default)]
pub struct CollectBuffer {
    read_limit: u64,
    buffer: Vec<u8>,
}

impl Handler for CollectBuffer {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        if self.read_limit < (self.buffer.len() + data.len()).try_into().unwrap() {
            // Do not handle data and tell curl that we didn't handle it;
            // this will make curl fail with a write error
            Ok(0)
        } else {
            self.buffer.extend_from_slice(data);
            Ok(data.len())
        }
    }
}

/// A [`curl`]-based resource handler for [`pulldown-cmark-mdcat`].
pub struct CurlResourceHandler {
    easy: RefCell<Easy2<CollectBuffer>>,
}

impl CurlResourceHandler {
    /// Create a new resource handler.
    ///
    /// `read_limit` is the maximum amount of data to be read from a resource.
    /// `useragent` is the value of the user agent header.
    pub fn create(read_limit: u64, useragent: &str) -> std::io::Result<Self> {
        let mut easy = Easy2::new(CollectBuffer {
            buffer: Vec::new(),
            read_limit,
        });
        // Use somewhat aggressive timeouts to avoid blocking rendering for long; we have graceful
        // fallbacks since we have to support terminals without image capabilities anyways.

        easy.timeout(Duration::from_secs(1))?;
        easy.connect_timeout(Duration::from_secs(1))?;
        easy.follow_location(true)?;
        easy.fail_on_error(true)?;
        easy.tcp_nodelay(true)?;
        easy.useragent(useragent)?;
        Ok(Self::new(easy))
    }

    /// Create a new resource handler.
    pub fn new(easy: Easy2<CollectBuffer>) -> Self {
        Self {
            easy: RefCell::new(easy),
        }
    }
}

impl ResourceUrlHandler for CurlResourceHandler {
    #[instrument(level = "debug", skip(self), fields(url = %url))]
    fn read_resource(
        &self,
        url: &Url,
    ) -> std::io::Result<pulldown_cmark_mdcat::resources::MimeData> {
        // See https://curl.se/docs/url-syntax.html for all schemas curl supports
        // We omit the more exotic ones :)
        filter_schemes(&["http", "https", "ftp", "ftps", "smb"], url).and_then(|url| {
            let mut easy = self.easy.borrow_mut();
            easy.url(url.as_str())?;
            easy.perform()?;

            let mime_type = easy.content_type()?.and_then(|content_type| {
                event!(
                    Level::DEBUG,
                    "Raw Content-Type of remote resource {}: {:?}",
                    &url,
                    content_type
                );
                content_type.parse::<Mime>().ok()
            });
            let data = easy.get_ref().buffer.clone();
            easy.get_mut().buffer.clear();
            Ok(MimeData { mime_type, data })
        })
    }
}
