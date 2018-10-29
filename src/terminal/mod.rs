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

mod ansi;
pub mod highlighting;
mod osc;
mod size;

#[cfg(feature = "terminology")]
mod terminology;

pub use self::ansi::AnsiStyle;
pub use self::size::Size as TerminalSize;

/// The capability of basic styling.
pub enum StyleCapability {
    /// The terminal supports no styles.
    None,
    /// The terminal supports ANSI styles.
    Ansi(AnsiStyle),
}

/// How the terminal supports inline links.
pub enum LinkCapability {
    /// The terminal does not support inline links.
    None,
    /// The terminal supports [OSC 8] inline links.
    ///
    /// [OSC 8]: https://git.io/vd4ee
    #[cfg(feature = "osc8_links")]
    OSC8(self::osc::OSC8Links),
}

/// The capability of the terminal to set marks.
pub enum MarkCapability {
    /// The terminal can't set marks.
    None,
}

/// The capability of the terminal to write images inline.
pub enum ImageCapability {
    /// The terminal can't write images inline.
    None,
    /// The terminal understands the terminology way of inline images.
    #[cfg(feature = "terminology")]
    Terminology(terminology::TerminologyImages),
}

/// The capabilities of a terminal.
pub struct TerminalCapabilities {
    /// How do we call this terminal?
    pub name: String,
    /// How the terminal supports basic styling.
    pub style: StyleCapability,
    /// How the terminal supports links.
    pub links: LinkCapability,
    /// How the terminal supports images.
    pub image: ImageCapability,
    /// How the terminal supports marks.
    pub marks: MarkCapability,
}

impl TerminalCapabilities {
    /// A terminal which supports nothing.
    pub fn none() -> TerminalCapabilities {
        TerminalCapabilities {
            name: "dumb".to_string(),
            style: StyleCapability::None,
            links: LinkCapability::None,
            image: ImageCapability::None,
            marks: MarkCapability::None,
        }
    }

    /// A terminal with basic ANSI formatting only.
    pub fn ansi() -> TerminalCapabilities {
        TerminalCapabilities {
            name: "Ansi".to_string(),
            style: StyleCapability::Ansi(AnsiStyle),
            links: LinkCapability::None,
            image: ImageCapability::None,
            marks: MarkCapability::None,
        }
    }

    /// Detect the capabilities of the current terminal.
    pub fn detect() -> TerminalCapabilities {
        // Pattern matching lets use feature-switch branches, depending on
        // enabled terminal support.  In an if chain we can't do this, so that's
        // why we have this weird match here.  Note: Don't use true here because
        // that makes clippy complain.
        match 1 {
            // #[cfg(feature = "iterm2")]
            // _ if iterm2::is_iterm2() =>
            // {
            //     Box::new(ITerm2::new(ansi))
            // }
            #[cfg(feature = "terminology")]
            _ if self::terminology::is_terminology() =>
            {
                TerminalCapabilities {
                    name: "Terminology".to_string(),
                    style: StyleCapability::Ansi(AnsiStyle),
                    links: LinkCapability::OSC8(self::osc::OSC8Links),
                    image: ImageCapability::Terminology(self::terminology::TerminologyImages),
                    marks: MarkCapability::None,
                }
            }
            #[cfg(feature = "vte50")]
            _ if get_vte_version().filter(|&v| v >= (50, 0)).is_some() =>
            {
                TerminalCapabilities {
                    name: "VTE 50".to_string(),
                    style: StyleCapability::Ansi(AnsiStyle),
                    links: LinkCapability::OSC8(self::osc::OSC8Links),
                    image: ImageCapability::None,
                    marks: MarkCapability::None,
                }
            }
            _ => TerminalCapabilities::ansi(),
        }
    }
}

/// Get the version of the underlying VTE terminal if any.
#[cfg(feature = "vte50")]
pub fn get_vte_version() -> Option<(u8, u8)> {
    std::env::var("VTE_VERSION").ok().and_then(|value| {
        value[..2]
            .parse::<u8>()
            .into_iter()
            .zip(value[2..4].parse::<u8>())
            .next()
    })
}
