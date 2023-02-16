// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Capabilities of terminal emulators.

use crate::terminal::AnsiStyle;

pub mod iterm2;
#[cfg(feature = "render-image")]
pub mod kitty;
#[cfg(feature = "render-image")]
pub mod terminology;

/// The capability of basic styling.
#[derive(Debug, Copy, Clone)]
pub enum StyleCapability {
    /// The terminal supports ANSI styles.
    Ansi(AnsiStyle),
}

/// How the terminal supports inline links.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum LinkCapability {
    /// The terminal supports [OSC 8] inline links.
    ///
    /// [OSC 8]: https://git.io/vd4ee
    Osc8(crate::terminal::osc::Osc8Links),
}

/// The capability of the terminal to set marks.
#[derive(Debug, Copy, Clone)]
pub enum MarkCapability {
    /// The terminal supports iTerm2 jump marks.
    ITerm2(self::iterm2::ITerm2Marks),
}

/// The capability of the terminal to write images inline.
#[cfg(feature = "render-image")]
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
    /// How the terminal supports basic styling.
    pub style: Option<StyleCapability>,
    /// How the terminal supports links.
    pub links: Option<LinkCapability>,
    /// How the terminal supports images.
    #[cfg(feature = "render-image")]
    pub image: Option<ImageCapability>,
    /// How the terminal supports marks.
    pub marks: Option<MarkCapability>,
}

impl Default for TerminalCapabilities {
    /// A terminal which supports nothing.
    fn default() -> Self {
        TerminalCapabilities {
            style: None,
            links: None,
            #[cfg(feature = "render-image")]
            image: None,
            marks: None,
        }
    }
}

impl TerminalCapabilities {
    #[cfg(feature = "render-image")]
    pub(crate) fn with_image_capability(mut self, cap: ImageCapability) -> Self {
        self.image = Some(cap);
        self
    }

    pub(crate) fn with_mark_capability(mut self, cap: MarkCapability) -> Self {
        self.marks = Some(cap);
        self
    }
}
