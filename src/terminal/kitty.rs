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

//! The kitty terminal.
//!
//! kitty is a fast, featureful, GPU based terminal emulator.
//!
//! See <https://sw.kovidgoyal.net/kitty/> for more information.

use super::magic;
use super::url::read_url;
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

/// Retrieve the terminal size by calling the command-line tool
/// > kitty +kitten icat --print-window-size
fn get_terminal_size_in_pixels() -> std::io::Result<(u32, u32)> {
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

        Ok((width, height))
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

pub struct KittyImages;

static ESCAPE_CODE_START: &str = "\x1b_G";
static ESCAPE_CODE_END: &str = "\x1b\\";

impl KittyImages {
    /// Output the KittyImage in the terminal
    pub fn write_inline_image<W: Write>(
        &self,
        writer: &mut W,
        image: KittyImage,
    ) -> Result<(), Error> {
        let mut cmd = KittyCommandPairs::new();
        cmd.append(KittyCommandAction::T);
        cmd.append(KittyTransmissionMedium::DIRECT);
        cmd.append(image.format);

        if let Some(dimensions) = image.dimensions {
            cmd.append(dimensions);
        }

        let image_data = base64::encode(&image.contents);
        let image_data_chunks = image_data.as_bytes().chunks(4096);
        let image_data_chunks_length = image_data_chunks.len();

        for (i, data) in image_data_chunks.enumerate() {
            cmd.append(KittyHasMoreChunkedData {
                flag: i < image_data_chunks_length - 1,
            });

            let cmd_str = cmd.as_string();

            let mut output = vec![];
            output.push(ESCAPE_CODE_START);
            output.push(&cmd_str);
            output.push(";");
            output.push(str::from_utf8(data)?);
            output.push(ESCAPE_CODE_END);

            writer.write(output.join("").as_bytes())?;
            writer.flush()?;

            cmd.clear();
        }

        Ok(())
    }

    /// Reads the image bytes into the KittyImage from the given URL
    /// It scales the image down if the image size is larger than the
    /// terminal window size.
    pub fn read_and_render(&self, url: &Url) -> Result<KittyImage, Error> {
        let contents = read_url(url)?;
        let mime = magic::detect_mime_type(&contents)?;
        let image = image::load_from_memory(&contents)?;
        let (terminal_width, terminal_height) = get_terminal_size_in_pixels()?;
        let (image_width, image_height) = image.dimensions();

        let needs_scaledown = image_width > terminal_width || image_height > terminal_height;

        if mime.type_() == mime::IMAGE && mime.subtype().as_str() == "png" && !needs_scaledown {
            self.render_as_png(contents)
        } else {
            self.render_as_rgb_or_rgba(image, (terminal_width, terminal_height))
        }
    }

    fn render_as_png(&self, contents: Vec<u8>) -> Result<KittyImage, Error> {
        Ok(KittyImage {
            contents: contents,
            format: KittyFormat::PNG,
            dimensions: None,
        })
    }

    fn render_as_rgb_or_rgba(
        &self,
        image: DynamicImage,
        terminal_size: (u32, u32),
    ) -> Result<KittyImage, Error> {
        let format = match image.color() {
            ColorType::RGB(_) => KittyFormat::RGB,
            _ => KittyFormat::RGBA,
        };

        let (image_width, image_height) = image.dimensions();
        let (available_width, available_height) = terminal_size;

        let image = if image_width > available_width || image_height > available_height {
            image.resize_to_fill(available_width, available_height, FilterType::Nearest)
        } else {
            image
        };

        Ok(KittyImage {
            contents: match format {
                KittyFormat::RGB => image.to_rgb().into_raw(),
                _ => image.to_rgba().into_raw(),
            },
            format: format,
            dimensions: Some(KittyImageDimension {
                width: image_width,
                height: image_height,
            }),
        })
    }
}

pub struct KittyImage {
    contents: Vec<u8>,
    format: KittyFormat,
    dimensions: Option<KittyImageDimension>,
}

pub struct KittyImageDimension {
    width: u32,
    height: u32,
}

/// The overall action this graphics command is performing.
/// I couldn't find the meaning of the possible values.
/// See https://sw.kovidgoyal.net/kitty/graphics-protocol.html#control-data-reference
pub enum KittyCommandAction {
    T,
}

/// The transmission medium
pub enum KittyTransmissionMedium {
    DIRECT,
}

/// The format (PNG, RGB or RGBA) of the image bytes to transmit
pub enum KittyFormat {
    PNG,
    RGB,
    RGBA,
}

pub struct KittyHasMoreChunkedData {
    flag: bool,
}

/// Manages the Kitty command pairs
pub struct KittyCommandPairs {
    cmd: Vec<(String, String)>,
}

impl KittyCommandPairs {
    fn new() -> Self {
        KittyCommandPairs { cmd: vec![] }
    }

    fn append<T>(&mut self, value: T) -> ()
    where
        T: KittyCommand<T>,
    {
        self.cmd.append(&mut T::cmd_pairs(value));
    }

    fn as_string(&self) -> String {
        self.cmd
            .iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect::<Vec<String>>()
            .join(",")
    }

    fn clear(&mut self) -> () {
        self.cmd.clear();
    }
}

pub trait KittyCommand<T> {
    fn cmd_pairs(value: T) -> Vec<(String, String)>;
}

/// See https://sw.kovidgoyal.net/kitty/graphics-protocol.html#transferring-pixel-data
impl KittyCommand<KittyFormat> for KittyFormat {
    fn cmd_pairs(value: KittyFormat) -> Vec<(String, String)> {
        vec![(
            "f".to_string(),
            match value {
                KittyFormat::PNG => "100",
                KittyFormat::RGBA => "32",
                KittyFormat::RGB => "24",
            }
            .to_string(),
        )]
    }
}

/// https://sw.kovidgoyal.net/kitty/graphics-protocol.html#control-data-reference
impl KittyCommand<KittyCommandAction> for KittyCommandAction {
    fn cmd_pairs(value: KittyCommandAction) -> Vec<(String, String)> {
        vec![(
            "a".to_string(),
            match value {
                KittyCommandAction::T => "T",
            }
            .to_string(),
        )]
    }
}

/// See https://sw.kovidgoyal.net/kitty/graphics-protocol.html#the-transmission-medium
impl KittyCommand<KittyTransmissionMedium> for KittyTransmissionMedium {
    fn cmd_pairs(value: KittyTransmissionMedium) -> Vec<(String, String)> {
        vec![(
            "t".to_string(),
            match value {
                KittyTransmissionMedium::DIRECT => "d",
            }
            .to_string(),
        )]
    }
}

/// See https://sw.kovidgoyal.net/kitty/graphics-protocol.html#rgb-and-rgba-data
/// See https://sw.kovidgoyal.net/kitty/graphics-protocol.html#control-data-reference
impl KittyCommand<KittyImageDimension> for KittyImageDimension {
    fn cmd_pairs(value: KittyImageDimension) -> Vec<(String, String)> {
        vec![
            ("s".to_string(), value.width.to_string()),
            ("v".to_string(), value.height.to_string()),
        ]
    }
}

impl KittyCommand<KittyHasMoreChunkedData> for KittyHasMoreChunkedData {
    fn cmd_pairs(value: KittyHasMoreChunkedData) -> Vec<(String, String)> {
        vec![(
            "m".to_string(),
            if value.flag {
                "1".to_string()
            } else {
                "0".to_string()
            },
        )]
    }
}
