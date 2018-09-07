// Copyright 2018 Sebastian Wiesner <sebastian@swsnr.de>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

// 	http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Iterm2 specific functions

use base64;
use failure::Error;
use mime;
use std;
use std::ffi::OsStr;
use std::io;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;

use super::super::magic;
use super::super::resources::{Resource, ResourceAccess};
use super::super::svg;
use super::ansi::AnsiTerminal;
use super::error::NotSupportedError;
use super::size::Size;
use super::write::Terminal;

/// The iTerm2 terminal.
///
/// iTerm2 is a powerful macOS terminal emulator with many formatting
/// features, including images and inline links.
///
/// See <https://www.iterm2.com> for more information.
pub struct ITerm2<W: Write> {
    ansi: AnsiTerminal<W>,
}

/// Whether we run inside iTerm2 or not.
pub fn is_iterm2() -> bool {
    std::env::var("TERM_PROGRAM")
        .map(|value| value.contains("iTerm.app"))
        .unwrap_or(false)
}

impl<W: Write> ITerm2<W> {
    /// Create an iTerm2 terminal over an underlying ANSI terminal.
    pub fn new(ansi: AnsiTerminal<W>) -> ITerm2<W> {
        ITerm2 { ansi }
    }

    fn write_image_contents<S: AsRef<OsStr>>(
        &mut self,
        name: S,
        contents: &[u8],
    ) -> io::Result<()> {
        self.ansi.write_osc(&format!(
            "1337;File=name={};inline=1:{}",
            base64::encode(name.as_ref().as_bytes()),
            base64::encode(contents)
        ))
    }

    /// Write an iterm2 inline image.
    ///
    /// `name` is the file name of the image, and `contents` holds the image
    /// contents.
    pub fn write_inline_image<S: AsRef<OsStr>>(
        &mut self,
        name: S,
        contents: &[u8],
    ) -> Result<(), Error> {
        let mime = magic::detect_mime_type(contents)?;
        match (mime.type_(), mime.subtype()) {
            (mime::IMAGE, mime::PNG)
            | (mime::IMAGE, mime::GIF)
            | (mime::IMAGE, mime::JPEG)
            | (mime::IMAGE, mime::BMP) => self
                .write_image_contents(name, contents)
                .map_err(Into::into),
            (mime::IMAGE, subtype) if subtype.as_str() == "svg" => {
                let png = svg::render_svg(contents)?;
                self.write_image_contents(name, &png).map_err(Into::into)
            }
            _ => Err(NotSupportedError {
                what: "inline image with mimetype",
            }.into()),
        }
    }
}

impl<W: Write> Terminal for ITerm2<W> {
    type TerminalWrite = W;

    fn name(&self) -> &'static str {
        "iTerm2"
    }

    fn write(&mut self) -> &mut W {
        self.ansi.write()
    }

    fn supports_styles(&self) -> bool {
        self.ansi.supports_styles()
    }

    fn set_link(&mut self, destination: &str) -> Result<(), Error> {
        self.ansi.write_osc(&format!("8;;{}", destination))?;
        Ok(())
    }

    fn set_mark(&mut self) -> Result<(), Error> {
        self.ansi.write_osc("1337;SetMark")?;
        Ok(())
    }

    fn write_inline_image(
        &mut self,
        _max_size: Size,
        resource: &Resource,
        access: ResourceAccess,
    ) -> Result<(), Error> {
        resource.read(access).and_then(|contents| {
            self.write_inline_image(resource.as_str().as_ref(), &contents)
                .map_err(Into::into)
        })?;
        Ok(())
    }
}
