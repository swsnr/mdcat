// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

// 	http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Standard ANSI styling.

use ansi_term::Style;
use std::io::{Result, Write};

/// Access to a terminal’s basic ANSI styling functionality.
pub struct AnsiStyle;

impl AnsiStyle {
    /// Write styled text to the given writer.
    pub fn write_styled<W: Write, V: AsRef<str>>(
        &self,
        write: &mut W,
        style: &Style,
        text: V,
    ) -> Result<()> {
        write!(write, "{}", style.paint(text.as_ref()))
    }
}
