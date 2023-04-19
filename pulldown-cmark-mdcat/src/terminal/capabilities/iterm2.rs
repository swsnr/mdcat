// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Support for specific iTerm2 features.
//!
//! This module provides the iTerm2 marks and the iTerm2 image protocol.

use std::borrow::Cow;
use std::io::{self, Result, Write};

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use tracing::{event, instrument, Level};

use crate::resources::{svg, InlineImageProtocol};
use crate::terminal::osc::write_osc;
use crate::ResourceUrlHandler;

/// Iterm2 marks.
#[derive(Debug, Copy, Clone)]
pub struct ITerm2;

impl ITerm2 {
    /// Write an iterm2 mark command to the given `writer`.
    pub fn set_mark<W: Write>(self, writer: &mut W) -> io::Result<()> {
        write_osc(writer, "1337;SetMark")
    }
}

/// The iterm2 inline image protocol.
///
/// See <https://iterm2.com/documentation-images.html> for details; effectively we write a base64
/// encoded dump of the pixel data.
///
/// This implementation does **not** validate whether iterm2 actually supports the image type;
/// it writes data opportunistically and hopes iTerm2 copes.  For rare formats which are not
/// supported by macOS, this may yield false positives, i.e. this implementation might not return
/// an error even though iTerm2 cannot actually display the image.
impl InlineImageProtocol for ITerm2 {
    #[instrument(skip(self, writer, _terminal_size), fields(url = %url))]
    fn write_inline_image(
        &self,
        writer: &mut dyn Write,
        resource_handler: &dyn ResourceUrlHandler,
        url: &url::Url,
        _terminal_size: &crate::TerminalSize,
    ) -> Result<()> {
        let mime_data = resource_handler.read_resource(url)?;
        event!(
            Level::DEBUG,
            "Received data of mime type {:?}",
            mime_data.mime_type
        );
        let contents = if let Some("image/svg+xml") = mime_data.mime_type_essence() {
            event!(Level::DEBUG, "Rendering SVG from {}", url);
            Cow::Owned(svg::render_svg_to_png(&mime_data.data)?)
        } else {
            event!(Level::DEBUG, "Rendering mime data literally");
            Cow::Borrowed(&mime_data.data)
        };
        // Determine the local file name to use, by taking the last segment of the URL.
        // If the URL has no last segment do not tell iterm about a file name.
        let name = url.path_segments().and_then(|s| s.last());
        let data = STANDARD.encode(contents.as_ref());
        write_osc(
            writer,
            &name.map_or_else(
                || format!("1337;inline=1:{}", data),
                |name| {
                    format!(
                        "1337;File=name={};inline=1:{}",
                        STANDARD.encode(name.as_bytes()),
                        data
                    )
                },
            ),
        )
    }
}
