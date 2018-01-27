#!/usr/bin/env python2.7
# -*- coding: utf8 -*-
# Copyright 2018 Sebastian Wiesner <sebastian@swsnr.de>

# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at

#	http://www.apache.org/licenses/LICENSE-2.0

# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
# License for the specific language governing permissions and limitations under
# the License.

from __future__ import print_function

import re
import sys
import os
from datetime import date
from subprocess import check_call


def is_prerelease(version):
    # Dumb but sufficient for our purposes
    return '-' in version


CHANGELOG = 'CHANGELOG.md'


def update_changelog(version):
    with open(CHANGELOG) as source:
        changelog = source.read()

    today = date.today().isoformat()
    with_new_version = re.sub(
        r'## \[Unreleased\]\n',
        "## [Unreleased]\n\n## [{0}] – {1}\n".format(version, today),
        changelog)
    with open(CHANGELOG, 'w') as sink:
        sink.write(with_new_version)

    # Add the changelog to Git index, for cargo release to commit it
    check_call(['git', 'add', CHANGELOG])


def main():
    previous_version = os.environ['PREV_VERSION']
    next_version = os.environ['NEW_VERSION']
    dry_run = os.environ.get('DRY_RUN') == 'true'

    if not dry_run:
        if not is_prerelease(next_version):
            update_changelog(next_version)

        # Make sure that we include all changes to the lockfile
        check_call(['git', 'add', 'Cargo.lock'])
    else:
        print('DRY RUN; skipping changelog update')


if __name__ == '__main__':
    main()
