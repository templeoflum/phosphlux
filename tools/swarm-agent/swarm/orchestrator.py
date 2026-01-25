"""
Orchestrator

Coordinates all agents, handles pausing, timeouts, and human interaction.
"""

import asyncio
import time
from typing import Dict, List, Optional, Callable, Any
from datetime import datetime, timedelta
import logging

from .shared_state import SharedState
from .event_bus import EventBus, Event
from .base_agent import BaseAgent
from .research_agent import ResearchAgent
from .development_agent import DevelopmentAgent
from .documentation_agent import DocumentationAgent
from .component_agent import ComponentAgent
from .token_tracker import TokenTracker
from .model_benchmark import ModelBenchmark


class Orchestrator:
    """
    Coordinates the agent swarm.

    Responsibilities:
    - Start/stop/pause agents
    - Monitor for timeouts and stuck agents
    - Handle human interrupts
    - Resolve conflicts requiring judgment
    - Track overall progress toward goal
    """

    def __init__(self, state_dir: str = "swarm_state",
                 research_dir: str = None,
                 components_dir: str = None,
                 ollama_url: str = None, ollama_model: str = None):
        self.state = SharedState(state_dir, research_dir=research_dir)
        self.events = EventBus(state_dir)
        self.logger = logging.getLogger("swarm.orchestrator")

        # Component library directory
        self.components_dir = components_dir

        # Token tracker for efficiency metrics
        self.token_tracker = TokenTracker(state_dir)

        # Model benchmark for self-improvement
        self.benchmark = ModelBenchmark(state_dir)

        # Ollama configuration for research agent
        self.ollama_url = ollama_url
        self.ollama_model = ollama_model

        # Agent instances
        self.agents: Dict[str, BaseAgent] = {}
        self._agent_tasks: Dict[str, asyncio.Task] = {}

        # State
        self._running = False
        self._paused = False
        self._goal = None
        self._start_time = None
        self._last_activity: Dict[str, datetime] = {}

        # Pending PDF downloads (for human-in-the-loop workflow)
        self._pending_pdfs: List[Dict] = []

        # Configuration
        self.config = {
            "timeout_seconds": 60,           # Agent timeout before intervention
            "max_iterations": 100,           # Max cycles before auto-pause
            "pause_on_conflict": True,       # Pause on critical conflicts
            "auto_commit": False,            # Auto-commit milestones
        }

        # Callbacks for human interaction
        self._on_pause_callback: Optional[Callable] = None
        self._on_decision_needed: Optional[Callable] = None
        self._on_pdf_needed: Optional[Callable] = None

        # Subscribe to events
        self.events.subscribe("orchestrator", ["*"])

    def set_callbacks(self,
                      on_pause: Callable = None,
                      on_decision: Callable = None,
                      on_pdf_needed: Callable = None):
        """Set callbacks for human interaction."""
        self._on_pause_callback = on_pause
        self._on_decision_needed = on_decision
        self._on_pdf_needed = on_pdf_needed

    async def start(self, goal: str):
        """Start the swarm with a goal."""
        self._goal = goal
        self._start_time = datetime.now()
        self._running = True
        self._paused = False

        self.state.set_goal(goal)
        self.logger.info(f"Starting swarm for goal: {goal}")

        # Create agents
        self.agents = {
            "research": ResearchAgent(
                self.state, self.events,
                token_tracker=self.token_tracker,
                ollama_url=self.ollama_url,
                ollama_model=self.ollama_model,
                benchmark=self.benchmark,
                auto_select_model=True,  # Use benchmark to pick optimal model
            ),
            "development": DevelopmentAgent(self.state, self.events),
            "documentation": DocumentationAgent(self.state, self.events),
            "component": ComponentAgent(
                self.state, self.events,
                components_dir=self.components_dir,
            ),
        }

        # Emit start directive
        self.events.emit(
            source="orchestrator",
            event_type="orch.directive",
            payload={"task": f"Begin work on: {goal}", "topic": goal},
        )

        # Start agent tasks
        for name, agent in self.agents.items():
            self._agent_tasks[name] = asyncio.create_task(
                self._run_agent(name, agent)
            )
            self._last_activity[name] = datetime.now()

        # Start monitoring loop
        try:
            await self._monitor_loop()
        except asyncio.CancelledError:
            self.logger.info("Orchestrator cancelled")
        finally:
            await self.stop()

    async def _run_agent(self, name: str, agent: BaseAgent):
        """Run an agent with error handling."""
        try:
            await agent.start()
        except Exception as e:
            self.logger.error(f"Agent {name} crashed: {e}", exc_info=True)
            self.state.update_agent_status(name, "error", {"error": str(e)})

    async def _monitor_loop(self):
        """Monitor agents and handle events."""
        cycle = 0

        while self._running:
            if self._paused:
                await asyncio.sleep(0.5)
                continue

            cycle += 1

            # Poll events
            events = self.events.poll("orchestrator")

            for event in events:
                await self._handle_event(event)

            # Check for timeouts
            await self._check_timeouts()

            # Check for completion
            if await self._check_completion():
                self.logger.info("Goal appears complete")
                await self._on_milestone("Goal completed")

            # Max iterations check
            if cycle >= self.config["max_iterations"]:
                self.logger.warning("Max iterations reached, pausing")
                await self.pause("max_iterations_reached")

            await asyncio.sleep(0.5)

    async def _handle_event(self, event: Event):
        """Handle events requiring orchestrator attention."""

        # Track activity
        if event.source in self._last_activity:
            self._last_activity[event.source] = datetime.now()

        # Handle specific events
        if event.event_type == "dev.blocked":
            needs = event.payload.get("needs", "")
            if needs == "human_decision":
                await self.pause(f"Human decision needed: {event.payload.get('reason', '')}")
                if self._on_decision_needed:
                    self._on_decision_needed(event.payload)

        elif event.event_type == "research.conflict":
            severity = event.payload.get("severity", "minor")
            if severity == "critical" and self.config["pause_on_conflict"]:
                await self.pause(f"Critical conflict: {event.payload}")

        elif event.event_type == "agent.pause_requested":
            reason = event.payload.get("reason", "")
            await self.pause(f"Agent requested pause: {reason}")

        elif event.event_type == "orch.milestone":
            await self._on_milestone(event.payload.get("description", ""))

        elif event.event_type == "research.pdf_needed":
            # Research agent found a PDF but needs human to download it
            pdf_info = {
                "request_id": event.payload.get("request_id"),
                "url": event.payload.get("url"),
                "description": event.payload.get("description"),
                "suggested_path": event.payload.get("suggested_path"),
            }
            self._pending_pdfs.append(pdf_info)
            self.logger.info(f"PDF download needed: {pdf_info['description']}")

            # Notify callback
            if self._on_pdf_needed:
                self._on_pdf_needed(pdf_info)

            # Pause for human to download
            await self.pause(f"PDF download required: {pdf_info['description']}")

        elif event.event_type == "component.datasheet_needed":
            # Component agent needs a datasheet downloaded
            part_number = event.payload.get("part_number", "unknown")
            search_url = event.payload.get("search_url", "")
            device = event.payload.get("device", "unknown")

            pdf_info = {
                "request_id": f"datasheet_{part_number}",
                "url": search_url,
                "description": f"Datasheet for {part_number} (used in {device})",
                "suggested_path": f"research/components/ics/{part_number.lower()}/",
            }
            self._pending_pdfs.append(pdf_info)
            self.logger.info(f"Datasheet needed: {part_number}")

            if self._on_pdf_needed:
                self._on_pdf_needed(pdf_info)

            await self.pause(f"Datasheet needed: {part_number}")

        elif event.event_type == "component.found":
            # Component found and documented
            part_number = event.payload.get("part_number", "unknown")
            comp_type = event.payload.get("type", "unknown")
            self.logger.info(f"Component documented: {part_number} ({comp_type})")

        elif event.event_type == "component.model_generated":
            # WGSL model generated for component
            part_number = event.payload.get("part_number", "unknown")
            path = event.payload.get("path", "")
            self.logger.info(f"WGSL model generated: {part_number} -> {path}")

    async def _check_timeouts(self):
        """Check for agent timeouts."""
        now = datetime.now()
        timeout = timedelta(seconds=self.config["timeout_seconds"])

        for name, last in self._last_activity.items():
            if now - last > timeout:
                status = self.state.get_agent_statuses().get(name, {})
                if status.get("status") == "running":
                    self.logger.warning(f"Agent {name} appears stuck (no activity for {timeout})")

                    # Emit timeout event
                    self.events.emit(
                        source="orchestrator",
                        event_type="orch.timeout",
                        payload={"agent": name, "last_activity": last.isoformat()},
                    )

                    # Pause for human review
                    await self.pause(f"Agent {name} timeout")

    async def _check_completion(self) -> bool:
        """Check if the goal appears complete."""
        # Check for milestone events
        recent = self.events.get_recent(limit=10)
        milestones = [e for e in recent if e.event_type == "orch.milestone"]

        if milestones:
            return True

        # Check if all agents are idle
        statuses = self.state.get_agent_statuses()
        all_idle = all(
            s.get("status") in ["idle", "paused"]
            for s in statuses.values()
        )

        return all_idle and len(recent) == 0

    async def _on_milestone(self, description: str):
        """Handle a milestone being reached."""
        self.logger.info(f"Milestone: {description}")

        if self.config["auto_commit"]:
            # Auto-commit (not implemented yet)
            pass
        else:
            # Pause for human review
            await self.pause(f"Milestone reached: {description}")

    async def pause(self, reason: str = "user_requested"):
        """Pause all agents."""
        self._paused = True
        self.logger.info(f"Pausing swarm: {reason}")

        # Pause all agents
        for name in self.agents:
            self.events.emit_pause(name, reason)

        # Notify callback
        if self._on_pause_callback:
            self._on_pause_callback(reason)

    async def resume(self):
        """Resume all agents."""
        self._paused = False
        self.logger.info("Resuming swarm")

        # Resume all agents
        for name in self.agents:
            self.events.emit_resume(name)

    async def stop(self):
        """Stop the swarm."""
        self._running = False
        self.logger.info("Stopping swarm")

        # Stop all agents
        for name, agent in self.agents.items():
            agent.stop()

        # Cancel tasks
        for name, task in self._agent_tasks.items():
            if not task.done():
                task.cancel()

        # Finalize documentation
        if "documentation" in self.agents:
            doc_agent = self.agents["documentation"]
            if isinstance(doc_agent, DocumentationAgent):
                doc_agent.finalize_session("stopped")

        # Print token usage summary
        self.token_tracker.print_summary()

    def get_status(self) -> Dict:
        """Get current swarm status."""
        token_stats = self.token_tracker.get_session_stats()
        best_model, best_variant = self.benchmark.get_best_model()

        return {
            "running": self._running,
            "paused": self._paused,
            "goal": self._goal,
            "uptime": str(datetime.now() - self._start_time) if self._start_time else None,
            "agents": self.state.get_agent_statuses(),
            "recent_events": [
                {"type": e.event_type, "source": e.source}
                for e in self.events.get_recent(limit=5)
            ],
            "pending_pdfs": len(self._pending_pdfs),
            "token_stats": {
                "cloud_tokens": token_stats["cloud_tokens"]["total"],
                "local_tokens": token_stats["local_tokens"]["total"],
                "savings_usd": token_stats["cost_analysis"]["savings_usd"],
            },
            "best_model": {
                "model": best_model,
                "variant": best_variant,
            },
        }

    # === Human interaction methods ===

    def send_directive(self, task: str, **kwargs):
        """Send a directive to agents."""
        self.events.emit(
            source="orchestrator",
            event_type="orch.directive",
            payload={"task": task, **kwargs},
        )

    def provide_decision(self, decision: str, context: Dict = None):
        """Provide a human decision."""
        self.events.emit(
            source="orchestrator",
            event_type="orch.decision",
            payload={
                "decision": decision,
                "context": context or {},
                "source": "human",
            },
        )

    def approve_commit(self) -> Dict:
        """Approve pending commit."""
        if "documentation" in self.agents:
            doc_agent = self.agents["documentation"]
            if isinstance(doc_agent, DocumentationAgent):
                # Get files and prepare commit
                # (Would gather changed files from state)
                return doc_agent.prepare_commit(
                    files=["src/devices/**/*.wgsl"],
                    summary=f"Implement: {self._goal}",
                )
        return {}

    def confirm_pdf_downloaded(self, request_id: str, file_path: str):
        """
        Confirm that a PDF has been downloaded by the user.

        Args:
            request_id: The ID from the pdf_needed event
            file_path: Where the user saved the file
        """
        # Emit event for research agent to pick up
        self.events.emit(
            source="orchestrator",
            event_type="user.pdf_downloaded",
            payload={
                "request_id": request_id,
                "file_path": file_path,
            },
        )

        # Remove from pending
        self._pending_pdfs = [
            p for p in self._pending_pdfs
            if p.get("request_id") != request_id
        ]

        self.logger.info(f"PDF confirmed: {file_path}")

    def get_pending_pdfs(self) -> List[Dict]:
        """Get list of PDFs waiting to be downloaded."""
        return self._pending_pdfs.copy()

    def get_token_stats(self) -> Dict:
        """Get token usage statistics."""
        return self.token_tracker.get_session_stats()

    def get_all_time_token_stats(self) -> Dict:
        """Get all-time token usage statistics."""
        return self.token_tracker.get_all_time_stats()

    def get_model_rankings(self) -> Dict:
        """Get model benchmark rankings."""
        return self.benchmark.get_model_rankings()

    def get_best_model(self) -> tuple:
        """Get the currently recommended model and variant."""
        return self.benchmark.get_best_model()

    def print_benchmark_rankings(self):
        """Print formatted benchmark rankings."""
        self.benchmark.print_rankings()


