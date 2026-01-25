"""
Patent Retriever for video synthesis research.

Searches and downloads patents from:
- Google Patents (primary)
- USPTO PatentsView API
- Espacenet (fallback)
"""

import re
import json
import logging
from datetime import datetime
from pathlib import Path
from typing import Optional, List, Dict, Any
from urllib.parse import quote_plus, urljoin

try:
    from bs4 import BeautifulSoup
    HAS_BS4 = True
except ImportError:
    HAS_BS4 = False

from .base import BaseRetriever
from .artifacts import PatentArtifact, PatentResult


logger = logging.getLogger(__name__)


class PatentRetriever(BaseRetriever):
    """Retriever for patent documents."""

    artifact_type = "patent"
    output_dir_name = "patents"

    # Source URLs
    GOOGLE_PATENTS_BASE = "https://patents.google.com"
    USPTO_API_BASE = "https://api.patentsview.org/patents/query"
    ESPACENET_BASE = "https://worldwide.espacenet.com"

    def __init__(self, **kwargs):
        """Initialize patent retriever."""
        super().__init__(**kwargs)

        if not HAS_BS4:
            logger.warning(
                "BeautifulSoup not installed. HTML parsing will be limited. "
                "Install with: pip install beautifulsoup4"
            )

    def search(
        self,
        query: str,
        inventor: Optional[str] = None,
        year_range: Optional[tuple] = None,
        max_results: int = 20,
    ) -> List[PatentResult]:
        """
        Search for patents.

        Args:
            query: Search terms
            inventor: Optional inventor name filter
            year_range: Optional (start_year, end_year) tuple
            max_results: Maximum results to return

        Returns:
            List of PatentResult objects
        """
        results = []

        # Try Google Patents first
        google_results = self._search_google_patents(query, inventor, year_range)
        results.extend(google_results)

        # Try USPTO API for additional results
        if len(results) < max_results:
            uspto_results = self._search_uspto_api(query, inventor, year_range)
            # Deduplicate by patent ID
            existing_ids = {r.patent_id for r in results}
            for r in uspto_results:
                if r.patent_id not in existing_ids:
                    results.append(r)

        return results[:max_results]

    def _search_google_patents(
        self,
        query: str,
        inventor: Optional[str] = None,
        year_range: Optional[tuple] = None,
    ) -> List[PatentResult]:
        """Search Google Patents."""
        results = []

        # Build search query
        search_terms = [query]
        if inventor:
            search_terms.append(f'inventor:"{inventor}"')
        if year_range:
            search_terms.append(f"before:filing:{year_range[1]+1}")
            search_terms.append(f"after:filing:{year_range[0]-1}")

        search_query = " ".join(search_terms)
        search_url = f"{self.GOOGLE_PATENTS_BASE}/?q={quote_plus(search_query)}"

        logger.info(f"Searching Google Patents: {search_query}")

        response = self._get_sync(search_url)
        if not response:
            logger.warning("Google Patents search failed")
            return results

        if HAS_BS4:
            soup = BeautifulSoup(response.text, "html.parser")

            # Find search result items
            for item in soup.select("search-result-item, article.result"):
                try:
                    result = self._parse_google_result(item)
                    if result:
                        results.append(result)
                except Exception as e:
                    logger.debug(f"Failed to parse result: {e}")

        return results

    def _parse_google_result(self, item) -> Optional[PatentResult]:
        """Parse a Google Patents search result item."""
        # Extract patent ID from link
        link = item.select_one("a[href*='/patent/']")
        if not link:
            return None

        href = link.get("href", "")
        patent_id_match = re.search(r"/patent/([A-Z]{2}\d+)", href)
        if not patent_id_match:
            return None

        patent_id = patent_id_match.group(1)

        # Extract title
        title_elem = item.select_one("h3, .title, span.style-scope")
        title = title_elem.get_text(strip=True) if title_elem else patent_id

        # Extract inventors
        inventors = []
        inventor_elem = item.select_one(".inventor, .inventors")
        if inventor_elem:
            inventors = [i.strip() for i in inventor_elem.get_text().split(",")]

        # Extract snippet
        snippet_elem = item.select_one(".abstract, .snippet")
        snippet = snippet_elem.get_text(strip=True) if snippet_elem else ""

        # Extract filing date
        date_elem = item.select_one(".date, .filing-date")
        filing_date = date_elem.get_text(strip=True) if date_elem else None

        return PatentResult(
            title=title,
            url=urljoin(self.GOOGLE_PATENTS_BASE, href),
            source="google_patents",
            snippet=snippet[:500] if snippet else "",
            patent_id=patent_id,
            inventors=inventors,
            filing_date=filing_date,
        )

    def _search_uspto_api(
        self,
        query: str,
        inventor: Optional[str] = None,
        year_range: Optional[tuple] = None,
    ) -> List[PatentResult]:
        """Search USPTO PatentsView API."""
        results = []

        # Build query criteria
        criteria = []

        # Text search
        criteria.append({"_text_any": {"patent_abstract": query}})

        if inventor:
            criteria.append({
                "_text_any": {"inventor_last_name": inventor}
            })

        if year_range:
            criteria.append({
                "_gte": {"patent_date": f"{year_range[0]}-01-01"}
            })
            criteria.append({
                "_lte": {"patent_date": f"{year_range[1]}-12-31"}
            })

        query_obj = {"_and": criteria} if len(criteria) > 1 else criteria[0]

        params = {
            "q": json.dumps(query_obj),
            "f": json.dumps([
                "patent_number",
                "patent_title",
                "patent_abstract",
                "patent_date",
                "inventor_first_name",
                "inventor_last_name",
            ]),
            "o": json.dumps({"page": 1, "per_page": 25}),
        }

        logger.info("Searching USPTO PatentsView API")

        try:
            response = self._get_sync(self.USPTO_API_BASE, params=params)
            if response:
                data = response.json()
                patents = data.get("patents", [])

                for patent in patents:
                    patent_num = patent.get("patent_number", "")
                    results.append(PatentResult(
                        title=patent.get("patent_title", ""),
                        url=f"https://patents.google.com/patent/US{patent_num}",
                        source="uspto_api",
                        snippet=patent.get("patent_abstract", "")[:500],
                        patent_id=f"US{patent_num}",
                        inventors=self._format_inventors(patent.get("inventors", [])),
                        filing_date=patent.get("patent_date"),
                    ))
        except Exception as e:
            logger.warning(f"USPTO API search failed: {e}")

        return results

    def _format_inventors(self, inventors: List[Dict]) -> List[str]:
        """Format inventor names from USPTO data."""
        names = []
        for inv in inventors:
            first = inv.get("inventor_first_name", "")
            last = inv.get("inventor_last_name", "")
            if first and last:
                names.append(f"{first} {last}")
            elif last:
                names.append(last)
        return names

    def download(
        self,
        patent_id: str,
        **kwargs
    ) -> Optional[PatentArtifact]:
        """
        Download a patent by ID.

        Args:
            patent_id: Patent ID (e.g., "US4011557", "EP1234567")

        Returns:
            PatentArtifact if successful, None otherwise
        """
        # Normalize patent ID
        patent_id = patent_id.upper().replace(" ", "")

        logger.info(f"Downloading patent: {patent_id}")

        # Create output directory
        output_dir = self.get_artifact_dir(patent_id)
        output_dir.mkdir(parents=True, exist_ok=True)

        # Fetch patent page for metadata
        patent_url = f"{self.GOOGLE_PATENTS_BASE}/patent/{patent_id}"
        response = self._get_sync(patent_url)

        if not response:
            logger.error(f"Failed to fetch patent page: {patent_id}")
            return None

        # Parse metadata
        metadata = self._parse_patent_page(response.text, patent_id)

        # Download PDF
        pdf_path = output_dir / "patent.pdf"
        pdf_url = f"{self.GOOGLE_PATENTS_BASE}/patent/{patent_id}/pdf"

        if not pdf_path.exists():
            pdf_downloaded = self._download_file_sync(pdf_url, pdf_path)
            if not pdf_downloaded:
                logger.warning(f"Could not download PDF for {patent_id}")

        # Save metadata
        self._save_metadata(output_dir, metadata)

        # Save claims as text
        if metadata.get("claims"):
            claims_path = output_dir / "claims.txt"
            with open(claims_path, "w", encoding="utf-8") as f:
                for i, claim in enumerate(metadata["claims"], 1):
                    f.write(f"Claim {i}:\n{claim}\n\n")

        # Create artifact
        artifact = PatentArtifact(
            id=f"patent:{patent_id}",
            name=metadata.get("title", patent_id),
            path=output_dir,
            source_url=patent_url,
            retrieved_at=datetime.now(),
            metadata=metadata,
            patent_id=patent_id,
            title=metadata.get("title", ""),
            inventors=metadata.get("inventors", []),
            assignee=metadata.get("assignee"),
            filing_date=metadata.get("filing_date"),
            publication_date=metadata.get("publication_date"),
            abstract=metadata.get("abstract", ""),
            claims=metadata.get("claims", []),
            figures_dir=output_dir / "figures" if (output_dir / "figures").exists() else None,
        )

        # Track in provenance database
        self._track_artifact(artifact)

        logger.info(f"Downloaded patent: {patent_id} -> {output_dir}")
        return artifact

    def _parse_patent_page(self, html: str, patent_id: str) -> Dict[str, Any]:
        """Parse patent page HTML for metadata."""
        metadata = {
            "patent_id": patent_id,
            "title": "",
            "inventors": [],
            "assignee": None,
            "filing_date": None,
            "publication_date": None,
            "abstract": "",
            "claims": [],
        }

        if not HAS_BS4:
            return metadata

        soup = BeautifulSoup(html, "html.parser")

        # Title
        title_elem = soup.select_one("h1#title, .patent-title, meta[name='DC.title']")
        if title_elem:
            if title_elem.name == "meta":
                metadata["title"] = title_elem.get("content", "")
            else:
                metadata["title"] = title_elem.get_text(strip=True)

        # Inventors
        for inv_elem in soup.select("dd[itemprop='inventor'], .inventor"):
            inv_text = inv_elem.get_text(strip=True)
            if inv_text:
                metadata["inventors"].append(inv_text)

        # Assignee
        assignee_elem = soup.select_one("dd[itemprop='assignee'], .assignee")
        if assignee_elem:
            metadata["assignee"] = assignee_elem.get_text(strip=True)

        # Dates
        for dt_elem in soup.select("dt"):
            dt_text = dt_elem.get_text(strip=True).lower()
            dd_elem = dt_elem.find_next_sibling("dd")
            if dd_elem:
                dd_text = dd_elem.get_text(strip=True)
                if "filing" in dt_text:
                    metadata["filing_date"] = dd_text
                elif "publication" in dt_text or "grant" in dt_text:
                    metadata["publication_date"] = dd_text

        # Abstract
        abstract_elem = soup.select_one("div.abstract, section#abstract, meta[name='DC.description']")
        if abstract_elem:
            if abstract_elem.name == "meta":
                metadata["abstract"] = abstract_elem.get("content", "")
            else:
                metadata["abstract"] = abstract_elem.get_text(strip=True)

        # Claims
        claims_section = soup.select_one("section#claims, div.claims")
        if claims_section:
            for claim_elem in claims_section.select("div.claim, .claim-text"):
                claim_text = claim_elem.get_text(strip=True)
                if claim_text:
                    metadata["claims"].append(claim_text)

        return metadata

    def extract_figures(self, patent_path: Path) -> List[Path]:
        """
        Extract figures from a patent PDF.

        This is a placeholder - actual implementation would use
        PDF extraction libraries like pdf2image or PyMuPDF.

        Args:
            patent_path: Path to patent directory or PDF

        Returns:
            List of paths to extracted figure images
        """
        figures = []

        if patent_path.is_file():
            patent_dir = patent_path.parent
        else:
            patent_dir = patent_path

        figures_dir = patent_dir / "figures"
        figures_dir.mkdir(exist_ok=True)

        # Check for existing figures
        for ext in ["*.png", "*.jpg", "*.gif"]:
            figures.extend(figures_dir.glob(ext))

        if figures:
            logger.info(f"Found {len(figures)} existing figures")
            return figures

        # TODO: Implement PDF figure extraction
        # This would require additional dependencies:
        # - pdf2image (for rasterizing pages)
        # - PyMuPDF/fitz (for extracting embedded images)
        # - opencv or PIL (for figure detection)

        pdf_path = patent_dir / "patent.pdf"
        if pdf_path.exists():
            logger.info(
                f"PDF figure extraction not implemented. "
                f"PDF available at: {pdf_path}"
            )

        return figures

    def get_related_patents(self, patent_id: str) -> List[PatentResult]:
        """
        Find patents related to a given patent.

        Args:
            patent_id: Patent ID to find related patents for

        Returns:
            List of related PatentResult objects
        """
        # Fetch the patent page
        patent_url = f"{self.GOOGLE_PATENTS_BASE}/patent/{patent_id}"
        response = self._get_sync(patent_url)

        if not response or not HAS_BS4:
            return []

        soup = BeautifulSoup(response.text, "html.parser")
        related = []

        # Find cited patents and citing patents
        for section in soup.select("section#patentCitations, section#citedBy"):
            for link in section.select("a[href*='/patent/']"):
                href = link.get("href", "")
                match = re.search(r"/patent/([A-Z]{2}\d+)", href)
                if match:
                    rel_id = match.group(1)
                    if rel_id != patent_id:
                        related.append(PatentResult(
                            title=link.get_text(strip=True) or rel_id,
                            url=urljoin(self.GOOGLE_PATENTS_BASE, href),
                            source="google_patents",
                            patent_id=rel_id,
                        ))

        return related


