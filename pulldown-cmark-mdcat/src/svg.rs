// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! SVG "rendering" for mdcat.

use anyhow::Context;
use once_cell::sync::Lazy;
use resvg::{tiny_skia, usvg};
use usvg::{fontdb, TreeParsing, TreeTextToPath};

/// Render an SVG image to a PNG pixel graphic for display.
pub fn render_svg(svg: &[u8]) -> anyhow::Result<Vec<u8>> {
    render_svg_with_resvg(svg)
}

static FONTS: Lazy<fontdb::Database> = Lazy::new(|| {
    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();
    fontdb
});

fn render_svg_with_resvg(svg: &[u8]) -> anyhow::Result<Vec<u8>> {
    let opt = usvg::Options::default();
    let mut tree =
        usvg::Tree::from_data(svg, &opt).with_context(|| "Failed to parse SVG".to_string())?;
    tree.convert_text(&FONTS);
    let size = tree.size.to_screen_size();
    let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height())
        .with_context(|| "Failed to create target pixmap".to_string())?;
    resvg::render(
        &tree,
        resvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .with_context(|| "Failed to render SVG".to_string())?;
    pixmap
        .encode_png()
        .with_context(|| "Failed to encode rendered pixmap to PNG".to_string())
}
