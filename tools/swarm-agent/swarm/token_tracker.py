"""
Token Tracker

Tracks token usage across cloud (Claude) and local (Ollama) models
to measure efficiency gains from the tiered architecture.
"""

import json
import time
from datetime import datetime
from pathlib import Path
from typing import Optional
from dataclasses import dataclass, asdict


@dataclass
class TokenUsage:
    """Single token usage record."""
    timestamp: str
    model_type: str       # "cloud" or "local"
    model_name: str       # "claude-opus-4-5", "llama3.2", etc.
    operation: str        # "research_query", "generate_shader", etc.
    input_tokens: int
    output_tokens: int
    latency_ms: int
    success: bool
    agent: str            # "research", "development", "documentation"


class TokenTracker:
    """
    Tracks token usage to measure cloud vs local efficiency.

    Estimates:
    - Cloud tokens saved by using local models
    - Cost savings (based on API pricing)
    - Latency differences
    """

    # Approximate pricing (per 1M tokens)
    CLOUD_INPUT_COST = 15.0   # Claude Opus input
    CLOUD_OUTPUT_COST = 75.0  # Claude Opus output
    LOCAL_COST = 0.0          # Ollama is free

    def __init__(self, state_dir: str = "swarm_state"):
        self.state_dir = Path(state_dir)
        self.usage_file = self.state_dir / "orchestrator/token_usage.jsonl"
        self.state_dir.mkdir(parents=True, exist_ok=True)
        self.usage_file.parent.mkdir(parents=True, exist_ok=True)

        # Session stats
        self._session_start = datetime.now()
        self._session_cloud_input = 0
        self._session_cloud_output = 0
        self._session_local_input = 0
        self._session_local_output = 0

    def record(self, model_type: str, model_name: str, operation: str,
               input_tokens: int, output_tokens: int, latency_ms: int,
               success: bool = True, agent: str = "unknown") -> TokenUsage:
        """Record a token usage event."""
        usage = TokenUsage(
            timestamp=datetime.now().isoformat(),
            model_type=model_type,
            model_name=model_name,
            operation=operation,
            input_tokens=input_tokens,
            output_tokens=output_tokens,
            latency_ms=latency_ms,
            success=success,
            agent=agent,
        )

        # Update session stats
        if model_type == "cloud":
            self._session_cloud_input += input_tokens
            self._session_cloud_output += output_tokens
        else:
            self._session_local_input += input_tokens
            self._session_local_output += output_tokens

        # Append to log
        with open(self.usage_file, 'a') as f:
            f.write(json.dumps(asdict(usage)) + '\n')

        return usage

    def record_cloud(self, operation: str, input_tokens: int, output_tokens: int,
                     latency_ms: int, agent: str = "orchestrator") -> TokenUsage:
        """Convenience method for cloud (Claude) usage."""
        return self.record(
            model_type="cloud",
            model_name="claude-opus-4-5",
            operation=operation,
            input_tokens=input_tokens,
            output_tokens=output_tokens,
            latency_ms=latency_ms,
            agent=agent,
        )

    def record_local(self, model_name: str, operation: str,
                     input_tokens: int, output_tokens: int,
                     latency_ms: int, agent: str = "research") -> TokenUsage:
        """Convenience method for local (Ollama) usage."""
        return self.record(
            model_type="local",
            model_name=model_name,
            operation=operation,
            input_tokens=input_tokens,
            output_tokens=output_tokens,
            latency_ms=latency_ms,
            agent=agent,
        )

    def get_session_stats(self) -> dict:
        """Get stats for current session."""
        total_cloud = self._session_cloud_input + self._session_cloud_output
        total_local = self._session_local_input + self._session_local_output
        total = total_cloud + total_local

        # Estimate what local work would have cost on cloud
        local_would_cost = (
            (self._session_local_input * self.CLOUD_INPUT_COST / 1_000_000) +
            (self._session_local_output * self.CLOUD_OUTPUT_COST / 1_000_000)
        )

        actual_cloud_cost = (
            (self._session_cloud_input * self.CLOUD_INPUT_COST / 1_000_000) +
            (self._session_cloud_output * self.CLOUD_OUTPUT_COST / 1_000_000)
        )

        return {
            "session_duration_seconds": (datetime.now() - self._session_start).total_seconds(),
            "cloud_tokens": {
                "input": self._session_cloud_input,
                "output": self._session_cloud_output,
                "total": total_cloud,
            },
            "local_tokens": {
                "input": self._session_local_input,
                "output": self._session_local_output,
                "total": total_local,
            },
            "totals": {
                "all_tokens": total,
                "cloud_percentage": (total_cloud / total * 100) if total > 0 else 0,
                "local_percentage": (total_local / total * 100) if total > 0 else 0,
            },
            "cost_analysis": {
                "actual_cloud_cost_usd": round(actual_cloud_cost, 4),
                "local_would_have_cost_usd": round(local_would_cost, 4),
                "savings_usd": round(local_would_cost, 4),  # Local is free
                "savings_percentage": (local_would_cost / (local_would_cost + actual_cloud_cost) * 100)
                                      if (local_would_cost + actual_cloud_cost) > 0 else 0,
            },
        }

    def get_all_time_stats(self) -> dict:
        """Get stats from all recorded sessions."""
        if not self.usage_file.exists():
            return {"error": "No usage data found"}

        cloud_input = 0
        cloud_output = 0
        local_input = 0
        local_output = 0
        cloud_latency = []
        local_latency = []
        operations = {}

        with open(self.usage_file, 'r') as f:
            for line in f:
                if line.strip():
                    record = json.loads(line)
                    op = record.get("operation", "unknown")
                    operations[op] = operations.get(op, 0) + 1

                    if record.get("model_type") == "cloud":
                        cloud_input += record.get("input_tokens", 0)
                        cloud_output += record.get("output_tokens", 0)
                        cloud_latency.append(record.get("latency_ms", 0))
                    else:
                        local_input += record.get("input_tokens", 0)
                        local_output += record.get("output_tokens", 0)
                        local_latency.append(record.get("latency_ms", 0))

        total_cloud = cloud_input + cloud_output
        total_local = local_input + local_output
        total = total_cloud + total_local

        # Cost analysis
        actual_cost = (
            (cloud_input * self.CLOUD_INPUT_COST / 1_000_000) +
            (cloud_output * self.CLOUD_OUTPUT_COST / 1_000_000)
        )
        would_have_cost = (
            ((cloud_input + local_input) * self.CLOUD_INPUT_COST / 1_000_000) +
            ((cloud_output + local_output) * self.CLOUD_OUTPUT_COST / 1_000_000)
        )

        return {
            "cloud_tokens": {
                "input": cloud_input,
                "output": cloud_output,
                "total": total_cloud,
                "avg_latency_ms": sum(cloud_latency) / len(cloud_latency) if cloud_latency else 0,
            },
            "local_tokens": {
                "input": local_input,
                "output": local_output,
                "total": total_local,
                "avg_latency_ms": sum(local_latency) / len(local_latency) if local_latency else 0,
            },
            "totals": {
                "all_tokens": total,
                "cloud_percentage": (total_cloud / total * 100) if total > 0 else 0,
                "local_percentage": (total_local / total * 100) if total > 0 else 0,
            },
            "cost_analysis": {
                "actual_cost_usd": round(actual_cost, 4),
                "would_have_cost_usd": round(would_have_cost, 4),
                "savings_usd": round(would_have_cost - actual_cost, 4),
                "savings_percentage": ((would_have_cost - actual_cost) / would_have_cost * 100)
                                      if would_have_cost > 0 else 0,
            },
            "operations": operations,
        }

    def print_summary(self):
        """Print a formatted summary."""
        stats = self.get_session_stats()

        print("\n" + "=" * 60)
        print("TOKEN USAGE SUMMARY")
        print("=" * 60)
        print(f"\nSession duration: {stats['session_duration_seconds']:.1f}s")
        print(f"\nCloud (Claude):")
        print(f"  Input:  {stats['cloud_tokens']['input']:,} tokens")
        print(f"  Output: {stats['cloud_tokens']['output']:,} tokens")
        print(f"  Total:  {stats['cloud_tokens']['total']:,} tokens")
        print(f"\nLocal (Ollama):")
        print(f"  Input:  {stats['local_tokens']['input']:,} tokens")
        print(f"  Output: {stats['local_tokens']['output']:,} tokens")
        print(f"  Total:  {stats['local_tokens']['total']:,} tokens")
        print(f"\nDistribution:")
        print(f"  Cloud: {stats['totals']['cloud_percentage']:.1f}%")
        print(f"  Local: {stats['totals']['local_percentage']:.1f}%")
        print(f"\nCost Analysis:")
        print(f"  Actual cloud cost: ${stats['cost_analysis']['actual_cloud_cost_usd']:.4f}")
        print(f"  Local saved:       ${stats['cost_analysis']['savings_usd']:.4f}")
        print(f"  Savings:           {stats['cost_analysis']['savings_percentage']:.1f}%")
        print("=" * 60 + "\n")


# Convenience function for timing operations
class TokenTimer:
    """Context manager for timing operations and recording tokens."""

    def __init__(self, tracker: TokenTracker, model_type: str, model_name: str,
                 operation: str, agent: str = "unknown"):
        self.tracker = tracker
        self.model_type = model_type
        self.model_name = model_name
        self.operation = operation
        self.agent = agent
        self.start_time = None
        self.input_tokens = 0
        self.output_tokens = 0

    def __enter__(self):
        self.start_time = time.time()
        return self

    def set_tokens(self, input_tokens: int, output_tokens: int):
        """Set token counts (call before exiting context)."""
        self.input_tokens = input_tokens
        self.output_tokens = output_tokens

    def __exit__(self, exc_type, exc_val, exc_tb):
        latency_ms = int((time.time() - self.start_time) * 1000)
        self.tracker.record(
            model_type=self.model_type,
            model_name=self.model_name,
            operation=self.operation,
            input_tokens=self.input_tokens,
            output_tokens=self.output_tokens,
            latency_ms=latency_ms,
            success=exc_type is None,
            agent=self.agent,
        )
        return False  # Don't suppress exceptions
