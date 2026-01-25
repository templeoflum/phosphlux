"""
Base retriever class for all document retrievers.

Provides common functionality for searching, downloading, and tracking artifacts.
"""

import asyncio
import hashlib
import re
import time
from abc import ABC, abstractmethod
from datetime import datetime
from pathlib import Path
from typing import Optional, List, Dict, Any, Type, TypeVar
from urllib.parse import urlparse, urljoin
import json
import logging

try:
    import aiohttp
    HAS_AIOHTTP = True
except ImportError:
    HAS_AIOHTTP = False

try:
    import requests
    HAS_REQUESTS = True
except ImportError:
    HAS_REQUESTS = False

from .artifacts import Artifact, SearchResult
from .database import ProvenanceDB, DEFAULT_DB_PATH


# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


T = TypeVar("T", bound=Artifact)
R = TypeVar("R", bound=SearchResult)


class BaseRetriever(ABC):
    """Base class for all document retrievers."""

    # Override in subclasses
    artifact_type: str = "unknown"
    output_dir_name: str = "artifacts"

    # Default request settings
    default_timeout: int = 30
    default_retries: int = 3
    retry_delay: float = 1.0
    user_agent: str = (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
        "(KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    )

    def __init__(
        self,
        base_dir: Optional[Path] = None,
        db: Optional[ProvenanceDB] = None,
    ):
        """Initialize the retriever."""
        self.base_dir = base_dir or Path(__file__).parent.parent.parent / "research"
        self.output_dir = self.base_dir / self.output_dir_name
        self.output_dir.mkdir(parents=True, exist_ok=True)

        self.db = db or ProvenanceDB(self.base_dir / "provenance.db")

        # Session for requests
        self._session = None

    @property
    def headers(self) -> Dict[str, str]:
        """Default HTTP headers."""
        return {
            "User-Agent": self.user_agent,
            "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            "Accept-Language": "en-US,en;q=0.5",
        }

    # === Abstract Methods ===

    @abstractmethod
    def search(self, query: str, **kwargs) -> List[SearchResult]:
        """Search for artifacts. Must be implemented by subclasses."""
        pass

    @abstractmethod
    def download(self, url: str, identifier: str, **kwargs) -> Optional[Artifact]:
        """Download an artifact. Must be implemented by subclasses."""
        pass

    # === HTTP Helpers ===

    def _get_sync(
        self,
        url: str,
        params: Optional[Dict] = None,
        timeout: Optional[int] = None,
    ) -> Optional[requests.Response]:
        """Synchronous HTTP GET with retries."""
        if not HAS_REQUESTS:
            raise ImportError("requests library required. Install with: pip install requests")

        timeout = timeout or self.default_timeout

        for attempt in range(self.default_retries):
            try:
                response = requests.get(
                    url,
                    params=params,
                    headers=self.headers,
                    timeout=timeout,
                )
                response.raise_for_status()
                return response
            except requests.RequestException as e:
                logger.warning(f"Request failed (attempt {attempt + 1}): {e}")
                if attempt < self.default_retries - 1:
                    time.sleep(self.retry_delay * (attempt + 1))

        return None

    def _download_file_sync(
        self,
        url: str,
        output_path: Path,
        timeout: Optional[int] = None,
    ) -> bool:
        """Download a file synchronously."""
        if not HAS_REQUESTS:
            raise ImportError("requests library required. Install with: pip install requests")

        timeout = timeout or self.default_timeout * 2  # Longer timeout for files

        try:
            response = requests.get(
                url,
                headers=self.headers,
                timeout=timeout,
                stream=True,
            )
            response.raise_for_status()

            output_path.parent.mkdir(parents=True, exist_ok=True)

            with open(output_path, "wb") as f:
                for chunk in response.iter_content(chunk_size=8192):
                    f.write(chunk)

            logger.info(f"Downloaded: {output_path}")
            return True

        except requests.RequestException as e:
            logger.error(f"Download failed: {e}")
            return False

    async def _get_async(
        self,
        url: str,
        params: Optional[Dict] = None,
        timeout: Optional[int] = None,
    ) -> Optional[str]:
        """Asynchronous HTTP GET with retries."""
        if not HAS_AIOHTTP:
            raise ImportError("aiohttp library required. Install with: pip install aiohttp")

        timeout_obj = aiohttp.ClientTimeout(total=timeout or self.default_timeout)

        for attempt in range(self.default_retries):
            try:
                async with aiohttp.ClientSession(
                    headers=self.headers,
                    timeout=timeout_obj,
                ) as session:
                    async with session.get(url, params=params) as response:
                        response.raise_for_status()
                        return await response.text()
            except aiohttp.ClientError as e:
                logger.warning(f"Async request failed (attempt {attempt + 1}): {e}")
                if attempt < self.default_retries - 1:
                    await asyncio.sleep(self.retry_delay * (attempt + 1))

        return None

    async def _download_file_async(
        self,
        url: str,
        output_path: Path,
        timeout: Optional[int] = None,
    ) -> bool:
        """Download a file asynchronously."""
        if not HAS_AIOHTTP:
            raise ImportError("aiohttp library required. Install with: pip install aiohttp")

        timeout_obj = aiohttp.ClientTimeout(total=timeout or self.default_timeout * 2)

        try:
            async with aiohttp.ClientSession(
                headers=self.headers,
                timeout=timeout_obj,
            ) as session:
                async with session.get(url) as response:
                    response.raise_for_status()

                    output_path.parent.mkdir(parents=True, exist_ok=True)

                    with open(output_path, "wb") as f:
                        async for chunk in response.content.iter_chunked(8192):
                            f.write(chunk)

            logger.info(f"Downloaded: {output_path}")
            return True

        except aiohttp.ClientError as e:
            logger.error(f"Async download failed: {e}")
            return False

    # === Metadata Helpers ===

    def _save_metadata(self, path: Path, metadata: Dict[str, Any]) -> None:
        """Save metadata JSON file."""
        metadata_path = path.parent / "metadata.json" if path.is_file() else path / "metadata.json"
        metadata_path.parent.mkdir(parents=True, exist_ok=True)

        with open(metadata_path, "w", encoding="utf-8") as f:
            json.dump(metadata, f, indent=2, default=str)

        logger.info(f"Saved metadata: {metadata_path}")

    def _load_metadata(self, path: Path) -> Optional[Dict[str, Any]]:
        """Load metadata JSON file."""
        metadata_path = path.parent / "metadata.json" if path.is_file() else path / "metadata.json"

        if metadata_path.exists():
            with open(metadata_path, "r", encoding="utf-8") as f:
                return json.load(f)
        return None

    # === Provenance Helpers ===

    def _track_artifact(self, artifact: Artifact) -> str:
        """Add artifact to provenance database."""
        return self.db.add_artifact(artifact)

    def _add_relationship(
        self,
        from_artifact: Artifact,
        to_artifact: Artifact,
        relationship: str,
    ) -> None:
        """Add relationship between artifacts."""
        self.db.add_relationship(from_artifact.id, to_artifact.id, relationship)

    # === Utility Methods ===

    @staticmethod
    def sanitize_filename(name: str) -> str:
        """Sanitize a string for use as a filename."""
        # Replace problematic characters
        name = re.sub(r'[<>:"/\\|?*]', "_", name)
        # Remove leading/trailing whitespace and dots
        name = name.strip(". ")
        # Limit length
        if len(name) > 100:
            name = name[:100]
        return name or "unnamed"

    @staticmethod
    def compute_file_hash(path: Path) -> str:
        """Compute SHA256 hash of a file."""
        sha256 = hashlib.sha256()
        with open(path, "rb") as f:
            for chunk in iter(lambda: f.read(8192), b""):
                sha256.update(chunk)
        return sha256.hexdigest()

    def get_artifact_dir(self, identifier: str) -> Path:
        """Get output directory for an artifact."""
        safe_id = self.sanitize_filename(identifier)
        return self.output_dir / safe_id

    # === Batch Operations ===

    def search_and_download(
        self,
        query: str,
        max_results: int = 5,
        **kwargs
    ) -> List[Artifact]:
        """Search and download multiple artifacts."""
        results = self.search(query, **kwargs)[:max_results]
        artifacts = []

        for result in results:
            try:
                artifact = self.download(result.url, result.title, **kwargs)
                if artifact:
                    artifacts.append(artifact)
            except Exception as e:
                logger.error(f"Failed to download {result.title}: {e}")

        return artifacts

    def list_local_artifacts(self) -> List[Dict[str, Any]]:
        """List all locally stored artifacts of this type."""
        artifacts = []

        for item in self.output_dir.iterdir():
            if item.is_dir():
                metadata = self._load_metadata(item)
                if metadata:
                    artifacts.append({
                        "path": item,
                        "metadata": metadata,
                    })
                else:
                    artifacts.append({
                        "path": item,
                        "metadata": {"name": item.name},
                    })

        return artifacts
