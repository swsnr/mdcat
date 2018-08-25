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

use super::super::magic;
use super::super::svg;
use super::error::NotSupportedError;
use super::TerminalWrite;
use base64;
use failure::Error;
use mime;
use std::ffi::OsStr;
use std::io;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;

/// Write an iterm2 mark;
pub fn write_mark<W: Write + TerminalWrite>(writer: &mut W) -> io::Result<()> {
    writer.write_osc("1337;SetMark")
}

fn write_image_contents<W: Write + TerminalWrite, S: AsRef<OsStr>>(
    writer: &mut W,
    name: S,
    contents: &[u8],
) -> io::Result<()> {
    writer.write_osc(&format!(
        "1337;File=name={};inline=1:{}",
        base64::encode(name.as_ref().as_bytes()),
        base64::encode(contents)
    ))
}

/// Write an iterm2 inline image.
///
/// `name` is the file name of the image, and `contents` holds the image contents.
pub fn write_inline_image<W: Write + TerminalWrite, S: AsRef<OsStr>>(
    writer: &mut W,
    name: S,
    contents: &[u8],
) -> Result<(), Error> {
    let mime = magic::detect_mime_type(contents)?;
    match (mime.type_(), mime.subtype()) {
        (mime::IMAGE, mime::PNG)
        | (mime::IMAGE, mime::GIF)
        | (mime::IMAGE, mime::JPEG)
        | (mime::IMAGE, mime::BMP) => {
            write_image_contents(writer, name, contents).map_err(Into::into)
        }
        (mime::IMAGE, subtype) if subtype.as_str() == "svg" => {
            let png = svg::render_svg(contents)?;
            write_image_contents(writer, name, &png).map_err(Into::into)
        }
        _ => Err(NotSupportedError {
            what: "inline image with mimetype",
        }.into()),
    }
}
