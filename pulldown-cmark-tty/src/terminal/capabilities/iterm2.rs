// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Support for specific iTerm2 features.
//!
//! This module provides the iTerm2 marks and the iTerm2 image protocol.

use std::io::{self, Write};

use anyhow::{Context, Result};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use url::Url;

use crate::resources::read_url;
use crate::svg;
use crate::terminal::osc::write_osc;
use crate::ResourceAccess;

/// Iterm2 marks.
#[derive(Debug, Copy, Clone)]
pub struct ITerm2Marks;

impl ITerm2Marks {
    /// Write an iterm2 mark command to the given `writer`.
    pub fn set_mark<W: Write>(self, writer: &mut W) -> io::Result<()> {
        write_osc(writer, "1337;SetMark")
    }
}

/// Iterm2 inline iamges.
#[derive(Debug, Copy, Clone)]
pub struct ITerm2Images;

impl ITerm2Images {
    /// Write an iterm2 inline image command to `writer`.
    ///
    /// `name` is the local file name and `contents` are the contents of the
    /// given file.
    pub fn write_inline_image<W: Write>(
        self,
        writer: &mut W,
        name: Option<&str>,
        contents: &[u8],
    ) -> io::Result<()> {
        write_osc(
            writer,
            &name.map_or_else(
                || format!("1337;inline=1:{}", STANDARD.encode(contents)),
                |name| {
                    format!(
                        "1337;File=name={};inline=1:{}",
                        STANDARD.encode(name.as_bytes()),
                        STANDARD.encode(contents)
                    )
                },
            ),
        )
    }

    /// Read `url` and render to an image if necessary.
    ///
    /// Render the binary content of the (rendered) image or an IO error if
    /// reading or rendering failed.
    pub fn read_and_render(self, url: &Url, access: ResourceAccess) -> Result<Vec<u8>> {
        let (mime_type, contents) = read_url(url, access)?;
        if mime_type == Some(mime::IMAGE_SVG) {
            svg::render_svg(&contents).with_context(|| format!("Failed to render SVG at URL {url}"))
        } else {
            Ok(contents)
        }
    }
}
