// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>
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

//! Kitty terminal extensions.
use std::io::{Error, ErrorKind, Write};
use std::str;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use thiserror::Error;
use tracing::{event, instrument, Level};

use crate::bufferline::BufferLines;
use crate::resources::image::*;
use crate::resources::MimeData;
use crate::terminal::size::{PixelSize, TerminalSize};

/// An error which occurred while rendering or writing an image with the Kitty image protocol.
#[derive(Debug, Error)]
pub enum KittyImageError {
    /// A general IO error.
    #[error("Failed to render kitty image: {0}")]
    IoError(#[from] std::io::Error),
    /// Processing a pixel image, e.g. for format conversion, failed
    #[error("Failed to process pixel image: {0}")]
    #[cfg(feature = "image-processing")]
    ImageError(#[from] image::ImageError),
}

impl From<KittyImageError> for std::io::Error {
    fn from(value: KittyImageError) -> Self {
        std::io::Error::new(ErrorKind::Other, value)
    }
}

/// Image data for the kitty graphics protocol.
///
/// See [Terminal graphics protocol][1] for a complete documentation.
///
/// [1]: https://sw.kovidgoyal.net/kitty/graphics-protocol/
enum KittyImageData {
    Png(Vec<u8>),
    #[cfg(feature = "image-processing")]
    Rgb(PixelSize, Vec<u8>),
    #[cfg(feature = "image-processing")]
    Rgba(PixelSize, Vec<u8>),
}

impl KittyImageData {
    /// Return the format code for this data for the `f` control data field.
    ///
    /// See the [Transferring pixel data][1] for reference.
    ///
    /// [1]: https://sw.kovidgoyal.net/kitty/graphics-protocol.html#transferring-pixel-data
    fn f_format_code(&self) -> &str {
        match self {
            KittyImageData::Png(_) => "100",
            #[cfg(feature = "image-processing")]
            KittyImageData::Rgb(_, _) => "24",
            #[cfg(feature = "image-processing")]
            KittyImageData::Rgba(_, _) => "32",
        }
    }

    /// Get the actual data.
    fn data(&self) -> &[u8] {
        match self {
            KittyImageData::Png(ref contents) => contents,
            #[cfg(feature = "image-processing")]
            KittyImageData::Rgb(_, ref contents) => contents,
            #[cfg(feature = "image-processing")]
            KittyImageData::Rgba(_, ref contents) => contents,
        }
    }

    /// Get the size of the image contained in this data.
    ///
    /// `Some` if the size is explicitly specified for this data, `None` otherwise, i.e. in PNG
    /// format).
    fn size(&self) -> Option<PixelSize> {
        match self {
            KittyImageData::Png(_) => None,
            #[cfg(feature = "image-processing")]
            KittyImageData::Rgb(size, _) => Some(*size),
            #[cfg(feature = "image-processing")]
            KittyImageData::Rgba(size, _) => Some(*size),
        }
    }

    /// The width of the image for the `s` control data field.
    fn s_width(&self) -> u32 {
        self.size().map_or(0, |s| s.x)
    }

    /// The height of the image for the `v` control data field.
    fn v_height(&self) -> u32 {
        self.size().map_or(0, |s| s.y)
    }
}

impl KittyImageData {
    fn write_to(&self, writer: &mut dyn Write) -> Result<(), Error> {
        let image_data = STANDARD.encode(self.data());
        let image_data_chunks = image_data.as_bytes().chunks(4096);
        let number_of_chunks = image_data_chunks.len();

        for (i, chunk_data) in image_data_chunks.enumerate() {
            let is_first_chunk = i == 0;
            // The value for the m field
            let m = if i < number_of_chunks - 1 { 1 } else { 0 };
            if is_first_chunk {
                // For the first chunk we must write the header for the image.
                //
                // a=T tells kitty that we transfer image data and want to show the image
                // immediately.
                //
                // t=d tells kitty that we transfer image data inline in the escape code.
                //
                // I=1 tells kitty that we want to treat every image as unique and not have kitty
                // reuse images.  At least wezterm requires this; otherwise past images disappear
                // because wezterm seems to assume that we're reusing some image ID.
                //
                // f tells kitty about the data format.
                //
                // s and v tell kitty about the size of our image.
                //
                // m tells kitty whether to expect more chunks or whether this is the last one.
                //
                // q=2 tells kitty never to respond to our image sequence; we're not reading these
                // responses anyway.
                //
                let f = self.f_format_code();
                let s = self.s_width();
                let v = self.v_height();
                write!(writer, "\x1b_Ga=T,t=d,I=1,f={f},s={s},v={v},m={m},q=2;")?;
            } else {
                // For follow up chunks we must not repeat the header, but only indicate whether we
                // expect a response and whether more data is to follow.
                write!(writer, "\x1b_Gm={m},q=2;")?;
            }
            writer.write_all(chunk_data)?;
            write!(writer, "\x1b\\")?;
        }

        Ok(())
    }
}

/// Provides access to printing images for kitty.
#[derive(Debug, Copy, Clone)]
pub struct KittyGraphicsProtocol;

impl KittyGraphicsProtocol {
    /// Render mime data obtained from `url` and wrap it in a `KittyImage`.
    ///
    /// This implementation processes the image to scale it to the given `terminal_size`, and
    /// supports various pixel image types, as well as SVG.
    #[cfg(feature = "image-processing")]
    fn render(
        self,
        mime_data: MimeData,
        terminal_size: TerminalSize,
    ) -> Result<KittyImageData, KittyImageError> {
        use image::ImageFormat;

        let image = if let Some("image/svg+xml") = mime_data.mime_type_essence() {
            event!(Level::DEBUG, "Rendering mime data to SVG");
            let png_data = crate::resources::svg::render_svg_to_png(&mime_data.data)?;
            image::load_from_memory_with_format(&png_data, ImageFormat::Png)?
        } else {
            let image_format = mime_data
                .mime_type_essence()
                .and_then(image::ImageFormat::from_mime_type);
            match image_format {
                // If we already have information about the mime type of the resource data let's
                // use it, and trust whoever provided it to have gotten it right.
                Some(format) => image::load_from_memory_with_format(&mime_data.data, format)?,
                // If we don't know the mime type of the original data have image guess the format.
                None => image::load_from_memory(&mime_data.data)?,
            }
        };

        match downsize_to_columns(&image, terminal_size) {
            Some(downsized_image) => {
                event!(
                    Level::DEBUG,
                    "Image scaled down to column limit, rendering RGB data"
                );
                Ok(self.render_as_rgb_or_rgba(downsized_image))
            }
            None if mime_data.mime_type_essence() == Some("image/png") => {
                event!(
                    Level::DEBUG,
                    "PNG image of appropriate size, rendering original image data"
                );
                Ok(self.render_as_png(mime_data.data))
            }
            None => {
                event!(Level::DEBUG, "Image not in PNG format, rendering RGB data");
                Ok(self.render_as_rgb_or_rgba(image))
            }
        }
    }

    /// Render mime data obtained from `url` and wrap it in a `KittyImageData`.
    ///
    /// This implementation does not support image processing, and only renders PNG images which
    /// kitty supports directly.
    #[cfg(not(feature = "image-processing"))]
    fn render(
        self,
        mime_data: MimeData,
        _terminal_size: TerminalSize,
    ) -> Result<KittyImageData, KittyImageError> {
        match mime_data.mime_type_essence() {
            Some("image/png") => Ok(self.render_as_png(mime_data.data)),
            _ => {
                event!(
                    Level::DEBUG,
                    "Only PNG images supported without image-processing feature, but got {:?}",
                    mime_data.mime_type
                );
                Err(std::io::Error::new(
                    ErrorKind::Unsupported,
                    format!(
                        "Image data with mime type {:?} not supported",
                        mime_data.mime_type
                    ),
                )
                .into())
            }
        }
    }

    /// Wrap the image bytes as PNG format in `KittyImage`.
    fn render_as_png(self, data: Vec<u8>) -> KittyImageData {
        KittyImageData::Png(data)
    }

    /// Render the image as RGB/RGBA format and wrap the image bytes in `KittyImage`.
    ///
    /// If the image size exceeds `terminal_size` in either dimension scale the
    /// image down to `terminal_size` (preserving aspect ratio).
    #[cfg(feature = "image-processing")]
    fn render_as_rgb_or_rgba(self, image: image::DynamicImage) -> KittyImageData {
        use image::{ColorType, GenericImageView};

        let size = PixelSize::from_xy(image.dimensions());
        match image.color() {
            ColorType::L8 | ColorType::Rgb8 | ColorType::L16 | ColorType::Rgb16 => {
                KittyImageData::Rgb(size, image.into_rgb8().into_raw())
            }
            // Default to RGBA format: We cannot match all colour types because
            // ColorType is marked non-exhaustive, but RGBA is a safe default
            // because we can convert any image to RGBA, at worth with additional
            // runtime costs.
            _ => KittyImageData::Rgba(size, image.into_rgba8().into_raw()),
        }
    }
}

/// Kitty's inline image protocol.
///
/// Kitty's escape sequence is like: Put the command key/value pairs together like "{}={}(,*)"
/// and write them along with the image bytes in 4096 bytes chunks to the stdout.
///
/// Its documentation gives the following python example:
///
/// ```python
/// import sys
/// from base64 import standard_b64encode
///
/// def serialize_gr_command(cmd, payload=None):
///   cmd = ','.join('{}={}'.format(k, v) for k, v in cmd.items())
///   ans = []
///   w = ans.append
///   w(b'\033_G'), w(cmd.encode('ascii'))
///   if payload:
///     w(b';')
///     w(payload)
///   w(b'\033\\')
///   return b''.join(ans)
///
/// def write_chunked(cmd, data):
///   cmd = {'a': 'T', 'f': 100}
///   data = standard_b64encode(data)
///   while data:
///     chunk, data = data[:4096], data[4096:]
///     m = 1 if data else 0
///     cmd['m'] = m
///     sys.stdout.buffer.write(serialize_gr_command(cmd, chunk))
///     sys.stdout.flush()
///     cmd.clear()
/// ```
///
/// See <https://sw.kovidgoyal.net/kitty/graphics-protocol.html#control-data-reference>
/// for reference.
impl InlineImageProtocol for KittyGraphicsProtocol {
    #[instrument(skip(self, writer, terminal_size))]
    fn write_inline_image(
        &self,
        writer: &mut BufferLines,
        resource_handler: &dyn crate::ResourceUrlHandler,
        url: &url::Url,
        terminal_size: crate::TerminalSize,
    ) -> std::io::Result<()> {
        let mime_data = resource_handler.read_resource(url)?;
        event!(
            Level::DEBUG,
            "Received data of mime type {:?}",
            mime_data.mime_type
        );
        let dynamic_image = image::load_from_memory(&mime_data.data).unwrap();
        let image = self.render(mime_data, terminal_size)?;
        image.write_to(writer)?;
        let cell = TerminalSize::detect().unwrap().cell.unwrap();
        writer.write_image((dynamic_image.height() / cell.y).try_into().unwrap());
        Ok(())
    }
}
