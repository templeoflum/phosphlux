"""
Datasheet Retriever for video synthesis research.

Searches and downloads IC datasheets from:
- AllDatasheet (primary, modern parts)
- DatasheetArchive (obsolete parts)
- BITSAVERS (historical PDFs)
- Direct manufacturer links (RCA databook for CA30xx)
"""

import re
import json
import logging
from datetime import datetime
from pathlib import Path
from typing import Optional, List, Dict, Any
from urllib.parse import quote_plus, urljoin, urlparse

try:
    from bs4 import BeautifulSoup
    HAS_BS4 = True
except ImportError:
    HAS_BS4 = False

from .base import BaseRetriever
from .artifacts import DatasheetArtifact, DatasheetResult


logger = logging.getLogger(__name__)


# Known datasheet sources for specific part families
KNOWN_SOURCES = {
    # RCA CA30xx series - common in video synthesis
    "CA3046": "http://www.bitsavers.org/components/rca/linear/CA3046_Transistor_Array.pdf",
    "CA3080": "http://www.bitsavers.org/components/rca/linear/CA3080_OTA.pdf",
    "CA3086": "http://www.bitsavers.org/components/rca/linear/CA3086_Transistor_Array.pdf",
    "CA3019": "http://www.bitsavers.org/components/rca/linear/CA3019_Diode_Array.pdf",
    "CA3054": "http://www.bitsavers.org/components/rca/linear/CA3054_Dual_Diff_Amp.pdf",

    # Other common video ICs
    "LM1881": "https://www.ti.com/lit/ds/symlink/lm1881.pdf",
    "AD8561": "https://www.analog.com/media/en/technical-documentation/data-sheets/AD8561.pdf",
    "EL2020": "https://www.renesas.com/en/document/dst/el2020-datasheet",
}


