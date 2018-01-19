# mdcat

[![Current release]( https://img.shields.io/crates/v/mdcat.svg)][crates]
![Actively developer](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)
[![Build status](https://img.shields.io/travis/lunaryorn/mdcat.rs/master.svg)][travis]

`cat` for [CommonMark][]: Show CommonMark (a standardized Markdown dialect)
documents on text terminals.

```
$ mdcat sample.md
```

[crates-badge]: https://img.shields.io/crates/v/mdcat.svg
[crates]: https://crates.io/crates/mdcat
[travis]: https://travis-ci.org/lunaryorn/mdcat
[CommonMark]: http://commonmark.org

## Installation and Requirements

Install [Rust][1] and run `cargo install mdcat`.

To keep mdcat up to date install [cargo-update][2] and run `cargo
install-update mdcat`.

mdcat needs a decent modern terminal with a good font; in particular mdcat
uses

- 256 colours,
- italic text, and
- iTerm2 escape codes for marks.

iTerm2 works; I do not know about other terminal emulators.

[1]: https://www.rustup.rs
[2]: https://github.com/nabijaczleweli/cargo-update
[3]: https://www.iterm2.com

## Status and future plans

`mdcat` supports all checked features in the list below.  For unsupported
syntax mdcat **panics**!

### Version 1

- [x] Inline formatting, with proper nesting of emphasis
- [x] Headings
- [x] Block quotes
- [x] Code blocks
- [x] Ordered lists
- [x] Numbered lists
- [x] Nested lists
- [x] Links
- [x] Syntax highlighting for code blocks
- [x] Show inline HTML and block HTML literally
- [x] iTerm2 integration: Set marks for headings to jump back and forth
- [x] iTerm2 integration: Show images inline

### Future plans

- [ ] Automatically select highlight theme according to terminal background [GH-5](https://github.com/lunaryorn/mdcat/issues/5)
- [ ] Figure out a better way to show HTML [GH-3](https://github.com/lunaryorn/mdcat/issues/3)
- [ ] CommonMark extensions: Footnotes [GH-1](https://github.com/lunaryorn/mdcat/issues/1)
- [ ] CommonMark extensions: Tables [GH-2](https://github.com/lunaryorn/mdcat/issues/2)
- [ ] Use basic ANSI colour highlighting instead of 24 colours to better fit the terminal theme
- [ ] Ignore soft wraps and wrap inline text a column limit instead [GH-4](https://github.com/lunaryorn/mdcat/issues/4)

## License

Copyright 2018 Sebastian Wiesner <sebastian@swsnr.de>

Licensed under the Apache License, Version 2.0 (the "License"); you may not use
this file except in compliance with the License. You may obtain a copy of the
License at <http://www.apache.org/licenses/LICENSE-2.0>.

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the
specific language governing permissions and limitations under the License.