"""
Shared State Manager

Filesystem-based shared state for swarm agents.
Research outputs are stored in a 4-layer knowledge base:

    Innovators  → Who built it and why (design philosophy, artistic intent)
    Techniques  → How the signal processing works (device-agnostic methods)
    Devices     → Specific machines: ALL device docs (schematics, service manuals,
                  patents, calibration data) organized per device
    Components  → Individual parts: vendor datasheets, SPICE params, transfer
                  functions, WGSL models - organized per part number

Each document can be flagged for creative synthesis (inspire_flag) to
mark materials that could inform novel artistic applications beyond
faithful emulation.
"""

import json
import os
import re
import sys
import time
from pathlib import Path
from typing import Any, Optional, List, Dict
from datetime import datetime
from threading import Lock

# Cross-platform file locking
if sys.platform == 'win32':
    import msvcrt
    def lock_file(f, exclusive=True):
        try:
            msvcrt.locking(f.fileno(), msvcrt.LK_NBLCK if exclusive else msvcrt.LK_NBRLCK, 1)
        except:
            pass
    def unlock_file(f):
        try:
            msvcrt.locking(f.fileno(), msvcrt.LK_UNLCK, 1)
        except:
            pass
else:
    import fcntl
    def lock_file(f, exclusive=True):
        fcntl.flock(f.fileno(), fcntl.LOCK_EX if exclusive else fcntl.LOCK_SH)
    def unlock_file(f):
        fcntl.flock(f.fileno(), fcntl.LOCK_UN)


