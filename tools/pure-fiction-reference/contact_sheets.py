#!/usr/bin/env python3

import argparse
import pathlib

from PIL import Image, ImageDraw

parser = argparse.ArgumentParser()
parser.add_argument("--input", type=pathlib.Path, required=True)
parser.add_argument("--output", type=pathlib.Path, required=True)
args = parser.parse_args()
args.output.mkdir(parents=True, exist_ok=False)
pages = sorted(args.input.glob("*.png"))
assert len(pages) == 37, len(pages)
for contact_index, start in enumerate(range(0, len(pages), 8), start=1):
    selected = pages[start:start + 8]
    thumbs = []
    for page in selected:
        image = Image.open(page).convert("RGB")
        width = 700
        height = round(image.height * width / image.width)
        thumbs.append((page.stem, image.resize((width, height))))
    cell_height = max(image.height for _, image in thumbs) + 24
    contact = Image.new("RGB", (1400, cell_height * 4), "white")
    draw = ImageDraw.Draw(contact)
    for offset, (label, image) in enumerate(thumbs):
        x = (offset % 2) * 700
        y = (offset // 2) * cell_height
        draw.text((x + 6, y + 4), label, fill="black")
        contact.paste(image, (x, y + 24))
    contact.save(args.output / f"contact-{contact_index:02d}.png")
print("Rendered 37 pages into five contact sheets.")
