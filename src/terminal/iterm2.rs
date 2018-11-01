// Copyright 2018 Sebastian Wiesner <sebastian@swsnr.de>

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

use super::osc::write_osc;
use std::io::{Result, Write};

/// Whether we run inside iTerm2 or not.
pub fn is_iterm2() -> bool {
    std::env::var("TERM_PROGRAM")
        .map(|value| value.contains("iTerm.app"))
        .unwrap_or(false)
}

pub struct Marks;

impl Marks {
    /// Write an iterm2 mark command to the given `writer`.
    pub fn set_mark<W: Write>(&self, writer: &mut W) -> Result<()> {
        write_osc(writer, "1337;SetMark")
    }
}

// impl<W: Write> ITerm2<W> {
//     /// Create an iTerm2 terminal over an underlying ANSI terminal.
//     pub fn new(ansi: AnsiTerminal<W>) -> ITerm2<W> {
//         ITerm2 { ansi }
//     }

//     fn write_image_contents<S: AsRef<OsStr>>(
//         &mut self,
//         name: S,
//         contents: &[u8],
//     ) -> io::Result<()> {
//         self.ansi.write_osc(&format!(
//             "1337;File=name={};inline=1:{}",
//             base64::encode(name.as_ref().as_bytes()),
//             base64::encode(contents)
//         ))
//     }

//     /// Write an iterm2 inline image.
//     ///
//     /// `name` is the file name of the image, and `contents` holds the image
//     /// contents.
//     pub fn write_inline_image<S: AsRef<OsStr>>(
//         &mut self,
//         name: S,
//         contents: &[u8],
//     ) -> Result<(), Error> {
//         let mime = magic::detect_mime_type(contents)?;
//         match (mime.type_(), mime.subtype()) {
//             (mime::IMAGE, mime::PNG)
//             | (mime::IMAGE, mime::GIF)
//             | (mime::IMAGE, mime::JPEG)
//             | (mime::IMAGE, mime::BMP) => self
//                 .write_image_contents(name, contents)
//                 .map_err(Into::into),
//             (mime::IMAGE, subtype) if subtype.as_str() == "svg" => {
//                 let png = svg::render_svg(contents)?;
//                 self.write_image_contents(name, &png).map_err(Into::into)
//             }
//             _ => Err(NotSupportedError {
//                 what: "inline image with mimetype",
//             }.into()),
//         }
//     }
// }
