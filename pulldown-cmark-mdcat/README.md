# pulldown-cmark-mdcat

[![Crates.io](https://img.shields.io/crates/v/pulldown-cmark-mdcat)](https://crates.io/crates/pulldown-cmark-mdcat)
[![docs.rs](https://img.shields.io/docsrs/pulldown-cmark-mdcat)](https://docs.rs/pulldown-cmark-mdcat)

Render [pulldown-cmark] events to a TTY.

This library backs the [mdcat] tool, and makes its rendering available to other crates.

It supports:

- All common mark syntax.
- Standard ANSI formatting with OCS-8 hyperlinks.
- Inline images on terminal emulators with either the iTerm2 or the Kitty protocol, as well as on Terminology.
- Jump marks in iTerm2.

It does not support commonmark table and footnote extension syntax.

[mdcat]: https://github.com/swsnr/mdcat
[pulldown-cmark]: https://github.com/raphlinus/pulldown-cmark

## License

Copyright Sebastian Wiesner <sebastian@swsnr.de>

Binaries are subject to the terms of the Mozilla Public
License, v. 2.0, see [LICENSE](LICENSE).

Most of the source is subject to the terms of the Mozilla Public
License, v. 2.0, see [LICENSE](LICENSE), unless otherwise noted;
some files are subject to the terms of the Apache 2.0 license,
see <http://www.apache.org/licenses/LICENSE-2.0>
