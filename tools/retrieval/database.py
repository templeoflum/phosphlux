"""
Provenance Database for artifact tracking.

Tracks all retrieved documents, component versions, R&D passes,
and relationships between artifacts.
"""

import sqlite3
import json
from datetime import datetime
from pathlib import Path
from typing import Optional, List, Dict, Any, Tuple
from contextlib import contextmanager

from .artifacts import (
    Artifact,
    PatentArtifact,
    DatasheetArtifact,
    SchematicArtifact,
    ManualArtifact,
    ComponentVersion,
)


# Default database location
DEFAULT_DB_PATH = Path(__file__).parent.parent.parent / "research" / "provenance.db"


class ProvenanceDB:
    """SQLite database for tracking artifact provenance."""

    def __init__(self, db_path: Optional[Path] = None):
        """Initialize database connection."""
        self.db_path = db_path or DEFAULT_DB_PATH
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        self._init_schema()

    @contextmanager
    def _connection(self):
        """Context manager for database connections."""
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        try:
            yield conn
            conn.commit()
        except Exception:
            conn.rollback()
            raise
        finally:
            conn.close()

    def _init_schema(self):
        """Initialize database schema."""
        with self._connection() as conn:
            conn.executescript("""
                -- Core artifact tracking
                CREATE TABLE IF NOT EXISTS artifacts (
                    id TEXT PRIMARY KEY,
                    type TEXT NOT NULL,
                    name TEXT NOT NULL,
                    path TEXT NOT NULL,
                    hash TEXT NOT NULL,
                    source_url TEXT,
                    retrieved_at TIMESTAMP,
                    metadata JSON
                );

                -- Version tracking for components
                CREATE TABLE IF NOT EXISTS versions (
                    id TEXT PRIMARY KEY,
                    artifact_id TEXT REFERENCES artifacts(id),
                    version INTEGER NOT NULL,
                    created_at TIMESTAMP,
                    created_by TEXT,
                    commit_hash TEXT,
                    notes TEXT,
                    UNIQUE(artifact_id, version)
                );

                -- R&D pass tracking
                CREATE TABLE IF NOT EXISTS rd_passes (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    device TEXT NOT NULL,
                    started_at TIMESTAMP,
                    completed_at TIMESTAMP,
                    goals TEXT,
                    outcomes TEXT
                );

                -- Link artifacts to R&D passes
                CREATE TABLE IF NOT EXISTS pass_artifacts (
                    pass_id TEXT REFERENCES rd_passes(id),
                    artifact_id TEXT REFERENCES artifacts(id),
                    role TEXT,
                    PRIMARY KEY(pass_id, artifact_id)
                );

                -- Source quality tracking
                CREATE TABLE IF NOT EXISTS sources (
                    id TEXT PRIMARY KEY,
                    url TEXT UNIQUE NOT NULL,
                    name TEXT,
                    type TEXT,
                    reliability INTEGER,
                    last_checked TIMESTAMP,
                    notes TEXT
                );

                -- Cross-references between artifacts
                CREATE TABLE IF NOT EXISTS relationships (
                    from_id TEXT REFERENCES artifacts(id),
                    to_id TEXT REFERENCES artifacts(id),
                    relationship TEXT,
                    PRIMARY KEY(from_id, to_id, relationship)
                );

                -- Create indexes for common queries
                CREATE INDEX IF NOT EXISTS idx_artifacts_type ON artifacts(type);
                CREATE INDEX IF NOT EXISTS idx_artifacts_name ON artifacts(name);
                CREATE INDEX IF NOT EXISTS idx_versions_artifact ON versions(artifact_id);
                CREATE INDEX IF NOT EXISTS idx_pass_artifacts_pass ON pass_artifacts(pass_id);
                CREATE INDEX IF NOT EXISTS idx_relationships_from ON relationships(from_id);
                CREATE INDEX IF NOT EXISTS idx_relationships_to ON relationships(to_id);
            """)

    # === Artifact Operations ===

    def add_artifact(self, artifact: Artifact) -> str:
        """Add or update an artifact in the database."""
        file_hash = artifact.compute_hash()

        with self._connection() as conn:
            conn.execute("""
                INSERT OR REPLACE INTO artifacts
                (id, type, name, path, hash, source_url, retrieved_at, metadata)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """, (
                artifact.id,
                artifact.type,
                artifact.name,
                str(artifact.path),
                file_hash,
                artifact.source_url,
                artifact.retrieved_at.isoformat() if artifact.retrieved_at else None,
                json.dumps(artifact.metadata),
            ))

        return artifact.id

    def get_artifact(self, artifact_id: str) -> Optional[Dict[str, Any]]:
        """Retrieve an artifact by ID."""
        with self._connection() as conn:
            row = conn.execute(
                "SELECT * FROM artifacts WHERE id = ?",
                (artifact_id,)
            ).fetchone()

            if row:
                return dict(row)
        return None

    def find_artifacts(
        self,
        artifact_type: Optional[str] = None,
        name_pattern: Optional[str] = None,
        limit: int = 100
    ) -> List[Dict[str, Any]]:
        """Find artifacts by type and/or name pattern."""
        query = "SELECT * FROM artifacts WHERE 1=1"
        params = []

        if artifact_type:
            query += " AND type = ?"
            params.append(artifact_type)

        if name_pattern:
            query += " AND name LIKE ?"
            params.append(f"%{name_pattern}%")

        query += " ORDER BY retrieved_at DESC LIMIT ?"
        params.append(limit)

        with self._connection() as conn:
            rows = conn.execute(query, params).fetchall()
            return [dict(row) for row in rows]

    def delete_artifact(self, artifact_id: str) -> bool:
        """Delete an artifact and its relationships."""
        with self._connection() as conn:
            # Delete relationships
            conn.execute(
                "DELETE FROM relationships WHERE from_id = ? OR to_id = ?",
                (artifact_id, artifact_id)
            )
            conn.execute(
                "DELETE FROM pass_artifacts WHERE artifact_id = ?",
                (artifact_id,)
            )
            conn.execute(
                "DELETE FROM versions WHERE artifact_id = ?",
                (artifact_id,)
            )

            # Delete artifact
            cursor = conn.execute(
                "DELETE FROM artifacts WHERE id = ?",
                (artifact_id,)
            )
            return cursor.rowcount > 0

    # === Version Operations ===

    def add_version(self, version: ComponentVersion) -> str:
        """Add a component version."""
        with self._connection() as conn:
            conn.execute("""
                INSERT INTO versions
                (id, artifact_id, version, created_at, created_by, commit_hash, notes)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            """, (
                version.id,
                f"component:{version.component}",
                version.version,
                version.created_at.isoformat(),
                version.created_by,
                version.commit_hash,
                version.notes,
            ))

        return version.id

    def get_versions(self, component: str) -> List[Dict[str, Any]]:
        """Get all versions for a component."""
        artifact_id = f"component:{component}"

        with self._connection() as conn:
            rows = conn.execute("""
                SELECT * FROM versions
                WHERE artifact_id = ?
                ORDER BY version DESC
            """, (artifact_id,)).fetchall()
            return [dict(row) for row in rows]

    def get_latest_version(self, component: str) -> Optional[int]:
        """Get the latest version number for a component."""
        artifact_id = f"component:{component}"

        with self._connection() as conn:
            row = conn.execute("""
                SELECT MAX(version) as max_version FROM versions
                WHERE artifact_id = ?
            """, (artifact_id,)).fetchone()

            if row and row["max_version"] is not None:
                return row["max_version"]
        return None

    # === R&D Pass Operations ===

    def start_rd_pass(
        self,
        name: str,
        device: str,
        goals: str = ""
    ) -> str:
        """Start a new R&D pass."""
        pass_id = f"rd:{device}:{name}:{datetime.now().strftime('%Y%m%d%H%M%S')}"

        with self._connection() as conn:
            conn.execute("""
                INSERT INTO rd_passes
                (id, name, device, started_at, goals)
                VALUES (?, ?, ?, ?, ?)
            """, (
                pass_id,
                name,
                device,
                datetime.now().isoformat(),
                goals,
            ))

        return pass_id

    def complete_rd_pass(self, pass_id: str, outcomes: str = "") -> bool:
        """Mark an R&D pass as complete."""
        with self._connection() as conn:
            cursor = conn.execute("""
                UPDATE rd_passes
                SET completed_at = ?, outcomes = ?
                WHERE id = ?
            """, (
                datetime.now().isoformat(),
                outcomes,
                pass_id,
            ))
            return cursor.rowcount > 0

    def link_artifact_to_pass(
        self,
        pass_id: str,
        artifact_id: str,
        role: str = "reference"
    ) -> None:
        """Link an artifact to an R&D pass."""
        with self._connection() as conn:
            conn.execute("""
                INSERT OR REPLACE INTO pass_artifacts
                (pass_id, artifact_id, role)
                VALUES (?, ?, ?)
            """, (pass_id, artifact_id, role))

    def get_pass_artifacts(self, pass_id: str) -> List[Dict[str, Any]]:
        """Get all artifacts linked to an R&D pass."""
        with self._connection() as conn:
            rows = conn.execute("""
                SELECT a.*, pa.role
                FROM artifacts a
                JOIN pass_artifacts pa ON a.id = pa.artifact_id
                WHERE pa.pass_id = ?
            """, (pass_id,)).fetchall()
            return [dict(row) for row in rows]

    # === Relationship Operations ===

    def add_relationship(
        self,
        from_id: str,
        to_id: str,
        relationship: str
    ) -> None:
        """Add a relationship between artifacts."""
        with self._connection() as conn:
            conn.execute("""
                INSERT OR IGNORE INTO relationships
                (from_id, to_id, relationship)
                VALUES (?, ?, ?)
            """, (from_id, to_id, relationship))

    def get_relationships(
        self,
        artifact_id: str,
        direction: str = "both"
    ) -> List[Dict[str, Any]]:
        """Get relationships for an artifact."""
        results = []

        with self._connection() as conn:
            if direction in ("from", "both"):
                rows = conn.execute("""
                    SELECT to_id as related_id, relationship, 'outgoing' as direction
                    FROM relationships WHERE from_id = ?
                """, (artifact_id,)).fetchall()
                results.extend([dict(row) for row in rows])

            if direction in ("to", "both"):
                rows = conn.execute("""
                    SELECT from_id as related_id, relationship, 'incoming' as direction
                    FROM relationships WHERE to_id = ?
                """, (artifact_id,)).fetchall()
                results.extend([dict(row) for row in rows])

        return results

    # === Source Operations ===

    def add_source(
        self,
        url: str,
        name: str,
        source_type: str,
        reliability: int = 3,
        notes: str = ""
    ) -> str:
        """Add or update a knowledge source."""
        source_id = f"source:{url.replace('://', '_').replace('/', '_')[:50]}"

        with self._connection() as conn:
            conn.execute("""
                INSERT OR REPLACE INTO sources
                (id, url, name, type, reliability, last_checked, notes)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            """, (
                source_id,
                url,
                name,
                source_type,
                reliability,
                datetime.now().isoformat(),
                notes,
            ))

        return source_id

    def get_sources(self, source_type: Optional[str] = None) -> List[Dict[str, Any]]:
        """Get all sources, optionally filtered by type."""
        with self._connection() as conn:
            if source_type:
                rows = conn.execute(
                    "SELECT * FROM sources WHERE type = ? ORDER BY reliability DESC",
                    (source_type,)
                ).fetchall()
            else:
                rows = conn.execute(
                    "SELECT * FROM sources ORDER BY reliability DESC"
                ).fetchall()
            return [dict(row) for row in rows]

    # === Statistics ===

    def get_stats(self) -> Dict[str, Any]:
        """Get database statistics."""
        with self._connection() as conn:
            stats = {}

            # Artifact counts by type
            rows = conn.execute("""
                SELECT type, COUNT(*) as count
                FROM artifacts
                GROUP BY type
            """).fetchall()
            stats["artifacts_by_type"] = {row["type"]: row["count"] for row in rows}

            # Total artifacts
            row = conn.execute("SELECT COUNT(*) as count FROM artifacts").fetchone()
            stats["total_artifacts"] = row["count"]

            # Version counts
            row = conn.execute("SELECT COUNT(*) as count FROM versions").fetchone()
            stats["total_versions"] = row["count"]

            # R&D pass counts
            row = conn.execute("SELECT COUNT(*) as count FROM rd_passes").fetchone()
            stats["total_rd_passes"] = row["count"]

            # Relationship counts
            row = conn.execute("SELECT COUNT(*) as count FROM relationships").fetchone()
            stats["total_relationships"] = row["count"]

            # Source counts
            row = conn.execute("SELECT COUNT(*) as count FROM sources").fetchone()
            stats["total_sources"] = row["count"]

            return stats

    def export_provenance(self, artifact_id: str) -> Dict[str, Any]:
        """Export full provenance for an artifact."""
        artifact = self.get_artifact(artifact_id)
        if not artifact:
            return {}

        return {
            "artifact": artifact,
            "relationships": self.get_relationships(artifact_id),
            "versions": self.get_versions(artifact_id.split(":")[-1])
                if artifact["type"] == "component" else [],
        }
