#!/usr/bin/env python3
"""
MCP Server Performance Benchmark Suite

Tests all tiers to establish:
- Cold-start latency (model loading)
- Warm latency (model already loaded)
- Tier switching overhead
- Recommended timeouts
- Failure detection thresholds
"""

import time
import json
import statistics
from datetime import datetime
from pathlib import Path
import subprocess

import ollama

# Models from mcp_server.py
MODELS = {
    "fast": "phi4-mini:latest",
    "deep": "llama3.1:latest",         # Matches mcp_server.py
    "vision": "moondream:latest",
    "embed": "qwen3-embedding:latest",
}

# Test queries per tier
TEST_QUERIES = {
    "fast": [
        "What is 2+2?",
        "Define: resistor",
        "Convert 1V to mV",
    ],
    "deep": [
        "Explain VCO in 2 sentences",
        "What is a folding amplifier?",
    ],
    "embed": [
        "analog video synthesis",
        "voltage controlled oscillator",
    ],
}

class BenchmarkResult:
    def __init__(self):
        self.results = {
            "timestamp": datetime.now().isoformat(),
            "tiers": {},
            "tier_switching": [],
            "recommendations": {}
        }

    def add_tier_result(self, tier: str, cold_time: float, warm_times: list, success: bool, error: str = None):
        self.results["tiers"][tier] = {
            "cold_start_sec": round(cold_time, 2),
            "warm_times_sec": [round(t, 2) for t in warm_times],
            "warm_mean_sec": round(statistics.mean(warm_times), 2) if warm_times else None,
            "warm_stdev_sec": round(statistics.stdev(warm_times), 2) if len(warm_times) > 1 else 0,
            "success": success,
            "error": error
        }

    def add_switch_result(self, from_tier: str, to_tier: str, time_sec: float):
        self.results["tier_switching"].append({
            "from": from_tier,
            "to": to_tier,
            "time_sec": round(time_sec, 2)
        })

    def compute_recommendations(self):
        """Calculate recommended timeouts based on empirical data."""
        recs = {}

        for tier, data in self.results["tiers"].items():
            if not data["success"]:
                recs[tier] = {"status": "FAILED", "reason": data["error"]}
                continue

            cold = data["cold_start_sec"]
            warm_mean = data["warm_mean_sec"] or cold
            warm_stdev = data["warm_stdev_sec"] or 0

            # Timeout = 3x warm mean + 2 stdev (covers 95% of cases)
            warm_timeout = warm_mean * 3 + warm_stdev * 2
            # Cold timeout = cold time + 50% buffer
            cold_timeout = cold * 1.5
            # Abort threshold = when it's clearly stuck (5x expected)
            abort_threshold = max(cold * 2, warm_mean * 5)

            recs[tier] = {
                "warm_timeout_sec": round(warm_timeout, 1),
                "cold_timeout_sec": round(cold_timeout, 1),
                "abort_threshold_sec": round(abort_threshold, 1),
                "expected_warm_sec": round(warm_mean, 1),
                "expected_cold_sec": round(cold, 1),
            }

        # Tier switching recommendation
        if self.results["tier_switching"]:
            switch_times = [s["time_sec"] for s in self.results["tier_switching"]]
            avg_switch = statistics.mean(switch_times)
            recs["tier_switching"] = {
                "expected_sec": round(avg_switch, 1),
                "timeout_sec": round(avg_switch * 1.5, 1),
                "abort_threshold_sec": round(avg_switch * 2.5, 1),
            }

        self.results["recommendations"] = recs

    def save(self, path: str):
        with open(path, "w") as f:
            json.dump(self.results, f, indent=2)

    def print_summary(self):
        print("\n" + "="*60)
        print("BENCHMARK RESULTS")
        print("="*60)

        for tier, data in self.results["tiers"].items():
            status = "[OK]" if data["success"] else "[FAIL]"
            print(f"\n{status} {tier.upper()} TIER ({MODELS[tier]})")
            if data["success"]:
                print(f"  Cold start: {data['cold_start_sec']}s")
                print(f"  Warm mean:  {data['warm_mean_sec']}s (±{data['warm_stdev_sec']}s)")
            else:
                print(f"  Error: {data['error']}")

        if self.results["tier_switching"]:
            print(f"\nTIER SWITCHING:")
            for switch in self.results["tier_switching"]:
                print(f"  {switch['from']} -> {switch['to']}: {switch['time_sec']}s")

        print("\n" + "-"*60)
        print("RECOMMENDED TIMEOUTS:")
        print("-"*60)
        for tier, rec in self.results["recommendations"].items():
            if isinstance(rec, dict) and "warm_timeout_sec" in rec:
                print(f"\n{tier}:")
                print(f"  Warm query timeout:  {rec['warm_timeout_sec']}s")
                print(f"  Cold start timeout:  {rec['cold_timeout_sec']}s")
                print(f"  Abort if exceeds:    {rec['abort_threshold_sec']}s")


