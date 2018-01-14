# mdless

[![Current release]( https://img.shields.io/crates/v/mdless.svg)][crates]
![Maintenance as is](https://img.shields.io/badge/maintenace-as--is-yellow.svg)
[![Build status](https://img.shields.io/travis/lunaryorn/mdless.rs/master.svg)][travis]

Less for markdown.

[crates-badge]: https://img.shields.io/crates/v/mdless.svg
[crates]: https://crates.io/crates/mdless
[travis]: https://travis-ci.org/lunaryorn/mdless.rs

## Status and future plans

- [x] Inline formatting, with proper nesting of emphasis
- [x] Headings
- [x] Block quotes
- [x] Code blocks
- [x] Ordered lists
- [x] Numbered lists
- [x] Nested lists
- [x] Links
- [ ] Syntax highlighting for code blocks
- [ ] Show inline HTML and block HTML literally
- [ ] iTerm2 integration: Set marks for headings to jump back and forth
- [ ] iTerm2 integration: Show images inline
- [ ] Figure out a better way to show HTML [GH-3](https://github.com/lunaryorn/mdless/issues/3)
- [ ] CommonMark extensions: Footnotes [GH-1](https://github.com/lunaryorn/mdless/issues/1)
- [ ] CommonMark extensions: Tables [GH-2](https://github.com/lunaryorn/mdless/issues/2)
- [ ] Ignore soft wraps and wrap inline text a column limit instead [GH-4](https://github.com/lunaryorn/mdless/issues/4)

## License

Copyright 2018 Sebastian Wiesner <sebastian@swsnr.de>

Licensed under the Apache License, Version 2.0 (the "License"); you may not use
this file except in compliance with the License. You may obtain a copy of the
License at <http://www.apache.org/licenses/LICENSE-2.0>.

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the
specific language governing permissions and limitations under the License.