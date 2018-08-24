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

//! Terminal utilities.

use super::resources::{Resource, ResourceAccess};
use atty;
use failure::Error;
use std;
use std::io;
use std::io::prelude::*;
use term_size;

#[cfg(target_os = "macos")]
mod iterm2;
#[cfg(all(unix, not(target_os = "macos")))]
mod terminology;

/// The size of a text terminal.
#[derive(Debug, Copy, Clone)]
pub struct Size {
    /// The width of the terminal, in characters aka columns.
    pub width: usize,
    /// The height of the terminal, in lines.
    pub height: usize,
}

impl Default for Size {
    /// A good default size assumption for a terminal: 80x24.
    fn default() -> Size {
        Size {
            width: 80,
            height: 24,
        }
    }
}

impl Size {
    fn new(width: usize, height: usize) -> Size {
        Size { width, height }
    }

    /// Get terminal size from `$COLUMNS` and `$LINES`.
    fn from_env() -> Option<Size> {
        let columns = std::env::var("COLUMNS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok());
        let rows = std::env::var("LINES")
            .ok()
            .and_then(|value| value.parse::<usize>().ok());

        match (columns, rows) {
            (Some(columns), Some(rows)) => Some(Size::new(columns, rows)),
            _ => None,
        }
    }

    /// Detect the terminal size.
    ///
    /// Get the terminal size from the underlying TTY, and fallback to
    /// `$COLUMNS` and `$LINES`.
    pub fn detect() -> Option<Size> {
        term_size::dimensions()
            .map(|(w, h)| Size::new(w, h))
            .or_else(Size::from_env)
    }
}

/// An ANSI colour.
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum AnsiColour {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
}

/// An ANSI style to enable on a terminal.
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum AnsiStyle {
    Reset,
    Bold,
    Italic,
    NoItalic,
    Underline,
    Foreground(AnsiColour),
    DefaultForeground,
}

/// A trait to provide terminal escape code for any `Write` implementation
pub trait TerminalWrite {
    /// Write a OSC `command`.
    fn write_osc(&mut self, command: &str) -> io::Result<()>;

    /// Write a CSI SGR `command`.
    ///
    /// See <https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_sequences>.
    fn write_sgr(&mut self, command: &str) -> io::Result<()>;

    /// Write an ANSI style.
    fn write_style(&mut self, style: AnsiStyle) -> io::Result<()> {
        match style {
            AnsiStyle::Reset => self.write_sgr(""),
            AnsiStyle::Bold => self.write_sgr("1"),
            AnsiStyle::Italic => self.write_sgr("3"),
            AnsiStyle::Underline => self.write_sgr("4"),
            AnsiStyle::NoItalic => self.write_sgr("23"),
            AnsiStyle::Foreground(AnsiColour::Red) => self.write_sgr("31"),
            AnsiStyle::Foreground(AnsiColour::Green) => self.write_sgr("32"),
            AnsiStyle::Foreground(AnsiColour::Yellow) => self.write_sgr("33"),
            AnsiStyle::Foreground(AnsiColour::Blue) => self.write_sgr("34"),
            AnsiStyle::Foreground(AnsiColour::Magenta) => self.write_sgr("35"),
            AnsiStyle::Foreground(AnsiColour::Cyan) => self.write_sgr("36"),
            AnsiStyle::Foreground(AnsiColour::LightRed) => self.write_sgr("91"),
            AnsiStyle::Foreground(AnsiColour::LightGreen) => self.write_sgr("92"),
            AnsiStyle::Foreground(AnsiColour::LightYellow) => self.write_sgr("93"),
            AnsiStyle::Foreground(AnsiColour::LightBlue) => self.write_sgr("94"),
            AnsiStyle::Foreground(AnsiColour::LightMagenta) => self.write_sgr("95"),
            AnsiStyle::Foreground(AnsiColour::LightCyan) => self.write_sgr("96"),
            AnsiStyle::DefaultForeground => self.write_sgr("39"),
        }
    }
}

impl<T> TerminalWrite for T
where
    T: Write,
{
    fn write_osc(&mut self, command: &str) -> io::Result<()> {
        self.write_all(&[0x1b, 0x5d])?;
        self.write_all(command.as_bytes())?;
        self.write_all(&[0x07])?;
        Ok(())
    }

    fn write_sgr(&mut self, command: &str) -> io::Result<()> {
        self.write_all(&[0x1b, 0x5b])?;
        self.write_all(command.as_bytes())?;
        self.write_all(&[0x6d])?;
        Ok(())
    }
}

/// The terminal mdcat writes to.
///
/// The terminal denotes what features mdcat can use when rendering markdown.
/// Features range from nothing at all on dumb terminals, to basic ANSI styling,
/// to inline links and inline images in some select terminal emulators.
#[derive(Debug, Copy, Clone)]
pub enum Terminal {
    /// iTerm2.
    ///
    /// iTerm2 is a powerful macOS terminal emulator with many formatting
    /// features, including images and inline links.
    ///
    /// See <https://www.iterm2.com> for more information.
    ITerm2,
    /// Terminology.
    ///
    /// Terminology is a terminal written for the Enlightenment window manager
    /// using the powerful EFL libraries.  It supports inline links and inline
    /// images.
    ///
    /// See <http://terminolo.gy/> for more information.
    Terminology,
    /// A generic terminal based on a modern VTE version.
    ///
    /// VTE is Gnome library for terminal emulators.  It powers some notable
    /// terminal emulators like Gnome Terminal, and embedded terminals in
    /// applications like GEdit.
    ///
    /// VTE 0.50 or newer support inline links.  Older versions do not; we
    /// recognize these as `BasicAnsi`.
    GenericVTE50,
    /// A terminal which supports basic ANSI sequences.
    ///
    /// Most terminal emulators fall into this category.
    BasicAnsi,
    /// A dumb terminal that supports no formatting.
    ///
    /// With this terminal mdcat will render no special formatting at all. Use
    /// when piping to other programs or when the terminal does not even support
    /// ANSI codes.
    Dumb,
}

/// The terminal does not support something.
#[derive(Debug, Fail)]
#[fail(display = "This terminal does not support {}.", what)]
pub struct NotSupportedError {
    /// The operation which the terminal did not support.
    pub what: &'static str,
}

/// Ignore a `NotSupportedError`.
pub trait IgnoreNotSupported {
    /// The type after ignoring `NotSupportedError`.
    type R;

