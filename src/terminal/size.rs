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
    pub columns: usize,
    /// The height of the terminal, in lines.
    pub rows: usize,
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
extern "C" {
    // Need to wrap ctermid explicitly because it's not (yet?) in libc, see
    // <https://github.com/rust-lang/libc/issues/1928>
    pub fn ctermid(c: *mut libc::c_char) -> *mut libc::c_char;
}

/// Query terminal size on Unix.
///
/// Open the underlying controlling terminal via ctermid and open, and issue a
/// TIOCGWINSZ ioctl to the device.
///
/// We do this manually because terminal_size currently doesn't support pixel
/// size see <https://github.com/eminence/terminal-size/issues/22>.
#[cfg(unix)]
#[inline]
fn from_terminal_impl() -> Option<TerminalSize> {
    unsafe {
        let mut winsize = libc::winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        // ctermid uses a static buffer if given NULL.  This isn't thread safe but
        // a) we open the path right away, and b) a process only has a single
        // controlling terminal anyway, so we're pretty safe here I guess.
        let cterm_path = ctermid(std::ptr::null_mut());
        if cterm_path.is_null() {
            None
        } else {
            let fd = libc::open(cterm_path, libc::O_RDONLY);
            // disable this check for the ioctl(2) call because different libcs
            // can have different types for the request parameter, see
            // https://github.com/lunaryorn/mdcat/issues/177
            #[allow(clippy::useless_conversion)]
            let result = libc::ioctl(fd, libc::TIOCGWINSZ.into(), &mut winsize);
            libc::close(fd);
            if result == -1 || winsize.ws_row == 0 || winsize.ws_col == 0 {
                None
            } else {
                Some(winsize)
            }
        }
    }
    .map(|winsize| {
        let window = if winsize.ws_xpixel != 0 && winsize.ws_ypixel != 0 {
            Some(PixelSize {
                x: winsize.ws_xpixel as u32,
                y: winsize.ws_ypixel as u32,
            })
        } else {
            None
        };
        TerminalSize {
            columns: winsize.ws_col as usize,
            rows: winsize.ws_row as usize,
            pixels: window,
        }
    })
}

#[cfg(windows)]
#[inline]
fn from_terminal_impl() -> Option<TerminalSize> {
    use terminal_size::{terminal_size, Height, Width};
    terminal_size().map(|(Width(w), Height(h))| TerminalSize {
        rows: h as usize,
        columns: w as usize,
        pixels: None,
    })
}

impl TerminalSize {
    /// Get terminal size from `$COLUMNS` and `$LINES`.
    ///
    /// Do not assume any knowledge about window size.
    pub fn from_env() -> Option<Self> {
        let columns = std::env::var("COLUMNS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok());
        let rows = std::env::var("LINES")
            .ok()
            .and_then(|value| value.parse::<usize>().ok());

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
        from_terminal_impl()
    }

    /// Detect the terminal size.
    ///
    /// Get the terminal size from the underlying TTY, and fallback to
    /// `$COLUMNS` and `$LINES`.
    pub fn detect() -> Option<Self> {
        Self::from_terminal().or_else(Self::from_env)
    }
}
