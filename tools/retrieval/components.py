"""
Component Version Manager for WGSL component models.

Manages versioned component models with provenance tracking:
- Tracks evolution across R&D passes
- Links models to source artifacts (datasheets, schematics)
- Supports diffing between versions
"""

import json
import shutil
import logging
import subprocess
from datetime import datetime
from pathlib import Path
from typing import Optional, List, Dict, Any
from dataclasses import asdict

from .artifacts import ComponentVersion
from .database import ProvenanceDB, DEFAULT_DB_PATH


logger = logging.getLogger(__name__)


# Component categories
COMPONENT_CATEGORIES = {
    "ics": "Integrated Circuits (CA3046, LM1881, etc.)",
    "functional": "Functional Blocks (folding amp, colorizer, etc.)",
    "generators": "Signal Generators (oscillators, ramps, etc.)",
    "processors": "Signal Processors (mixers, keyers, etc.)",
}


class ComponentManager:
    """Manages versioned WGSL component models."""

    def __init__(
        self,
        base_dir: Optional[Path] = None,
        db: Optional[ProvenanceDB] = None,
    ):
        """Initialize component manager."""
        self.base_dir = base_dir or Path(__file__).parent.parent.parent / "research"
        self.components_dir = self.base_dir / "components"
        self.components_dir.mkdir(parents=True, exist_ok=True)

        self.db = db or ProvenanceDB(self.base_dir / "provenance.db")

    def _get_component_dir(self, component: str) -> Path:
        """Get directory for a component."""
        # component format: "category/name" e.g., "ics/ca3046"
        parts = component.lower().split("/")
        return self.components_dir.joinpath(*parts)

    def list_components(self, category: Optional[str] = None) -> List[Dict[str, Any]]:
        """
        List all components.

        Args:
            category: Optional category filter ('ics', 'functional', etc.)

        Returns:
            List of component info dictionaries
        """
        components = []

        search_dirs = []
        if category:
            cat_dir = self.components_dir / category.lower()
            if cat_dir.exists():
                search_dirs = [cat_dir]
        else:
            search_dirs = [
                d for d in self.components_dir.iterdir()
                if d.is_dir() and not d.name.startswith(".")
            ]

        for cat_dir in search_dirs:
            category_name = cat_dir.name
            for comp_dir in cat_dir.iterdir():
                if comp_dir.is_dir() and not comp_dir.name.startswith("."):
                    component_name = f"{category_name}/{comp_dir.name}"

                    # Find versions
                    versions = sorted([
                        int(v.name[1:])
                        for v in comp_dir.iterdir()
                        if v.is_dir() and v.name.startswith("v")
                    ])

                    # Load latest version metadata
                    latest_meta = {}
                    if versions:
                        latest_dir = comp_dir / f"v{versions[-1]}"
                        spec_path = latest_dir / "spec.json"
                        if spec_path.exists():
                            with open(spec_path) as f:
                                latest_meta = json.load(f)

                    components.append({
                        "component": component_name,
                        "category": category_name,
                        "name": comp_dir.name,
                        "versions": versions,
                        "latest_version": versions[-1] if versions else 0,
                        "latest_spec": latest_meta,
                        "path": comp_dir,
                    })

        return components

    def get_component(self, component: str, version: Optional[int] = None) -> Optional[Dict[str, Any]]:
        """
        Get component information.

        Args:
            component: Component path (e.g., "ics/ca3046")
            version: Optional specific version (default: latest)

        Returns:
            Component info dictionary or None
        """
        comp_dir = self._get_component_dir(component)
        if not comp_dir.exists():
            return None

        # Get all versions
        versions = sorted([
            int(v.name[1:])
            for v in comp_dir.iterdir()
            if v.is_dir() and v.name.startswith("v")
        ])

        if not versions:
            return None

        # Select version
        if version is None:
            version = versions[-1]
        elif version not in versions:
            logger.warning(f"Version {version} not found for {component}")
            return None

        version_dir = comp_dir / f"v{version}"

        # Load spec
        spec_path = version_dir / "spec.json"
        spec = {}
        if spec_path.exists():
            with open(spec_path) as f:
                spec = json.load(f)

        # Load provenance
        prov_path = version_dir / "provenance.json"
        provenance = {}
        if prov_path.exists():
            with open(prov_path) as f:
                provenance = json.load(f)

        # Find WGSL files
        wgsl_files = list(version_dir.glob("*.wgsl"))

        return {
            "component": component,
            "version": version,
            "versions_available": versions,
            "path": version_dir,
            "spec": spec,
            "provenance": provenance,
            "wgsl_files": wgsl_files,
        }

    def create_version(
        self,
        component: str,
        spec: Dict[str, Any],
        source_artifacts: Optional[List[str]] = None,
        created_by: str = "manual",
        notes: str = "",
        wgsl_files: Optional[Dict[str, str]] = None,
    ) -> ComponentVersion:
        """
        Create a new version of a component.

        Args:
            component: Component path (e.g., "ics/ca3046")
            spec: Specification dictionary
            source_artifacts: List of artifact IDs this version is based on
            created_by: Who/what created this version
            notes: Version notes
            wgsl_files: Dict of filename -> WGSL code

        Returns:
            ComponentVersion object
        """
        comp_dir = self._get_component_dir(component)
        comp_dir.mkdir(parents=True, exist_ok=True)

        # Determine new version number
        existing = sorted([
            int(v.name[1:])
            for v in comp_dir.iterdir()
            if v.is_dir() and v.name.startswith("v")
        ])
        new_version = (existing[-1] + 1) if existing else 1

        version_dir = comp_dir / f"v{new_version}"
        version_dir.mkdir()

        logger.info(f"Creating {component} v{new_version}")

        # Save spec
        spec_path = version_dir / "spec.json"
        with open(spec_path, "w") as f:
            json.dump(spec, f, indent=2, default=str)

        # Save WGSL files
        if wgsl_files:
            for filename, code in wgsl_files.items():
                wgsl_path = version_dir / filename
                with open(wgsl_path, "w") as f:
                    f.write(code)

        # Get git commit hash if in a repo
        commit_hash = self._get_git_commit()

        # Create provenance record
        provenance = {
            "component": component,
            "version": new_version,
            "created_at": datetime.now().isoformat(),
            "created_by": created_by,
            "commit_hash": commit_hash,
            "notes": notes,
            "source_artifacts": source_artifacts or [],
        }

        prov_path = version_dir / "provenance.json"
        with open(prov_path, "w") as f:
            json.dump(provenance, f, indent=2)

        # Create ComponentVersion object
        version_obj = ComponentVersion(
            id=f"component:{component}:v{new_version}",
            component=component,
            version=new_version,
            path=version_dir,
            created_at=datetime.now(),
            created_by=created_by,
            commit_hash=commit_hash,
            notes=notes,
            spec=spec,
            source_artifacts=source_artifacts or [],
        )

        # Track in database
        self.db.add_version(version_obj)

        # Create relationships to source artifacts
        if source_artifacts:
            for artifact_id in source_artifacts:
                self.db.add_relationship(
                    f"component:{component}",
                    artifact_id,
                    "derived_from"
                )

        logger.info(f"Created {component} v{new_version} at {version_dir}")
        return version_obj

    def get_history(self, component: str) -> List[Dict[str, Any]]:
        """
        Get version history for a component.

        Args:
            component: Component path

        Returns:
            List of version info dictionaries (newest first)
        """
        # Try database first
        db_versions = self.db.get_versions(component)
        if db_versions:
            return db_versions

        # Fall back to filesystem
        comp_dir = self._get_component_dir(component)
        if not comp_dir.exists():
            return []

        history = []
        for v_dir in sorted(comp_dir.iterdir(), reverse=True):
            if not v_dir.is_dir() or not v_dir.name.startswith("v"):
                continue

            version_num = int(v_dir.name[1:])

            # Load provenance
            prov_path = v_dir / "provenance.json"
            if prov_path.exists():
                with open(prov_path) as f:
                    prov = json.load(f)
            else:
                prov = {"version": version_num}

            history.append({
                "version": version_num,
                "path": v_dir,
                **prov,
            })

        return history

    def compare_versions(
        self,
        component: str,
        v1: int,
        v2: int,
    ) -> Dict[str, Any]:
        """
        Compare two versions of a component.

        Args:
            component: Component path
            v1: First version number
            v2: Second version number

        Returns:
            Diff dictionary with changes
        """
        comp_dir = self._get_component_dir(component)

        v1_dir = comp_dir / f"v{v1}"
        v2_dir = comp_dir / f"v{v2}"

        if not v1_dir.exists() or not v2_dir.exists():
            return {"error": "Version not found"}

        diff = {
            "component": component,
            "from_version": v1,
            "to_version": v2,
            "spec_changes": {},
            "wgsl_changes": {},
        }

        # Compare specs
        spec1_path = v1_dir / "spec.json"
        spec2_path = v2_dir / "spec.json"

        if spec1_path.exists() and spec2_path.exists():
            with open(spec1_path) as f:
                spec1 = json.load(f)
            with open(spec2_path) as f:
                spec2 = json.load(f)

            diff["spec_changes"] = self._diff_dicts(spec1, spec2)

        # Compare WGSL files
        v1_wgsl = {f.name: f.read_text() for f in v1_dir.glob("*.wgsl")}
        v2_wgsl = {f.name: f.read_text() for f in v2_dir.glob("*.wgsl")}

        all_files = set(v1_wgsl.keys()) | set(v2_wgsl.keys())
        for filename in all_files:
            if filename not in v1_wgsl:
                diff["wgsl_changes"][filename] = {"status": "added"}
            elif filename not in v2_wgsl:
                diff["wgsl_changes"][filename] = {"status": "removed"}
            elif v1_wgsl[filename] != v2_wgsl[filename]:
                diff["wgsl_changes"][filename] = {
                    "status": "modified",
                    "lines_v1": len(v1_wgsl[filename].splitlines()),
                    "lines_v2": len(v2_wgsl[filename].splitlines()),
                }

        return diff

    def _diff_dicts(self, d1: Dict, d2: Dict) -> Dict[str, Any]:
        """Compute differences between two dictionaries."""
        changes = {}

        all_keys = set(d1.keys()) | set(d2.keys())
        for key in all_keys:
            if key not in d1:
                changes[key] = {"status": "added", "value": d2[key]}
            elif key not in d2:
                changes[key] = {"status": "removed", "value": d1[key]}
            elif d1[key] != d2[key]:
                changes[key] = {
                    "status": "modified",
                    "old": d1[key],
                    "new": d2[key],
                }

        return changes

    def copy_version(
        self,
        component: str,
        from_version: int,
        notes: str = "",
    ) -> ComponentVersion:
        """
        Copy a version as the basis for a new version.

        Args:
            component: Component path
            from_version: Version to copy from
            notes: Notes for the new version

        Returns:
            New ComponentVersion object
        """
        source = self.get_component(component, from_version)
        if not source:
            raise ValueError(f"Version {from_version} not found for {component}")

        # Load source files
        wgsl_files = {
            f.name: f.read_text()
            for f in source["wgsl_files"]
        }

        return self.create_version(
            component=component,
            spec=source["spec"],
            source_artifacts=source["provenance"].get("source_artifacts", []),
            created_by=f"copy_from_v{from_version}",
            notes=notes or f"Copied from v{from_version}",
            wgsl_files=wgsl_files,
        )

    def export_component(
        self,
        component: str,
        version: Optional[int] = None,
        output_dir: Optional[Path] = None,
    ) -> Path:
        """
        Export a component version to a standalone directory.

        Args:
            component: Component path
            version: Version to export (default: latest)
            output_dir: Output directory (default: current dir)

        Returns:
            Path to exported directory
        """
        source = self.get_component(component, version)
        if not source:
            raise ValueError(f"Component {component} not found")

        version = source["version"]
        output_dir = output_dir or Path.cwd()

        export_name = f"{component.replace('/', '_')}_v{version}"
        export_path = output_dir / export_name

        # Copy version directory
        shutil.copytree(source["path"], export_path)

        logger.info(f"Exported {component} v{version} to {export_path}")
        return export_path

    def _get_git_commit(self) -> Optional[str]:
        """Get current git commit hash."""
        try:
            result = subprocess.run(
                ["git", "rev-parse", "HEAD"],
                capture_output=True,
                text=True,
                cwd=self.base_dir,
            )
            if result.returncode == 0:
                return result.stdout.strip()[:8]
        except Exception:
            pass
        return None


