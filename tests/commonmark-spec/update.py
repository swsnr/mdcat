#!/usr/bin/env python3
# Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# 	http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.


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