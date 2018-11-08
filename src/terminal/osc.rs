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

//! OSC commands on terminals.

use std::io::{Result, Write};

/// Write an OSC `command` to this terminal.
pub fn write_osc<W: Write>(writer: &mut W, command: &str) -> Result<()> {
    writer.write_all(&[0x1b, 0x5d])?;
    writer.write_all(command.as_bytes())?;
    writer.write_all(&[0x07])?;
    Ok(())
}

#[cfg(feature = "osc8_links")]
pub struct OSC8Links;

#[cfg(feature = "osc8_links")]
impl OSC8Links {
    pub fn set_link<W: Write>(&self, writer: &mut W, destination: &str) -> Result<()> {
        write_osc(writer, &format!("8;;{}", destination))
    }

    pub fn clear_link<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.set_link(writer, "")
    }
}