class DatasheetRetriever(BaseRetriever):
    """Retriever for IC datasheets."""

    artifact_type = "datasheet"
    output_dir_name = "datasheets"

    # Source URLs
    ALLDATASHEET_BASE = "https://www.alldatasheet.com"
    DATASHEET_ARCHIVE_BASE = "https://www.datasheetarchive.com"
    BITSAVERS_BASE = "http://bitsavers.org"

    def __init__(self, **kwargs):
        """Initialize datasheet retriever."""
        super().__init__(**kwargs)

        if not HAS_BS4:
            logger.warning(
                "BeautifulSoup not installed. HTML parsing will be limited. "
                "Install with: pip install beautifulsoup4"
            )

    def search(
        self,
        part_number: str,
        manufacturer: Optional[str] = None,
        max_results: int = 10,
    ) -> List[DatasheetResult]:
        """
        Search for datasheets.

        Args:
            part_number: Part number to search for (e.g., "CA3046")
            manufacturer: Optional manufacturer filter
            max_results: Maximum results to return

        Returns:
            List of DatasheetResult objects
        """
        results = []

        # Normalize part number
        part_number = part_number.upper().strip()

        # Check known sources first
        if part_number in KNOWN_SOURCES:
            results.append(DatasheetResult(
                title=f"{part_number} Datasheet",
                url=KNOWN_SOURCES[part_number],
                source="known_source",
                snippet="Direct manufacturer or archive link",
                part_number=part_number,
                relevance=1.0,
            ))

        # Search AllDatasheet
        alldatasheet_results = self._search_alldatasheet(part_number)
        results.extend(alldatasheet_results)

        # Search DatasheetArchive (better for obsolete parts)
        archive_results = self._search_datasheet_archive(part_number)

        # Deduplicate
        seen_urls = {r.url for r in results}
        for r in archive_results:
            if r.url not in seen_urls:
                results.append(r)
                seen_urls.add(r.url)

        # Filter by manufacturer if specified
        if manufacturer:
            manufacturer_lower = manufacturer.lower()
            results = [
                r for r in results
                if not r.manufacturer or manufacturer_lower in r.manufacturer.lower()
            ]

        return results[:max_results]

    def _search_alldatasheet(self, part_number: str) -> List[DatasheetResult]:
        """Search AllDatasheet.com."""
        results = []

        search_url = f"{self.ALLDATASHEET_BASE}/view.jsp?Searchword={quote_plus(part_number)}"

        logger.info(f"Searching AllDatasheet: {part_number}")

        response = self._get_sync(search_url)
        if not response or not HAS_BS4:
            return results

        soup = BeautifulSoup(response.text, "html.parser")

        # Find result table
        for row in soup.select("table.main tr"):
            try:
                cells = row.select("td")
                if len(cells) < 3:
                    continue

                # Extract part number link
                part_link = cells[0].select_one("a")
                if not part_link:
                    continue

                part_text = part_link.get_text(strip=True)
                href = part_link.get("href", "")

                if not href or not part_text:
                    continue

                # Extract manufacturer
                mfr_text = cells[1].get_text(strip=True) if len(cells) > 1 else ""

                # Extract description
                desc_text = cells[2].get_text(strip=True) if len(cells) > 2 else ""

                results.append(DatasheetResult(
                    title=f"{part_text} - {desc_text[:50]}" if desc_text else part_text,
                    url=urljoin(self.ALLDATASHEET_BASE, href),
                    source="alldatasheet",
                    snippet=desc_text[:200],
                    part_number=part_text,
                    manufacturer=mfr_text,
                ))

            except Exception as e:
                logger.debug(f"Failed to parse row: {e}")

        return results

    def _search_datasheet_archive(self, part_number: str) -> List[DatasheetResult]:
        """Search DatasheetArchive.com (good for obsolete parts)."""
        results = []

        search_url = f"{self.DATASHEET_ARCHIVE_BASE}/search?q={quote_plus(part_number)}"

        logger.info(f"Searching DatasheetArchive: {part_number}")

        response = self._get_sync(search_url)
        if not response or not HAS_BS4:
            return results

        soup = BeautifulSoup(response.text, "html.parser")

        # Find search results
        for item in soup.select("div.search-result, article.result"):
            try:
                link = item.select_one("a[href*='/pdf/']")
                if not link:
                    continue

                href = link.get("href", "")
                title_text = link.get_text(strip=True)

                # Extract manufacturer if present
                mfr_elem = item.select_one(".manufacturer, .mfr")
                mfr_text = mfr_elem.get_text(strip=True) if mfr_elem else ""

                # Extract snippet
                snippet_elem = item.select_one(".snippet, .description")
                snippet_text = snippet_elem.get_text(strip=True) if snippet_elem else ""

                results.append(DatasheetResult(
                    title=title_text or part_number,
                    url=urljoin(self.DATASHEET_ARCHIVE_BASE, href),
                    source="datasheet_archive",
                    snippet=snippet_text[:200],
                    part_number=part_number,
                    manufacturer=mfr_text,
                ))

            except Exception as e:
                logger.debug(f"Failed to parse result: {e}")

        return results

    def download(
        self,
        url_or_part: str,
        part_number: Optional[str] = None,
        **kwargs
    ) -> Optional[DatasheetArtifact]:
        """
        Download a datasheet.

        Args:
            url_or_part: Either a direct URL or a part number
            part_number: Optional part number (if url_or_part is a URL)

        Returns:
            DatasheetArtifact if successful, None otherwise
        """
        # Determine if input is URL or part number
        if url_or_part.startswith("http"):
            url = url_or_part
            if not part_number:
                # Try to extract part number from URL
                match = re.search(r"([A-Z]{2,3}[0-9]{3,5}[A-Z0-9]*)", url, re.IGNORECASE)
                part_number = match.group(1).upper() if match else "unknown"
        else:
            # It's a part number, search for it
            part_number = url_or_part.upper().strip()

            # Check known sources
            if part_number in KNOWN_SOURCES:
                url = KNOWN_SOURCES[part_number]
            else:
                # Search and get first result
                results = self.search(part_number, max_results=1)
                if not results:
                    logger.error(f"No datasheet found for: {part_number}")
                    return None
                url = results[0].url

        logger.info(f"Downloading datasheet: {part_number} from {url}")

        # Create output directory
        output_dir = self.get_artifact_dir(part_number)
        output_dir.mkdir(parents=True, exist_ok=True)

        # Handle different source types
        pdf_path = output_dir / "datasheet.pdf"

        if not pdf_path.exists():
            # AllDatasheet requires navigating through a landing page
            if "alldatasheet" in url:
                pdf_url = self._resolve_alldatasheet_pdf(url)
                if pdf_url:
                    self._download_file_sync(pdf_url, pdf_path)
            else:
                # Direct download
                self._download_file_sync(url, pdf_path)

        if not pdf_path.exists():
            logger.error(f"Failed to download datasheet PDF for {part_number}")
            return None

        # Create metadata
        metadata = {
            "part_number": part_number,
            "source_url": url,
            "retrieved_at": datetime.now().isoformat(),
        }

        # Try to extract more metadata from the page
        if "alldatasheet" in url and HAS_BS4:
            page_metadata = self._extract_alldatasheet_metadata(url)
            metadata.update(page_metadata)

        self._save_metadata(output_dir, metadata)

        # Create artifact
        artifact = DatasheetArtifact(
            id=f"datasheet:{part_number}",
            name=f"{part_number} Datasheet",
            path=pdf_path,
            source_url=url,
            retrieved_at=datetime.now(),
            metadata=metadata,
            part_number=part_number,
            manufacturer=metadata.get("manufacturer", ""),
            description=metadata.get("description", ""),
            package_types=metadata.get("package_types", []),
        )

        # Track in provenance database
        self._track_artifact(artifact)

        logger.info(f"Downloaded datasheet: {part_number} -> {output_dir}")
        return artifact

    def _resolve_alldatasheet_pdf(self, page_url: str) -> Optional[str]:
        """Resolve actual PDF URL from AllDatasheet landing page."""
        response = self._get_sync(page_url)
        if not response or not HAS_BS4:
            return None

        soup = BeautifulSoup(response.text, "html.parser")

        # Look for PDF download link
        pdf_link = soup.select_one("a[href*='.pdf'], a[href*='pdf-down']")
        if pdf_link:
            href = pdf_link.get("href", "")
            return urljoin(page_url, href)

        # Look for embedded iframe with PDF
        iframe = soup.select_one("iframe[src*='.pdf']")
        if iframe:
            return iframe.get("src")

        return None

    def _extract_alldatasheet_metadata(self, page_url: str) -> Dict[str, Any]:
        """Extract metadata from AllDatasheet page."""
        metadata = {}

        response = self._get_sync(page_url)
        if not response or not HAS_BS4:
            return metadata

        soup = BeautifulSoup(response.text, "html.parser")

        # Manufacturer
        mfr_elem = soup.select_one("td:contains('Manufacturer') + td, .manufacturer")
        if mfr_elem:
            metadata["manufacturer"] = mfr_elem.get_text(strip=True)

        # Description
        desc_elem = soup.select_one("td:contains('Description') + td, .description")
        if desc_elem:
            metadata["description"] = desc_elem.get_text(strip=True)

        # Package types
        pkg_elem = soup.select_one("td:contains('Package') + td, .package")
        if pkg_elem:
            pkg_text = pkg_elem.get_text(strip=True)
            metadata["package_types"] = [p.strip() for p in pkg_text.split(",")]

        return metadata

    def extract_specs(self, datasheet_path: Path) -> Dict[str, Any]:
        """
        Extract specifications from a datasheet PDF.

        This is a placeholder for AI-assisted extraction.
        Actual implementation would use PDF parsing and/or
        vision models to extract key specifications.

        Args:
            datasheet_path: Path to datasheet PDF or directory

        Returns:
            Dictionary of extracted specifications
        """
        if datasheet_path.is_dir():
            pdf_path = datasheet_path / "datasheet.pdf"
            specs_path = datasheet_path / "specs.json"
        else:
            pdf_path = datasheet_path
            specs_path = datasheet_path.parent / "specs.json"

        # Check for existing specs
        if specs_path.exists():
            with open(specs_path) as f:
                return json.load(f)

        specs = {
            "extracted": False,
            "note": "Spec extraction requires AI assistance or manual entry",
        }

        if pdf_path.exists():
            specs["pdf_path"] = str(pdf_path)
            specs["file_size"] = pdf_path.stat().st_size

        return specs

    def get_part_family(self, part_number: str) -> List[str]:
        """
        Get related parts in the same family.

        Args:
            part_number: Part number to find family for

        Returns:
            List of related part numbers
        """
        # Extract base part number
        match = re.match(r"([A-Z]+[0-9]+)", part_number.upper())
        if not match:
            return []

        base = match.group(1)

        # Common suffix patterns
        suffixes = ["", "A", "B", "C", "E", "N", "S", "T"]
        family = [f"{base}{s}" for s in suffixes]

        return [p for p in family if p != part_number.upper()]


