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

//! VTE newer than 50.

use failure::Error;
use std;
use std::io::Write;

use super::super::resources::{Resource, ResourceAccess};
use super::ansi::AnsiTerminal;
use super::size::Size;
use super::write::Terminal;

/// Get the version of VTE underlying this terminal.
///
/// Return `(minor, patch)` if this terminal uses VTE, otherwise return `None`.
pub fn get_vte_version() -> Option<(u8, u8)> {
    std::env::var("VTE_VERSION").ok().and_then(|value| {
        value[..2]
            .parse::<u8>()
            .into_iter()
            .zip(value[2..4].parse::<u8>())
            .next()
    })
}

/// A generic terminal based on a modern VTE (>= 50) version.
///
/// VTE is Gnome library for terminal emulators.  It powers some notable
/// terminal emulators like Gnome Terminal, and embedded terminals in
/// applications like GEdit.
///
/// VTE 0.50 or newer support inline links.  Older versions do not; we
/// recognize these as `BasicAnsi`.
pub struct VTE50Terminal<W: Write> {
    ansi: AnsiTerminal<W>,
}

impl<W: Write> VTE50Terminal<W> {
    /// Create a VTE 50 terminal over an underlying ANSI terminal.
    pub fn new(ansi: AnsiTerminal<W>) -> VTE50Terminal<W> {
        VTE50Terminal { ansi }
    }
}

impl<W: Write> Terminal for VTE50Terminal<W> {
    type TerminalWrite = W;

    fn name(&self) -> &'static str {
        "VTE 50"
    }

    fn write(&mut self) -> &mut W {
        self.ansi.write()
    }

    fn supports_styles(&self) -> bool {
        self.ansi.supports_styles()
    }

    fn set_link(&mut self, destination: &str) -> Result<(), Error> {
        self.ansi.write_osc(&format!("8;;{}", destination))?;
        Ok(())
    }

    fn set_mark(&mut self) -> Result<(), Error> {
        self.ansi.set_mark()
    }

    fn write_inline_image(
        &mut self,
        max_size: Size,
        resources: &Resource,
        access: ResourceAccess,
    ) -> Result<(), Error> {
        self.ansi.write_inline_image(max_size, resources, access)
    }
}
