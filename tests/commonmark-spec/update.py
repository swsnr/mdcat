#!/usr/bin/env python3
# Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.


import json
from pathlib import Path
from urllib.request import urlopen


SPEC_VERSION = '0.29'
SPEC_URL = f'https://spec.commonmark.org/{SPEC_VERSION}/spec.json'

DIR = Path(__file__).parent.absolute()


def main():
    for file in DIR.glob('*.md'):
        file.unlink()
    for file in DIR.glob('*.expected'):
        file.unlink()
    for file in DIR.glob('*.actual'):
        file.unlink()

    with urlopen(SPEC_URL) as source:
        examples = json.load(source)
    for example in examples:
        number = example['example']
        section = example['section'].lower().replace(' ', '_')
        filename = f'{number:0>3}-{section}.md'
        (DIR / filename).write_text(example['markdown'], encoding='utf-8')


if __name__ == '__main__':
    main()