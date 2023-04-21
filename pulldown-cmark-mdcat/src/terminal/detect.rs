// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Detect the terminal application mdcat is running on.

use crate::terminal::capabilities::iterm2::ITerm2;
use crate::terminal::capabilities::*;
use crate::terminal::AnsiStyle;
use std::fmt::{Display, Formatter};

/// A terminal application.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TerminalProgram {
    /// A dumb terminal which does not support any formatting.
    Dumb,
    /// A plain ANSI terminal which supports only standard ANSI formatting.
    Ansi,
    /// iTerm2.
    ///
    /// iTerm2 is a powerful macOS terminal emulator with many formatting features, including images
    /// and inline links.
    ///
    /// See <https://www.iterm2.com> for more information.
    ITerm2,
    /// Terminology.
    ///
    /// See <http://terminolo.gy> for more information.
    Terminology,
    /// Kitty.
    ///
    /// kitty is a fast, featureful, GPU-based terminal emulator with a lot of extensions to the
    /// terminal protocol.
    ///
    /// See <https://sw.kovidgoyal.net/kitty/> for more information.
    Kitty,
    /// WezTerm
    ///
    /// WezTerm is a GPU-accelerated cross-platform terminal emulator and multiplexer.  It's highly
    /// customizable and supports some terminal extensions.
    ///
    /// See <https://wezfurlong.org/wezterm/> for more information.
    WezTerm,
}

impl Display for TerminalProgram {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match *self {
            TerminalProgram::Dumb => "dumb",
            TerminalProgram::Ansi => "ansi",
            TerminalProgram::ITerm2 => "iTerm2",
            TerminalProgram::Terminology => "Terminology",
            TerminalProgram::Kitty => "kitty",
            TerminalProgram::WezTerm => "WezTerm",
        };
        write!(f, "{name}")
    }
}

impl TerminalProgram {
    fn detect_term() -> Option<Self> {
        match std::env::var("TERM").ok().as_deref() {
            Some("wezterm") => Some(Self::WezTerm),
            Some("xterm-kitty") => Some(Self::Kitty),
            _ => None,
        }
    }

    fn detect_term_program() -> Option<Self> {
        match std::env::var("TERM_PROGRAM").ok().as_deref() {
            Some("WezTerm") => Some(Self::WezTerm),
            Some("iTerm.app") => Some(Self::ITerm2),
            _ => None,
        }
    }

    /// Attempt to detect the terminal program mdcat is running on.
    ///
    /// This function looks at various environment variables to identify the terminal program.
    ///
    /// It first looks at `$TERM` to determine the terminal program, then at `$TERM_PROGRAM`, and
    /// finally at `$TERMINOLOGY`.
    ///
    /// If `$TERM` is set to anything other than `xterm-256colors` it's definitely accurate, since
    /// it points to the terminfo entry to use.  `$TERM` also propagates across most boundaries
    /// (e.g. `sudo`, `ssh`), and thus the most reliable place to check.
    ///
    /// However, `$TERM` only works if the terminal has a dedicated entry in terminfo database. Many
    /// terminal programs avoid this complexity (even WezTerm only sets `$TERM` if explicitly
    /// configured to do so), so `mdcat` proceeds to look at other variables.  However, these are
    /// generally not as reliable as `$TERM`, because they often do not propagate across SSH or
    /// sudo, and may leak if one terminal program is started from another one.
    ///
    /// # Returns
    ///
    /// - [`TerminalProgram::Kitty`] if `$TERM` is `xterm-kitty`.
    /// - [`TerminalProgram::WezTerm`] if `$TERM` is `wezterm`.
    /// - [`TerminalProgram::WezTerm`] if `$TERM_PROGRAM` is `WezTerm`.
    /// - [`TerminalProgram::ITerm2`] if `$TERM_PROGRAM` is `iTerm.app`.
    /// - [`TerminalProgram::Terminology`] if `$TERMINOLOGY` is `1`.
    /// - [`TerminalProgram::Ansi`] otherwise.
    pub fn detect() -> Self {
        Self::detect_term()
            .or_else(Self::detect_term_program)
            .or_else(|| match std::env::var("TERMINOLOGY").ok().as_deref() {
                Some("1") => Some(Self::Terminology),
                _ => None,
            })
            .unwrap_or(Self::Ansi)
    }

