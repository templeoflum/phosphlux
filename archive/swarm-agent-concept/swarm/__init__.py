"""
Swarm Agent System

Multi-agent architecture for parallel research, development, and documentation.
"""

from .shared_state import SharedState
from .event_bus import EventBus, Event
from .base_agent import BaseAgent
from .research_agent import ResearchAgent
from .development_agent import DevelopmentAgent
from .documentation_agent import DocumentationAgent
from .component_agent import ComponentAgent
from .orchestrator import Orchestrator, run_swarm
from .token_tracker import TokenTracker, TokenTimer
from .model_benchmark import ModelBenchmark, PromptVariant, BenchmarkResult, SelfImprovementLoop

__all__ = [
    "SharedState",
    "EventBus",
    "Event",
    "BaseAgent",
    "ResearchAgent",
    "DevelopmentAgent",
    "DocumentationAgent",
    "ComponentAgent",
    "Orchestrator",
    "run_swarm",
    "TokenTracker",
    "TokenTimer",
    "ModelBenchmark",
    "PromptVariant",
    "BenchmarkResult",
    "SelfImprovementLoop",
]
