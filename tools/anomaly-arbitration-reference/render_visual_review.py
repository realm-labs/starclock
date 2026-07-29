#!/usr/bin/env python3
"""Render Goal 13 review PDFs and compose labeled contact sheets."""

from __future__ import annotations

import argparse
from pathlib import Path

import pypdfium2 as pdfium
from PIL import Image, ImageDraw


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    source = args.input.resolve()
    output = args.output.resolve()
    if output.exists():
        raise FileExistsError(f"refusing to overwrite rendered review {output}")
    pages = output / "pages"
    contacts = output / "contacts"
    pages.mkdir(parents=True)
    contacts.mkdir()
    rendered: list[tuple[str, Path]] = []
    for pdf_path in sorted(source.glob("AnomalyArbitration*.pdf")):
        document = pdfium.PdfDocument(str(pdf_path))
        for index in range(len(document)):
            image = document[index].render(scale=1.25).to_pil().convert("RGB")
            label = f"{pdf_path.stem} page {index + 1:02d}"
            target = pages / f"{pdf_path.stem}-{index + 1:02d}.png"
            image.save(target)
            rendered.append((label, target))
    if len(rendered) != 37:
        raise ValueError(f"expected 37 rendered pages, got {len(rendered)}")
    per_contact = 8
    thumb_width = 900
    label_height = 28
    for contact_index, start in enumerate(
        range(0, len(rendered), per_contact), start=1
    ):
        selected = rendered[start : start + per_contact]
        thumbnails: list[tuple[str, Image.Image]] = []
        for label, image_path in selected:
            page = Image.open(image_path).convert("RGB")
            height = round(page.height * thumb_width / page.width)
            thumbnails.append((label, page.resize((thumb_width, height))))
        cell_height = max(page.height for _, page in thumbnails) + label_height
        contact = Image.new("RGB", (thumb_width * 2, cell_height * 4), "white")
        draw = ImageDraw.Draw(contact)
        for offset, (label, page) in enumerate(thumbnails):
            x = (offset % 2) * thumb_width
            y = (offset // 2) * cell_height
            draw.text((x + 8, y + 6), label, fill="black")
            contact.paste(page, (x, y + label_height))
        contact.save(contacts / f"contact-{contact_index:02d}.png")
    print(
        f"Rendered {len(rendered)} PDF pages into "
        f"{(len(rendered) + per_contact - 1) // per_contact} contact sheets."
    )


if __name__ == "__main__":
    main()
