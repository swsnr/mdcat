// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The iTerm2 terminal.
//!
//! iTerm2 is a powerful macOS terminal emulator with many formatting
//! features, including images and inline links.
//!
//! See <https://www.iterm2.com> for more information.

use super::osc::write_osc;
use crate::resources::read_url;
use crate::{magic, ResourceAccess};
use std::error::Error;
use std::ffi::OsStr;
use std::io::{self, Write};
use url::Url;

use super::super::svg;

/// Whether we run inside iTerm2 or not.
pub fn is_iterm2() -> bool {
    cfg!(unix)
        && std::env::var("TERM_PROGRAM")
            .map(|value| value.contains("iTerm.app"))
            .unwrap_or(false)
}

/// Iterm2 marks.
#[derive(Debug, Copy, Clone)]
pub struct ITerm2Marks;

impl ITerm2Marks {
    /// Write an iterm2 mark command to the given `writer`.
    pub fn set_mark<W: Write>(self, writer: &mut W) -> io::Result<()> {
        write_osc(writer, "1337;SetMark")
    }
}

/// Iterm2 inline iamges.
#[derive(Debug, Copy, Clone)]
pub struct ITerm2Images;

impl ITerm2Images {
    /// Write an iterm2 inline image command to `writer`.
    ///
    /// `name` is the local file name and `contents` are the contents of the
    /// given file.
    #[cfg(unix)]
    pub fn write_inline_image<W: Write, S: AsRef<OsStr>>(
        self,
        writer: &mut W,
        name: S,
        contents: &[u8],
    ) -> io::Result<()> {
        use std::os::unix::ffi::OsStrExt;
        write_osc(
            writer,
            &format!(
                "1337;File=name={};inline=1:{}",
                base64::encode(name.as_ref().as_bytes()),
                base64::encode(contents)
            ),
        )
    }

    #[cfg(windows)]
    pub fn write_inline_image<W: Write, S: AsRef<OsStr>>(
        self,
        _writer: &mut W,
        _name: S,
        _contents: &[u8],
    ) -> io::Result<()> {
        unimplemented!()
    }

    /// Read `url` and render to an image if necessary.
    ///
    /// Render the binary content of the (rendered) image or an IO error if
    /// reading or rendering failed.
    pub fn read_and_render(
        self,
        url: &Url,
        access: ResourceAccess,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let contents = read_url(&url, access)?;
        if magic::is_svg(&magic::detect_mime_type(&contents)?) {
            svg::render_svg(&contents).map_err(Into::into)
        } else {
            Ok(contents)
        }
    }
}
