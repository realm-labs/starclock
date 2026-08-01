#!/usr/bin/env python3

import argparse
import pathlib

from author_workbooks import normalize_xlsx

parser = argparse.ArgumentParser()
parser.add_argument("directory", type=pathlib.Path)
args = parser.parse_args()
templates = sorted(args.directory.glob("*.xlsx"))
assert [path.name for path in templates] == [
    "PureFiction.xlsx", "PureFictionBindings.xlsx", "PureFictionReview.xlsx"
]
for template in templates:
    normalize_xlsx(template)
print("Normalized three Sora template archives.")
