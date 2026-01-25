"""
Schematic Retriever for video synthesis research.

Searches and downloads circuit schematics from:
- Vasulka Archive (primary for video art)
- Archive.org (historical documents)
- GitHub repositories (LZX, Syntonie, etc.)
- Experimental TV Center
- Fondation Langlois
"""

import re
import json
import logging
from datetime import datetime
from pathlib import Path
from typing import Optional, List, Dict, Any
from urllib.parse import quote_plus, urljoin, urlparse
import subprocess

try:
    from bs4 import BeautifulSoup
    HAS_BS4 = True
except ImportError:
    HAS_BS4 = False

from .base import BaseRetriever
from .artifacts import SchematicArtifact, SchematicResult


logger = logging.getLogger(__name__)


# Known schematic sources for video synthesis devices
KNOWN_SOURCES = {
    # Paik/Abe devices
    "paik-abe-synthesizer": {
        "url": "http://vasulka.org/archive/Artists1/PaikAbe/PaikAbeSynthesizer.pdf",
        "designer": "Nam June Paik / Shuya Abe",
        "year": 1970,
    },
    "paik-abe-colorizer": {
        "url": "http://vasulka.org/archive/Artists1/PaikAbe/Colorizer.pdf",
        "designer": "Nam June Paik / Shuya Abe",
        "year": 1972,
    },

    # Sandin Image Processor
    "sandin-ip": {
        "url": "http://vasulka.org/archive/Artists1/Sandin/SandinIP.pdf",
        "designer": "Dan Sandin",
        "year": 1973,
    },

    # Jones Colorizer
    "jones-colorizer": {
        "url": "http://www.analogarts.net/circuits/jones_colorizer.pdf",
        "designer": "Dave Jones",
        "year": 1983,
    },

    # LZX modules (GitHub)
    "lzx-cadet": {
        "url": "https://github.com/lzxindustries/cadet",
        "designer": "LZX Industries",
        "year": 2018,
        "type": "github",
    },
    "lzx-castle": {
        "url": "https://github.com/lzxindustries/castle",
        "designer": "LZX Industries",
        "year": 2020,
        "type": "github",
    },
}


# GitHub repositories for video synthesis schematics
GITHUB_REPOS = [
    {"owner": "lzxindustries", "repo": "cadet", "desc": "LZX Cadet DIY modules"},
    {"owner": "lzxindustries", "repo": "castle", "desc": "LZX Castle DIY modules"},
    {"owner": "syntonie", "repo": "DIY", "desc": "Syntonie DIY video synth"},
    {"owner": "reverselandfill", "repo": "reverselandfill.org", "desc": "Reverselandfill open video synth"},
]


