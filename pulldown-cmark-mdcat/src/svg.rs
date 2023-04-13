// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! SVG "rendering" for mdcat.

use std::io::ErrorKind;

use image::{
    error::{EncodingError, ImageFormatHint},
    ImageFormat,
};
use once_cell::sync::Lazy;
use resvg::{
    tiny_skia,
    usvg::{self, ScreenSize},
};
use thiserror::Error;
use usvg::{fontdb, TreeParsing, TreeTextToPath};

#[derive(Debug, Error)]
pub enum RenderSvgError {
    #[error("Failed to parse SVG: {0}")]
    ParseError(#[from] usvg::Error),
    #[error("Failed to create pixmap of size {0}")]
    FailedToCreatePixmap(ScreenSize),
    #[error("Failed to encode pixmap to pixel image of format {}: {}", .0.format_hint(), .0)]
    EncodePngError(#[from] EncodingError),
}

impl From<RenderSvgError> for std::io::Error {
    fn from(value: RenderSvgError) -> Self {
        std::io::Error::new(ErrorKind::Other, value)
    }
}

/// Render an SVG image to a PNG pixel graphic for display.
pub fn render_svg_to_png(svg: &[u8]) -> Result<Vec<u8>, RenderSvgError> {
    render_svg_with_resvg(svg)
}

static FONTS: Lazy<fontdb::Database> = Lazy::new(|| {
    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();
    fontdb
});

fn render_svg_with_resvg(svg: &[u8]) -> Result<Vec<u8>, RenderSvgError> {
    let opt = usvg::Options::default();
    let mut tree = usvg::Tree::from_data(svg, &opt)?;
    tree.convert_text(&FONTS);
    let size = tree.size.to_screen_size();
    let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height())
        .ok_or(RenderSvgError::FailedToCreatePixmap(size))?;
    // We create a pixmap of the appropriate size so the size transform in render cannot fail, so
    // if it fails it's a bug in our code or in resvg which we should fix and not hide.  Hence we
    // unwrap the result.
    resvg::render(
        &tree,
        resvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();
    pixmap
        .encode_png()
        .map_err(|err| EncodingError::new(ImageFormatHint::Exact(ImageFormat::Png), err).into())
}
