"""
Base Agent Class

Abstract base class for all swarm agents.
Handles common functionality: event polling, state access, lifecycle.
"""

import asyncio
from abc import ABC, abstractmethod
from typing import List, Optional, Dict, Any
from datetime import datetime
import logging

from .shared_state import SharedState
from .event_bus import EventBus, Event


class BaseAgent(ABC):
    """
    Abstract base class for swarm agents.

    Lifecycle:
        1. __init__: Setup state and event connections
        2. start(): Begin processing loop
        3. cycle(): Called repeatedly - poll events, do work, emit events
        4. pause(): Temporarily stop processing
        5. resume(): Continue processing
        6. stop(): Shut down agent

    Subclasses must implement:
        - agent_type: Property returning agent type string
        - subscriptions: Property returning event patterns to subscribe to
        - process_event(event): Handle a single event
        - do_work(): Perform agent's main work (called each cycle)
    """

    def __init__(self, state: SharedState, event_bus: EventBus):
        self.state = state
        self.events = event_bus
        self.logger = logging.getLogger(f"swarm.{self.agent_type}")

        self._running = False
        self._paused = False
        self._cycle_count = 0
        self._last_activity = None

        # Subscribe to events
        self.events.subscribe(
            agent_id=self.agent_type,
            patterns=self.subscriptions,
        )

        # Update status
        self.state.update_agent_status(self.agent_type, "idle")

    @property
    @abstractmethod
    def agent_type(self) -> str:
        """Return agent type identifier (e.g., 'research', 'development')."""
        pass

    @property
    @abstractmethod
    def subscriptions(self) -> List[str]:
        """Return list of event patterns to subscribe to."""
        pass

    @abstractmethod
    async def process_event(self, event: Event) -> Optional[List[Event]]:
        """
        Process a single event.

        Args:
            event: The event to process

        Returns:
            List of events to emit in response, or None
        """
        pass

    @abstractmethod
    async def do_work(self) -> Optional[List[Event]]:
        """
        Perform agent's main work for this cycle.

        Called each cycle after processing events.
        May return events to emit.
        """
        pass

    async def start(self):
        """Start the agent's processing loop."""
        self._running = True
        self._paused = False
        self.state.update_agent_status(self.agent_type, "running")
        self.logger.info(f"{self.agent_type} agent started")

        try:
            while self._running:
                if self._paused:
                    await asyncio.sleep(0.5)
                    continue

                await self.cycle()
                self._cycle_count += 1

                # Small delay between cycles
                await asyncio.sleep(0.1)

        except Exception as e:
            self.logger.error(f"Agent error: {e}", exc_info=True)
            self.state.update_agent_status(self.agent_type, "error", {"error": str(e)})
            raise

    async def cycle(self):
        """Run one processing cycle."""
        self._last_activity = datetime.now()

        # 1. Poll for new events
        events = self.events.poll(self.agent_type)

        # 2. Process each event
        for event in events:
            self.logger.debug(f"Processing event: {event.event_type}")

            # Check for pause/stop directives
            if event.event_type == "orch.pause" and event.payload.get("agent") == self.agent_type:
                await self.pause()
                continue
            if event.event_type == "orch.resume" and event.payload.get("agent") == self.agent_type:
                await self.resume()
                continue

            # Process the event
            try:
                response_events = await self.process_event(event)
                if response_events:
                    for resp in response_events:
                        self.events.emit(
                            source=self.agent_type,
                            event_type=resp["type"],
                            payload=resp.get("payload", {}),
                            confidence=resp.get("confidence", 1.0),
                        )
            except Exception as e:
                self.logger.error(f"Error processing event {event.id}: {e}")

        # 3. Do main work
        try:
            work_events = await self.do_work()
            if work_events:
                for evt in work_events:
                    self.events.emit(
                        source=self.agent_type,
                        event_type=evt["type"],
                        payload=evt.get("payload", {}),
                        confidence=evt.get("confidence", 1.0),
                    )
        except Exception as e:
            self.logger.error(f"Error in do_work: {e}")

        # 4. Save checkpoint periodically
        if self._cycle_count % 10 == 0:
            self._save_checkpoint()

    async def pause(self):
        """Pause agent processing."""
        self._paused = True
        self.state.update_agent_status(self.agent_type, "paused")
        self.logger.info(f"{self.agent_type} agent paused")

    async def resume(self):
        """Resume agent processing."""
        self._paused = False
        self.state.update_agent_status(self.agent_type, "running")
        self.logger.info(f"{self.agent_type} agent resumed")

    def stop(self):
        """Stop the agent."""
        self._running = False
        self._save_checkpoint()
        self.state.update_agent_status(self.agent_type, "stopped")
        self.logger.info(f"{self.agent_type} agent stopped")

    def _save_checkpoint(self):
        """Save current state for resume capability."""
        checkpoint = {
            "cycle_count": self._cycle_count,
            "last_activity": self._last_activity.isoformat() if self._last_activity else None,
            "paused": self._paused,
            "agent_state": self._get_agent_state(),
        }
        self.state.save_checkpoint(self.agent_type, checkpoint)

    def _get_agent_state(self) -> dict:
        """
        Get agent-specific state for checkpointing.
        Override in subclasses to save additional state.
        """
        return {}

    def _restore_from_checkpoint(self):
        """Restore agent state from checkpoint."""
        checkpoint = self.state.load_checkpoint(self.agent_type)
        if checkpoint:
            self._cycle_count = checkpoint.get("cycle_count", 0)
            self._paused = checkpoint.get("paused", False)
            self._restore_agent_state(checkpoint.get("agent_state", {}))
            self.logger.info(f"Restored from checkpoint (cycle {self._cycle_count})")

    def _restore_agent_state(self, agent_state: dict):
        """
        Restore agent-specific state from checkpoint.
        Override in subclasses.
        """
        pass

    # === Utility methods for subclasses ===

    def emit(self, event_type: str, payload: dict, confidence: float = 1.0) -> Event:
        """Convenience method to emit an event."""
        return self.events.emit(
            source=self.agent_type,
            event_type=event_type,
            payload=payload,
            confidence=confidence,
        )

    def log_action(self, action: str, details: dict = None):
        """Log an action with optional details."""
        self.logger.info(f"{action}: {details}" if details else action)

    async def request_pause(self, reason: str):
        """Request orchestrator to pause for clarification."""
        self.events.emit(
            source=self.agent_type,
            event_type="agent.pause_requested",
            payload={
                "agent": self.agent_type,
                "reason": reason,
            },
        )
        await self.pause()

    def is_confident(self, confidence: float, threshold: float = 0.7) -> bool:
        """Check if confidence meets threshold for autonomous action."""
        return confidence >= threshold

    def needs_verification(self, confidence: float) -> bool:
        """Check if action needs verification before proceeding."""
        return confidence < 0.9
