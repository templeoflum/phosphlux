"""
Event Bus

File-based event system for inter-agent communication.
Events are appended to a log file and subscribers poll for new events.
"""

import json
import time
import fnmatch
from pathlib import Path
from typing import Callable, Optional, List, Dict, Any
from datetime import datetime
from dataclasses import dataclass, asdict
from threading import Lock
import uuid


@dataclass
class Event:
    """An event in the system."""
    id: str
    timestamp: str
    source: str           # Agent that emitted: "research", "development", "documentation", "orchestrator"
    event_type: str       # e.g., "research.finding", "dev.screenshot"
    payload: Dict[str, Any]
    confidence: float = 1.0  # For verification tracking
    triggers: List[str] = None  # Events this might trigger

    def to_dict(self) -> dict:
        return asdict(self)

    @classmethod
    def from_dict(cls, data: dict) -> 'Event':
        return cls(**data)


class EventBus:
    """
    File-based event bus for agent communication.

    Events are:
    - Appended to events.jsonl (append-only log)
    - Subscribers poll for new events matching their patterns
    - Orchestrator can see all events
    """

    def __init__(self, state_dir: str = "swarm_state"):
        self.state_dir = Path(state_dir)
        self.events_file = self.state_dir / "orchestrator/events.jsonl"
        self.state_dir.mkdir(parents=True, exist_ok=True)
        self.events_file.parent.mkdir(parents=True, exist_ok=True)

        # In-memory state for this process
        self._subscriptions: Dict[str, List[str]] = {}  # agent_id -> [patterns]
        self._last_read_position: Dict[str, int] = {}   # agent_id -> file position
        self._handlers: Dict[str, Callable] = {}        # agent_id -> handler function
        self._lock = Lock()

        # Ensure events file exists
        if not self.events_file.exists():
            self.events_file.touch()

    def emit(self, source: str, event_type: str, payload: dict,
             confidence: float = 1.0, triggers: List[str] = None) -> Event:
        """
        Emit an event to the bus.

        Args:
            source: Agent emitting the event
            event_type: Type like "research.finding", "dev.screenshot"
            payload: Event data
            confidence: Confidence level (for verification tracking)
            triggers: List of event types this might trigger

        Returns:
            The created Event
        """
        event = Event(
            id=f"evt_{uuid.uuid4().hex[:12]}",
            timestamp=datetime.now().isoformat(),
            source=source,
            event_type=event_type,
            payload=payload,
            confidence=confidence,
            triggers=triggers or [],
        )

        # Append to log file
        with self._lock:
            with open(self.events_file, 'a') as f:
                f.write(json.dumps(event.to_dict()) + '\n')

        return event

    def subscribe(self, agent_id: str, patterns: List[str], handler: Callable = None):
        """
        Subscribe an agent to event patterns.

        Args:
            agent_id: Subscribing agent
            patterns: List of patterns like "research.*", "dev.screenshot"
            handler: Optional callback function(event) -> None
        """
        self._subscriptions[agent_id] = patterns
        self._last_read_position[agent_id] = self._get_file_end()
        if handler:
            self._handlers[agent_id] = handler

    def poll(self, agent_id: str) -> List[Event]:
        """
        Poll for new events matching agent's subscriptions.

        Returns events since last poll that match subscribed patterns.
        """
        if agent_id not in self._subscriptions:
            return []

        patterns = self._subscriptions[agent_id]
        last_pos = self._last_read_position.get(agent_id, 0)

        events = []
        with open(self.events_file, 'r') as f:
            f.seek(last_pos)
            for line in f:
                if line.strip():
                    event = Event.from_dict(json.loads(line))
                    if self._matches_patterns(event.event_type, patterns):
                        events.append(event)
            self._last_read_position[agent_id] = f.tell()

        return events

    def get_recent(self, since: str = None, event_types: List[str] = None,
                   limit: int = 100) -> List[Event]:
        """
        Get recent events, optionally filtered.

        Args:
            since: ISO timestamp - only events after this
            event_types: Filter to these types
            limit: Max events to return
        """
        events = []
        with open(self.events_file, 'r') as f:
            for line in f:
                if line.strip():
                    event = Event.from_dict(json.loads(line))

                    if since and event.timestamp < since:
                        continue
                    if event_types and event.event_type not in event_types:
                        continue

                    events.append(event)

        return events[-limit:]

    def get_all_since(self, timestamp: str) -> List[Event]:
        """Get all events since a timestamp (for orchestrator)."""
        return self.get_recent(since=timestamp, limit=10000)

    def _matches_patterns(self, event_type: str, patterns: List[str]) -> bool:
        """Check if event type matches any pattern."""
        for pattern in patterns:
            if fnmatch.fnmatch(event_type, pattern):
                return True
        return False

    def _get_file_end(self) -> int:
        """Get current end position of events file."""
        if not self.events_file.exists():
            return 0
        return self.events_file.stat().st_size

    # === Convenience methods for common events ===

    def emit_finding(self, topic: str, finding: str, confidence: float,
                     sources: list) -> Event:
        """Emit a research finding event."""
        return self.emit(
            source="research",
            event_type="research.finding",
            payload={
                "topic": topic,
                "finding": finding,
                "sources": sources,
            },
            confidence=confidence,
            triggers=["dev.review_requested"] if confidence >= 0.7 else [],
        )

    def emit_conflict(self, expected: str, actual: str, severity: str) -> Event:
        """Emit a research conflict event."""
        return self.emit(
            source="research",
            event_type="research.conflict",
            payload={
                "expected": expected,
                "actual": actual,
                "severity": severity,  # "minor", "major", "critical"
            },
            triggers=["dev.code_change_requested"],
        )

    def emit_screenshot(self, path: str, params: dict, signal: str) -> Event:
        """Emit a development screenshot event."""
        return self.emit(
            source="development",
            event_type="dev.screenshot",
            payload={
                "path": path,
                "params": params,
                "signal": signal,
            },
            triggers=["docs.logged", "research.validation_requested"],
        )

    def emit_code_changed(self, file: str, summary: str) -> Event:
        """Emit a code change event."""
        return self.emit(
            source="development",
            event_type="dev.code_changed",
            payload={
                "file": file,
                "summary": summary,
            },
            triggers=["docs.logged", "research.validation_requested"],
        )

    def emit_blocked(self, reason: str, needs: str) -> Event:
        """Emit a blocked event (needs input)."""
        return self.emit(
            source="development",
            event_type="dev.blocked",
            payload={
                "reason": reason,
                "needs": needs,
            },
            triggers=["orch.attention_needed"],
        )

    def emit_milestone(self, description: str) -> Event:
        """Emit a milestone event."""
        return self.emit(
            source="orchestrator",
            event_type="orch.milestone",
            payload={"description": description},
            triggers=["docs.ready_to_commit"],
        )

    def emit_pause(self, agent: str, reason: str) -> Event:
        """Emit a pause directive."""
        return self.emit(
            source="orchestrator",
            event_type="orch.pause",
            payload={
                "agent": agent,
                "reason": reason,
            },
        )

    def emit_resume(self, agent: str) -> Event:
        """Emit a resume directive."""
        return self.emit(
            source="orchestrator",
            event_type="orch.resume",
            payload={"agent": agent},
        )
