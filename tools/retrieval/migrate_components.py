"""
Migrate existing components to versioned directory structure.

This script:
1. Moves flat component files into v1/ subdirectories
2. Registers them in the provenance database
"""

import json
import shutil
from datetime import datetime
from pathlib import Path

from .database import ProvenanceDB
from .artifacts import ComponentVersion


def migrate_components(base_dir: Path = None, dry_run: bool = False):
    """
    Migrate existing flat component structure to versioned structure.

    Before: components/ics/ca3046/{spec.json, model.wgsl, ...}
    After:  components/ics/ca3046/v1/{spec.json, model.wgsl, ...}
    """
    base_dir = base_dir or Path(__file__).parent.parent.parent / "research"
    components_dir = base_dir / "components"
    db = ProvenanceDB(base_dir / "provenance.db")

    migrated = []

    for category_dir in components_dir.iterdir():
        if not category_dir.is_dir() or category_dir.name.startswith("."):
            continue

        for comp_dir in category_dir.iterdir():
            if not comp_dir.is_dir() or comp_dir.name.startswith("."):
                continue

            component_name = f"{category_dir.name}/{comp_dir.name}"

            # Check if already has version directories
            has_versions = any(
                d.is_dir() and d.name.startswith("v")
                for d in comp_dir.iterdir()
            )

            if has_versions:
                print(f"  {component_name}: already versioned, skipping")
                continue

            # Check if has component files
            files_to_migrate = [
                f for f in comp_dir.iterdir()
                if f.is_file() and not f.name.startswith(".")
            ]

            if not files_to_migrate:
                print(f"  {component_name}: no files to migrate, skipping")
                continue

            print(f"  {component_name}: migrating {len(files_to_migrate)} files to v1/")

            if dry_run:
                for f in files_to_migrate:
                    print(f"    - {f.name}")
                continue

            # Create v1 directory
            v1_dir = comp_dir / "v1"
            v1_dir.mkdir(exist_ok=True)

            # Move files to v1
            for f in files_to_migrate:
                dest = v1_dir / f.name
                shutil.move(str(f), str(dest))

            # Create provenance.json if it doesn't exist
            prov_path = v1_dir / "provenance.json"
            if not prov_path.exists():
                provenance = {
                    "component": component_name,
                    "version": 1,
                    "created_at": datetime.now().isoformat(),
                    "created_by": "migration",
                    "notes": "Migrated from flat structure",
                    "source_artifacts": [],
                }
                with open(prov_path, "w") as f:
                    json.dump(provenance, f, indent=2)

            # Load spec if exists
            spec_path = v1_dir / "spec.json"
            spec = {}
            if spec_path.exists():
                with open(spec_path) as f:
                    spec = json.load(f)

            # Register in database
            version_obj = ComponentVersion(
                id=f"component:{component_name}:v1",
                component=component_name,
                version=1,
                path=v1_dir,
                created_at=datetime.now(),
                created_by="migration",
                commit_hash=None,
                notes="Migrated from flat structure",
                spec=spec,
                source_artifacts=[],
            )
            db.add_version(version_obj)

            migrated.append(component_name)

    return migrated


def main():
    import argparse

    parser = argparse.ArgumentParser(description="Migrate components to versioned structure")
    parser.add_argument("--dry-run", "-n", action="store_true", help="Show what would be done")
    args = parser.parse_args()

    print("\nMigrating components to versioned structure...\n")

    migrated = migrate_components(dry_run=args.dry_run)

    if args.dry_run:
        print(f"\n[DRY RUN] Would migrate {len(migrated)} components")
    else:
        print(f"\nMigrated {len(migrated)} components")


if __name__ == "__main__":
    main()
