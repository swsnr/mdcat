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

use crate::bufferline::BufferLines;
use crate::resources::{svg, InlineImageProtocol};
use crate::terminal::osc::write_osc;
use crate::{ResourceUrlHandler, TerminalSize};

/// Iterm2 terminal protocols.
#[derive(Debug, Copy, Clone)]
pub struct ITerm2Protocol;

impl ITerm2Protocol {
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
impl InlineImageProtocol for ITerm2Protocol {
    #[instrument(skip(self, writer, _terminal_size), fields(url = %url))]
    fn write_inline_image(
        &self,
        writer: &mut BufferLines,
        resource_handler: &dyn ResourceUrlHandler,
        url: &url::Url,
        _terminal_size: crate::TerminalSize,
    ) -> Result<()> {
        let mime_data = resource_handler.read_resource(url)?;
        event!(
            Level::DEBUG,
            "Received data of mime type {:?}",
            mime_data.mime_type
        );

        // Determine the local file name to use, by taking the last segment of the URL.
        // If the URL has no last segment do not tell iterm about a file name.
        let name = url
            .path_segments()
            .and_then(|s| s.last())
            .map(Cow::Borrowed);
        let (name, contents) = if let Some("image/svg+xml") = mime_data.mime_type_essence() {
            event!(Level::DEBUG, "Rendering SVG from {}", url);
            (
                name.map(|n| {
                    let mut name = String::new();
                    name.push_str(&n);
                    name.push_str(".png");
                    Cow::Owned(name)
                }),
                Cow::Owned(svg::render_svg_to_png(&mime_data.data)?),
            )
        } else {
            event!(Level::DEBUG, "Rendering mime data literally");
            (name, Cow::Borrowed(&mime_data.data))
        };
        let data = STANDARD.encode(contents.as_ref());
        write_osc(
            writer,
            &name.map_or_else(
                || format!("1337;File=size={};inline=1:{}", contents.len(), data),
                |name| {
                    format!(
                        "1337;File=name={};size={};inline=1:{}",
                        STANDARD.encode(name.as_bytes()),
                        contents.len(),
                        data
                    )
                },
            ),
        )?;
        let dynamic_image = image::load_from_memory(&contents).unwrap();
        let cell = TerminalSize::detect().unwrap().cell.unwrap();
        writer.write_image((dynamic_image.height() / cell.y).try_into().unwrap());
        Ok(())
    }
}
