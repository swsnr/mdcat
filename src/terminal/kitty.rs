// Copyright 2019 Fabian Spillner <fabian.spillner@gmail.com>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//  http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The kitty terminal.
//!
//! kitty is a fast, featureful, GPU based terminal emulator.
//!
//! See <https://sw.kovidgoyal.net/kitty/> for more information.

use super::magic;
use super::resources::read_url;
use failure::Error;
use image::{ColorType, FilterType};
use image::{DynamicImage, GenericImageView};
use std::io::Write;
use std::process::{Command, Stdio};
use std::str;
use url::Url;

/// Whether we run in Kitty or not.
pub fn is_kitty() -> bool {
    std::env::var("TERM")
        .map(|value| value == "xterm-kitty")
        .unwrap_or(false)
}

/// Retrieve the terminal size in pixels by calling the command-line tool `kitty`.
///
///     kitty +kitten icat --print-window-size
///
/// We cannot use the terminal size information from Context.output.size, because
/// the size information are in columns / rows instead of pixel.
fn get_terminal_size() -> std::io::Result<KittyDimension> {
    use std::io::{Error, ErrorKind};

    let process = Command::new("kitty")
        .arg("+kitten")
        .arg("icat")
        .arg("--print-window-size")
        .stdout(Stdio::piped())
        .spawn()?;

    let output = process.wait_with_output()?;

    if output.status.success() {
        let terminal_size_str = std::str::from_utf8(&output.stdout).or(Err(Error::new(
            ErrorKind::Other,
            format!("The terminal size could not be read."),
        )))?;
        let terminal_size = terminal_size_str.split('x').collect::<Vec<&str>>();

        let (width, height) = (
            terminal_size[0].parse::<u32>().or(Err(Error::new(
                ErrorKind::Other,
                format!(
                    "The terminal width could not be parsed: {}",
                    terminal_size_str
                ),
            )))?,
            terminal_size[1].parse::<u32>().or(Err(Error::new(
                ErrorKind::Other,
                format!(
                    "The terminal height could not be parsed: {}",
                    terminal_size_str
                ),
            )))?,
        );

        Ok(KittyDimension { width, height })
    } else {
        Err(Error::new(
            ErrorKind::Other,
            format!(
                "kitty +kitten icat --print-window-size failed with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ),
        ))
    }
}

/// Provides access to printing images for kitty.
pub struct KittyImages;

impl KittyImages {
    /// Write an inline image for kitty.
    pub fn write_inline_image<W: Write>(
        &self,
        writer: &mut W,
        image: KittyImage,
    ) -> Result<(), Error> {
        // Kitty's escape sequence is like: Put the command key/value pairs together like "{}={}(,*)"
        // and write them along with the image bytes in 4096 bytes chunks to the stdout.
        // Documentation gives the following python example:
        //
        //  import sys
        //  from base64 import standard_b64encode
        //
        //  def serialize_gr_command(cmd, payload=None):
        //    cmd = ','.join('{}={}'.format(k, v) for k, v in cmd.items())
        //    ans = []
        //    w = ans.append
        //    w(b'\033_G'), w(cmd.encode('ascii'))
        //    if payload:
        //      w(b';')
        //      w(payload)
        //    w(b'\033\\')
        //    return b''.join(ans)
        //
        //  def write_chunked(cmd, data):
        //    cmd = {'a': 'T', 'f': 100}
        //    data = standard_b64encode(data)
        //    while data:
        //      chunk, data = data[:4096], data[4096:]
        //      m = 1 if data else 0
        //      cmd['m'] = m
        //      sys.stdout.buffer.write(serialize_gr_command(cmd, chunk))
        //      sys.stdout.flush()
        //      cmd.clear()
        //
        // Check at <https://sw.kovidgoyal.net/kitty/graphics-protocol.html#control-data-reference>
        // for the reference.
        let mut cmd_header: Vec<String> = vec![
            "a=T".into(),
            "t=d".into(),
            format!("f={}", image.format.control_data_value()),
        ];

        if let Some(dimension) = image.dimension {
            cmd_header.push(format!("s={}", dimension.width));
            cmd_header.push(format!("v={}", dimension.height));
        }

        let image_data = base64::encode(&image.contents);
        let image_data_chunks = image_data.as_bytes().chunks(4096);
        let image_data_chunks_length = image_data_chunks.len();

        for (i, data) in image_data_chunks.enumerate() {
            if i < image_data_chunks_length - 1 {
                cmd_header.push("m=1".into());
            } else {
                cmd_header.push("m=0".into());
            }

            let cmd = format!(
                "\x1b_G{};{}\x1b\\",
                cmd_header.join(","),
                str::from_utf8(data)?
            );
            writer.write(cmd.as_bytes())?;
            writer.flush()?;

            cmd_header.clear();
        }

        Ok(())
    }

    /// Read the image bytes from the given URL and wrap them in a `KittyImage`.
    /// It scales the image down, if the image size exceeds the terminal window size.
    pub fn read_and_render(&self, url: &Url) -> Result<KittyImage, Error> {
        let contents = read_url(url)?;
        let mime = magic::detect_mime_type(&contents)?;
        let image = image::load_from_memory(&contents)?;
        let terminal_size = get_terminal_size()?;
        let (image_width, image_height) = image.dimensions();

        let needs_scaledown =
            image_width > terminal_size.width || image_height > terminal_size.height;

        if mime.type_() == mime::IMAGE && mime.subtype().as_str() == "png" && !needs_scaledown {
            self.render_as_png(contents)
        } else {
            self.render_as_rgb_or_rgba(image, terminal_size)
        }
    }

    /// Wrap the image bytes as PNG format in `KittyImage`.
    fn render_as_png(&self, contents: Vec<u8>) -> Result<KittyImage, Error> {
        Ok(KittyImage {
            contents,
            format: KittyFormat::PNG,
            dimension: None,
        })
    }

    /// Render the image as RGB/RGBA format and wrap the image bytes in `KittyImage`.
    /// It scales the image down if its size exceeds the terminal size.
    fn render_as_rgb_or_rgba(
        &self,
        image: DynamicImage,
        terminal_size: KittyDimension,
    ) -> Result<KittyImage, Error> {
        let format = match image.color() {
            ColorType::RGB(_) => KittyFormat::RGB,
            _ => KittyFormat::RGBA,
        };

        let (image_width, image_height) = image.dimensions();

        let image = if image_width > terminal_size.width || image_height > terminal_size.height {
            image.resize_to_fill(
                terminal_size.width,
                terminal_size.height,
                FilterType::Nearest,
            )
        } else {
            image
        };

        Ok(KittyImage {
            contents: match format {
                KittyFormat::RGB => image.to_rgb().into_raw(),
                _ => image.to_rgba().into_raw(),
            },
            format,
            dimension: Some(KittyDimension {
                width: image_width,
                height: image_height,
            }),
        })
    }
}

/// Holds the image bytes with its image format and dimensions.
pub struct KittyImage {
    contents: Vec<u8>,
    format: KittyFormat,
    dimension: Option<KittyDimension>,
}

/// The image format (PNG, RGB or RGBA) of the image bytes.
enum KittyFormat {
    PNG,
    RGB,
    RGBA,
}

impl KittyFormat {
    /// Return the control data value of the image format.
    /// See the [documentation] for the reference and explanation.
    ///
    /// [documentation]: https://sw.kovidgoyal.net/kitty/graphics-protocol.html#transferring-pixel-data
    fn control_data_value(&self) -> &str {
        match *self {
            KittyFormat::PNG => "100",
            KittyFormat::RGB => "24",
            KittyFormat::RGBA => "32",
        }
    }
}

/// The dimension encapsulate the width and height in the pixel unit.
struct KittyDimension {
    width: u32,
    height: u32,
}
