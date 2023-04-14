// Copyright Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Inline image handling

use std::io::Write;

use url::Url;

use crate::{ResourceUrlHandler, TerminalSize};

/// An implementation of an inline image protocol.
pub trait InlineImageProtocol {
    /// Write an inline image to `writer`.
    ///
    /// `url` is the URL pointing to the image was obtained from.  If the underlying terminal does
    /// not support URLs directly the protocol should use `resource_handler` to load the image data
    /// from `url`.
    ///
    /// `size` denotes the dimensions of the current terminal, to be used as indication for the
    /// size the image should be rendered at.
    ///
    /// Implementations are encouraged to return an IO error with [`std::io::ErrorKind::Unsupported`]
    /// if either the underlying terminal does not support images currently or if it does not
    /// support the given image format.
    fn write_inline_image(
        &self,
        writer: &mut dyn Write,
        resource_handler: &dyn ResourceUrlHandler,
        url: &Url,
        terminal_size: &TerminalSize,
    ) -> std::io::Result<()>;
}
