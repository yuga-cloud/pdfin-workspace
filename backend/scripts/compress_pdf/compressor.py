#!/usr/bin/env python3
"""Lossless PDF compaction for PDFin's optional PyMuPDF fast path."""

from __future__ import annotations

import argparse
from pathlib import Path

import pymupdf


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("quality", choices=("high", "medium", "low"))
    parser.add_argument("input", type=Path)
    parser.add_argument("output", type=Path)
    return parser.parse_args()


def main() -> None:
    args = parse_args()

    # Keep this path lossless: quality-specific lossy recompression remains
    # delegated to the Ghostscript fallback, while PyMuPDF provides a fast
    # object-stream / Flate compaction pass when available.
    garbage = 4 if args.quality == "low" else 3

    document = pymupdf.open(args.input)
    try:
        document.save(
            args.output,
            garbage=garbage,
            deflate=True,
            deflate_images=True,
            deflate_fonts=True,
            use_objstms=True,
        )
    finally:
        document.close()


if __name__ == "__main__":
    main()