    /// Get the capabilities of this terminal emulator.
    pub fn capabilities(self) -> TerminalCapabilities {
        let ansi = TerminalCapabilities {
            style: Some(StyleCapability::Ansi(AnsiStyle)),
            links: Some(LinkCapability::Osc8(crate::terminal::osc::Osc8Links)),
            image: None,
            marks: None,
        };
        match self {
            TerminalProgram::Dumb => TerminalCapabilities::default(),
            TerminalProgram::Ansi => ansi,
            TerminalProgram::ITerm2 => ansi
                .with_mark_capability(MarkCapability::ITerm2(ITerm2))
                .with_image_capability(ImageCapability::ITerm2(ITerm2)),
            TerminalProgram::Terminology => {
                ansi.with_image_capability(ImageCapability::Terminology(terminology::Terminology))
            }
            TerminalProgram::Kitty => {
                ansi.with_image_capability(ImageCapability::Kitty(self::kitty::KittyImages))
            }
            TerminalProgram::WezTerm => {
                ansi.with_image_capability(ImageCapability::Kitty(self::kitty::KittyImages))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::terminal::TerminalProgram;

    use temp_env::with_vars;

    #[test]
    pub fn detect_term_kitty() {
        with_vars(vec![("TERM", Some("xterm-kitty"))], || {
            assert_eq!(TerminalProgram::detect(), TerminalProgram::Kitty)
        })
    }

    #[test]
    pub fn detect_term_wezterm() {
        with_vars(vec![("TERM", Some("wezterm"))], || {
            assert_eq!(TerminalProgram::detect(), TerminalProgram::WezTerm)
        })
    }

    #[test]
    pub fn detect_term_program_wezterm() {
        with_vars(
            vec![
                ("TERM", Some("xterm-256color")),
                ("TERM_PROGRAM", Some("WezTerm")),
            ],
            || assert_eq!(TerminalProgram::detect(), TerminalProgram::WezTerm),
        )
    }

    #[test]
    pub fn detect_term_program_iterm2() {
        with_vars(
            vec![
                ("TERM", Some("xterm-256color")),
                ("TERM_PROGRAM", Some("iTerm.app")),
            ],
            || assert_eq!(TerminalProgram::detect(), TerminalProgram::ITerm2),
        )
    }

    #[test]
    pub fn detect_terminology() {
        with_vars(
            vec![
                ("TERM", Some("xterm-256color")),
                ("TERM_PROGRAM", None),
                ("TERMINOLOGY", Some("1")),
            ],
            || assert_eq!(TerminalProgram::detect(), TerminalProgram::Terminology),
        );
        with_vars(
            vec![
                ("TERM", Some("xterm-256color")),
                ("TERM_PROGRAM", None),
                ("TERMINOLOGY", Some("0")),
            ],
            || assert_eq!(TerminalProgram::detect(), TerminalProgram::Ansi),
        );
    }

    #[test]
    pub fn detect_ansi() {
        with_vars(
            vec![
                ("TERM", Some("xterm-256color")),
                ("TERM_PROGRAM", None),
                ("TERMINOLOGY", None),
            ],
            || assert_eq!(TerminalProgram::detect(), TerminalProgram::Ansi),
        )
    }

    /// Regression test for <https://github.com/swsnr/mdcat/issues/230>
    #[test]
    #[allow(non_snake_case)]
    pub fn GH_230_detect_nested_kitty_from_iterm2() {
        with_vars(
            vec![
                ("TERM_PROGRAM", Some("iTerm.app")),
                ("TERM", Some("xterm-kitty")),
            ],
            || assert_eq!(TerminalProgram::detect(), TerminalProgram::Kitty),
        )
    }
}
