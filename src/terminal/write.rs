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

use super::super::resources::{Resource, ResourceAccess};
use super::types::{AnsiStyle, Size};
use failure::Error;
use std::io::Write;

/// Write to terminals.
pub trait Terminal {
    /// The associated writer of this terminal.
    type TerminalWrite: Write;

    /// Get a writer for this terminal.
    fn write(&mut self) -> &mut Self::TerminalWrite;

    /// Whether this terminal supports styles.
    fn supports_styles(&self) -> bool;

    /// Active a style on the terminal.
    ///
    /// The default implementation errors with `NotSupportedError`.
    fn set_style(&mut self, style: AnsiStyle) -> Result<(), Error>;

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
