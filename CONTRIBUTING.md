# Contributing to mdcat

Thank you for your interest in this little tool.  I appreciate all kinds of
contribution; feel free to report issues or open pull requests, but please
understand that I address issues or merge pull requests at my sole discretion.
I make no promises whatsoever about whether I add your changes or wishes to
mdcat.

## Maintainer documentation

### Make a release

1. Check whether `~/.cargo/credentials` contains a valid API token for
   <https://crates.io>.  If it doesn't, run `cargo login` and follow the
   instructions.
2. Install [cargo-release][] 0.9 or newer if not already present, with `cargo
   install cargo-release`.
3. Run `cargo release` and follow the instructions.

[cargo-release]: https://github.com/sunng87/cargo-release
