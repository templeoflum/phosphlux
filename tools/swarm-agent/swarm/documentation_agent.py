"""
Documentation Agent

Records everything as it happens.
Maintains dev logs, captures artifacts, prepares commits.
"""

import asyncio
from typing import List, Optional, Dict, Any
from datetime import datetime
from pathlib import Path

from .base_agent import BaseAgent
from .shared_state import SharedState
from .event_bus import EventBus, Event


class DocumentationAgent(BaseAgent):
    """
    Documentation agent responsibilities:
    - Log all significant events
    - Capture and organize screenshots
    - Track research findings
    - Prepare commit messages
    - Identify documentation gaps
    """

    def __init__(self, state: SharedState, event_bus: EventBus,
                 docs_dir: str = "docs/dev-logs"):
        self.docs_dir = Path(docs_dir)
        self.docs_dir.mkdir(parents=True, exist_ok=True)

        self._pending_logs = []
        self._current_session_file = None
        self._session_start = datetime.now()
        self._event_count = 0
        self._findings_logged = set()
        self._screenshots_logged = set()

        super().__init__(state, event_bus)
        self._start_session()

    @property
    def agent_type(self) -> str:
        return "documentation"

    @property
    def subscriptions(self) -> List[str]:
        return [
            "research.*",       # All research events
            "dev.*",            # All development events
            "orch.*",           # All orchestrator events
        ]

    def _start_session(self):
        """Start a new documentation session."""
        timestamp = self._session_start.strftime("%Y%m%d_%H%M%S")
        goal = self.state.get_goal() or "unnamed_session"
        safe_goal = "".join(c if c.isalnum() or c in "-_" else "_" for c in goal[:30])

        self._current_session_file = self.docs_dir / f"{timestamp}_{safe_goal}.md"

        # Write session header
        header = f"""# Development Session: {goal}

**Started:** {self._session_start.isoformat()}
**Status:** In Progress

---

## Event Log

"""
        with open(self._current_session_file, 'w') as f:
            f.write(header)

    async def process_event(self, event: Event) -> Optional[List[Dict]]:
        """Process and log incoming events."""
        self._event_count += 1

        # Categorize and format the event
        log_entry = self._format_event(event)

        if log_entry:
            self._pending_logs.append(log_entry)

        # Check for gaps
        gaps = self._check_for_gaps(event)
        if gaps:
            return gaps

        return None

    async def do_work(self) -> Optional[List[Dict]]:
        """Write pending log entries."""
        if not self._pending_logs:
            return None

        # Write all pending logs
        with open(self._current_session_file, 'a') as f:
            for entry in self._pending_logs:
                f.write(entry + "\n")

        self._pending_logs = []

        return [{
            "type": "docs.logged",
            "payload": {"entries_written": len(self._pending_logs)},
        }]

    def _format_event(self, event: Event) -> Optional[str]:
        """Format an event as a markdown log entry."""
        timestamp = event.timestamp.split("T")[1].split(".")[0]  # HH:MM:SS
        source = event.source
        event_type = event.event_type.split(".")[-1]  # Last part of type

        # Format based on event type
        if event.event_type == "research.finding":
            topic = event.payload.get("topic", "unknown")
            finding = event.payload.get("finding", "")[:200]
            confidence = event.payload.get("confidence", 0)
            sources = event.payload.get("sources", [])

            finding_id = event.payload.get("finding_id", "")
            if finding_id in self._findings_logged:
                return None
            self._findings_logged.add(finding_id)

            return f"""### [{timestamp}] Research Finding

**Topic:** {topic}
**Confidence:** {confidence:.0%}

{finding}

**Sources:** {', '.join(sources) if sources else 'None cited'}

---
"""

        elif event.event_type == "research.conflict":
            expected = event.payload.get("expected", "")
            actual = event.payload.get("actual", "")
            severity = event.payload.get("severity", "minor")

            return f"""### [{timestamp}] Conflict Detected ({severity.upper()})

**Expected:** {expected}
**Actual:** {actual}

---
"""

        elif event.event_type == "dev.screenshot":
            path = event.payload.get("path", "")
            params = event.payload.get("params", {})

            if path in self._screenshots_logged:
                return None
            self._screenshots_logged.add(path)

            params_str = ", ".join(f"{k}={v}" for k, v in params.items())

            return f"""### [{timestamp}] Screenshot Captured

**Path:** `{path}`
**Parameters:** {params_str or 'default'}

![Screenshot]({path})

---
"""

        elif event.event_type == "dev.code_changed":
            file = event.payload.get("file", "unknown")
            summary = event.payload.get("summary", "")

            return f"""### [{timestamp}] Code Changed

**File:** `{file}`
**Summary:** {summary}

---
"""

        elif event.event_type == "dev.blocked":
            reason = event.payload.get("reason", "")
            needs = event.payload.get("needs", "")

            return f"""### [{timestamp}] Development Blocked

**Reason:** {reason}
**Needs:** {needs}

---
"""

        elif event.event_type == "orch.milestone":
            description = event.payload.get("description", "")

            return f"""### [{timestamp}] MILESTONE

**{description}**

---
"""

        elif event.event_type == "orch.directive":
            task = event.payload.get("task", "")

            return f"""### [{timestamp}] Orchestrator Directive

**Task:** {task}

---
"""

        # Default format for other events
        payload_str = ", ".join(f"{k}={v}" for k, v in event.payload.items())
        return f"- `[{timestamp}]` **{source}.{event_type}**: {payload_str}\n"

    def _check_for_gaps(self, event: Event) -> Optional[List[Dict]]:
        """Check if event reveals documentation gaps."""
        gaps = []

        # Check for unexplained changes
        if event.event_type == "dev.code_changed":
            # Do we have a finding that explains this change?
            findings = self.state.get_findings()
            if not findings:
                gaps.append({
                    "type": "docs.gap_identified",
                    "payload": {
                        "question": "What research informed this code change?",
                        "context": event.payload.get("summary", ""),
                    },
                })

        # Check for unvalidated screenshots
        if event.event_type == "dev.screenshot":
            # Will research validate this?
            # (Gap would be identified later if no validation comes)
            pass

        return gaps if gaps else None

    def prepare_commit(self, files: List[str], summary: str) -> Dict:
        """Prepare a commit with message and metadata."""
        # Gather context from recent events
        recent_events = self.events.get_recent(limit=20)

        findings = [e for e in recent_events if e.event_type == "research.finding"]
        changes = [e for e in recent_events if e.event_type == "dev.code_changed"]
        screenshots = [e for e in recent_events if e.event_type == "dev.screenshot"]

        # Build commit message
        message_lines = [summary, ""]

        if findings:
            message_lines.append("Research findings incorporated:")
            for f in findings[:3]:
                topic = f.payload.get("topic", "")
                message_lines.append(f"- {topic}")
            message_lines.append("")

        if screenshots:
            message_lines.append(f"Visual validation: {len(screenshots)} screenshots captured")
            message_lines.append("")

        message_lines.append("Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>")

        return {
            "files": files,
            "message": "\n".join(message_lines),
            "findings_count": len(findings),
            "screenshots_count": len(screenshots),
        }

    def finalize_session(self, outcome: str = "completed"):
        """Finalize the documentation session."""
        duration = datetime.now() - self._session_start

        footer = f"""
---

## Session Summary

**Outcome:** {outcome}
**Duration:** {duration}
**Events logged:** {self._event_count}
**Findings:** {len(self._findings_logged)}
**Screenshots:** {len(self._screenshots_logged)}

**Ended:** {datetime.now().isoformat()}
"""
        with open(self._current_session_file, 'a') as f:
            f.write(footer)

        # Emit completion event
        self.emit("docs.session_complete", {
            "file": str(self._current_session_file),
            "outcome": outcome,
            "events": self._event_count,
        })

    def _get_agent_state(self) -> dict:
        """Save agent-specific state."""
        return {
            "pending_logs": self._pending_logs,
            "session_file": str(self._current_session_file),
            "event_count": self._event_count,
            "findings_logged": list(self._findings_logged),
            "screenshots_logged": list(self._screenshots_logged),
        }

    def _restore_agent_state(self, state: dict):
        """Restore agent-specific state."""
        self._pending_logs = state.get("pending_logs", [])
        if state.get("session_file"):
            self._current_session_file = Path(state["session_file"])
        self._event_count = state.get("event_count", 0)
        self._findings_logged = set(state.get("findings_logged", []))
        self._screenshots_logged = set(state.get("screenshots_logged", []))
