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
use tracing::{event, Level};

use crate::resources::{InlineImageProtocol, MimeData};
use crate::terminal::size::PixelSize;

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

/// The image format (PNG, RGB or RGBA) of the image bytes.
enum KittyFormat {
    Png,
    #[cfg(feature = "image-processing")]
    Rgb,
    #[cfg(feature = "image-processing")]
    Rgba,
}

impl KittyFormat {
    /// Return the control data value of the image format.
    /// See the [documentation] for the reference and explanation.
    ///
    /// [documentation]: https://sw.kovidgoyal.net/kitty/graphics-protocol.html#transferring-pixel-data
    fn control_data_value(&self) -> &str {
        match *self {
            KittyFormat::Png => "100",
            #[cfg(feature = "image-processing")]
            KittyFormat::Rgb => "24",
            #[cfg(feature = "image-processing")]
            KittyFormat::Rgba => "32",
        }
    }
}

/// Holds the image bytes with its image format and dimensions.
struct KittyImage {
    contents: Vec<u8>,
    format: KittyFormat,
    size: Option<PixelSize>,
}

impl KittyImage {
    fn write_to(&self, writer: &mut dyn Write) -> Result<(), Error> {
        let mut cmd_header: Vec<String> = vec![
            "a=T".into(),
            "t=d".into(),
            format!("f={}", self.format.control_data_value()),
        ];

        if let Some(size) = self.size {
            cmd_header.push(format!("s={}", size.x));
            cmd_header.push(format!("v={}", size.y));
        }

        let image_data = STANDARD.encode(&self.contents);
        let image_data_chunks = image_data.as_bytes().chunks(4096);
        let image_data_chunks_length = image_data_chunks.len();

        for (i, data) in image_data_chunks.enumerate() {
            if i < image_data_chunks_length - 1 {
                cmd_header.push("m=1".into());
            } else {
                cmd_header.push("m=0".into());
            }

            write!(writer, "\x1b_G{};", cmd_header.join(","))?;
            writer.write_all(data)?;
            write!(writer, "\x1b\\")?;
            // FIXME: Remove this? Why do we flush here?
            writer.flush()?;

            cmd_header.clear();
        }

        Ok(())
    }
}

/// Provides access to printing images for kitty.
#[derive(Debug, Copy, Clone)]
pub struct KittyImages;

impl KittyImages {
    /// Render mime data obtained from `url` and wrap it in a `KittyImage`.
    ///
    /// This implemention processes the image to scale it to the given `terminal_size`, and
    /// supports various pixel image types, as well as SVG.
    #[cfg(feature = "image-processing")]
    fn render(
        self,
        mime_data: MimeData,
        terminal_size: PixelSize,
    ) -> Result<KittyImage, KittyImageError> {
        use image::{GenericImageView, ImageFormat};
        let image = if let Some("image/svg+xml") = mime_data.mime_type_essence() {
            let png_data = crate::resources::svg::render_svg_to_png(&mime_data.data)?;
            image::load_from_memory_with_format(&png_data, ImageFormat::Png)?
        } else {
            let image_format = mime_data
                .mime_type
                .as_ref()
                .and_then(image::ImageFormat::from_mime_type);
            match image_format {
                // If we already have information about the mime type of the resource data let's
                // use it, and trust whoever provided it to have gotten it right.
                Some(format) => image::load_from_memory_with_format(&mime_data.data, format)?,
                // If we don't know the mime type of the original data have image guess the format.
                None => image::load_from_memory(&mime_data.data)?,
            }
        };

        if mime_data.mime_type == Some(mime::IMAGE_PNG)
            && PixelSize::from_xy(image.dimensions()) <= terminal_size
        {
            event!(
                Level::DEBUG,
                "PNG image of appropriate size, rendering original data"
            );
            // If we know that the original data is in PNG format and of sufficient size we can
            // discard the decoded image and instead render the original data directly.
            //
            // We kinda wasted the decoded image here (we do need it to check dimensions tho) but
            // at least we don't have to encode it again.
            Ok(self.render_as_png(mime_data.data))
        } else {
            event!(
                Level::DEBUG,
                "Image of other format or larger than terminal, rendering RGB data"
            );
            // The original data was not in PNG format, or we have to resize the image to terminal
            // dimensions, so we need to encode the RGB data of the decoded image explicitly.
            Ok(self.render_as_rgb_or_rgba(image, terminal_size))
        }
    }

    /// Render mime data obtained from `url` and wrap it in a `KittyImage`.
    ///
    /// This implementation does not support image processing, and only renders PNG images which
    /// kitty supports directly.
    #[cfg(not(feature = "image-processing"))]
    fn render(
        self,
        mime_data: MimeData,
        _terminal_size: PixelSize,
    ) -> Result<KittyImage, KittyImageError> {
        match mime_data.mime_type {
            Some(t) if t == mime::IMAGE_PNG => Ok(self.render_as_png(mime_data.data)),
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
    fn render_as_png(self, contents: Vec<u8>) -> KittyImage {
        KittyImage {
            contents,
            format: KittyFormat::Png,
            size: None,
        }
    }

    /// Render the image as RGB/RGBA format and wrap the image bytes in `KittyImage`.
    ///
    /// If the image size exceeds `terminal_size` in either dimension scale the
    /// image down to `terminal_size` (preserving aspect ratio).
    #[cfg(feature = "image-processing")]
    fn render_as_rgb_or_rgba(
        self,
        image: image::DynamicImage,
        terminal_size: PixelSize,
    ) -> KittyImage {
        use image::{ColorType, GenericImageView};
        let format = match image.color() {
            ColorType::L8 | ColorType::Rgb8 | ColorType::L16 | ColorType::Rgb16 => KittyFormat::Rgb,
            // Default to RGBA format: We cannot match all colour types because
            // ColorType is marked non-exhaustive, but RGBA is a safe default
            // because we can convert any image to RGBA, at worth with additional
            // runtime costs.
            _ => KittyFormat::Rgba,
        };

        let image = if PixelSize::from_xy(image.dimensions()) <= terminal_size {
            image
        } else {
            image.resize(
                terminal_size.x,
                terminal_size.y,
                image::imageops::FilterType::Nearest,
            )
        };

        let size = PixelSize::from_xy(image.dimensions());

        KittyImage {
            contents: match format {
                KittyFormat::Rgb => image.into_rgb8().into_raw(),
                _ => image.into_rgba8().into_raw(),
            },
            format,
            size: Some(size),
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
impl InlineImageProtocol for KittyImages {
    fn write_inline_image(
        &self,
        writer: &mut dyn Write,
        resource_handler: &dyn crate::ResourceUrlHandler,
        url: &url::Url,
        terminal_size: &crate::TerminalSize,
    ) -> std::io::Result<()> {
        let pixel_size = terminal_size.pixels.ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                "kitty did not report pixel size, cannot write image",
            )
        })?;
        let mime_data = resource_handler.read_resource(url)?;
        let image = self.render(mime_data, pixel_size)?;
        image.write_to(writer)
    }
}