# CLI interface
def main():
    """Command-line interface for patent retrieval."""
    import argparse

    parser = argparse.ArgumentParser(description="Patent Retriever")
    subparsers = parser.add_subparsers(dest="command", help="Command")

    # Search command
    search_parser = subparsers.add_parser("search", help="Search for patents")
    search_parser.add_argument("query", help="Search query")
    search_parser.add_argument("--inventor", "-i", help="Inventor name")
    search_parser.add_argument("--from-year", type=int, help="Start year")
    search_parser.add_argument("--to-year", type=int, help="End year")
    search_parser.add_argument("--max", "-n", type=int, default=10, help="Max results")

    # Download command
    download_parser = subparsers.add_parser("download", help="Download a patent")
    download_parser.add_argument("patent_id", help="Patent ID (e.g., US4011557)")

    # Extract figures command
    extract_parser = subparsers.add_parser("extract-figures", help="Extract figures from patent")
    extract_parser.add_argument("path", help="Path to patent directory or PDF")

    args = parser.parse_args()

    retriever = PatentRetriever()

    if args.command == "search":
        year_range = None
        if args.from_year or args.to_year:
            year_range = (
                args.from_year or 1900,
                args.to_year or datetime.now().year,
            )

        results = retriever.search(
            args.query,
            inventor=args.inventor,
            year_range=year_range,
            max_results=args.max,
        )

        print(f"\nFound {len(results)} patents:\n")
        for r in results:
            print(f"  {r.patent_id}: {r.title}")
            if r.inventors:
                print(f"    Inventors: {', '.join(r.inventors)}")
            if r.filing_date:
                print(f"    Filed: {r.filing_date}")
            print(f"    URL: {r.url}\n")

    elif args.command == "download":
        artifact = retriever.download(args.patent_id)
        if artifact:
            print(f"\nDownloaded: {artifact.title}")
            print(f"  Path: {artifact.path}")
            print(f"  Inventors: {', '.join(artifact.inventors)}")
        else:
            print(f"\nFailed to download patent: {args.patent_id}")

    elif args.command == "extract-figures":
        figures = retriever.extract_figures(Path(args.path))
        print(f"\nFound {len(figures)} figures:")
        for f in figures:
            print(f"  {f}")

    else:
        parser.print_help()


if __name__ == "__main__":
    main()
