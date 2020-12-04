# mdcat

Fancy `cat` for Markdown (that is, [CommonMark][]):

```
$ mdcat sample.md
```

![mdcat showcase with different colour themes][sxs]

mdcat in [kitty], with Tango Light, [Dracula], and [Solarized] Light (from left to
right), and [PragmataPro] as font.

[CommonMark]: http://commonmark.org
[Solarized]: http://ethanschoonover.com/solarized
[dracula]: https://draculatheme.com/iterm/
[kitty]: https://sw.kovidgoyal.net/kitty/index.html
[PragmataPro]: https://www.fsd.it/shop/fonts/pragmatapro/
[sxs]: ./screenshots/side-by-side.png

## Features

`mdcat` works best with [iTerm2] or [Kitty], and a good terminal font with italic characters.
Then it

* nicely renders all basic CommonMark syntax (no [tables][GH-2] or [footnotes][GH-1] though),
* highlights code blocks with [syntect],
* shows [links][osc8] and images inline in supported terminals (see above, where "Pixabay" is a clickable link!),
* adds jump marks for headings in [iTerm2] (jump forwards and backwards with <key>⇧⌘↓</key> and <key>⇧⌘↑</key>).

| Terminal                   |  Basic syntax | Syntax highlighting | Links | Images | Jump marks |
| :------------------------- | :-----------: | :-----------------: | :---: | :----: | :--------: |
| Basic ANSI                 | ✓             | ✓                   |       |        |            |
| Windows [ConEmu][]         | ✓             | ✓                   |       |        |            |
| Windows 10 console         | ✓             | ✓                   |       |        |            |
| Generic VTE 0.50 or newer¹ | ✓             | ✓                   | ✓     |        |            |
| [Terminology][]            | ✓             | ✓                   | ✓     | ✓      |            |
| [iTerm2][]                 | ✓             | ✓                   | ✓     | ✓ 2)   | ✓          |
| [kitty][]                  | ✓             | ✓                   | ✓     | ✓ 2)   |            |

1) VTE is Gnome’s terminal emulation library used by many popular terminal emulators on Linux, including Gnome Terminal, Xfce Terminal, Tilix, etc.
2) SVG images require `rsvg-convert` from librsvg.

Not supported:

* CommonMark extensions: [Footnotes][GH-1] and [tables][GH-2]
* [Re-filling paragraphs][GH-4]

[syntect]: https://github.com/trishume/syntect
[osc8]: https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda
[Terminology]: http://terminolo.gy
[ConEmu]: https://conemu.github.io
[iterm2]: https://www.iterm2.com

## Usage

Try `mdcat --help` or read the [mdcat(1)](./mdcat.1.adoc) manpage.

## Installation

* The [Releases] page provides pre-build binaries for Linux, macOS and Windows.
    * **Tip:** You can copy or hard-link `mdcat` to `mdless` for a variant of `mdcat` which paginates by default (like `mdcat -p`).
* 3rd party packages:
    * [Homebrew]: `brew install mdcat`
    * [Arch Linux]: `pacman -S mdcat`
    * Void Linux: `xbps-install -S mdcat`
    * Nixpkgs: `nix-env -i mdcat`
    * [Scoop]: `scoop install mdcat`
* You can also build `mdcat` manually with `cargo install mdcat`.

[Releases]: https://github.com/lunaryorn/mdcat/releases
[Homebrew]: https://brew.sh
[Arch Linux]: https://www.archlinux.org/packages/community/x86_64/mdcat/
[scoop]: https://github.com/lukesampson/scoop

## Future plans

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
