// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Standard ANSI styling.

use std::io::{Result, Write};

use anstyle::Style;

/// Access to a terminal’s basic ANSI styling functionality.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct AnsiStyle;

impl AnsiStyle {
    /// Write styled text to the given writer.
    pub fn write_styled<W: Write, V: AsRef<str>>(
        self,
        write: &mut W,
        style: &Style,
        text: V,
    ) -> Result<()> {
        write!(
            write,
            "{}{}{}",
            style.render(),
            text.as_ref(),
            style.render_reset()
        )
    }
}