class SchematicRetriever(BaseRetriever):
    """Retriever for circuit schematics."""

    artifact_type = "schematic"
    output_dir_name = "schematics"

    # Source URLs
    VASULKA_BASE = "http://vasulka.org/archive"
    ARCHIVE_ORG_BASE = "https://archive.org"
    ETC_BASE = "http://experimentaltvcenter.org"
    LANGLOIS_BASE = "https://www.fondation-langlois.org"

    def __init__(self, **kwargs):
        """Initialize schematic retriever."""
        super().__init__(**kwargs)

        if not HAS_BS4:
            logger.warning(
                "BeautifulSoup not installed. HTML parsing will be limited. "
                "Install with: pip install beautifulsoup4"
            )

    def search(
        self,
        device: str,
        designer: Optional[str] = None,
        max_results: int = 10,
    ) -> List[SchematicResult]:
        """
        Search for schematics.

        Args:
            device: Device name to search for
            designer: Optional designer/inventor filter
            max_results: Maximum results to return

        Returns:
            List of SchematicResult objects
        """
        results = []
        device_lower = device.lower()

        # Check known sources first
        for key, source in KNOWN_SOURCES.items():
            if device_lower in key.lower():
                results.append(SchematicResult(
                    title=f"{key.replace('-', ' ').title()} Schematic",
                    url=source["url"],
                    source="known_source",
                    snippet=f"Designer: {source.get('designer', 'Unknown')}",
                    device=key,
                    archive=source.get("type", "direct"),
                    relevance=1.0,
                ))

        # Search Vasulka Archive
        vasulka_results = self._search_vasulka(device)
        results.extend(vasulka_results)

        # Search Archive.org
        archive_results = self._search_archive_org(device)
        results.extend(archive_results)

        # Search GitHub repos
        github_results = self._search_github(device)
        results.extend(github_results)

        # Filter by designer if specified
        if designer:
            designer_lower = designer.lower()
            results = [
                r for r in results
                if designer_lower in r.snippet.lower() or
                   designer_lower in r.title.lower()
            ]

        # Deduplicate by URL
        seen_urls = set()
        unique_results = []
        for r in results:
            if r.url not in seen_urls:
                unique_results.append(r)
                seen_urls.add(r.url)

        return unique_results[:max_results]

    def _search_vasulka(self, device: str) -> List[SchematicResult]:
        """Search Vasulka Archive."""
        results = []

        # Vasulka archive is organized by artist/device
        # Try common artist directories
        artists = ["PaikAbe", "Sandin", "Hearn", "Siegel", "Beck", "Rutt"]

        for artist in artists:
            url = f"{self.VASULKA_BASE}/Artists1/{artist}/"

            response = self._get_sync(url)
            if not response or not HAS_BS4:
                continue

            soup = BeautifulSoup(response.text, "html.parser")

            # Look for PDF and image links
            for link in soup.select("a[href*='.pdf'], a[href*='.jpg'], a[href*='.png']"):
                href = link.get("href", "")
                link_text = link.get_text(strip=True).lower()

                if device.lower() in link_text or device.lower() in href.lower():
                    results.append(SchematicResult(
                        title=link.get_text(strip=True) or f"{artist} - {device}",
                        url=urljoin(url, href),
                        source="vasulka",
                        snippet=f"From Vasulka Archive - {artist} collection",
                        device=device,
                        archive="vasulka",
                    ))

        return results

    def _search_archive_org(self, device: str) -> List[SchematicResult]:
        """Search Archive.org."""
        results = []

        # Archive.org search API
        search_url = f"{self.ARCHIVE_ORG_BASE}/advancedsearch.php"
        params = {
            "q": f'"{device}" AND (schematic OR circuit OR diagram)',
            "fl[]": ["identifier", "title", "description"],
            "output": "json",
            "rows": 10,
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
                if isinstance(desc, list):
                    desc = " ".join(desc)

                results.append(SchematicResult(
                    title=title,
                    url=f"{self.ARCHIVE_ORG_BASE}/details/{identifier}",
                    source="archive_org",
                    snippet=desc[:200] if desc else "",
                    device=device,
                    archive="archive.org",
                ))

        except Exception as e:
            logger.warning(f"Archive.org search error: {e}")

        return results

    def _search_github(self, device: str) -> List[SchematicResult]:
        """Search GitHub repositories for schematics."""
        results = []

        for repo in GITHUB_REPOS:
            # Use GitHub API to search repo contents
            api_url = f"https://api.github.com/search/code"
            params = {
                "q": f"{device} repo:{repo['owner']}/{repo['repo']} extension:pdf extension:svg extension:kicad_sch",
            }

            response = self._get_sync(api_url, params=params)
            if not response:
                continue

            try:
                data = response.json()
                items = data.get("items", [])

                for item in items[:3]:  # Limit per repo
                    path = item.get("path", "")
                    name = item.get("name", "")
                    html_url = item.get("html_url", "")

                    results.append(SchematicResult(
                        title=f"{repo['repo']}: {name}",
                        url=html_url,
                        source="github",
                        snippet=f"From {repo['desc']} - {path}",
                        device=device,
                        archive="github",
                    ))

            except Exception as e:
                logger.debug(f"GitHub search error for {repo['repo']}: {e}")

        return results

    def download(
        self,
        url_or_device: str,
        device: Optional[str] = None,
        **kwargs
    ) -> Optional[SchematicArtifact]:
        """
        Download a schematic.

        Args:
            url_or_device: Either a URL or a known device name
            device: Device name (if url_or_device is a URL)

        Returns:
            SchematicArtifact if successful, None otherwise
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

                # Handle GitHub repos
                if source.get("type") == "github":
                    return self._clone_github_repo(url, device, source)
            else:
                # Search for it
                results = self.search(device, max_results=1)
                if not results:
                    logger.error(f"No schematic found for: {device}")
                    return None
                url = results[0].url

        logger.info(f"Downloading schematic: {device} from {url}")

        # Create output directory
        output_dir = self.get_artifact_dir(device)
        output_dir.mkdir(parents=True, exist_ok=True)

        # Determine file type and download
        parsed = urlparse(url)
        ext = Path(parsed.path).suffix.lower() or ".pdf"

        schematic_path = output_dir / f"schematic{ext}"

        if not schematic_path.exists():
            if "archive.org" in url:
                # Archive.org may need special handling
                schematic_path = self._download_from_archive_org(url, output_dir)
            else:
                self._download_file_sync(url, schematic_path)

        if not schematic_path or not schematic_path.exists():
            logger.error(f"Failed to download schematic for {device}")
            return None

        # Create metadata
        metadata = {
            "device": device,
            "source_url": url,
            "retrieved_at": datetime.now().isoformat(),
        }

        # Add known source metadata
        if device in KNOWN_SOURCES:
            source = KNOWN_SOURCES[device]
            metadata["designer"] = source.get("designer")
            metadata["year"] = source.get("year")

        self._save_metadata(output_dir, metadata)

        # Create artifact
        artifact = SchematicArtifact(
            id=f"schematic:{device}",
            name=f"{device} Schematic",
            path=schematic_path,
            source_url=url,
            retrieved_at=datetime.now(),
            metadata=metadata,
            device=device,
            designer=metadata.get("designer"),
            year=metadata.get("year"),
        )

        # Track in provenance database
        self._track_artifact(artifact)

        logger.info(f"Downloaded schematic: {device} -> {output_dir}")
        return artifact

    def _extract_device_from_url(self, url: str) -> str:
        """Extract device name from URL."""
        parsed = urlparse(url)
        path = Path(parsed.path)

        # Try filename
        name = path.stem.lower()
        name = re.sub(r'[_\-]+', '-', name)
        name = re.sub(r'schematic|circuit|diagram', '', name, flags=re.I)
        name = name.strip('-')

        return name or "unknown"

    def _clone_github_repo(
        self,
        url: str,
        device: str,
        source: Dict
    ) -> Optional[SchematicArtifact]:
        """Clone a GitHub repository containing schematics."""
        output_dir = self.get_artifact_dir(device)

        if output_dir.exists():
            logger.info(f"Repository already cloned: {output_dir}")
        else:
            try:
                subprocess.run(
                    ["git", "clone", "--depth", "1", url, str(output_dir)],
                    check=True,
                    capture_output=True,
                )
                logger.info(f"Cloned repository: {url}")
            except subprocess.CalledProcessError as e:
                logger.error(f"Git clone failed: {e}")
                return None
            except FileNotFoundError:
                logger.error("Git not found. Please install git.")
                return None

        # Create metadata
        metadata = {
            "device": device,
            "source_url": url,
            "type": "github_repo",
            "designer": source.get("designer"),
            "year": source.get("year"),
            "retrieved_at": datetime.now().isoformat(),
        }
        self._save_metadata(output_dir, metadata)

        # Create artifact
        artifact = SchematicArtifact(
            id=f"schematic:{device}",
            name=f"{device} Schematics",
            path=output_dir,
            source_url=url,
            retrieved_at=datetime.now(),
            metadata=metadata,
            device=device,
            designer=source.get("designer"),
            year=source.get("year"),
        )

        self._track_artifact(artifact)
        return artifact

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

            # Look for schematic files
            for f in files:
                name = f.get("name", "").lower()
                if any(ext in name for ext in [".pdf", ".svg", ".png", ".jpg"]):
                    if "schematic" in name or "circuit" in name or "diagram" in name:
                        file_url = f"{self.ARCHIVE_ORG_BASE}/download/{identifier}/{f['name']}"
                        ext = Path(f["name"]).suffix
                        output_path = output_dir / f"schematic{ext}"
                        self._download_file_sync(file_url, output_path)
                        return output_path

            # Fallback: download first PDF
            for f in files:
                if f.get("name", "").lower().endswith(".pdf"):
                    file_url = f"{self.ARCHIVE_ORG_BASE}/download/{identifier}/{f['name']}"
                    output_path = output_dir / "schematic.pdf"
                    self._download_file_sync(file_url, output_path)
                    return output_path

        except Exception as e:
            logger.error(f"Archive.org download error: {e}")

        return None

    def extract_parts(self, schematic_path: Path) -> List[str]:
        """
        Extract component parts list from a schematic.

        This is a placeholder for AI-assisted extraction.

        Args:
            schematic_path: Path to schematic file or directory

        Returns:
            List of component part numbers
        """
        if schematic_path.is_dir():
            parts_path = schematic_path / "parts_list.json"
        else:
            parts_path = schematic_path.parent / "parts_list.json"

        # Check for existing parts list
        if parts_path.exists():
            with open(parts_path) as f:
                data = json.load(f)
                return data.get("parts", [])

        return []


# CLI interface
def main():
    """Command-line interface for schematic retrieval."""
    import argparse

    parser = argparse.ArgumentParser(description="Schematic Retriever")
    subparsers = parser.add_subparsers(dest="command", help="Command")

    # Search command
    search_parser = subparsers.add_parser("search", help="Search for schematics")
    search_parser.add_argument("device", help="Device name to search")
    search_parser.add_argument("--designer", "-d", help="Designer filter")
    search_parser.add_argument("--max", "-n", type=int, default=10, help="Max results")

    # Download command
    download_parser = subparsers.add_parser("download", help="Download a schematic")
    download_parser.add_argument("device", help="Device name or URL")

    # List known command
    list_parser = subparsers.add_parser("list-known", help="List known schematic sources")

    args = parser.parse_args()

    retriever = SchematicRetriever()

    if args.command == "search":
        results = retriever.search(
            args.device,
            designer=args.designer,
            max_results=args.max,
        )

        print(f"\nFound {len(results)} schematics:\n")
        for r in results:
            print(f"  {r.title}")
            print(f"    Archive: {r.archive}")
            print(f"    URL: {r.url}\n")

    elif args.command == "download":
        artifact = retriever.download(args.device)
        if artifact:
            print(f"\nDownloaded: {artifact.device}")
            print(f"  Path: {artifact.path}")
            if artifact.designer:
                print(f"  Designer: {artifact.designer}")
        else:
            print(f"\nFailed to download schematic: {args.device}")

    elif args.command == "list-known":
        print("\nKnown schematic sources:\n")
        for key, source in KNOWN_SOURCES.items():
            print(f"  {key}")
            print(f"    Designer: {source.get('designer', 'Unknown')}")
            print(f"    Year: {source.get('year', 'Unknown')}")
            print(f"    URL: {source['url']}\n")

    else:
        parser.print_help()


if __name__ == "__main__":
    main()
