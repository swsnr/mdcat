// Copyright 2018 Vinícius dos Santos Oliveira <vini.ipsmaker@gmail.com>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

// 	http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Support for terminology extensions.
//!
//! This module implements the terminology image protocol.

use crate::{
    bufferline::BufferLines, resources::InlineImageProtocol, terminal::TerminalSize,
    ResourceUrlHandler,
};
use std::io::{Result, Write};
use tracing::{event, Level};
use url::Url;

/// Whether we run in terminology or not.
pub fn is_terminology() -> bool {
    std::env::var("TERMINOLOGY")
        .map(|value| value.trim() == "1")
        .unwrap_or(false)
}

/// Provides access to printing images for Terminology.
#[derive(Debug, Copy, Clone)]
pub struct Terminology;

#[cfg(feature = "image-processing")]
fn get_image_dimensions(url: &Url) -> Option<(u32, u32)> {
    if url.scheme() == "file" {
        let path = url.to_file_path().ok()?;
        event!(
            Level::DEBUG,
            "Inspecting image dimensions at {}",
            path.display()
        );
        image::image_dimensions(&path)
            .map_err(|error| {
                event!(
                    Level::INFO,
                    "Failed to read image dimensions from {}: {}",
                    path.display(),
                    error
                );
                error
            })
            .ok()
    } else {
        None
    }
}

#[cfg(not(feature = "image-processing"))]
fn get_image_dimensions(_url: &Url) -> Option<(u32, u32)> {
    event!(
        Level::DEBUG,
        "Inspecting image dimensions not supported without image-processing feature"
    );
    None
}

/// The terminology image protocol

/// Terminology escape sequences work like this: Set texture to path, then draw a rectangle of a
/// chosen character which Terminology will then replace with the texture
///
/// The documentation gives the following C example:
///
/// ```c
/// printf("\033}is#5;3;%s\000"
///        "\033}ib\000#####\033}ie\000\n"
///        "\033}ib\000#####\033}ie\000\n"
///        "\033}ib\000#####\033}ie\000\n", "/tmp/icon.png");
/// ```
///
/// To determine the optimal size this implementation attempts to determine the image dimensions:
/// If the URL refers to a local path it'll read the image header from the path and extracts the
/// size information.
///
/// For remote URLs the implementation falls back to a rectangle covering half of the screen, and
/// does not attempt to determine more precise dimensions, to avoid downloading the resource twice
/// (once to determine the size, and then again when terminology renders the image).
impl InlineImageProtocol for Terminology {
    fn write_inline_image(
        &self,
        writer: &mut BufferLines,
        _resource_handler: &dyn ResourceUrlHandler,
        url: &Url,
        terminal_size: TerminalSize,
    ) -> Result<()> {
        let columns = terminal_size.columns;
        let lines = match get_image_dimensions(url) {
            Some((w, h)) => ((h as f64) * (columns / 2) as f64 / (w as f64)) as usize,
            None => terminal_size.rows as usize / 2,
        };

        let mut command = format!("\x1b}}ic#{};{};{}\x00", columns, lines, url.as_str());
        for _ in 0..lines {
            command.push_str("\x1b}ib\x00");
            for _ in 0..columns {
                command.push('#');
            }
            command.push_str("\x1b}ie\x00\n");
        }
        writer.write_all(command.as_bytes())?;
        /*
        TODO: how to do ?
        buffer_lines.push(BufferLine {
            occupied_lines: dynamic_image.height().try_into().unwrap(),
            offset: writer.len(),
        });
         */
        Ok(())
    }
}
