# mdcat

[![Current release](https://img.shields.io/crates/v/mdcat.svg)][crates]
[![License](https://img.shields.io/github/license/lunaryorn/mdcat.svg)][license]
[![CI status](https://github.com/lunaryorn/mdcat/workflows/CI/badge.svg)][workflows]
[![Dependabot Status](https://api.dependabot.com/badges/status?host=github&repo=lunaryorn/mdcat)](https://dependabot.com)

`cat` for Markdown (that is, [CommonMark][]):

```
$ mdcat sample.md
```

![mdcat showcase with different colour themes][sxs]

mdcat in [iTerm2], with [Dracula], and [Solarized] Light and Dark (from left to
right), and [PragmataPro] as font.

[crates]: https://crates.io/crates/mdcat
[license]: https://github.com/lunaryorn/mdcat/blob/master/LICENSE
[workflows]: https://github.com/lunaryorn/mdcat/actions
[CommonMark]: http://commonmark.org
[Solarized]: http://ethanschoonover.com/solarized
[dracula]: https://draculatheme.com/iterm/
[iterm2]: https://www.iterm2.com
[PragmataPro]: https://www.fsd.it/shop/fonts/pragmatapro/
[sxs]: https://raw.githubusercontent.com/lunaryorn/mdcat/master/screenshots/side-by-side.png

## Features

`mdcat` works best with [iTerm2] or a compatible terminal emulator, and a good
terminal font with italic characters.  Then it

* nicely renders all basic CommonMark syntax (no [tables][GH-2] or [footnotes][GH-1] though),
* highlights code blocks with [syntect],
* shows [links][osc8] and images inline in supported terminals (see above, where "Pixabay" is a clickable link!),
* adds jump marks for headings in [iTerm2] (jump forwards and backwards with
  <key>⇧⌘↓</key> and <key>⇧⌘↑</key>).

| Terminal                |  Basic syntax | Syntax highlighting | Links | Images | Jump marks |
| :---------------------- | :-----------: | :-----------------: | :---: | :----: | :--------: |
| Basic ANSI              | ✓             | ✓                   |       |        |            |
| Windows [ConEmu][]¹     | ✓             | ✓                   |       |        |            |
| Windows 10 console¹     | ✓             | ✓                   |       |        |            |
| VTE 0.50 or newer based | ✓             | ✓                   | ✓     |        |            |
| [Terminology][]         | ✓             | ✓                   | ✓     | ✓      |            |
| [iTerm2][]              | ✓             | ✓                   | ✓     | ✓      | ✓          |

1: On Windows you need to install with `--no-default-features` (see below).

Not supported:

* CommonMark extensions: [Footnotes][GH-1] and [tables][GH-2]
* [Re-filling paragraphs][GH-4]

[syntect]: https://github.com/trishume/syntect
[osc8]: https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda
[Terminology]: http://terminolo.gy
[ConEmu]: https://conemu.github.io

## Installation

### Binaries

The [Releases] page provides pre-build binaries for Linux, macOS and Windows.
Use these binaries to try `mdcat`, particularly on Windows.

[Releases]: https://github.com/lunaryorn/mdcat/releases

### 3rd party packages

Some package managers include `mdcat`:

* [Homebrew]: `brew install mdcat`
* Arch Linux: [mdcat in AUR][aur]
* Void Linux: `xbps-install -S mdcat`
* OpenBSD: [port][epbsd_port] via [EPBSD][epbsd]

While these packages may not always be up to date we generally recommend to use
these to keep `mdcat` updated with the rest of the system.

[Homebrew]: https://brew.sh
[aur]: https://aur.archlinux.org/packages/mdcat/
[epbsd]: https://epbsd.org/
[epbsd_port]: https://gitlab.com/epbsd/ports/tree/master/textproc/mdcat

### Building with rustup

You can also build `mdcat` manually with `cargo`.  Install Rust via [rustup] and
run `cargo install mdcat`.  To keep mdcat up to date install [cargo-update] and
run `cargo install-update mdcat`.

**Note:** On Windows you likely need `--no-default-features` to make `mdcat`
build.

[rustup]: https://www.rustup.rs
[cargo-update]: https://github.com/nabijaczleweli/cargo-update

### SVG support

`mdcat` needs `rsvg-convert` to show SVG images in [iTerm2]; otherwise `mdcat`
only shows the image title and URL for SVG images.  Install with `brew install
librsvg`.

[Terminology] supports SVG out of the box and needs no additional tools.

### Future plans

- [ ] Figure out a better way to show HTML [GH-3].
- [ ] CommonMark extensions: Footnotes [GH-1].
- [ ] CommonMark extensions: Tables [GH-2].
- [ ] Ignore soft wraps and wrap inline text a column limit instead [GH-4].

[GH-1]: https://github.com/lunaryorn/mdcat/issues/1
[GH-2]: https://github.com/lunaryorn/mdcat/issues/2
[GH-3]: https://github.com/lunaryorn/mdcat/issues/3
[GH-4]: https://github.com/lunaryorn/mdcat/issues/4

## License

Copyright 2018-2019 Sebastian Wiesner <sebastian@swsnr.de>

Licensed under the Apache License, Version 2.0 (the "License"); you may not use
this file except in compliance with the License. You may obtain a copy of the
License at <http://www.apache.org/licenses/LICENSE-2.0>.

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the
specific language governing permissions and limitations under the License.
