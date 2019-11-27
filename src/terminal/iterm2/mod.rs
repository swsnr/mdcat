// Copyright 2018-2019 Sebastian Wiesner <sebastian@swsnr.de>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//  http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The iTerm2 terminal.
//!
//! iTerm2 is a powerful macOS terminal emulator with many formatting
//! features, including images and inline links.
//!
//! See <https://www.iterm2.com> for more information.

use super::magic;
use super::osc::write_osc;
use super::resources::read_url;
use failure::Error;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use url::Url;

pub mod svg;

/// Whether we run inside iTerm2 or not.
pub fn is_iterm2() -> bool {
    std::env::var("TERM_PROGRAM")
        .map(|value| value.contains("iTerm.app"))
        .unwrap_or(false)
}

/// Iterm2 marks.
pub struct ITerm2Marks;

impl ITerm2Marks {
    /// Write an iterm2 mark command to the given `writer`.
    pub fn set_mark<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_osc(writer, "1337;SetMark")
    }
}

/// Iterm2 inline iamges.
pub struct ITerm2Images;

impl ITerm2Images {
    /// Write an iterm2 inline image command to `writer`.
    ///
    /// `name` is the local file name and `contents` are the contents of the
    /// given file.
    pub fn write_inline_image<W: Write, S: AsRef<OsStr>>(
        &self,
        writer: &mut W,
        name: S,
        contents: &[u8],
    ) -> io::Result<()> {
        write_osc(
            writer,
            &format!(
                "1337;File=name={};inline=1:{}",
                base64::encode(name.as_ref().as_bytes()),
                base64::encode(contents)
            ),
        )
    }

    /// Read `url` and render to an image if necessary.
    ///
    /// Render the binary content of the (rendered) image or an IO error if
    /// reading or rendering failed.
    pub fn read_and_render(&self, url: &Url) -> Result<Vec<u8>, Error> {
        let contents = read_url(&url)?;
        let mime = magic::detect_mime_type(&contents)?;
        if mime.type_() == mime::IMAGE && mime.subtype().as_str() == "svg" {
            svg::render_svg(&contents).map_err(Into::into)
        } else {
            Ok(contents)
        }
    }
}
