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

use super::super::resources::{Resource, ResourceAccess};
use super::error::NotSupportedError;
use super::types::{AnsiColour, AnsiStyle, Size};
use super::write::Terminal;

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

    /// Write an ANSI style to this terminal.
    pub fn write_style(&mut self, style: AnsiStyle) -> io::Result<()> {
        match style {
            AnsiStyle::Reset => self.write_sgr(""),
            AnsiStyle::Bold => self.write_sgr("1"),
            AnsiStyle::Italic => self.write_sgr("3"),
            AnsiStyle::Underline => self.write_sgr("4"),
            AnsiStyle::NoItalic => self.write_sgr("23"),
            AnsiStyle::Foreground(AnsiColour::Red) => self.write_sgr("31"),
            AnsiStyle::Foreground(AnsiColour::Green) => self.write_sgr("32"),
            AnsiStyle::Foreground(AnsiColour::Yellow) => self.write_sgr("33"),
            AnsiStyle::Foreground(AnsiColour::Blue) => self.write_sgr("34"),
            AnsiStyle::Foreground(AnsiColour::Magenta) => self.write_sgr("35"),
            AnsiStyle::Foreground(AnsiColour::Cyan) => self.write_sgr("36"),
            AnsiStyle::Foreground(AnsiColour::LightRed) => self.write_sgr("91"),
            AnsiStyle::Foreground(AnsiColour::LightGreen) => self.write_sgr("92"),
            AnsiStyle::Foreground(AnsiColour::LightYellow) => self.write_sgr("93"),
            AnsiStyle::Foreground(AnsiColour::LightBlue) => self.write_sgr("94"),
            AnsiStyle::Foreground(AnsiColour::LightMagenta) => self.write_sgr("95"),
            AnsiStyle::Foreground(AnsiColour::LightCyan) => self.write_sgr("96"),
            AnsiStyle::DefaultForeground => self.write_sgr("39"),
        }
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

    fn set_style(&mut self, style: AnsiStyle) -> Result<(), Error> {
        self.write_style(style)?;
        Ok(())
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
