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

use termion;
use std;
use std::io;
use std::fmt;
use term_size;
use super::resources::Resource;

mod iterm2;

/// Terminal size.
#[derive(Debug, Copy, Clone)]
pub struct Size {
    /// The terminal width, in characters.
    pub width: usize,
    /// The terminal height, in lines.
    pub height: usize,
}

impl Default for Size {
    /// A default terminal size: 80x24
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

/// The terminal we use.
#[derive(Debug, Copy, Clone)]
pub enum Terminal {
    /// iTerm2.
    ///
    /// iTerm2 is a powerful macOS terminal emulator with many formatting
    /// features, including images and inline links.
    ///
    /// See <https://www.iterm2.com> for more information.
    ITerm2,
    /// A generic terminal based on a modern VTE version.
    ///
    /// VTE is Gnome library for terminal emulators.  It powers some notable
    /// terminal emulators like Gnome Terminal, and embedded terminals in
    /// applications like GEdit.
    ///
    /// We require 0.50 or newer; these versions support inline links.
    GenericVTE50,
    /// A terminal which supports basic ANSI sequences.
    BasicAnsi,
    /// A dumb terminal that supports no formatting.
    Dumb,
}

/// A terminal error.
#[derive(Debug)]
pub enum TerminalError {
    /// The terminal does not support this operation.
    NotSupported,
    /// An I/O error occured.
    IoError(io::Error),
}

impl TerminalError {
    /// Turn a terminal error into an IO result.
    ///
    /// Map `NotSupported` to `Ok(())`.
    pub fn to_io(self) -> io::Result<()> {
        match self {
            TerminalError::NotSupported => Ok(()),
            TerminalError::IoError(err) => Err(err),
        }
    }
}

/// The result of a terminal operation.
pub type TerminalResult<T> = Result<T, TerminalError>;

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
    /// If stdout links to a TTY find out what terminal emulator we run in.
    ///
    /// Otherwise assume a dumb terminal that cannot format anything.
    pub fn detect() -> Terminal {
        if termion::is_tty(&io::stdout()) {
            if std::env::var("TERM_PROGRAM")
                .map(|value| value.contains("iTerm.app"))
                .unwrap_or(false)
            {
                Terminal::ITerm2
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

    /// Write an inline image.
    ///
    /// Supported on iTerm2, all other terminal emulators return a not supported
    /// error.
    pub fn write_inline_image<W: io::Write>(
        self,
        writer: &mut W,
        resource: &Resource,
    ) -> TerminalResult<()> {
        match self {
            Terminal::ITerm2 => resource
                .read()
                .and_then(|contents| {
                    iterm2::write_inline_image(writer, resource.as_str().as_ref(), &contents)
                })
                .map_err(|e| TerminalError::IoError(e)),
            _ => Err(TerminalError::NotSupported),
        }
    }

    /// Set the link for the subsequent text.
    ///
    /// To stop a link write a link to an empty destination.
    pub fn set_link<W: io::Write>(self, writer: &mut W, destination: &str) -> TerminalResult<()> {
        match self {
            Terminal::ITerm2 | Terminal::GenericVTE50 => {
                let command = format!("8;;{}", destination);
                write!(writer, "{}", osc(&command)).map_err(|err| TerminalError::IoError(err))
            }
            _ => Err(TerminalError::NotSupported),
        }
    }

    /// Set a mark in the current terminal.
    pub fn set_mark<W: io::Write>(self, writer: &mut W) -> TerminalResult<()> {
        if let Terminal::ITerm2 = self {
            iterm2::write_mark(writer).map_err(|err| TerminalError::IoError(err))
        } else {
            Err(TerminalError::NotSupported)
        }
    }
}

/// An OSC command for a terminal.
#[derive(Debug, Copy, Clone)]
pub struct OSC<'a> {
    command: &'a str,
}

/// Create an OSC command for the terminal.
pub fn osc(command: &str) -> OSC {
    OSC { command }
}

impl<'a> fmt::Display for OSC<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\x1B]{}\x07", self.command)
    }
}