# CLI interface
def main():
    """Command-line interface for component management."""
    import argparse

    parser = argparse.ArgumentParser(description="Component Version Manager")
    subparsers = parser.add_subparsers(dest="command", help="Command")

    # List command
    list_parser = subparsers.add_parser("list", help="List components")
    list_parser.add_argument("--category", "-c", help="Filter by category")

    # Get command
    get_parser = subparsers.add_parser("get", help="Get component info")
    get_parser.add_argument("component", help="Component path (e.g., ics/ca3046)")
    get_parser.add_argument("--version", "-v", type=int, help="Specific version")

    # History command
    history_parser = subparsers.add_parser("history", help="Get version history")
    history_parser.add_argument("component", help="Component path")

    # New version command
    new_parser = subparsers.add_parser("new-version", help="Create new version")
    new_parser.add_argument("component", help="Component path")
    new_parser.add_argument("--notes", "-n", default="", help="Version notes")
    new_parser.add_argument("--from-version", type=int, help="Copy from version")

    # Compare command
    compare_parser = subparsers.add_parser("compare", help="Compare versions")
    compare_parser.add_argument("component", help="Component path")
    compare_parser.add_argument("v1", type=int, help="First version")
    compare_parser.add_argument("v2", type=int, help="Second version")

    args = parser.parse_args()

    manager = ComponentManager()

    if args.command == "list":
        components = manager.list_components(category=args.category)

        print(f"\nFound {len(components)} components:\n")
        for comp in components:
            print(f"  {comp['component']}")
            print(f"    Versions: {comp['versions']}")
            if comp['latest_spec'].get('description'):
                print(f"    Description: {comp['latest_spec']['description'][:60]}")
            print()

    elif args.command == "get":
        comp = manager.get_component(args.component, version=args.version)

        if comp:
            print(f"\nComponent: {comp['component']}")
            print(f"Version: {comp['version']}")
            print(f"Path: {comp['path']}")
            print(f"\nSpec:")
            print(json.dumps(comp['spec'], indent=2))
            print(f"\nWGSL files: {[f.name for f in comp['wgsl_files']]}")
        else:
            print(f"\nComponent not found: {args.component}")

    elif args.command == "history":
        history = manager.get_history(args.component)

        print(f"\nVersion history for {args.component}:\n")
        for h in history:
            print(f"  v{h['version']}")
            if h.get('created_at'):
                print(f"    Created: {h['created_at']}")
            if h.get('created_by'):
                print(f"    By: {h['created_by']}")
            if h.get('notes'):
                print(f"    Notes: {h['notes']}")
            print()

    elif args.command == "new-version":
        if args.from_version:
            version = manager.copy_version(
                args.component,
                args.from_version,
                notes=args.notes,
            )
        else:
            # Create empty version (need spec)
            version = manager.create_version(
                args.component,
                spec={"description": "New component version"},
                notes=args.notes,
            )

        print(f"\nCreated {args.component} v{version.version}")
        print(f"Path: {version.path}")

    elif args.command == "compare":
        diff = manager.compare_versions(args.component, args.v1, args.v2)

        print(f"\nComparing {args.component} v{args.v1} -> v{args.v2}:\n")

        if diff.get("spec_changes"):
            print("Spec changes:")
            for key, change in diff["spec_changes"].items():
                print(f"  {key}: {change['status']}")

        if diff.get("wgsl_changes"):
            print("\nWGSL changes:")
            for filename, change in diff["wgsl_changes"].items():
                print(f"  {filename}: {change['status']}")

    else:
        parser.print_help()


if __name__ == "__main__":
    main()
