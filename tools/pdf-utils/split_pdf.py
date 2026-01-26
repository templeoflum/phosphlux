#!/usr/bin/env python3
"""
PDF Utilities for Research Workflow

Handles large PDFs that exceed Claude's 20MB limit by:
- Splitting into smaller page ranges
- Extracting specific pages
- Converting pages to images
- Basic compression
"""

import argparse
import os
import sys
from pathlib import Path

try:
    import pypdf
    from pypdf import PdfReader, PdfWriter
    HAS_PYPDF = True
except ImportError:
    HAS_PYPDF = False

try:
    import fitz  # PyMuPDF
    HAS_PYMUPDF = True
except ImportError:
    HAS_PYMUPDF = False


def get_pdf_info(pdf_path: str) -> dict:
    """Get basic info about a PDF."""
    path = Path(pdf_path)
    size_mb = path.stat().st_size / (1024 * 1024)

    info = {
        "path": str(path),
        "size_mb": round(size_mb, 2),
        "exceeds_limit": size_mb > 20,
    }

    if HAS_PYPDF:
        reader = PdfReader(pdf_path)
        info["page_count"] = len(reader.pages)
        info["suggested_splits"] = max(1, int(size_mb / 15))  # ~15MB chunks
    elif HAS_PYMUPDF:
        doc = fitz.open(pdf_path)
        info["page_count"] = len(doc)
        info["suggested_splits"] = max(1, int(size_mb / 15))
        doc.close()

    return info


def split_pdf(pdf_path: str, output_dir: str = None, pages_per_split: int = None) -> list:
    """
    Split a PDF into smaller files.

    Args:
        pdf_path: Path to the PDF
        output_dir: Directory for output files (default: same as input)
        pages_per_split: Pages per output file (default: auto-calculated)

    Returns:
        List of output file paths
    """
    if not HAS_PYPDF:
        raise ImportError("pypdf required: pip install pypdf")

    path = Path(pdf_path)
    output_dir = Path(output_dir) if output_dir else path.parent
    output_dir.mkdir(parents=True, exist_ok=True)

    reader = PdfReader(pdf_path)
    total_pages = len(reader.pages)

    # Auto-calculate pages per split to stay under 15MB
    if pages_per_split is None:
        size_mb = path.stat().st_size / (1024 * 1024)
        mb_per_page = size_mb / total_pages
        pages_per_split = max(1, int(15 / mb_per_page))

    output_files = []

    for i in range(0, total_pages, pages_per_split):
        writer = PdfWriter()
        end_page = min(i + pages_per_split, total_pages)

        for page_num in range(i, end_page):
            writer.add_page(reader.pages[page_num])

        output_name = f"{path.stem}_pages_{i+1}-{end_page}.pdf"
        output_path = output_dir / output_name

        with open(output_path, "wb") as f:
            writer.write(f)

        output_files.append(str(output_path))
        print(f"Created: {output_name} ({end_page - i} pages)")

    return output_files


def extract_pages(pdf_path: str, start: int, end: int, output_path: str = None) -> str:
    """Extract a range of pages to a new PDF."""
    if not HAS_PYPDF:
        raise ImportError("pypdf required: pip install pypdf")

    path = Path(pdf_path)
    reader = PdfReader(pdf_path)
    writer = PdfWriter()

    # Convert to 0-indexed
    for page_num in range(start - 1, min(end, len(reader.pages))):
        writer.add_page(reader.pages[page_num])

    if output_path is None:
        output_path = path.parent / f"{path.stem}_pages_{start}-{end}.pdf"

    with open(output_path, "wb") as f:
        writer.write(f)

    print(f"Extracted pages {start}-{end} to: {output_path}")
    return str(output_path)


def pages_to_images(pdf_path: str, output_dir: str = None,
                    pages: list = None, dpi: int = 150) -> list:
    """
    Convert PDF pages to images.

    Args:
        pdf_path: Path to PDF
        output_dir: Output directory (default: pdf_name_images/)
        pages: List of page numbers (1-indexed), None for all
        dpi: Resolution (150 is good balance of size/quality)

    Returns:
        List of image paths
    """
    if not HAS_PYMUPDF:
        raise ImportError("PyMuPDF required: pip install pymupdf")

    path = Path(pdf_path)
    output_dir = Path(output_dir) if output_dir else path.parent / f"{path.stem}_images"
    output_dir.mkdir(parents=True, exist_ok=True)

    doc = fitz.open(pdf_path)
    image_paths = []

    # Default to all pages
    if pages is None:
        pages = list(range(1, len(doc) + 1))

    zoom = dpi / 72  # 72 is default PDF DPI
    matrix = fitz.Matrix(zoom, zoom)

    for page_num in pages:
        page = doc[page_num - 1]  # 0-indexed
        pix = page.get_pixmap(matrix=matrix)

        output_path = output_dir / f"page_{page_num:03d}.png"
        pix.save(str(output_path))
        image_paths.append(str(output_path))
        print(f"Saved: {output_path.name}")

    doc.close()
    return image_paths


def main():
    parser = argparse.ArgumentParser(description="PDF utilities for research workflow")
    subparsers = parser.add_subparsers(dest="command", required=True)

    # Info command
    info_parser = subparsers.add_parser("info", help="Get PDF info")
    info_parser.add_argument("pdf", help="Path to PDF")

    # Split command
    split_parser = subparsers.add_parser("split", help="Split PDF into smaller files")
    split_parser.add_argument("pdf", help="Path to PDF")
    split_parser.add_argument("-o", "--output-dir", help="Output directory")
    split_parser.add_argument("-p", "--pages", type=int, help="Pages per split")

    # Extract command
    extract_parser = subparsers.add_parser("extract", help="Extract page range")
    extract_parser.add_argument("pdf", help="Path to PDF")
    extract_parser.add_argument("start", type=int, help="Start page (1-indexed)")
    extract_parser.add_argument("end", type=int, help="End page (1-indexed)")
    extract_parser.add_argument("-o", "--output", help="Output path")

    # Images command
    images_parser = subparsers.add_parser("images", help="Convert pages to images")
    images_parser.add_argument("pdf", help="Path to PDF")
    images_parser.add_argument("-o", "--output-dir", help="Output directory")
    images_parser.add_argument("-p", "--pages", type=int, nargs="+", help="Page numbers")
    images_parser.add_argument("-d", "--dpi", type=int, default=150, help="DPI (default: 150)")

    args = parser.parse_args()

    if args.command == "info":
        info = get_pdf_info(args.pdf)
        print(f"File: {info['path']}")
        print(f"Size: {info['size_mb']} MB")
        print(f"Exceeds 20MB limit: {info['exceeds_limit']}")
        if 'page_count' in info:
            print(f"Pages: {info['page_count']}")
            print(f"Suggested splits: {info['suggested_splits']}")

    elif args.command == "split":
        split_pdf(args.pdf, args.output_dir, args.pages)

    elif args.command == "extract":
        extract_pages(args.pdf, args.start, args.end, args.output)

    elif args.command == "images":
        pages_to_images(args.pdf, args.output_dir, args.pages, args.dpi)


if __name__ == "__main__":
    main()
