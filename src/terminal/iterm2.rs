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

//! Iterm2 specific functions

use std::io::{Result, Write};
use base64;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use super::osc;

/// Write an iterm2 mark;
pub fn write_mark<W: Write>(writer: &mut W) -> Result<()> {
    write!(writer, "{}", osc("1337;SetMark"))
}

/// Write an iterm2 inline image.
///
/// `name` is the file name of the image, and `contents` holds the image contents.
pub fn write_inline_image<W: Write, S: AsRef<OsStr>>(
    writer: &mut W,
    name: S,
    contents: &[u8],
) -> Result<()> {
    let command = format!(
        "1337;File=name={};inline=1:{}",
        base64::encode(name.as_ref().as_bytes()),
        base64::encode(contents)
    );
    write!(writer, "{}", osc(&command))
}
