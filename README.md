# mdcat

[![Current release]( https://img.shields.io/crates/v/mdcat.svg)][crates]
![Actively developer](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)
[![Build status](https://img.shields.io/travis/lunaryorn/mdcat/master.svg)][travis]
[![Windows build status](https://img.shields.io/appveyor/ci/lunaryorn/mdcat/master.svg)][appveyor]

`cat` for [CommonMark][]: Show CommonMark (a standardized Markdown dialect)
documents on text terminals.

```
$ mdcat sample.md
```

![mdcat showcase with different colour themes][sxs]

mdcat in [iTerm2][], with [Dracula][], and [Solarized][] Light and Dark (from
left to right), and [PragmataPro][pp] as font.

[crates-badge]: https://img.shields.io/crates/v/mdcat.svg
[crates]: https://crates.io/crates/mdcat
[travis]: https://travis-ci.org/lunaryorn/mdcat
[appveyor]: https://ci.appveyor.com/project/lunaryorn/mdcat
[CommonMark]: http://commonmark.org
[Solarized]: http://ethanschoonover.com/solarized
[dracula]: https://draculatheme.com/iterm/
[iterm2]: https://www.iterm2.com
[pp]: https://www.fsd.it/shop/fonts/pragmatapro/
[sxs]: https://raw.githubusercontent.com/lunaryorn/mdcat/master/screenshots/side-by-side.png

## Features

`mdcat` works best with [iTerm2][] or a compatible terminal emulator, and a
good terminal font which includes italic characters.  It supports

* All CommonMark syntax
* Syntax highlighting for code blocks
* Inline links (note the dashed underline like in the screenshot above, in some
  terminals)
* Inline images like in the screenshot above (in some terminals), even from
  HTTP(S) URLs (use `--local` to disable remote images)
* Jump marks for headings (in iTerm2 jump forwards and backwards with
  <key>⇧⌘↓</key> and <key>⇧⌘↑</key>)

| Terminal                |  Basic syntax | Syntax highlighting | Links | Images | Jump marks |
| :---------------------- | :-----------: | :-----------------: | :---: | :----: | :--------: |
| Basic ANSI              | ✓             | ✓                   |       |        |            |
| Windows 10 console¹     | ✓             | ✓                   |       |        |            |
| VTE 0.50 or newer based | ✓             | ✓                   | ✓     |        |            |
| [Terminology][]         | ✓             | ✓                   | ✓     | ✓      |            |
| [iTerm2][]              | ✓             | ✓                   | ✓     | ✓      | ✓          |

1: On Windows you need to install with `--no-default-features`.

Not supported:

* CommonMark extensions: Footnotes and Tables
* Re-filling paragraphs

[Terminology]: http://terminolo.gy

## Installation

Install [Rust][rustup] and run `cargo install mdcat`.  To keep mdcat up to date
install [cargo-update][] and run `cargo install-update mdcat`.

**Note:** On Windows you likely need `--no-default-features` to make `mdcat`
build.

### SVG support

`mdcat` needs `rsvg-convert` to show SVG images in [iTerm2][]; otherwise `mdcat`
only shows the image title and URL for SVG images.  Install with `brew install
librsvg`.

[Terminology][] supports SVG out of the box and needs no additional tools.

### 3rd party packages

* [Homebrew][]: `brew install mdcat`
* Arch Linux: [mdcat in AUR][aur]
* Void Linux: `xbps-install -S mdcat`

[rustup]: https://www.rustup.rs
[cargo-update]: https://github.com/nabijaczleweli/cargo-update
[Homebrew]: https://brew.sh
[aur]: https://aur.archlinux.org/packages/mdcat/

### Future plans

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