# CLI interface
def main():
    """Command-line interface for datasheet retrieval."""
    import argparse

    parser = argparse.ArgumentParser(description="Datasheet Retriever")
    subparsers = parser.add_subparsers(dest="command", help="Command")

    # Search command
    search_parser = subparsers.add_parser("search", help="Search for datasheets")
    search_parser.add_argument("part_number", help="Part number to search")
    search_parser.add_argument("--manufacturer", "-m", help="Manufacturer filter")
    search_parser.add_argument("--max", "-n", type=int, default=10, help="Max results")

    # Download command
    download_parser = subparsers.add_parser("download", help="Download a datasheet")
    download_parser.add_argument("part", help="Part number or URL")

    # Family command
    family_parser = subparsers.add_parser("family", help="Get part family")
    family_parser.add_argument("part_number", help="Part number")

    args = parser.parse_args()

    retriever = DatasheetRetriever()

    if args.command == "search":
        results = retriever.search(
            args.part_number,
            manufacturer=args.manufacturer,
            max_results=args.max,
        )

        print(f"\nFound {len(results)} datasheets:\n")
        for r in results:
            print(f"  {r.part_number}")
            if r.manufacturer:
                print(f"    Manufacturer: {r.manufacturer}")
            print(f"    Source: {r.source}")
            print(f"    URL: {r.url}\n")

    elif args.command == "download":
        artifact = retriever.download(args.part)
        if artifact:
            print(f"\nDownloaded: {artifact.part_number}")
            print(f"  Path: {artifact.path}")
            if artifact.manufacturer:
                print(f"  Manufacturer: {artifact.manufacturer}")
        else:
            print(f"\nFailed to download datasheet: {args.part}")

    elif args.command == "family":
        family = retriever.get_part_family(args.part_number)
        print(f"\nPart family for {args.part_number}:")
        for p in family:
            print(f"  {p}")

    else:
        parser.print_help()


if __name__ == "__main__":
    main()