class SharedState:
    """
    Manages shared state across all agents.

    4-Layer Knowledge Base (research/):
        research/
        ├── innovators/     ← Key inventors, artists, engineers
        ├── techniques/     ← Abstract methods (device-agnostic)
        ├── devices/        ← Per-device: schematics, patents, manuals, calibration
        │   └── paik_abe/   ← All Paik/Abe docs together
        ├── components/     ← Per-part: vendor datasheets, spec.json, model.wgsl
        │   ├── ics/        ← Integrated circuit models
        │   └── functional/ ← Higher-level functional blocks
        ├── notes/          ← Working research notes
        └── RESEARCH_INDEX.md

    Operational state (.swarm/):
        .swarm/
        ├── orchestrator/
        │   ├── current_goal.json
        │   ├── agent_status.json
        │   └── pending_documents.json
        ├── codebase/
        │   ├── current_shader.wgsl
        │   └── parameters.json
        ├── visuals/screenshots/
        └── checkpoints/
    """

    # 4-layer knowledge base categories
    KNOWLEDGE_LAYERS = ["innovators", "techniques", "devices", "components"]

    # Document sub-types within the devices layer
    DEVICE_DOC_TYPES = ["schematics", "patents", "manuals", "calibration", "reference"]

    # Creative synthesis flags - documents marked for artistic exploration
    INSPIRE_FLAGS = [
        "glitch_potential",     # Circuit behavior under non-standard conditions
        "happy_accident",       # Unexpected visual results worth exploring
        "cross_device",         # Technique transferable between devices
        "power_starve",         # Behavior when power rails are degraded
        "component_abuse",      # Operating outside spec for creative effect
        "signal_feedback",      # Feedback loop possibilities
        "novel_topology",       # Unusual circuit connections
    ]

    def __init__(self, state_dir: str = ".swarm", research_dir: str = None):
        self.state_dir = Path(state_dir)
        self.research_dir = Path(research_dir) if research_dir else None
        self._init_directories()

    def _init_directories(self):
        """Create directory structure if it doesn't exist."""
        # Operational state directories
        state_dirs = [
            "codebase",
            "codebase/history",
            "visuals/screenshots",
            "orchestrator",
        ]
        for d in state_dirs:
            (self.state_dir / d).mkdir(parents=True, exist_ok=True)

        # Initialize state JSON files
        state_defaults = {
            "codebase/parameters.json": {},
            "orchestrator/current_goal.json": {"goal": None, "started": None},
            "orchestrator/agent_status.json": {},
        }
        for path, default in state_defaults.items():
            full_path = self.state_dir / path
            if not full_path.exists():
                self._write_json(full_path, default)

        # Research directories (if research_dir specified)
        if self.research_dir:
            for layer in self.KNOWLEDGE_LAYERS:
                (self.research_dir / layer).mkdir(parents=True, exist_ok=True)
            # Always ensure notes dir exists for working notes
            (self.research_dir / "notes").mkdir(parents=True, exist_ok=True)
            # Component sub-structure
            (self.research_dir / "components" / "ics").mkdir(parents=True, exist_ok=True)
            (self.research_dir / "components" / "functional").mkdir(parents=True, exist_ok=True)

    def _read_json(self, path: Path) -> Any:
        """Read JSON file with locking."""
        if not path.exists():
            return None
        with open(path, 'r', encoding='utf-8') as f:
            try:
                lock_file(f, exclusive=False)
                return json.load(f)
            except:
                return json.load(f)
            finally:
                unlock_file(f)

    def _write_json(self, path: Path, data: Any):
        """Write JSON file with locking."""
        path.parent.mkdir(parents=True, exist_ok=True)
        with open(path, 'w', encoding='utf-8') as f:
            try:
                lock_file(f, exclusive=True)
                json.dump(data, f, indent=2, ensure_ascii=False)
            except:
                json.dump(data, f, indent=2, ensure_ascii=False)
            finally:
                unlock_file(f)

    # === Research Documents ===

    def add_research_note(self, device: str, title: str, content: str,
                          sources: List[str] = None) -> str:
        """
        Add a research note as a markdown file.

        Args:
            device: Device being researched (e.g., "paik-abe")
            title: Note title
            content: Markdown content
            sources: List of source references

        Returns:
            Path to the created note file
        """
        if not self.research_dir:
            return ""

        notes_dir = self.research_dir / "notes"
        notes_dir.mkdir(parents=True, exist_ok=True)

        # Create filename from device and title
        slug = re.sub(r'[^a-z0-9]+', '-', f"{device}-{title}".lower()).strip('-')
        date_str = datetime.now().strftime("%Y-%m-%d")
        filename = f"{date_str}-{slug}.md"
        filepath = notes_dir / filename

        # Build note content
        note = f"# {title}\n\n"
        note += f"**Device:** {device}\n"
        note += f"**Date:** {date_str}\n\n"
        note += "---\n\n"
        note += content
        if sources:
            note += "\n\n---\n\n## Sources\n\n"
            for source in sources:
                note += f"- {source}\n"

        filepath.write_text(note, encoding='utf-8')
        return str(filepath)

    def get_research_notes(self, device: str = None) -> List[Dict]:
        """Get list of research notes, optionally filtered by device."""
        if not self.research_dir:
            return []

        notes_dir = self.research_dir / "notes"
        if not notes_dir.exists():
            return []

        notes = []
        for f in notes_dir.glob("*.md"):
            content = f.read_text(encoding='utf-8')
            # Parse device from content
            device_match = re.search(r'\*\*Device:\*\* (.+)', content)
            note_device = device_match.group(1) if device_match else "unknown"

            if device is None or device.lower() in note_device.lower():
                notes.append({
                    "path": str(f),
                    "filename": f.name,
                    "device": note_device,
                })

        return notes

    def get_collected_documents(self, layer: str = None) -> Dict[str, List[str]]:
        """
        Get list of collected documents by knowledge layer.

        Returns dict of layer -> list of filenames.
        For devices, walks subdirectories to find per-device docs.
        """
        if not self.research_dir:
            return {}

        result = {}
        layers = [layer] if layer else self.KNOWLEDGE_LAYERS + ["notes"]

        for lyr in layers:
            lyr_dir = self.research_dir / lyr
            if lyr_dir.exists():
                files = []
                for f in lyr_dir.rglob("*"):
                    if f.is_file() and f.name != "README.md":
                        # Show path relative to layer dir
                        files.append(str(f.relative_to(lyr_dir)))
                result[lyr] = files

        return result

    # === Device Document Management ===

    def add_device_document(self, device: str, doc_type: str, filename: str,
                            source: str, inspire_flags: List[str] = None) -> str:
        """
        Register a document for a specific device.

        Args:
            device: Device slug (e.g., "paik_abe")
            doc_type: One of DEVICE_DOC_TYPES (schematics, patents, manuals, etc.)
            filename: Filename within the device directory
            source: Where the document came from
            inspire_flags: Creative synthesis flags from INSPIRE_FLAGS

        Returns:
            Path to the document's metadata entry
        """
        if not self.research_dir:
            return ""

        device_dir = self.research_dir / "devices" / device
        device_dir.mkdir(parents=True, exist_ok=True)

        # Load or create device manifest
        manifest_path = device_dir / "manifest.json"
        manifest = self._read_json(manifest_path) or {
            "device": device,
            "documents": [],
            "inspire_flags": {},
        }

        # Add document entry
        doc_entry = {
            "filename": filename,
            "doc_type": doc_type,
            "source": source,
            "added": datetime.now().isoformat(),
            "inspire_flags": inspire_flags or [],
        }
        manifest["documents"].append(doc_entry)

        # Merge inspire flags
        for flag in (inspire_flags or []):
            if flag not in manifest["inspire_flags"]:
                manifest["inspire_flags"][flag] = []
            manifest["inspire_flags"][flag].append(filename)

        self._write_json(manifest_path, manifest)
        return str(manifest_path)

    def get_device_documents(self, device: str, doc_type: str = None) -> List[Dict]:
        """Get documents for a device, optionally filtered by type."""
        if not self.research_dir:
            return []

        manifest_path = self.research_dir / "devices" / device / "manifest.json"
        manifest = self._read_json(manifest_path)
        if not manifest:
            return []

        docs = manifest.get("documents", [])
        if doc_type:
            docs = [d for d in docs if d.get("doc_type") == doc_type]
        return docs

    # === Creative Synthesis Flags ===

    def flag_for_synthesis(self, layer: str, path: str, flags: List[str],
                           notes: str = "") -> Dict:
        """
        Flag a document or component for creative synthesis exploration.

        Args:
            layer: Knowledge layer (innovators, techniques, devices, components)
            path: Relative path within the layer
            flags: List of INSPIRE_FLAGS
            notes: Freeform notes about creative potential

        Returns:
            The flag entry
        """
        if not self.research_dir:
            return {}

        flags_file = self.research_dir / "creative_synthesis_flags.json"
        all_flags = self._read_json(flags_file) or {"entries": []}

        entry = {
            "layer": layer,
            "path": path,
            "flags": [f for f in flags if f in self.INSPIRE_FLAGS],
            "notes": notes,
            "flagged_at": datetime.now().isoformat(),
        }
        all_flags["entries"].append(entry)
        self._write_json(flags_file, all_flags)
        return entry

    def get_synthesis_flags(self, flag_type: str = None) -> List[Dict]:
        """Get all creative synthesis flags, optionally filtered by type."""
        if not self.research_dir:
            return []

        flags_file = self.research_dir / "creative_synthesis_flags.json"
        all_flags = self._read_json(flags_file) or {"entries": []}
        entries = all_flags.get("entries", [])

        if flag_type:
            entries = [e for e in entries if flag_type in e.get("flags", [])]
        return entries

    def update_research_index(self, device: str, layer: str,
                              filename: str, source: str, status: str = "Complete"):
        """
        Update the RESEARCH_INDEX.md with a new document entry.

        Args:
            device: Device name
            layer: Knowledge layer (innovators, techniques, devices, components)
            filename: Filename within the layer directory
            source: Where the document came from
            status: Collection status
        """
        if not self.research_dir:
            return

        index_path = self.research_dir / "RESEARCH_INDEX.md"
        if not index_path.exists():
            return

        content = index_path.read_text()

        # Build the path based on layer
        if layer == "devices" and device:
            doc_path = f"devices/{device}/{filename}"
        elif layer == "components":
            doc_path = f"components/{filename}"
        else:
            doc_path = f"{layer}/{filename}"

        # Find the appropriate table and add entry
        table_header = f"### {layer.title()}"

        if table_header in content:
            if filename not in content:
                lines = content.split('\n')
                new_lines = []
                in_table = False
                added = False

                for line in lines:
                    new_lines.append(line)
                    if table_header in line:
                        in_table = True
                    elif in_table and line.startswith('|') and '*none yet*' in line.lower():
                        new_lines[-1] = f"| {device} | `{doc_path}` | {source} | {status} |"
                        added = True
                        in_table = False
                    elif in_table and (line.startswith('###') or line.startswith('---')):
                        if not added:
                            new_lines.insert(-1, f"| {device} | `{doc_path}` | {source} | {status} |")
                        in_table = False

                content = '\n'.join(new_lines)
                index_path.write_text(content)

    def add_document_request(self, layer: str, description: str,
                             url: str = None, device: str = None,
                             doc_type: str = None) -> Dict:
        """
        Record a request for a document that needs to be downloaded.

        Args:
            layer: Knowledge layer (innovators, techniques, devices, components)
            description: What document is needed
            url: Where to find it
            device: Device slug if layer is "devices"
            doc_type: Document sub-type if layer is "devices"

        Returns request info that can be shown to the user.
        """
        # Build suggested path based on layer
        if layer == "devices" and device:
            suggested = f"research/devices/{device}/"
        elif layer == "components":
            suggested = f"research/components/ics/"
        else:
            suggested = f"research/{layer}/"

        request = {
            "id": f"doc_{int(time.time()*1000)}",
            "layer": layer,
            "doc_type": doc_type,
            "description": description,
            "url": url,
            "device": device,
            "requested_at": datetime.now().isoformat(),
            "suggested_path": suggested,
        }

        # Save to pending requests file
        requests_file = self.state_dir / "orchestrator/pending_documents.json"
        requests = self._read_json(requests_file) or []
        requests.append(request)
        self._write_json(requests_file, requests)

        return request

    def get_pending_documents(self) -> List[Dict]:
        """Get list of documents waiting to be downloaded."""
        requests_file = self.state_dir / "orchestrator/pending_documents.json"
        return self._read_json(requests_file) or []

    def confirm_document_downloaded(self, request_id: str, filename: str):
        """Mark a document request as fulfilled."""
        requests_file = self.state_dir / "orchestrator/pending_documents.json"
        requests = self._read_json(requests_file) or []
        requests = [r for r in requests if r.get("id") != request_id]
        self._write_json(requests_file, requests)

    # === Codebase ===

    def set_current_shader(self, code: str, name: str = "current"):
        """Set the current shader code."""
        path = self.state_dir / "codebase/current_shader.wgsl"

        # Archive previous version
        if path.exists():
            history_path = self.state_dir / f"codebase/history/{name}_{int(time.time())}.wgsl"
            path.rename(history_path)

        with open(path, 'w') as f:
            f.write(code)

    def get_current_shader(self) -> Optional[str]:
        """Get the current shader code."""
        path = self.state_dir / "codebase/current_shader.wgsl"
        if path.exists():
            return path.read_text()
        return None

    def set_parameters(self, params: dict):
        """Set current parameters."""
        self._write_json(self.state_dir / "codebase/parameters.json", params)

    def get_parameters(self) -> dict:
        """Get current parameters."""
        return self._read_json(self.state_dir / "codebase/parameters.json") or {}

    # === Orchestrator ===

    def set_goal(self, goal: str):
        """Set the current goal."""
        self._write_json(self.state_dir / "orchestrator/current_goal.json", {
            "goal": goal,
            "started": datetime.now().isoformat(),
        })

    def get_goal(self) -> Optional[str]:
        """Get the current goal."""
        data = self._read_json(self.state_dir / "orchestrator/current_goal.json")
        return data.get("goal") if data else None

    def update_agent_status(self, agent_id: str, status: str, metadata: dict = None):
        """Update an agent's status."""
        statuses = self._read_json(self.state_dir / "orchestrator/agent_status.json") or {}
        statuses[agent_id] = {
            "status": status,
            "updated": datetime.now().isoformat(),
            "metadata": metadata or {},
        }
        self._write_json(self.state_dir / "orchestrator/agent_status.json", statuses)

    def get_agent_statuses(self) -> dict:
        """Get all agent statuses."""
        return self._read_json(self.state_dir / "orchestrator/agent_status.json") or {}

    # === Legacy compatibility (for existing code) ===

    def add_finding(self, topic: str, finding: str, confidence: float,
                    sources: list = None, agent_id: str = None) -> str:
        """
        Legacy method - creates a research note instead of JSON.
        """
        note_path = self.add_research_note(
            device=topic.split()[0] if topic else "unknown",
            title=f"Finding: {topic[:50]}",
            content=finding,
            sources=sources,
        )
        return note_path

    def get_findings(self, topic: Optional[str] = None) -> list:
        """Legacy method - returns research notes."""
        notes = self.get_research_notes(device=topic)
        return [{"topic": n["device"], "path": n["path"]} for n in notes]

    def add_reference(self, url: str, title: str, ref_type: str, notes: str = ""):
        """Legacy method - adds document request."""
        # Map old category names to new layers
        layer_map = {
            "schematic": "devices",
            "schematics": "devices",
            "patent": "devices",
            "patents": "devices",
            "manual": "devices",
            "manuals": "devices",
            "datasheet": "components",
            "datasheets": "components",
        }
        layer = layer_map.get(ref_type, "notes")
        return self.add_document_request(
            layer=layer,
            description=title,
            url=url,
            doc_type=ref_type,
        )

    def add_open_question(self, question: str, context: str, priority: str = "normal"):
        """Legacy method - creates a note with the question."""
        return self.add_research_note(
            device="questions",
            title=question[:50],
            content=f"## Question\n\n{question}\n\n## Context\n\n{context}\n\n## Priority\n\n{priority}",
        )

    def save_checkpoint(self, agent_id: str, checkpoint: dict):
        """Save agent checkpoint for recovery."""
        checkpoint_dir = self.state_dir / "checkpoints"
        checkpoint_dir.mkdir(parents=True, exist_ok=True)
        checkpoint_file = checkpoint_dir / f"{agent_id}.json"
        self._write_json(checkpoint_file, checkpoint)

    def load_checkpoint(self, agent_id: str) -> Optional[dict]:
        """Load agent checkpoint if it exists."""
        checkpoint_file = self.state_dir / "checkpoints" / f"{agent_id}.json"
        if checkpoint_file.exists():
            return self._read_json(checkpoint_file)
        return None
