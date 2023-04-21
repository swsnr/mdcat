// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Terminal size.

use std::cmp::Ordering;

/// The size of a terminal window in pixels.
///
/// This type is partially ordered; a value is smaller than another if all fields
/// are smaller, and greater if all fields are greater.
///
/// If either field is greater and the other smaller values aren't orderable.
#[derive(Debug, Copy, Clone)]
pub struct PixelSize {
    /// The width of the window, in pixels.
    pub x: u32,
    // The height of the window, in pixels.
    pub y: u32,
}

impl PixelSize {
    /// Create a pixel size for a `(x, y)` pair.
    pub fn from_xy((x, y): (u32, u32)) -> Self {
        Self { x, y }
    }
}

impl PartialEq for PixelSize {
    fn eq(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Equal))
    }
}

impl PartialOrd for PixelSize {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.x == other.x && self.y == other.y {
            Some(Ordering::Equal)
        } else if self.x < other.x && self.y < other.y {
            Some(Ordering::Less)
        } else if self.x > other.x && self.y > other.y {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}

/// The size of a terminal.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TerminalSize {
    /// The width of the terminal, in characters aka columns.
    pub columns: u16,
    /// The height of the terminal, in lines.
    pub rows: u16,
    /// The size in pixels, if available.
    pub pixels: Option<PixelSize>,
}

impl Default for TerminalSize {
    fn default() -> Self {
        TerminalSize {
            columns: 80,
            rows: 24,
            pixels: None,
        }
    }
}

#[cfg(unix)]
mod implementation {
    use rustix::termios::{tcgetwinsize, Winsize};
    use tracing::{event, Level};

    use crate::TerminalSize;
    use std::fs::File;
    use std::io::Result;
    use std::path::Path;

    use super::PixelSize;

    /// Get the ID of the controlling terminal.
    ///
    /// This implementation currently just returns `/dev/tty`, which refers to the current TTY on
    /// Linux and macOS at least.
    fn ctermid() -> &'static Path {
        Path::new("/dev/tty")
    }

    fn from_cterm() -> Result<Winsize> {
        let tty = File::open(ctermid())?;
        tcgetwinsize(&tty).map_err(Into::into)
    }

    /// Query terminal size on Unix.
    ///
    /// Open the underlying controlling terminal via ctermid and open, and issue a
    /// TIOCGWINSZ ioctl to the device.
    ///
    /// We do this manually because terminal_size currently doesn't support pixel
    /// size see <https://github.com/eminence/terminal-size/issues/22>.
    pub fn from_terminal() -> Option<TerminalSize> {
        let winsize = from_cterm()
            .map_err(|error| {
                event!(
                    Level::ERROR,
                    "Failed to read terminal size from controlling terminal: {}",
                    error
                );
                error
            })
            .ok()?;
        if winsize.ws_row == 0 || winsize.ws_col == 0 {
            event!(
                Level::WARN,
                "Invalid terminal size returned, columns or rows were 0: {:?}",
                winsize
            );
            None
        } else {
            let pixels = if winsize.ws_xpixel != 0 && winsize.ws_ypixel != 0 {
                Some(PixelSize {
                    x: winsize.ws_xpixel as u32,
                    y: winsize.ws_ypixel as u32,
                })
            } else {
                None
            };
            Some(TerminalSize {
                columns: winsize.ws_col,
                rows: winsize.ws_row,
                pixels,
            })
        }
    }
}

#[cfg(windows)]
mod implementation {
    use terminal_size::{terminal_size, Height, Width};

    use super::TerminalSize;

    pub fn from_terminal() -> Option<TerminalSize> {
        terminal_size().map(|(Width(columns), Height(rows))| TerminalSize {
            rows,
            columns,
            pixels: None,
        })
    }
}

impl TerminalSize {
    /// Get terminal size from `$COLUMNS` and `$LINES`.
    ///
    /// Do not assume any knowledge about window size.
    pub fn from_env() -> Option<Self> {
        let columns = std::env::var("COLUMNS")
            .ok()
            .and_then(|value| value.parse::<u16>().ok());
        let rows = std::env::var("LINES")
            .ok()
            .and_then(|value| value.parse::<u16>().ok());

        match (columns, rows) {
            (Some(columns), Some(rows)) => Some(Self {
                columns,
                rows,
                pixels: None,
            }),
            _ => None,
        }
    }

    /// Detect the terminal size by querying the underlying terminal.
    ///
    /// On unix this issues a ioctl to the controlling terminal.
    ///
    /// On Windows this uses the [terminal_size] crate which does some magic windows API calls.
    ///
    /// [terminal_size]: https://docs.rs/terminal_size/
    pub fn from_terminal() -> Option<Self> {
        implementation::from_terminal()
    }

    /// Detect the terminal size.
    ///
    /// Get the terminal size from the underlying TTY, and fallback to
    /// `$COLUMNS` and `$LINES`.
    pub fn detect() -> Option<Self> {
        Self::from_terminal().or_else(Self::from_env)
    }
}
