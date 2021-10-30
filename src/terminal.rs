// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Terminal utilities.

// Support modules for terminal writing.

mod ansi;
pub mod highlighting;
mod size;

mod iterm2;
mod kitty;
mod osc;
mod terminology;

pub use self::ansi::AnsiStyle;
pub use self::size::TerminalSize;

/// The capability of basic styling.
#[derive(Debug, Copy, Clone)]
pub enum StyleCapability {
    /// The terminal supports ANSI styles.
    Ansi(AnsiStyle),
}

/// How the terminal supports inline links.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LinkCapability {
    /// The terminal supports [OSC 8] inline links.
    ///
    /// [OSC 8]: https://git.io/vd4ee
    Osc8(self::osc::Osc8Links),
}

/// The capability of the terminal to set marks.
#[derive(Debug, Copy, Clone)]
pub enum MarkCapability {
    /// The terminal supports iTerm2 jump marks.
    ITerm2(self::iterm2::ITerm2Marks),
}

/// The capability of the terminal to write images inline.
#[derive(Debug, Copy, Clone)]
pub enum ImageCapability {
    /// The terminal understands the terminology way of inline images.
    Terminology(self::terminology::TerminologyImages),
    /// The terminal understands the iterm2 way of inline images.
    ITerm2(self::iterm2::ITerm2Images),
    /// The terminal understands the Kitty way of inline images.
    Kitty(self::kitty::KittyImages),
}

/// The capabilities of a terminal.
#[derive(Debug)]
pub struct TerminalCapabilities {
    /// How do we call this terminal?
    pub name: String,
    /// How the terminal supports basic styling.
    pub style: Option<StyleCapability>,
    /// How the terminal supports links.
    pub links: Option<LinkCapability>,
    /// How the terminal supports images.
    pub image: Option<ImageCapability>,
    /// How the terminal supports marks.
    pub marks: Option<MarkCapability>,
}

/// Get the version of the underlying VTE terminal if any.
fn get_vte_version() -> Option<(u8, u8)> {
    std::env::var("VTE_VERSION").ok().and_then(|value| {
        value[..2]
            .parse::<u8>()
            .into_iter()
            .zip(value[2..4].parse::<u8>())
            .next()
    })
}

/// Checks if the current terminal is WezTerm.
fn is_wezterm() -> bool {
    std::env::var("TERM_PROGRAM").map_or(false, |value| value == "WezTerm")
        || std::env::var("TERM").map_or(false, |value| value == "wezterm")
}

/// Checks if the current terminal is foot.
fn is_foot() -> bool {
    std::env::var("TERM").map_or(false, |value| value == "foot")
}

impl TerminalCapabilities {
    /// A terminal which supports nothing.
    pub fn none() -> TerminalCapabilities {
        TerminalCapabilities {
            name: "dumb".to_string(),
            style: None,
            links: None,
            image: None,
            marks: None,
        }
    }

    /// A terminal with basic ANSI formatting only.
    pub fn ansi() -> TerminalCapabilities {
        TerminalCapabilities {
            name: "Ansi".to_string(),
            style: Some(StyleCapability::Ansi(AnsiStyle)),
            links: None,
            image: None,
            marks: None,
        }
    }

    /// Terminal capabilities of iTerm2.
    pub fn iterm2() -> TerminalCapabilities {
        TerminalCapabilities {
            name: "iTerm2".to_string(),
            style: Some(StyleCapability::Ansi(AnsiStyle)),
            links: Some(LinkCapability::Osc8(self::osc::Osc8Links)),
            image: Some(ImageCapability::ITerm2(self::iterm2::ITerm2Images)),
            marks: Some(MarkCapability::ITerm2(self::iterm2::ITerm2Marks)),
        }
    }

    /// Terminal capabilities of Terminology.
    pub fn terminology() -> TerminalCapabilities {
        TerminalCapabilities {
            name: "Terminology".to_string(),
            style: Some(StyleCapability::Ansi(AnsiStyle)),
            links: Some(LinkCapability::Osc8(self::osc::Osc8Links)),
            image: Some(ImageCapability::Terminology(
                self::terminology::TerminologyImages,
            )),
            marks: None,
        }
    }

    /// Terminal capabilities of Kitty.
    pub fn kitty() -> TerminalCapabilities {
        TerminalCapabilities {
            name: "Kitty".to_string(),
            style: Some(StyleCapability::Ansi(AnsiStyle)),
            links: Some(LinkCapability::Osc8(self::osc::Osc8Links)),
            image: Some(ImageCapability::Kitty(self::kitty::KittyImages)),
            marks: None,
        }
    }

    /// Terminal capabilities of VET 0.50 or newer.
    pub fn vte50() -> TerminalCapabilities {
        TerminalCapabilities {
            name: "VTE 50".to_string(),
            style: Some(StyleCapability::Ansi(AnsiStyle)),
            links: Some(LinkCapability::Osc8(self::osc::Osc8Links)),
            image: None,
            marks: None,
        }
    }

    /// Terminal capabilities of WezTerm (Wez's Terminal Emulator).
    ///
    /// WezTerm is a GPU-accelerated cross-platform
    /// terminal emulator and multiplexer written by @wez
    /// and implemented in Rust.
    ///
    /// See <https://wezfurlong.org/wezterm/> for more details.
    pub fn wezterm() -> TerminalCapabilities {
        TerminalCapabilities {
            name: "WezTerm".to_string(),
            style: Some(StyleCapability::Ansi(AnsiStyle)),
            links: Some(LinkCapability::Osc8(self::osc::Osc8Links)),
            image: Some(ImageCapability::ITerm2(self::iterm2::ITerm2Images)),
            marks: None,
        }
    }

    /// Terminal capabilities of foot
    /// Foot is a fast, lightweight and minimalistic Wayland terminal emulator
    pub fn foot() -> TerminalCapabilities {
        TerminalCapabilities {
            name: "foot".to_string(),
            style: Some(StyleCapability::Ansi(AnsiStyle)),
            links: Some(LinkCapability::Osc8(self::osc::Osc8Links)),
            image: None,
            marks: None,
        }
    }

    /// Detect the capabilities of the current terminal.
    pub fn detect() -> TerminalCapabilities {
        if self::iterm2::is_iterm2() {
            Self::iterm2()
        } else if self::terminology::is_terminology() {
            Self::terminology()
        } else if self::kitty::is_kitty() {
            Self::kitty()
        } else if is_wezterm() {
            Self::wezterm()
        } else if is_foot() {
            Self::foot()
        } else if get_vte_version().filter(|&v| v >= (50, 0)).is_some() {
            Self::vte50()
        } else {
            Self::ansi()
        }
    }
}
