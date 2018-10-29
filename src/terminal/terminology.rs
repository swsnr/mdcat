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

//! [Terminology][] specific functions.
//!
//! [Terminology]: http://terminolo.gy

use super::TerminalSize;
use std::io::{Result, Write};
use url::Url;

/// Whether we run in terminology or not.
pub fn is_terminology() -> bool {
    std::env::var("TERMINOLOGY")
        .map(|value| value.trim() == "1")
        .unwrap_or(false)
}

/// Provides access to printing images for Terminology.
pub struct TerminologyImages;

impl TerminologyImages {
    /// Write an inline image for Terminology.
    pub fn write_inline_image<W: Write>(
        &mut self,
        writer: &mut W,
        max_size: TerminalSize,
        url: &Url,
    ) -> Result<()> {
        // Terminology escape sequence is like: set texture to path, then draw a
        // rectangle of chosen character to be replaced by the given texture.
        // Documentation gives the following C example:
        //
        //  printf("\033}is#5;3;%s\000"
        //         "\033}ib\000#####\033}ie\000\n"
        //         "\033}ib\000#####\033}ie\000\n"
        //         "\033}ib\000#####\033}ie\000\n", "/tmp/icon.png");
        //
        // We need to compute image proportion to draw the appropriate
        // rectangle. If we can't compute the image proportion (e.g. it's an
        // external URL), we fallback to a rectangle that is half of the screen.
        let columns = max_size.width;

        let lines = Some(url)
            .filter(|url| url.scheme() == "file")
            .and_then(|url| url.to_file_path().ok())
            .and_then(|path| immeta::load_from_file(path).ok())
            .map(|m| {
                let d = m.dimensions();
                let (w, h) = (f64::from(d.width), f64::from(d.height));
                // We divide by 2 because terminal cursor/font most likely has a
                // 1:2 proportion
                (h * (columns / 2) as f64 / w) as usize
            }).unwrap_or(max_size.height / 2);

        let mut command = format!("\x1b}}ic#{};{};{}\x00", columns, lines, url.as_str());
        for _ in 0..lines {
            command.push_str("\x1b}ib\x00");
            for _ in 0..columns {
                command.push('#');
            }
            command.push_str("\x1b}ie\x00\n");
        }
        writer.write_all(command.as_bytes())?;
        Ok(())
    }
}
