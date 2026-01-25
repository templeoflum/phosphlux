"""
Artifact dataclasses for the document retrieval system.

Each artifact type represents a specific kind of research document
with appropriate metadata fields.
"""

from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Optional, List, Dict, Any
import hashlib
import json


@dataclass
class Artifact:
    """Base artifact with common fields."""
    id: str
    type: str  # 'patent', 'datasheet', 'schematic', 'manual', 'component'
    name: str
    path: Path
    source_url: Optional[str] = None
    retrieved_at: Optional[datetime] = None
    metadata: Dict[str, Any] = field(default_factory=dict)

    def compute_hash(self) -> str:
        """Compute SHA256 hash of the artifact file."""
        if not self.path.exists():
            return ""
        sha256 = hashlib.sha256()
        with open(self.path, "rb") as f:
            for chunk in iter(lambda: f.read(8192), b""):
                sha256.update(chunk)
        return sha256.hexdigest()

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return {
            "id": self.id,
            "type": self.type,
            "name": self.name,
            "path": str(self.path),
            "source_url": self.source_url,
            "retrieved_at": self.retrieved_at.isoformat() if self.retrieved_at else None,
            "metadata": self.metadata,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Artifact":
        """Create from dictionary."""
        data = data.copy()
        data["path"] = Path(data["path"])
        if data.get("retrieved_at"):
            data["retrieved_at"] = datetime.fromisoformat(data["retrieved_at"])
        return cls(**data)


@dataclass
class PatentArtifact(Artifact):
    """Patent document artifact."""
    patent_id: str = ""
    title: str = ""
    inventors: List[str] = field(default_factory=list)
    assignee: Optional[str] = None
    filing_date: Optional[str] = None
    publication_date: Optional[str] = None
    abstract: str = ""
    claims: List[str] = field(default_factory=list)
    figures_dir: Optional[Path] = None

    def __post_init__(self):
        self.type = "patent"
        if not self.id:
            self.id = f"patent:{self.patent_id}"

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        data = super().to_dict()
        data.update({
            "patent_id": self.patent_id,
            "title": self.title,
            "inventors": self.inventors,
            "assignee": self.assignee,
            "filing_date": self.filing_date,
            "publication_date": self.publication_date,
            "abstract": self.abstract,
            "claims": self.claims,
            "figures_dir": str(self.figures_dir) if self.figures_dir else None,
        })
        return data


@dataclass
class DatasheetArtifact(Artifact):
    """IC datasheet artifact."""
    part_number: str = ""
    manufacturer: str = ""
    description: str = ""
    package_types: List[str] = field(default_factory=list)
    specs: Dict[str, Any] = field(default_factory=dict)

    def __post_init__(self):
        self.type = "datasheet"
        if not self.id:
            self.id = f"datasheet:{self.part_number}"

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        data = super().to_dict()
        data.update({
            "part_number": self.part_number,
            "manufacturer": self.manufacturer,
            "description": self.description,
            "package_types": self.package_types,
            "specs": self.specs,
        })
        return data


@dataclass
class SchematicArtifact(Artifact):
    """Circuit schematic artifact."""
    device: str = ""
    designer: Optional[str] = None
    year: Optional[int] = None
    parts_list: List[str] = field(default_factory=list)
    circuit_type: Optional[str] = None  # 'video processor', 'oscillator', etc.

    def __post_init__(self):
        self.type = "schematic"
        if not self.id:
            self.id = f"schematic:{self.device}"

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        data = super().to_dict()
        data.update({
            "device": self.device,
            "designer": self.designer,
            "year": self.year,
            "parts_list": self.parts_list,
            "circuit_type": self.circuit_type,
        })
        return data


@dataclass
class ManualArtifact(Artifact):
    """Operation or service manual artifact."""
    device: str = ""
    manual_type: str = "service"  # 'service', 'operation', 'user'
    manufacturer: Optional[str] = None
    model_number: Optional[str] = None
    year: Optional[int] = None
    page_count: Optional[int] = None

    def __post_init__(self):
        self.type = "manual"
        if not self.id:
            self.id = f"manual:{self.device}:{self.manual_type}"

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        data = super().to_dict()
        data.update({
            "device": self.device,
            "manual_type": self.manual_type,
            "manufacturer": self.manufacturer,
            "model_number": self.model_number,
            "year": self.year,
            "page_count": self.page_count,
        })
        return data


@dataclass
class ComponentVersion:
    """Versioned component model (WGSL shaders, specs)."""
    id: str
    component: str
    version: int
    path: Path
    created_at: datetime
    created_by: str  # 'manual' or 'agent:{name}'
    commit_hash: Optional[str] = None
    notes: str = ""
    spec: Dict[str, Any] = field(default_factory=dict)
    source_artifacts: List[str] = field(default_factory=list)  # Artifact IDs

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return {
            "id": self.id,
            "component": self.component,
            "version": self.version,
            "path": str(self.path),
            "created_at": self.created_at.isoformat(),
            "created_by": self.created_by,
            "commit_hash": self.commit_hash,
            "notes": self.notes,
            "spec": self.spec,
            "source_artifacts": self.source_artifacts,
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "ComponentVersion":
        """Create from dictionary."""
        data = data.copy()
        data["path"] = Path(data["path"])
        data["created_at"] = datetime.fromisoformat(data["created_at"])
        return cls(**data)


@dataclass
class SearchResult:
    """Generic search result."""
    title: str
    url: str
    source: str
    snippet: str = ""
    relevance: float = 0.0
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class PatentResult(SearchResult):
    """Patent search result."""
    patent_id: str = ""
    inventors: List[str] = field(default_factory=list)
    filing_date: Optional[str] = None

    def __post_init__(self):
        self.source = "patents"


@dataclass
class DatasheetResult(SearchResult):
    """Datasheet search result."""
    part_number: str = ""
    manufacturer: str = ""

    def __post_init__(self):
        self.source = "datasheets"


@dataclass
class SchematicResult(SearchResult):
    """Schematic search result."""
    device: str = ""
    archive: str = ""  # 'vasulka', 'archive.org', 'github', etc.

    def __post_init__(self):
        self.source = "schematics"


@dataclass
class ManualResult(SearchResult):
    """Manual search result."""
    device: str = ""
    manual_type: str = ""

    def __post_init__(self):
        self.source = "manuals"
