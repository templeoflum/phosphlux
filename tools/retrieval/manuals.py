"""
Manual Retriever for video synthesis research.

Searches and downloads operation/service manuals from:
- Archive.org (primary)
- Elektrotanya (service manuals)
- HiFi Engine (requires account)
- ManualsLib (user manuals)
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
from .artifacts import ManualArtifact, ManualResult


logger = logging.getLogger(__name__)


# Known manual sources for video equipment
KNOWN_SOURCES = {
    # Video synthesizers
    "fairlight-cvi": {
        "url": "https://archive.org/details/fairlight-cvi-manual",
        "manufacturer": "Fairlight",
        "type": "operation",
    },
    "sandin-ip-manual": {
        "url": "https://archive.org/details/sandin-image-processor-manual",
        "manufacturer": "Dan Sandin",
        "type": "operation",
    },

    # Video processors
    "sony-sev-1": {
        "url": "http://elektrotanya.com/sony_sev-1.pdf",
        "manufacturer": "Sony",
        "type": "service",
    },

    # Time base correctors
    "fortel-tbc": {
        "url": "https://archive.org/details/fortel-tbc-service-manual",
        "manufacturer": "Fortel",
        "type": "service",
    },
}


class ManualRetriever(BaseRetriever):
    """Retriever for operation and service manuals."""

    artifact_type = "manual"
    output_dir_name = "manuals"

    # Source URLs
    ARCHIVE_ORG_BASE = "https://archive.org"
    ELEKTROTANYA_BASE = "https://elektrotanya.com"
    HIFIENGINE_BASE = "https://www.hifiengine.com"
    MANUALSLIB_BASE = "https://www.manualslib.com"

    def __init__(self, **kwargs):
        """Initialize manual retriever."""
        super().__init__(**kwargs)

        if not HAS_BS4:
            logger.warning(
                "BeautifulSoup not installed. HTML parsing will be limited. "
                "Install with: pip install beautifulsoup4"
            )

    def search(
        self,
        device: str,
        manual_type: str = "any",
        manufacturer: Optional[str] = None,
        max_results: int = 10,
    ) -> List[ManualResult]:
        """
        Search for manuals.

        Args:
            device: Device name to search for
            manual_type: Type of manual ('service', 'operation', 'user', 'any')
            manufacturer: Optional manufacturer filter
            max_results: Maximum results to return

        Returns:
            List of ManualResult objects
        """
        results = []
        device_lower = device.lower()

        # Check known sources first
        for key, source in KNOWN_SOURCES.items():
            if device_lower in key.lower():
                if manual_type == "any" or source.get("type") == manual_type:
                    results.append(ManualResult(
                        title=f"{key.replace('-', ' ').title()} Manual",
                        url=source["url"],
                        source="known_source",
                        snippet=f"Type: {source.get('type', 'Unknown')}",
                        device=key,
                        manual_type=source.get("type", "unknown"),
                        relevance=1.0,
                    ))

        # Search Archive.org
        archive_results = self._search_archive_org(device, manual_type)
        results.extend(archive_results)

        # Search Elektrotanya for service manuals
        if manual_type in ("any", "service"):
            elektrotanya_results = self._search_elektrotanya(device)
            results.extend(elektrotanya_results)

        # Search ManualsLib for user manuals
        if manual_type in ("any", "user", "operation"):
            manualslib_results = self._search_manualslib(device)
            results.extend(manualslib_results)

        # Filter by manufacturer if specified
        if manufacturer:
            manufacturer_lower = manufacturer.lower()
            results = [
                r for r in results
                if manufacturer_lower in r.title.lower() or
                   manufacturer_lower in r.snippet.lower()
            ]

        # Deduplicate by URL
        seen_urls = set()
        unique_results = []
        for r in results:
            if r.url not in seen_urls:
                unique_results.append(r)
                seen_urls.add(r.url)

        return unique_results[:max_results]

    def _search_archive_org(
        self,
        device: str,
        manual_type: str
    ) -> List[ManualResult]:
        """Search Archive.org for manuals."""
        results = []

        # Build search query
        type_terms = {
            "service": "service manual",
            "operation": "operation manual OR user manual",
            "user": "user manual OR owner manual",
            "any": "manual",
        }
        type_query = type_terms.get(manual_type, "manual")

        search_url = f"{self.ARCHIVE_ORG_BASE}/advancedsearch.php"
        params = {
            "q": f'"{device}" AND ({type_query})',
            "fl[]": ["identifier", "title", "description", "creator"],
            "output": "json",
            "rows": 20,
        }

        response = self._get_sync(search_url, params=params)
        if not response:
            return results

        try:
            data = response.json()
            docs = data.get("response", {}).get("docs", [])

            for doc in docs:
                identifier = doc.get("identifier", "")
                title = doc.get("title", identifier)
                desc = doc.get("description", "")
                creator = doc.get("creator", "")

                if isinstance(desc, list):
                    desc = " ".join(desc)
                if isinstance(creator, list):
                    creator = ", ".join(creator)

                # Determine manual type from title/description
                detected_type = self._detect_manual_type(title, desc)

                results.append(ManualResult(
                    title=title,
                    url=f"{self.ARCHIVE_ORG_BASE}/details/{identifier}",
                    source="archive_org",
                    snippet=desc[:200] if desc else "",
                    device=device,
                    manual_type=detected_type,
                ))

        except Exception as e:
            logger.warning(f"Archive.org search error: {e}")

        return results

    def _search_elektrotanya(self, device: str) -> List[ManualResult]:
        """Search Elektrotanya for service manuals."""
        results = []

        search_url = f"{self.ELEKTROTANYA_BASE}/search.php"
        params = {"q": device}

        response = self._get_sync(search_url, params=params)
        if not response or not HAS_BS4:
            return results

        soup = BeautifulSoup(response.text, "html.parser")

        # Find search result links
        for link in soup.select("a[href*='.pdf']"):
            href = link.get("href", "")
            title = link.get_text(strip=True)

            if device.lower() in title.lower() or device.lower() in href.lower():
                results.append(ManualResult(
                    title=title or device,
                    url=urljoin(self.ELEKTROTANYA_BASE, href),
                    source="elektrotanya",
                    snippet="Service manual from Elektrotanya",
                    device=device,
                    manual_type="service",
                ))

        return results[:5]  # Limit results

    def _search_manualslib(self, device: str) -> List[ManualResult]:
        """Search ManualsLib for user manuals."""
        results = []

        search_url = f"{self.MANUALSLIB_BASE}/search"
        params = {"q": device}

        response = self._get_sync(search_url, params=params)
        if not response or not HAS_BS4:
            return results

        soup = BeautifulSoup(response.text, "html.parser")

        # Find search result items
        for item in soup.select("div.search-result, article.result"):
            link = item.select_one("a[href*='/manual/']")
            if not link:
                continue

            href = link.get("href", "")
            title = link.get_text(strip=True)

            # Extract manufacturer if present
            mfr_elem = item.select_one(".manufacturer, .brand")
            mfr = mfr_elem.get_text(strip=True) if mfr_elem else ""

            results.append(ManualResult(
                title=title or device,
                url=urljoin(self.MANUALSLIB_BASE, href),
                source="manualslib",
                snippet=f"Manufacturer: {mfr}" if mfr else "",
                device=device,
                manual_type="user",
            ))

        return results[:5]  # Limit results

    def _detect_manual_type(self, title: str, description: str) -> str:
        """Detect manual type from title and description."""
        text = f"{title} {description}".lower()

        if "service" in text or "repair" in text or "schematic" in text:
            return "service"
        elif "operation" in text or "operator" in text:
            return "operation"
        elif "user" in text or "owner" in text:
            return "user"
        else:
            return "unknown"

    def download(
        self,
        url_or_device: str,
        device: Optional[str] = None,
        manual_type: str = "service",
        **kwargs
    ) -> Optional[ManualArtifact]:
        """
        Download a manual.

        Args:
            url_or_device: Either a URL or a device name
            device: Device name (if url_or_device is a URL)
            manual_type: Type of manual to download

        Returns:
            ManualArtifact if successful, None otherwise
        """
        # Determine if input is URL or device name
        if url_or_device.startswith("http"):
            url = url_or_device
            if not device:
                device = self._extract_device_from_url(url)
        else:
            # It's a device name
            device = url_or_device.lower().replace(" ", "-")

            # Check known sources
            if device in KNOWN_SOURCES:
                source = KNOWN_SOURCES[device]
                url = source["url"]
                manual_type = source.get("type", manual_type)
            else:
                # Search for it
                results = self.search(device, manual_type=manual_type, max_results=1)
                if not results:
                    logger.error(f"No manual found for: {device}")
                    return None
                url = results[0].url
                manual_type = results[0].manual_type

        logger.info(f"Downloading manual: {device} ({manual_type}) from {url}")

        # Create output directory
        output_dir = self.get_artifact_dir(f"{device}-{manual_type}")
        output_dir.mkdir(parents=True, exist_ok=True)

        # Download the manual
        manual_path = output_dir / "manual.pdf"

        if not manual_path.exists():
            if "archive.org" in url:
                manual_path = self._download_from_archive_org(url, output_dir)
            else:
                self._download_file_sync(url, manual_path)

        if not manual_path or not manual_path.exists():
            logger.error(f"Failed to download manual for {device}")
            return None

        # Create metadata
        metadata = {
            "device": device,
            "manual_type": manual_type,
            "source_url": url,
            "retrieved_at": datetime.now().isoformat(),
        }

        # Add known source metadata
        if device in KNOWN_SOURCES:
            source = KNOWN_SOURCES[device]
            metadata["manufacturer"] = source.get("manufacturer")

        self._save_metadata(output_dir, metadata)

        # Get page count if possible
        page_count = self._get_pdf_page_count(manual_path)

        # Create artifact
        artifact = ManualArtifact(
            id=f"manual:{device}:{manual_type}",
            name=f"{device} {manual_type.title()} Manual",
            path=manual_path,
            source_url=url,
            retrieved_at=datetime.now(),
            metadata=metadata,
            device=device,
            manual_type=manual_type,
            manufacturer=metadata.get("manufacturer"),
            page_count=page_count,
        )

        # Track in provenance database
        self._track_artifact(artifact)

        logger.info(f"Downloaded manual: {device} -> {output_dir}")
        return artifact

    def _extract_device_from_url(self, url: str) -> str:
        """Extract device name from URL."""
        parsed = urlparse(url)
        path = Path(parsed.path)

        # Try filename
        name = path.stem.lower()
        name = re.sub(r'[_\-]+', '-', name)
        name = re.sub(r'manual|service|operation|user', '', name, flags=re.I)
        name = name.strip('-')

        return name or "unknown"

    def _download_from_archive_org(self, url: str, output_dir: Path) -> Optional[Path]:
        """Download from Archive.org, handling their page structure."""
        # Extract identifier from URL
        match = re.search(r"/details/([^/]+)", url)
        if not match:
            return None

        identifier = match.group(1)

        # Get item metadata to find downloadable files
        metadata_url = f"{self.ARCHIVE_ORG_BASE}/metadata/{identifier}"
        response = self._get_sync(metadata_url)

        if not response:
            return None

        try:
            data = response.json()
            files = data.get("files", [])

            # Look for PDF files
            for f in files:
                name = f.get("name", "").lower()
                if name.endswith(".pdf"):
                    file_url = f"{self.ARCHIVE_ORG_BASE}/download/{identifier}/{f['name']}"
                    output_path = output_dir / "manual.pdf"
                    self._download_file_sync(file_url, output_path)
                    return output_path

        except Exception as e:
            logger.error(f"Archive.org download error: {e}")

        return None

    def _get_pdf_page_count(self, pdf_path: Path) -> Optional[int]:
        """Get page count from PDF file."""
        try:
            # Try PyPDF2 if available
            import PyPDF2
            with open(pdf_path, "rb") as f:
                reader = PyPDF2.PdfReader(f)
                return len(reader.pages)
        except ImportError:
            pass
        except Exception:
            pass

        return None


# CLI interface
def main():
    """Command-line interface for manual retrieval."""
    import argparse

    parser = argparse.ArgumentParser(description="Manual Retriever")
    subparsers = parser.add_subparsers(dest="command", help="Command")

    # Search command
    search_parser = subparsers.add_parser("search", help="Search for manuals")
    search_parser.add_argument("device", help="Device name to search")
    search_parser.add_argument(
        "--type", "-t",
        choices=["service", "operation", "user", "any"],
        default="any",
        help="Manual type"
    )
    search_parser.add_argument("--manufacturer", "-m", help="Manufacturer filter")
    search_parser.add_argument("--max", "-n", type=int, default=10, help="Max results")

    # Download command
    download_parser = subparsers.add_parser("download", help="Download a manual")
    download_parser.add_argument("device", help="Device name or URL")
    download_parser.add_argument(
        "--type", "-t",
        choices=["service", "operation", "user"],
        default="service",
        help="Manual type"
    )

    # List known command
    list_parser = subparsers.add_parser("list-known", help="List known manual sources")

    args = parser.parse_args()

    retriever = ManualRetriever()

    if args.command == "search":
        results = retriever.search(
            args.device,
            manual_type=args.type,
            manufacturer=args.manufacturer,
            max_results=args.max,
        )

        print(f"\nFound {len(results)} manuals:\n")
        for r in results:
            print(f"  {r.title}")
            print(f"    Type: {r.manual_type}")
            print(f"    Source: {r.source}")
            print(f"    URL: {r.url}\n")

    elif args.command == "download":
        artifact = retriever.download(args.device, manual_type=args.type)
        if artifact:
            print(f"\nDownloaded: {artifact.device}")
            print(f"  Type: {artifact.manual_type}")
            print(f"  Path: {artifact.path}")
            if artifact.page_count:
                print(f"  Pages: {artifact.page_count}")
        else:
            print(f"\nFailed to download manual: {args.device}")

    elif args.command == "list-known":
        print("\nKnown manual sources:\n")
        for key, source in KNOWN_SOURCES.items():
            print(f"  {key}")
            print(f"    Manufacturer: {source.get('manufacturer', 'Unknown')}")
            print(f"    Type: {source.get('type', 'Unknown')}")
            print(f"    URL: {source['url']}\n")

    else:
        parser.print_help()


if __name__ == "__main__":
    main()
