"""
Document Retrieval System for Analog Video Synthesis Research

This package provides layer-specific retrievers for:
- Patents (Google Patents, USPTO, Espacenet)
- Datasheets (AllDatasheet, DatasheetArchive, BITSAVERS)
- Schematics (Vasulka, Archive.org, GitHub)
- Manuals (Archive.org, Elektrotanya, HiFi Engine)

All artifacts are tracked in a SQLite provenance database.
"""

from .artifacts import (
    Artifact,
    PatentArtifact,
    DatasheetArtifact,
    SchematicArtifact,
    ManualArtifact,
    ComponentVersion,
    SearchResult,
    PatentResult,
    DatasheetResult,
    SchematicResult,
    ManualResult,
)
from .database import ProvenanceDB
from .base import BaseRetriever
from .patents import PatentRetriever
from .datasheets import DatasheetRetriever
from .schematics import SchematicRetriever
from .manuals import ManualRetriever
from .components import ComponentManager

__all__ = [
    # Artifacts
    "Artifact",
    "PatentArtifact",
    "DatasheetArtifact",
    "SchematicArtifact",
    "ManualArtifact",
    "ComponentVersion",
    # Search results
    "SearchResult",
    "PatentResult",
    "DatasheetResult",
    "SchematicResult",
    "ManualResult",
    # Database
    "ProvenanceDB",
    # Retrievers
    "BaseRetriever",
    "PatentRetriever",
    "DatasheetRetriever",
    "SchematicRetriever",
    "ManualRetriever",
    # Component manager
    "ComponentManager",
]
