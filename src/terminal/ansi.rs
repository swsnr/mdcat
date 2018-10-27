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

//! A standard Ansi terminal with no special features.

use failure::Error;
use std::io;
use std::io::Write;

use crate::error::NotSupportedError;
use crate::resources::{Resource, ResourceAccess};
use crate::terminal::size::Size;
use crate::terminal::write::Terminal;

/// A simple ANSI terminal with support for basic ANSI styles.
///
/// This represents most ordinary terminal emulators.
pub struct AnsiTerminal<W: Write> {
    writer: W,
}

impl<W: Write> AnsiTerminal<W> {
    /// Create a new ANSI terminal for th given writer.
    pub fn new(writer: W) -> AnsiTerminal<W> {
        AnsiTerminal { writer }
    }

    /// Write an OSC `command` to this terminal.
    pub fn write_osc(&mut self, command: &str) -> io::Result<()> {
        self.writer.write_all(&[0x1b, 0x5d])?;
        self.writer.write_all(command.as_bytes())?;
        self.writer.write_all(&[0x07])?;
        Ok(())
    }

    /// Write a CSI SGR `command` to this terminal.
    ///
    /// See <https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_sequences>.
    pub fn write_sgr(&mut self, command: &str) -> io::Result<()> {
        self.writer.write_all(&[0x1b, 0x5b])?;
        self.writer.write_all(command.as_bytes())?;
        self.writer.write_all(&[0x6d])?;
        Ok(())
    }
}

impl<W: Write> Terminal for AnsiTerminal<W> {
    type TerminalWrite = W;

    fn name(&self) -> &'static str {
        "ANSI"
    }

    fn write(&mut self) -> &mut W {
        &mut self.writer
    }

    fn supports_styles(&self) -> bool {
        true
    }

    fn set_link(&mut self, _destination: &str) -> Result<(), Error> {
        Err(NotSupportedError {
            what: "inline links",
        })?
    }

    fn set_mark(&mut self) -> Result<(), Error> {
        Err(NotSupportedError { what: "marks" })?
    }

    fn write_inline_image(
        &mut self,
        _max_size: Size,
        _resources: &Resource,
        _access: ResourceAccess,
    ) -> Result<(), Error> {
        Err(NotSupportedError {
            what: "inline images",
        })?
    }
}