def clear_models():
    """Unload all models for clean cold-start test."""
    print("Clearing loaded models...")
    try:
        running = ollama.ps()
        for model in running.get("models", []):
            name = model.get("name", "")
            if name:
                subprocess.run(["ollama", "stop", name], capture_output=True)
        time.sleep(2)
    except Exception as e:
        print(f"Warning: Could not clear models: {e}")


def test_chat_tier(model: str, query: str) -> tuple[float, bool, str]:
    """Test a chat-based tier, return (time, success, error)."""
    start = time.time()
    try:
        response = ollama.chat(
            model=model,
            messages=[{"role": "user", "content": query}]
        )
        elapsed = time.time() - start
        return elapsed, True, None
    except Exception as e:
        elapsed = time.time() - start
        return elapsed, False, str(e)


def test_embed_tier(model: str, text: str) -> tuple[float, bool, str]:
    """Test embedding tier, return (time, success, error)."""
    start = time.time()
    try:
        response = ollama.embeddings(model=model, prompt=text)
        elapsed = time.time() - start
        return elapsed, True, None
    except Exception as e:
        elapsed = time.time() - start
        return elapsed, False, str(e)


def run_tier_benchmark(tier: str, results: BenchmarkResult):
    """Benchmark a single tier."""
    model = MODELS[tier]
    queries = TEST_QUERIES.get(tier, ["test"])

    print(f"\nTesting {tier} tier ({model})...")

    # Cold start test
    clear_models()
    print(f"  Cold start test...")

    if tier == "embed":
        cold_time, success, error = test_embed_tier(model, queries[0])
    else:
        cold_time, success, error = test_chat_tier(model, queries[0])

    if not success:
        print(f"  FAILED: {error}")
        results.add_tier_result(tier, cold_time, [], False, error)
        return

    print(f"  Cold start: {cold_time:.2f}s")

    # Warm tests (model now loaded)
    warm_times = []
    for i, query in enumerate(queries[1:] if len(queries) > 1 else queries):
        print(f"  Warm test {i+1}...")
        if tier == "embed":
            t, s, e = test_embed_tier(model, query)
        else:
            t, s, e = test_chat_tier(model, query)

        if s:
            warm_times.append(t)
            print(f"    {t:.2f}s")
        else:
            print(f"    Failed: {e}")

    results.add_tier_result(tier, cold_time, warm_times, True)


def run_switch_benchmark(results: BenchmarkResult):
    """Benchmark tier switching overhead."""
    print("\nTesting tier switching...")

    switches = [
        ("fast", "deep"),
        ("deep", "fast"),
        ("fast", "embed"),
    ]

    for from_tier, to_tier in switches:
        # Ensure from_tier is loaded
        from_model = MODELS[from_tier]
        to_model = MODELS[to_tier]

        print(f"  Loading {from_tier}...")
        if from_tier == "embed":
            test_embed_tier(from_model, "warmup")
        else:
            test_chat_tier(from_model, "warmup")

        # Now switch
        print(f"  Switching {from_tier} -> {to_tier}...")
        start = time.time()
        if to_tier == "embed":
            test_embed_tier(to_model, "test")
        else:
            test_chat_tier(to_model, "test")
        switch_time = time.time() - start

        print(f"    {switch_time:.2f}s")
        results.add_switch_result(from_tier, to_tier, switch_time)


def main():
    print("="*60)
    print("MCP SERVER PERFORMANCE BENCHMARK")
    print("="*60)

    results = BenchmarkResult()

    # Test each tier
    for tier in ["fast", "deep", "embed"]:
        run_tier_benchmark(tier, results)

    # Test tier switching
    run_switch_benchmark(results)

    # Compute recommendations
    results.compute_recommendations()

    # Save and display
    output_path = Path(__file__).parent / "benchmark_results.json"
    results.save(str(output_path))
    results.print_summary()

    print(f"\nResults saved to: {output_path}")

    return results


if __name__ == "__main__":
    main()
