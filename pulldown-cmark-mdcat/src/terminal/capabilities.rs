// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Capabilities of terminal emulators.

use anyhow::Result;
use ratatui::layout::Rect;

use crate::resources::InlineImageProtocol;

pub mod iterm2;
pub mod kitty;
pub mod terminology;

/// The capability of basic styling.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StyleCapability {
    /// The terminal supports ANSI styles including OSC 8.
    Ansi,
}

/// The capability of the terminal to set marks.
#[derive(Debug, Copy, Clone)]
pub enum MarkCapability {
    /// The terminal supports iTerm2 jump marks.
    ITerm2(iterm2::ITerm2Protocol),
}

/// The capability of the terminal to write images inline.
#[derive(Debug, Copy, Clone)]
pub enum ImageCapability {
    /// The terminal understands the terminology image protocol.
    Terminology(terminology::Terminology),
    /// The terminal understands the iterm2 image protocol.
    ITerm2(iterm2::ITerm2Protocol),
    /// The terminal understands the kitty graphics protocol.
    Kitty(kitty::KittyGraphicsProtocol),
}

impl ImageCapability {
    pub(crate) fn image_protocol(&self) -> &dyn InlineImageProtocol {
        match self {
            ImageCapability::Terminology(t) => t,
            ImageCapability::ITerm2(t) => t,
            ImageCapability::Kitty(t) => t,
        }
    }

    /// copy from yazi
    pub fn image_erase(self, rect: Rect) -> Result<()> {
        match self {
            Self::Kitty(_) => yazi_adaptor::kitty::Kitty::image_erase(rect),
            Self::ITerm2(_) => yazi_adaptor::iterm2::Iterm2::image_erase(rect),
            _ => todo!(),
        }
    }
}

/// The capabilities of a terminal.
///
/// See [`crate::TerminalProgram`] for a way to detect a terminal and derive known capabilities.
/// To obtain capabilities for the current terminal program use [`crate::TerminalProgram::detect`]
/// to detect the terminal and then [`crate::TerminalProgram::capabilities`] to get its
/// capabilities.
#[derive(Debug)]
pub struct TerminalCapabilities {
    /// Whether the terminal supports basic ANSI styling.
    pub style: Option<StyleCapability>,
    /// How the terminal supports images.
    pub image: Option<ImageCapability>,
    /// How the terminal supports marks.
    pub marks: Option<MarkCapability>,
}

impl Default for TerminalCapabilities {
    /// A terminal which supports nothing.
    fn default() -> Self {
        TerminalCapabilities {
            style: None,
            image: None,
            marks: None,
        }
    }
}

impl TerminalCapabilities {
    pub(crate) fn with_image_capability(mut self, cap: ImageCapability) -> Self {
        self.image = Some(cap);
        self
    }

    pub(crate) fn with_mark_capability(mut self, cap: MarkCapability) -> Self {
        self.marks = Some(cap);
        self
    }
}
