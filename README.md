# mdcat

Fancy `cat` for Markdown (that is, [CommonMark][]):

```
$ mdcat sample.md
```

![mdcat showcase with different colour themes][sxs]

mdcat in [iTerm2], with [Dracula], and [Solarized] Light and Dark (from left to
right), and [PragmataPro] as font.

[CommonMark]: http://commonmark.org
[Solarized]: http://ethanschoonover.com/solarized
[dracula]: https://draculatheme.com/iterm/
[iterm2]: https://www.iterm2.com
[PragmataPro]: https://www.fsd.it/shop/fonts/pragmatapro/
[sxs]: ./screenshots/side-by-side.png

## Features

`mdcat` works best with [iTerm2] or a compatible terminal emulator, and a good
terminal font with italic characters.  Then it

* nicely renders all basic CommonMark syntax (no [tables][GH-2] or [footnotes][GH-1] though),
* highlights code blocks with [syntect],
* shows [links][osc8] and images inline in supported terminals (see above, where "Pixabay" is a clickable link!),
* adds jump marks for headings in [iTerm2] (jump forwards and backwards with
  <key>⇧⌘↓</key> and <key>⇧⌘↑</key>).

| Terminal                   |  Basic syntax | Syntax highlighting | Links | Images | Jump marks |
| :------------------------- | :-----------: | :-----------------: | :---: | :----: | :--------: |
| Basic ANSI                 | ✓             | ✓                   |       |        |            |
| Windows [ConEmu][]         | ✓             | ✓                   |       |        |            |
| Windows 10 console         | ✓             | ✓                   |       |        |            |
| Generic VTE 0.50 or newer¹ | ✓             | ✓                   | ✓     |        |            |
| [Terminology][]            | ✓             | ✓                   | ✓     | ✓      |            |
| [iTerm2][]                 | ✓             | ✓                   | ✓     | ✓      | ✓          |
| [kitty][]                  | ✓             | ✓                   | ✓     | ✓      |            |

¹) VTE is Gnome’s terminal emulation library used by many popular terminal emulators on Linux, including
Gnome Terminal, Xfce Terminal, Tilix, etc.

Not supported:

* CommonMark extensions: [Footnotes][GH-1] and [tables][GH-2]
* [Re-filling paragraphs][GH-4]

[syntect]: https://github.com/trishume/syntect
[osc8]: https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda
[Terminology]: http://terminolo.gy
[ConEmu]: https://conemu.github.io
[kitty]: https://sw.kovidgoyal.net/kitty/index.html

## Installation

### Binaries

The [Releases] page provides pre-build binaries for Linux, macOS and Windows.

**Note:** The Linux build is statically linked and requires the `curl` command
to fetch images from HTTP(S).

**Tip:** You can copy or hard-link `mdcat` to `mdless` for a variant of `mdcat` which paginates by default (like `mdcat -p`).

[Releases]: https://github.com/lunaryorn/mdcat/releases

### 3rd party packages

Some package repositories include `mdcat`:

* [Homebrew]: `brew install mdcat`
* Arch Linux: `pacman -S mdcat`
* Void Linux: `xbps-install -S mdcat`
* Nixpkgs: `nix-env -i mdcat`

[Homebrew]: https://brew.sh

### Building with rustup

You can also build `mdcat` manually with `cargo install mdcat`.

### SVG support

`mdcat` needs `rsvg-convert` to show SVG images in [iTerm2] and [kitty];
otherwise `mdcat` only shows the image title and URL for SVG images.  On macOS
you can install the `librsvg` formula from Homebrew, on Linux the tool is
typically part of the `librsvg-bin` package (or similar).

[Terminology] renders SVG directly and needs no additional tools.

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

Copyright Sebastian Wiesner <sebastian@swsnr.de>

Binaries are subject to the terms of the Mozilla Public
License, v. 2.0, see [LICENSE](LICENSE).

Most of the source is subject to the terms of the Mozilla Public
License, v. 2.0, see [LICENSE](LICENSE), unless otherwise noted;
some files are subject to the terms of the Apache 2.0 license,
see <http://www.apache.org/licenses/LICENSE-2.0>
