# mdcat

Fancy `cat` for Markdown (that is, [CommonMark][]):

```
$ mdcat sample.md
```

![mdcat showcase with different colour themes][sxs]

mdcat in [WezTerm], with "One Light (base16)", "Gruvbox Light", and "Darcula
(base16)" (from left to right), and [JetBrains Mono] as font.

[CommonMark]: http://commonmark.org
[Solarized]: http://ethanschoonover.com/solarized
[dracula]: https://draculatheme.com/iterm/
[wezterm]: https://wezfurlong.org/wezterm/
[JetBrains Mono]: https://www.jetbrains.com/lp/mono/
[sxs]: ./screenshots/side-by-side.png

## Features

`mdcat` works best with [iTerm2] or [Kitty], and a good terminal font with italic characters.
Then it

* nicely renders all basic CommonMark syntax (no [tables][#2] or [footnotes][#1] though),
* highlights code blocks with [syntect],
* shows [links][osc8], and also images inline in supported terminals (see above, where "Rust" is a clickable link!),
* adds jump marks for headings in [iTerm2] (jump forwards and backwards with <key>⇧⌘↓</key> and <key>⇧⌘↑</key>).

| Terminal                   |  Basic syntax | Syntax highlighting | Images | Jump marks |
| :------------------------- | :-----------: | :-----------------: | :----: | :--------: |
| Basic ANSI                 | ✓             | ✓                   |        |            |
| Windows [ConEmu][]         | ✓             | ✓                   |        |            |
| Windows 10 console         | ✓             | ✓                   |        |            |
| Generic VTE 0.50 or newer¹ | ✓             | ✓                   |        |            |
| [Terminology][]            | ✓             | ✓                   | ✓      |            |
| [iTerm2][]                 | ✓             | ✓                   | ✓ 2)   | ✓          |
| [kitty][]                  | ✓             | ✓                   | ✓ 2)   |            |
| [WezTerm][]                | ✓             | ✓                   | ✓ 2)   |            |
| [foot][]                   | ✓             | ✓                   |        |            |

1) VTE is Gnome’s terminal emulation library used by many popular terminal emulators on Linux, including Gnome Terminal, Xfce Terminal, Tilix, etc.
2) SVG images are rendered with [resvg], see [SVG support].

Not supported:

* CommonMark extensions: [Footnotes][#1] and [tables][#2]

[syntect]: https://github.com/trishume/syntect
[osc8]: https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda
[Terminology]: http://terminolo.gy
[ConEmu]: https://conemu.github.io
[iterm2]: https://www.iterm2.com
[WezTerm]: https://wezfurlong.org/wezterm/
[foot]: https://codeberg.org/dnkl/foot/
[resvg]: https://github.com/RazrFalcon/resvg
[SVG support]: https://github.com/RazrFalcon/resvg#svg-support

## Usage

Try `mdcat --help` or read the [mdcat(1)](./mdcat.1.adoc) manpage.

## Installation

* 3rd party packages:
    * [Homebrew]: `brew install mdcat`
    * [MacPorts]: `sudo port install mdcat`
    * [Arch Linux]: `pacman -S mdcat`
    * Void Linux: `xbps-install -S mdcat`
    * Nixpkgs: `nix-env -i mdcat`
    * [Scoop]: `scoop install mdcat`
    * [Chocolatey]: `choco install mdcat`
* You can also build `mdcat` manually with `cargo install mdcat`.

[Homebrew]: https://formulae.brew.sh/formula/mdcat#default
[MacPorts]: https://ports.macports.org/port/mdcat/
[Arch Linux]: https://www.archlinux.org/packages/community/x86_64/mdcat/
[scoop]: https://scoop.sh/#/apps?q=mdcat
[chocolatey]: https://community.chocolatey.org/packages/mdcat

## Building

Run `cargo build --release`.
The resulting `mdcat` executable links against the system's SSL library, i.e. openssl on Linux.
To build a self-contained executable use `cargo build --features=static`; the resulting executable uses a pure Rust SSL implementation.
It still uses the system's CA roots however.

The build process also generates the following additional files in `$OUT_DIR`:

* Completions for Bash, Zsh, Fish, and Powershell, for both `mdcat` and `mdless`, in `completions` sub-directory.
* A `mdcat.1` manpage, build from `mdcat.1.adoc` with the `asciidoctor` command from [AsciiDoctor].
  If `asciidoctor` is not found the build script prints a warning.

These additional artifacts are included in the release builds.
If you package mdcat you may want to include these files too.

[AsciiDoctor]: https://asciidoctor.org/

## Troubleshooting

`mdcat` can output extensive tracing information when asked to.
Run `mdcat` with `$MDCAT_LOG=trace` for complete tracing information, or with `$MDCAT_LOG=mdcat::render=trace` to trace only rendering.

## Future plans

- [ ] Figure out a better way to show HTML [#3].
- [ ] CommonMark extensions: Footnotes [#1].
- [ ] CommonMark extensions: Tables [#2].

[#1]: https://github.com/swsnr/mdcat/issues/1
[#2]: https://github.com/swsnr/mdcat/issues/2
[#3]: https://github.com/swsnr/mdcat/issues/3
[#4]: https://github.com/swsnr/mdcat/issues/4

## License

Copyright Sebastian Wiesner <sebastian@swsnr.de>

Binaries are subject to the terms of the Mozilla Public
License, v. 2.0, see [LICENSE](LICENSE).

Most of the source is subject to the terms of the Mozilla Public
License, v. 2.0, see [LICENSE](LICENSE), unless otherwise noted;
some files are subject to the terms of the Apache 2.0 license,
see <http://www.apache.org/licenses/LICENSE-2.0>