    /// Elide a `NotSupportedError` from this value.
    fn ignore_not_supported(self) -> Self::R;
}

impl IgnoreNotSupported for Error {
    type R = Result<(), Error>;

    fn ignore_not_supported(self) -> Self::R {
        self.downcast::<NotSupportedError>().map(|_| ())
    }
}

impl IgnoreNotSupported for Result<(), Error> {
    type R = Result<(), Error>;

    fn ignore_not_supported(self) -> Self::R {
        self.or_else(|err| err.ignore_not_supported())
    }
}

/// Get the version of VTE underlying this terminal.
///
/// Return `(minor, patch)` if this terminal uses VTE, otherwise return `None`.
fn get_vte_version() -> Option<(u8, u8)> {
    std::env::var("VTE_VERSION").ok().and_then(|value| {
        value[..2]
            .parse::<u8>()
            .into_iter()
            .zip(value[2..4].parse::<u8>())
            .next()
    })
}

impl Terminal {
    /// Detect the underlying terminal application.
    ///
    /// If stdout links to a TTY look at various pieces of information, in
    /// particular environment variables set by terminal emulators, to figure
    /// out what terminal emulator we run in.
    ///
    /// If stdout does not link to a TTY assume a `Dumb` terminal which cannot
    /// format anything.
    pub fn detect() -> Terminal {
        if atty::is(atty::Stream::Stdout) {
            if cfg!(feature = "iterm")
                && std::env::var("TERM_PROGRAM")
                    .map(|value| value.contains("iTerm.app"))
                    .unwrap_or(false)
            {
                Terminal::ITerm2
            } else if std::env::var("TERMINOLOGY")
                .map(|value| value.trim() == "1")
                .unwrap_or(false)
            {
                Terminal::Terminology
            } else {
                match get_vte_version() {
                    Some(version) if version >= (50, 0) => Terminal::GenericVTE50,
                    _ => Terminal::BasicAnsi,
                }
            }
        } else {
            Terminal::Dumb
        }
    }

    /// Whether this terminal supports colours.
    pub fn supports_colours(self) -> bool {
        if let Terminal::Dumb = self {
            false
        } else {
            true
        }
    }

    /// Set a style on this terminal.
    pub fn set_style<W: TerminalWrite>(
        self,
        writer: &mut W,
        style: AnsiStyle,
    ) -> Result<(), Error> {
        if self.supports_colours() {
            writer.write_style(style)?;
            Ok(())
        } else {
            Err(NotSupportedError {
                what: "ANSI styles",
            }.into())
        }
    }

    /// Write an inline image.
    ///
    /// Only supported for some terminal emulators.
    #[cfg(unix)]
    #[allow(unused_variables)]
    pub fn write_inline_image<W: io::Write>(
        self,
        writer: &mut W,
        max_size: Size,
        resource: &Resource,
        resource_access: ResourceAccess,
    ) -> Result<(), Error> {
        match self {
            #[cfg(target_os = "macos")]
            Terminal::ITerm2 => resource.read(resource_access).and_then(|contents| {
                iterm2::write_inline_image(writer, resource.as_str().as_ref(), &contents)
                    .map_err(Into::into)
            })?,
            #[cfg(all(unix, not(target_os = "macos")))]
            Terminal::Terminology => {
                terminology::write_inline_image(writer, max_size, resource, resource_access)?
            }
            _ => Err(NotSupportedError {
                what: "inline images",
            })?,
        }
        Ok(())
    }

    /// Write an inline image.
    ///
    /// Not supported on windows at all.
    #[cfg(windows)]
    pub fn write_inline_image<W: io::Write>(
        self,
        _writer: &mut W,
        _max_size: Size,
        _resource: &Resource,
        _resource_access: ResourceAccess,
    ) -> Result<(), Error> {
        Err(NotSupportedError {
            what: "inline images",
        })?
    }

    /// Set the link for the subsequent text.
    ///
    /// To stop a link write a link to an empty destination.
    pub fn set_link<W: io::Write>(self, writer: &mut W, destination: &str) -> Result<(), Error> {
        match self {
            #[cfg(target_os = "macos")]
            Terminal::ITerm2 => writer.write_osc(&format!("8;;{}", destination))?,
            Terminal::Terminology | Terminal::GenericVTE50 => {
                writer.write_osc(&format!("8;;{}", destination))?
            }
            _ => Err(NotSupportedError {
                what: "inline links",
            })?,
        }
        Ok(())
    }

    /// Set a mark in the current terminal.
    ///
    /// Only supported by iTerm2 currently.
    #[cfg(target_os = "macos")]
    pub fn set_mark<W: io::Write>(self, writer: &mut W) -> Result<(), Error> {
        if let Terminal::ITerm2 = self {
            iterm2::write_mark(writer)?
        } else {
            Err(NotSupportedError { what: "marks" })?
        };
        Ok(())
    }

    /// Set a mark in the current terminal.
    #[cfg(not(target_os = "macos"))]
    pub fn set_mark<W: io::Write>(self, _writer: &mut W) -> Result<(), Error> {
        Err(NotSupportedError { what: "marks" })?
    }
}
