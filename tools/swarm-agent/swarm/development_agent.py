"""
Development Agent

Writes code, tunes parameters, captures visual output.
Interfaces with the workbench for real-time shader testing.
"""

import asyncio
import aiohttp
from typing import List, Optional, Dict, Any
from datetime import datetime
from pathlib import Path

from .base_agent import BaseAgent
from .shared_state import SharedState
from .event_bus import EventBus, Event


class DevelopmentAgent(BaseAgent):
    """
    Development agent responsibilities:
    - Write and modify shader code
    - Load shaders into workbench
    - Tune parameters based on visual feedback
    - Capture screenshots for validation
    - Track iterations and changes
    """

    def __init__(self, state: SharedState, event_bus: EventBus,
                 workbench_url: str = "http://localhost:3030"):
        self.workbench_url = workbench_url
        self._pending_tasks = []
        self._current_shader = None
        self._current_params = {}
        self._iteration_count = 0
        self._last_screenshot = None
        self._awaiting_validation = False
        super().__init__(state, event_bus)

    @property
    def agent_type(self) -> str:
        return "development"

    @property
    def subscriptions(self) -> List[str]:
        return [
            "research.finding",         # New information to incorporate
            "research.conflict",        # Need to adjust implementation
            "research.validation",      # Validation result
            "orch.directive",           # Implementation tasks
            "orch.pause",
            "orch.resume",
        ]

    async def process_event(self, event: Event) -> Optional[List[Dict]]:
        """Process incoming events."""

        if event.event_type == "research.finding":
            # New research finding - might need to incorporate
            finding = event.payload.get("finding", "")
            topic = event.payload.get("topic", "")
            confidence = event.payload.get("confidence", 0.5)

            self.logger.info(f"Received finding: {topic} (confidence: {confidence})")

            # Only act on high-confidence findings
            if confidence >= 0.7:
                self._pending_tasks.append({
                    "type": "incorporate_finding",
                    "finding": finding,
                    "topic": topic,
                    "confidence": confidence,
                })

        elif event.event_type == "research.conflict":
            # Research found a conflict with current implementation
            expected = event.payload.get("expected", "")
            actual = event.payload.get("actual", "")
            severity = event.payload.get("severity", "minor")

            self.logger.warning(f"Conflict detected: expected '{expected}', got '{actual}'")

            self._pending_tasks.append({
                "type": "resolve_conflict",
                "expected": expected,
                "actual": actual,
                "severity": severity,
                "priority": "high" if severity == "critical" else "normal",
            })

        elif event.event_type == "research.validation":
            # Validation result for our screenshot
            result = event.payload.get("result", "")
            self._awaiting_validation = False

            if result == "match":
                self.logger.info("Validation passed!")
            elif result == "mismatch":
                self.logger.warning("Validation failed - need to adjust")
                self._pending_tasks.append({
                    "type": "adjust_for_validation",
                    "details": event.payload,
                })

        elif event.event_type == "orch.directive":
            # Handle orchestrator directives
            task = event.payload.get("task", "")
            if "implement" in task.lower() or "develop" in task.lower():
                self._pending_tasks.append({
                    "type": "implement",
                    "description": task,
                    "details": event.payload,
                })

        return None

    async def do_work(self) -> Optional[List[Dict]]:
        """Execute pending development tasks."""
        events_to_emit = []

        # Don't do new work if awaiting validation
        if self._awaiting_validation:
            return None

        # Process tasks
        if self._pending_tasks:
            task = self._pending_tasks.pop(0)
            result = await self._execute_task(task)
            if result:
                events_to_emit.extend(result)

        return events_to_emit if events_to_emit else None

    async def _execute_task(self, task: Dict) -> Optional[List[Dict]]:
        """Execute a development task."""
        task_type = task.get("type")

        if task_type == "implement":
            return await self._implement(task)
        elif task_type == "incorporate_finding":
            return await self._incorporate_finding(task)
        elif task_type == "resolve_conflict":
            return await self._resolve_conflict(task)
        elif task_type == "adjust_for_validation":
            return await self._adjust_for_validation(task)

        return None

    async def _implement(self, task: Dict) -> List[Dict]:
        """Implement a new feature or device."""
        description = task.get("description", "")
        self.logger.info(f"Implementing: {description}")

        # For Phase 1, we scaffold the implementation
        # Real implementation would generate shader code

        events = []

        # Emit that we're starting
        events.append({
            "type": "dev.started",
            "payload": {
                "task": description,
                "iteration": self._iteration_count,
            },
        })

        # Check if we have enough information
        findings = self.state.get_findings()
        if not findings:
            # Need research first
            events.append({
                "type": "dev.blocked",
                "payload": {
                    "reason": f"No research findings for: {description}",
                    "needs": description,
                },
            })
            return events

        # We have findings, proceed with implementation
        # (In real implementation, this would generate shader code)
        self._iteration_count += 1

        return events

    async def _incorporate_finding(self, task: Dict) -> List[Dict]:
        """Incorporate a research finding into the implementation."""
        finding = task.get("finding", "")
        confidence = task.get("confidence", 0.5)

        self.logger.info(f"Incorporating finding (confidence: {confidence})")

        events = []

        # Mark that we've incorporated this
        events.append({
            "type": "dev.finding_incorporated",
            "payload": {
                "finding_summary": finding[:200],
                "iteration": self._iteration_count,
            },
            "confidence": confidence,
        })

        return events

    async def _resolve_conflict(self, task: Dict) -> List[Dict]:
        """Resolve a conflict between research and implementation."""
        expected = task.get("expected", "")
        actual = task.get("actual", "")
        severity = task.get("severity", "minor")

        self.logger.info(f"Resolving conflict: {expected} vs {actual}")

        events = []

        if severity == "critical":
            # Need human input for critical conflicts
            events.append({
                "type": "dev.blocked",
                "payload": {
                    "reason": f"Critical conflict: expected '{expected}', implemented '{actual}'",
                    "needs": "human_decision",
                    "options": [expected, actual],
                },
            })
        else:
            # For non-critical, implement both variants
            events.append({
                "type": "dev.variant_created",
                "payload": {
                    "variant_a": expected,
                    "variant_b": actual,
                    "reason": "Implementing both configurations per conflict resolution policy",
                },
            })

        return events

    async def _adjust_for_validation(self, task: Dict) -> List[Dict]:
        """Adjust implementation based on validation feedback."""
        details = task.get("details", {})

        self.logger.info("Adjusting based on validation feedback")

        # Queue parameter tuning or code adjustment
        self._iteration_count += 1

        return [{
            "type": "dev.adjustment_made",
            "payload": {
                "iteration": self._iteration_count,
                "reason": "validation_feedback",
            },
        }]

    # === Workbench interaction methods ===

    async def load_shader(self, code: str, name: str = "current") -> bool:
        """Load shader into workbench."""
        try:
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    f"{self.workbench_url}/shader",
                    json={"code": code, "name": name}
                ) as resp:
                    if resp.status == 200:
                        self._current_shader = code
                        self.state.set_current_shader(code, name)
                        self.emit("dev.code_changed", {
                            "file": f"{name}.wgsl",
                            "summary": f"Loaded shader: {name}",
                        })
                        return True
        except Exception as e:
            self.logger.error(f"Failed to load shader: {e}")
        return False

    async def set_params(self, params: Dict[str, float]) -> bool:
        """Set workbench parameters."""
        try:
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    f"{self.workbench_url}/params",
                    json=params
                ) as resp:
                    if resp.status == 200:
                        self._current_params.update(params)
                        self.state.set_parameters(self._current_params)
                        return True
        except Exception as e:
            self.logger.error(f"Failed to set params: {e}")
        return False

    async def take_screenshot(self, name: str = "output") -> Optional[str]:
        """Take screenshot from workbench."""
        try:
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    f"{self.workbench_url}/screenshot",
                    json={"name": name}
                ) as resp:
                    if resp.status == 200:
                        data = await resp.json()
                        path = data.get("path")
                        if path:
                            self._last_screenshot = path
                            self.state.add_screenshot(
                                path=path,
                                params=self._current_params,
                                signal="unknown",
                            )
                            self._awaiting_validation = True
                            self.events.emit_screenshot(
                                path=path,
                                params=self._current_params,
                                signal="unknown",
                            )
                            return path
        except Exception as e:
            self.logger.error(f"Failed to take screenshot: {e}")
        return None

    async def inject_signal(self, signal_type: str) -> bool:
        """Inject test signal into workbench."""
        try:
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    f"{self.workbench_url}/signal",
                    json={"type": signal_type}
                ) as resp:
                    return resp.status == 200
        except Exception as e:
            self.logger.error(f"Failed to inject signal: {e}")
        return False

    def _get_agent_state(self) -> dict:
        """Save agent-specific state."""
        return {
            "pending_tasks": self._pending_tasks,
            "current_shader": self._current_shader,
            "current_params": self._current_params,
            "iteration_count": self._iteration_count,
            "last_screenshot": self._last_screenshot,
            "awaiting_validation": self._awaiting_validation,
        }

    def _restore_agent_state(self, state: dict):
        """Restore agent-specific state."""
        self._pending_tasks = state.get("pending_tasks", [])
        self._current_shader = state.get("current_shader")
        self._current_params = state.get("current_params", {})
        self._iteration_count = state.get("iteration_count", 0)
        self._last_screenshot = state.get("last_screenshot")
        self._awaiting_validation = state.get("awaiting_validation", False)
