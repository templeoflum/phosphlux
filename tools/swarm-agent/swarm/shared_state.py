"""
Shared State Manager

Filesystem-based shared state for swarm agents.
Research outputs are stored as actual documents (markdown notes, PDFs, etc.)
in the project's research folder structure.
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

    Research outputs go to the project's research folder as actual documents:
        research/
        ├── schematics/     ← PDF schematics
        ├── datasheets/     ← Component datasheets
        ├── patents/        ← Patent documents
        ├── manuals/        ← Device manuals
        ├── notes/          ← Markdown research notes
        └── RESEARCH_INDEX.md  ← Master index of collected materials

    Operational state stays in a local .swarm folder:
        .swarm/
        ├── orchestrator/
        │   ├── current_goal.json
        │   └── agent_status.json
        ├── codebase/
        │   ├── current_shader.wgsl
        │   └── parameters.json
        └── visuals/
            └── screenshots/
    """

    # Document categories in research folder
    DOCUMENT_CATEGORIES = ["schematics", "datasheets", "patents", "manuals", "notes"]

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
            for category in self.DOCUMENT_CATEGORIES:
                (self.research_dir / category).mkdir(parents=True, exist_ok=True)

    def _read_json(self, path: Path) -> Any:
        """Read JSON file with locking."""
        if not path.exists():
            return None
        with open(path, 'r') as f:
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
        with open(path, 'w') as f:
            try:
                lock_file(f, exclusive=True)
                json.dump(data, f, indent=2)
            except:
                json.dump(data, f, indent=2)
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

        filepath.write_text(note)
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
            content = f.read_text()
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

    def get_collected_documents(self, category: str = None) -> Dict[str, List[str]]:
        """
        Get list of collected documents by category.

        Returns dict of category -> list of filenames
        """
        if not self.research_dir:
            return {}

        result = {}
        categories = [category] if category else self.DOCUMENT_CATEGORIES

        for cat in categories:
            cat_dir = self.research_dir / cat
            if cat_dir.exists():
                files = [f.name for f in cat_dir.iterdir() if f.is_file()]
                result[cat] = files

        return result

    def update_research_index(self, device: str, category: str,
                              filename: str, source: str, status: str = "Complete"):
        """
        Update the RESEARCH_INDEX.md with a new document entry.

        Args:
            device: Device name
            category: Document category (schematics, datasheets, etc.)
            filename: Filename in the category folder
            source: Where the document came from
            status: Collection status
        """
        if not self.research_dir:
            return

        index_path = self.research_dir / "RESEARCH_INDEX.md"
        if not index_path.exists():
            return

        content = index_path.read_text()

        # Find the appropriate table and add entry
        # This is a simple approach - could be more sophisticated
        table_header = f"### {category.title()}"

        if table_header in content:
            # Find the table and add entry if not already there
            if filename not in content:
                # Find end of table (next ### or ---)
                lines = content.split('\n')
                new_lines = []
                in_table = False
                added = False

                for line in lines:
                    new_lines.append(line)
                    if table_header in line:
                        in_table = True
                    elif in_table and line.startswith('|') and '*none yet*' in line:
                        # Replace "none yet" row
                        new_lines[-1] = f"| {device} | `{category}/{filename}` | {source} | {status} |"
                        added = True
                        in_table = False
                    elif in_table and (line.startswith('###') or line.startswith('---')):
                        if not added:
                            # Insert before this line
                            new_lines.insert(-1, f"| {device} | `{category}/{filename}` | {source} | {status} |")
                        in_table = False

                content = '\n'.join(new_lines)
                index_path.write_text(content)

    def add_document_request(self, category: str, description: str,
                             url: str = None, device: str = None) -> Dict:
        """
        Record a request for a document that needs to be downloaded.

        Returns request info that can be shown to the user.
        """
        request = {
            "id": f"doc_{int(time.time()*1000)}",
            "category": category,
            "description": description,
            "url": url,
            "device": device,
            "requested_at": datetime.now().isoformat(),
            "suggested_path": f"research/{category}/",
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
        return self.add_document_request(
            category=ref_type if ref_type in self.DOCUMENT_CATEGORIES else "notes",
            description=title,
            url=url,
        )

    def add_open_question(self, question: str, context: str, priority: str = "normal"):
        """Legacy method - creates a note with the question."""
        return self.add_research_note(
            device="questions",
            title=question[:50],
            content=f"## Question\n\n{question}\n\n## Context\n\n{context}\n\n## Priority\n\n{priority}",
        )
