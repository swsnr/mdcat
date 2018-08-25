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

// Support modules for terminal writing.
mod error;
mod highlighting;
mod types;
mod write;

// Terminal implementations;
mod ansi;
mod dumb;
#[cfg(feature = "iterm2")]
mod iterm2;
#[cfg(feature = "terminology")]
mod terminology;
mod vte50;

use atty;
use std::io;

#[cfg(feature = "iterm2")]
use self::iterm2::*;
#[cfg(feature = "terminology")]
use self::terminology::*;
use self::vte50::*;

// Export types.
pub use self::ansi::AnsiTerminal;
pub use self::dumb::DumbTerminal;
pub use self::error::IgnoreNotSupported;
pub use self::highlighting::write_as_ansi;
pub use self::types::{AnsiColour, AnsiStyle, Size};
pub use self::write::Terminal;

/// Detect the terminal on stdout.
///
/// If stdout links to a TTY look at different pieces of information, in
/// particular environment variables set by terminal emulators, to figure
/// out what terminal emulator we run in.
///
/// If stdout does not link to a TTY assume a `Dumb` terminal which cannot
/// format anything.
pub fn detect_terminal() -> Box<Terminal<TerminalWrite = io::Stdout>> {
    if atty::is(atty::Stream::Stdout) {
        let ansi = AnsiTerminal::new(io::stdout());
        // Pattern matching lets use feature-switch branches, depending on
        // enabled terminal support.  In an if chain we can't do this, so that's
        // why we have this weird match here.  Note: Don't use true here because
        // that makes clippy complain.
        match 1 {
            #[cfg(feature = "iterm2")]
            _ if iterm2::is_iterm2() =>
            {
                Box::new(ITerm2::new(ansi))
            }
            #[cfg(feature = "terminology")]
            _ if terminology::is_terminology() =>
            {
                Box::new(Terminology::new(ansi))
            }
            _ => match vte50::get_vte_version() {
                Some(version) if version >= (50, 0) => Box::new(VTE50Terminal::new(ansi)),
                _ => Box::new(ansi),
            },
        }
    } else {
        Box::new(DumbTerminal::new(io::stdout()))
    }
}
