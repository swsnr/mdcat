// Copyright 2018-2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! SVG rendering for mdcat.

use std::io::Result;

/// Render an SVG image to a PNG pixel graphic for display.
pub fn render_svg_to_png(svg: &[u8]) -> Result<Vec<u8>> {
    implementation::render_svg_to_png(svg)
}

#[cfg(feature = "svg")]
mod implementation {
    use std::fmt::Display;
    use std::sync::{Arc, OnceLock};
    use std::{error::Error, io::ErrorKind};

    use resvg::tiny_skia::{IntSize, Pixmap, Transform};
    use resvg::usvg::{self, Tree};
    use usvg::fontdb;

    #[derive(Debug)]
    pub enum RenderSvgError {
        ParseError(usvg::Error),
        FailedToCreatePixmap(IntSize),
        EncodePngError(Box<dyn Error + Send + Sync>),
    }

    impl Display for RenderSvgError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                RenderSvgError::ParseError(error) => write!(f, "Failed to parse SVG: {error}"),
                RenderSvgError::FailedToCreatePixmap(int_size) => {
                    write!(f, "Failed to create pixmap of size {int_size:?}")
                }
                RenderSvgError::EncodePngError(error) => {
                    write!(f, "Failed to encode pixmap to PNG image: {error}")
                }
            }
        }
    }

    impl std::error::Error for RenderSvgError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                RenderSvgError::ParseError(error) => Some(error),
                RenderSvgError::FailedToCreatePixmap(_) => None,
                RenderSvgError::EncodePngError(error) => Some(error.as_ref()),
            }
        }
    }

    impl From<RenderSvgError> for std::io::Error {
        fn from(value: RenderSvgError) -> Self {
            std::io::Error::new(ErrorKind::Other, value)
        }
    }

    impl From<usvg::Error> for RenderSvgError {
        fn from(value: usvg::Error) -> Self {
            Self::ParseError(value)
        }
    }

    static FONTS: OnceLock<Arc<fontdb::Database>> = OnceLock::new();

    fn parse_svg(svg: &[u8]) -> Result<Tree, RenderSvgError> {
        let fonts = FONTS.get_or_init(|| {
            let mut fontdb = fontdb::Database::new();
            fontdb.load_system_fonts();
            Arc::new(fontdb)
        });
        let options = usvg::Options {
            fontdb: fonts.clone(),
            ..Default::default()
        };
        Ok(usvg::Tree::from_data(svg, &options)?)
    }

    fn render_svg_to_png_with_resvg(svg: &[u8]) -> Result<Vec<u8>, RenderSvgError> {
        let tree = parse_svg(svg)?;
        let size = tree.size().to_int_size();
        let mut pixmap = Pixmap::new(size.width(), size.height())
            .ok_or(RenderSvgError::FailedToCreatePixmap(size))?;
        // We create a pixmap of the appropriate size so the size transform in render cannot fail, so
        // if it fails it's a bug in our code or in resvg which we should fix and not hide.  Hence we
        // unwrap the result.
        resvg::render(&tree, Transform::default(), &mut pixmap.as_mut());
        pixmap
            .encode_png()
            .map_err(|err| RenderSvgError::EncodePngError(Box::new(err)))
    }

    pub fn render_svg_to_png(svg: &[u8]) -> std::io::Result<Vec<u8>> {
        render_svg_to_png_with_resvg(svg).map_err(Into::into)
    }
}

#[cfg(not(feature = "svg"))]
mod implementation {
    use std::io::{Error, ErrorKind, Result};
    pub fn render_svg_to_png(_svg: &[u8]) -> Result<Vec<u8>> {
        Err(Error::new(
            ErrorKind::Unsupported,
            "SVG rendering not enabled in this build",
        ))
    }
}
