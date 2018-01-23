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
use std::io::stdout;

/// Get the number of columns for the terminal from `$COLUMNS`.
///
/// Return `None` if the variable is not set or does not contain a valid number.
fn columns_from_env() -> Option<u16> {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
}

/// Get the number of columns from the TTY device.
///
/// Return `None` if TTY access fails.
fn columns_from_tty() -> Option<u16> {
    termion::terminal_size().map(|size| size.0).ok()
}

/// Make a best effort to get the number of columns for the terminal.
///
/// Try to get the terminal size from the TTY device, or from the `$COLUMNS`
/// environment variable, and eventually assume a default of 80 for safety.
pub fn columns() -> u16 {
    columns_from_tty().or_else(columns_from_env).unwrap_or(80)
}

/// The terminal we use.
#[derive(Debug)]
enum Terminal {
    /// iTerm2
    ITerm2,
    /// An unknown terminal application.
    Unknown,
}

impl Terminal {
    /// Detect the underlying terminal application.
    fn detect() -> Terminal {
        if std::env::var("TERM_PROGRAM")
            .map(|value| value.contains("iTerm.app"))
            .unwrap_or(false)
        {
            Terminal::ITerm2
        } else {
            Terminal::Unknown
        }
    }
}

#[derive(Debug, Copy, Clone)]
/// What kind of format to use.
pub enum Format {
    /// No colours and no styles.
    NoColours,
    /// Basic colours and styles.
    Colours,
    /// Colours and additional formatting for iTerm.
    ITermColours,
}

impl Format {
    /// Auto-detect the format to use.
    ///
    /// If `force_colours` is true enforce colours, otherwise use colours if we run
    /// on a TTY.  If we run on a TTY and detect that we run within iTerm, enable
    /// additional formatting for iTerm.
    pub fn auto_detect(force_colours: bool) -> Format {
        if termion::is_tty(&stdout()) {
            match Terminal::detect() {
                Terminal::ITerm2 => Format::ITermColours,
                _ => Format::Colours,
            }
        } else if force_colours {
            Format::Colours
        } else {
            Format::NoColours
        }
    }

    /// Whether this format enables colours.
    pub fn enables_colours(self) -> bool {
        match self {
            Format::NoColours => false,
            _ => true,
        }
    }

    /// Whether this format enables inline images.
    pub fn enables_inline_images(self) -> bool {
        match self {
            Format::ITermColours => true,
            _ => false,
        }
    }

    /// Whether this format enables marks.
    pub fn enables_marks(self) -> bool {
        match self {
            Format::ITermColours => true,
            _ => false,
        }
    }
}
