// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Support for specific iTerm2 features.
//!
//! This module provides the iTerm2 marks and the iTerm2 image protocol.

use std::io::{self, Write};

#[cfg(feature = "render-image")]
use {
    anyhow::{Context, Result},
    base64::engine::general_purpose::STANDARD,
    base64::Engine,
    url::Url,
};

use crate::terminal::osc::write_osc;
#[cfg(feature = "render-image")]
use crate::{magic, resources::read_url, svg, ResourceAccess};

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
#[cfg(feature = "render-image")]
#[derive(Debug, Copy, Clone)]
pub struct ITerm2Images;

#[cfg(feature = "render-image")]
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
        let contents = read_url(url, access)?;
        if magic::is_svg(&contents) {
            svg::render_svg(&contents).with_context(|| format!("Failed to render SVG at URL {url}"))
        } else {
            Ok(contents)
        }
    }
}