async def run_swarm(goal: str, state_dir: str = "swarm_state",
                    research_dir: str = None,
                    components_dir: str = None,
                    ollama_url: str = None, ollama_model: str = None):
    """
    Convenience function to run the swarm.

    Usage:
        asyncio.run(run_swarm(
            "Implement Sandin IP differencing amplifier",
            research_dir="research",
            components_dir="research/components",
        ))
    """
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s [%(name)s] %(levelname)s: %(message)s",
    )

    orchestrator = Orchestrator(
        state_dir,
        research_dir=research_dir,
        components_dir=components_dir,
        ollama_url=ollama_url,
        ollama_model=ollama_model,
    )

    def on_pause(reason):
        print(f"\n[PAUSED] {reason}")
        print("Commands: 'resume', 'status', 'stop', 'decide <choice>', 'tokens'")

    def on_pdf_needed(pdf_info):
        print(f"\n{'='*60}")
        print("PDF DOWNLOAD REQUIRED")
        print(f"{'='*60}")
        print(f"Description: {pdf_info.get('description', 'Unknown')}")
        print(f"URL: {pdf_info.get('url', 'N/A')}")
        print(f"Save to: {pdf_info.get('suggested_path', 'research/schematics/')}")
        print(f"Request ID: {pdf_info.get('request_id')}")
        print(f"{'='*60}")
        print("\nAfter downloading, use: pdf_done <request_id> <file_path>")
        print("Then 'resume' to continue.\n")

    orchestrator.set_callbacks(
        on_pause=on_pause,
        on_pdf_needed=on_pdf_needed,
    )

    await orchestrator.start(goal)
