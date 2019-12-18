#!/bin/bash
# Copyright 2019 Sebastian Wiesnser <sebastian@swsnr.de>

# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at

#   http://www.apache.org/licenses/LICENSE-2.0

# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

set -e

next_version="$1"
if [[ -z "${next_version}" ]]; then
    echo "Next version missing; aborting"
    exit 1
fi

changes=""
#changes="$(git status --porcelain)"
if [[ -n "${changes}" ]]; then
    git status
    echo "Working directory not clean; aborting"
    exit 1
fi

if [[ "$(git symbolic-ref --short HEAD)" != "master" ]]; then
    echo "Not on master branch; aborting"
    exit 1
fi

# TODO: Check status of HEAD commit

latest_version="$(git tag --sort '-v:refname' | grep '^mdcat-' | head -n1 | cut -d'-' -f2)"

# Substitute version in Cargo.toml
sed -i '' "1,/^version =/ s/^version = .*$/version = \"$next_version\"/" Cargo.toml
# Make cargo update the version in the lockfile as well
cargo metadata --format-version 1 > /dev/null
# Add both files to Git
git add Cargo.toml Cargo.lock

# Update the changelog:
# 1. Append a headline for the current version right after [Unreleased] headline
# 2. Append link references for the new version and the new unreleased version
# 3. Delete the old link reference for the Unreleased header
sed -e "/^## \[Unreleased\]\$/ a\\
\\
## [${next_version}] – $(date +%Y-%m-%d)\\" \
    -e "\$a\\
[$next_version]: https://github.com/lunaryorn/mdcat/compare/mdcat-${latest_version}...mdcat-${next_version}\\
[Unreleased]: https://github.com/lunaryorn/mdcat/compare/mdcat-${next_version}...HEAD" \
    -e '/^\[Unreleased\]:/ D' \
    -i '' CHANGELOG.md
git add CHANGELOG.md

git commit -m "Release $next_version"
git tag -m "mdcat $next_version" "mdcat-$next_version"
cargo publish --no-verify
git push --follow-tags origin master

