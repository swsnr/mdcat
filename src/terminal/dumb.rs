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

//! A terminal with no features.

use failure::Error;
use std::io::Write;

use super::super::resources::{Resource, ResourceAccess};
use super::error::NotSupportedError;
use super::types::{AnsiStyle, Size};
use super::write::Terminal;

/// A dumb terminal with no style support.
///
/// With this terminal mdcat will render no special formatting at all. Use
/// when piping to other programs or when the terminal does not even support
/// ANSI codes.
pub struct DumbTerminal<W: Write> {
    writer: W,
}

impl<W: Write> DumbTerminal<W> {
    /// Create a new bump terminal for the given writer.
    pub fn new(writer: W) -> DumbTerminal<W> {
        DumbTerminal { writer }
    }
}

impl<W: Write> Terminal for DumbTerminal<W> {
    type TerminalWrite = W;

    fn name(&self) -> &'static str {
        "dumb"
    }

    fn write(&mut self) -> &mut W {
        &mut self.writer
    }

    fn supports_styles(&self) -> bool {
        false
    }

    fn set_style(&mut self, _style: AnsiStyle) -> Result<(), Error> {
        Err(NotSupportedError {
            what: "ANSI styles",
        })?
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
