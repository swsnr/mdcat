# mdcat

[![Current release]( https://img.shields.io/crates/v/mdcat.svg)][crates]
![Actively developer](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)
[![Build status](https://img.shields.io/travis/lunaryorn/mdcat/master.svg)][travis]

`cat` for [CommonMark][]: Show CommonMark (a standardized Markdown dialect)
documents on text terminals.

```
$ mdcat sample.md
```

![mdcat showcase with different colour themes](screenshots/side-by-side.png)

[Solarized][] Light, Dark, and [Dracula][] (from left to right), with
[PragmataPro font][pp] in [iTerm2][].

[crates-badge]: https://img.shields.io/crates/v/mdcat.svg
[crates]: https://crates.io/crates/mdcat
[travis]: https://travis-ci.org/lunaryorn/mdcat
[CommonMark]: http://commonmark.org
[Solarized]: http://ethanschoonover.com/solarized
[dracula]: https://draculatheme.com/iterm/
[iterm2]: https://www.iterm2.com
[pp]: https://www.fsd.it/shop/fonts/pragmatapro/

## Features

* All CommonMark syntax
* Syntax highlighting for code blocks
* Image support with [iTerm2][]
* [iTerm2][] jump marks for headings: Jump forwards and backwards to headings
  with <key>⇧⌘↓</key> and <key>⇧⌘↑</key> respectively.

Not supported:

* CommonMark extensions: Footnotes and Tables
* Images from remote URLs
* Re-filling paragraphs

## Installation and Requirements

Install [Rust][1] and run `cargo install mdcat`.

To keep mdcat up to date install [cargo-update][2] and run `cargo
install-update mdcat`.

`mdcat` works best with [iTerm2][] or a compatible terminal emulator, and a
good terminal font which includes italic characters.

[1]: https://www.rustup.rs
[2]: https://github.com/nabijaczleweli/cargo-update

### Future plans

- [ ] Fetch remote images to show them inline.
- [ ] Use basic ANSI colour highlighting instead of 24 colours to better fit the terminal theme.
- [ ] Automatically select highlight theme according to terminal background [GH-5](https://github.com/lunaryorn/mdcat/issues/5).
- [ ] Figure out a better way to show HTML [GH-3](https://github.com/lunaryorn/mdcat/issues/3).
- [ ] CommonMark extensions: Footnotes [GH-1](https://github.com/lunaryorn/mdcat/issues/1).
- [ ] CommonMark extensions: Tables [GH-2](https://github.com/lunaryorn/mdcat/issues/2).
- [ ] Ignore soft wraps and wrap inline text a column limit instead [GH-4](https://github.com/lunaryorn/mdcat/issues/4).

## License

Copyright 2018 Sebastian Wiesner <sebastian@swsnr.de>

Licensed under the Apache License, Version 2.0 (the "License"); you may not use
this file except in compliance with the License. You may obtain a copy of the
License at <http://www.apache.org/licenses/LICENSE-2.0>.

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the
specific language governing permissions and limitations under the License.