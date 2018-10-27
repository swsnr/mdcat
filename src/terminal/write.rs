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

//! Writer for terminals.

use ansi_term::Style;
use crate::resources::{Resource, ResourceAccess};
use crate::terminal::size::Size;
use failure::Error;
use std::io::Write;

/// Write to terminals.
pub trait Terminal {
    /// The associated writer of this terminal.
    type TerminalWrite: Write;

    /// Get a descriptive name for this terminal.
    fn name(&self) -> &str;

    /// Get a writer for this terminal.
    fn write(&mut self) -> &mut Self::TerminalWrite;

    /// Whether this terminal supports styles.
    fn supports_styles(&self) -> bool;

    /// Set a link to the given destination on the terminal.
    ///
    /// To stop a link write a link with an empty destination.
    ///
    /// The default implementation errors with `NotSupportedError`.
    fn set_link(&mut self, destination: &str) -> Result<(), Error>;

    /// Set a jump mark on the terminal.
    ///
    /// The default implementation errors with `NotSupportedError`.
    fn set_mark(&mut self) -> Result<(), Error>;

    /// Write an inline image from the given resource to the terminal.
    ///
    /// The default implementation errors with `NotSupportedError`.
    fn write_inline_image(
        &mut self,
        max_size: Size,
        resource: &Resource,
        access: ResourceAccess,
    ) -> Result<(), Error>;
}

/// Write a styled text to a `terminal`.
///
/// If the terminal supports styles use `style` to paint `text`, otherwise just
/// write `text` and ignore `style`.
pub fn write_styled<W: Write, V: AsRef<str>>(
    terminal: &mut dyn Terminal<TerminalWrite = W>,
    style: &Style,
    text: V,
) -> Result<(), Error> {
    if terminal.supports_styles() {
        write!(terminal.write(), "{}", style.paint(text.as_ref()))?;
    } else {
        write!(terminal.write(), "{}", text.as_ref())?;
    }
    Ok(())
}
