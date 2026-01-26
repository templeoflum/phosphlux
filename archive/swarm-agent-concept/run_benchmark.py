#!/usr/bin/env python3
"""
Model Benchmark Runner

Compare models and prompt variants for optimal research performance.
Tracks speed, accuracy, and finds the best configuration.
"""

import asyncio
import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from swarm import ModelBenchmark


# Ground truth from Paik/Abe schematics
PAIK_ABE_GROUND_TRUTH = {
    "colorizer": {
        "topic": "Paik/Abe colorizer",
        "facts": [
            "Uses 7 camera input channels (CH-1 through CH-7)",
            "Matrix and Mixing Amp combines channels",
            "Outputs to separate R, G, B channels",
            "Uses amplitude windows, not wavefolding",
            "Color banding from overlapping amplitude thresholds",
            "Key ICs include CA-3019, CA-3022, CA-3023",
            "Final output goes to encoder for NTSC signal",
        ],
        "source": "Paik-Abe Video Synthesizer Technical Documentation",
        "confidence": 0.95,
    },
    "amplitude windows": {
        "topic": "Paik/Abe amplitude windows",
        "facts": [
            "Input signal compared against multiple threshold levels",
            "Each threshold triggers a different color channel",
            "Overlapping windows create smooth color transitions",
            "Window positions set by potentiometers",
            "Window widths determine color band width",
        ],
        "source": "Paik-Abe Video Synthesizer - Schematics",
        "confidence": 0.9,
    },
}


async def run_benchmark(models: list, topic: str, state_dir: str):
    """Run benchmark across models and variants."""

    benchmark = ModelBenchmark(state_dir)

    # Add ground truth if not already present
    for key, gt in PAIK_ABE_GROUND_TRUTH.items():
        if key not in benchmark.ground_truth:
            benchmark.add_ground_truth(
                topic=gt["topic"],
                facts=gt["facts"],
                source=gt["source"],
                confidence=gt["confidence"],
            )
            print(f"Added ground truth: {gt['topic']}")

    print(f"\nBenchmarking {len(models)} models on topic: {topic}")
    print(f"Testing prompt variants: {list(benchmark.variants.keys())}")
    print("-" * 60)

    results = await benchmark.benchmark_models(
        models=models,
        topic=topic,
        context="Researching for accurate circuit emulation",
        focus="implementation",
    )

    print(f"\nCompleted {len(results)} benchmark runs")

    # Show results
    for result in results:
        print(f"\n{result.model_name} / {result.prompt_variant}:")
        print(f"  Latency: {result.latency_ms}ms")
        print(f"  Tokens/sec: {result.tokens_per_second:.1f}")
        print(f"  Composite score: {result.composite_score:.3f}")

    # Print rankings
    benchmark.print_rankings()

    # Show recommendation
    best_model, best_variant = benchmark.get_best_model()
    print(f"\nRECOMMENDATION: Use {best_model} with {best_variant} variant")

    return results


async def compare_prompts(model: str, topic: str, state_dir: str):
    """Compare different prompt variants on a single model."""

    benchmark = ModelBenchmark(state_dir)

    print(f"\nComparing prompt variants on {model}")
    print(f"Topic: {topic}")
    print("-" * 60)

    results = await benchmark.benchmark_models(
        models=[model],
        topic=topic,
    )

    # Sort by composite score
    results.sort(key=lambda r: r.composite_score, reverse=True)

    print("\nRanked by effectiveness:")
    for i, result in enumerate(results, 1):
        variant = benchmark.variants[result.prompt_variant]
        print(f"\n{i}. {variant.name} ({result.prompt_variant})")
        print(f"   Latency: {result.latency_ms}ms")
        print(f"   Score: {result.composite_score:.3f}")


def main():
    parser = argparse.ArgumentParser(description="Benchmark models and prompts")

    parser.add_argument(
        "command",
        choices=["benchmark", "compare-prompts", "rankings", "add-variant"],
        help="Command to run",
    )
    parser.add_argument(
        "--models",
        nargs="+",
        default=["phi3", "gemma3", "llama3.1"],
        help="Models to benchmark",
    )
    parser.add_argument(
        "--model",
        default="phi3",
        help="Single model for prompt comparison",
    )
    parser.add_argument(
        "--topic",
        default="Paik/Abe colorizer amplitude windows",
        help="Topic to test",
    )
    parser.add_argument(
        "--state-dir",
        default="swarm_state",
        help="State directory",
    )
    parser.add_argument(
        "--variant-id",
        help="Variant ID for add-variant",
    )
    parser.add_argument(
        "--variant-name",
        help="Variant name for add-variant",
    )
    parser.add_argument(
        "--system-prompt",
        help="System prompt for add-variant",
    )
    parser.add_argument(
        "--user-template",
        help="User template for add-variant",
    )

    args = parser.parse_args()

    if args.command == "benchmark":
        asyncio.run(run_benchmark(args.models, args.topic, args.state_dir))

    elif args.command == "compare-prompts":
        asyncio.run(compare_prompts(args.model, args.topic, args.state_dir))

    elif args.command == "rankings":
        benchmark = ModelBenchmark(args.state_dir)
        benchmark.print_rankings()

    elif args.command == "add-variant":
        if not all([args.variant_id, args.variant_name, args.system_prompt, args.user_template]):
            print("Error: add-variant requires --variant-id, --variant-name, --system-prompt, --user-template")
            sys.exit(1)

        benchmark = ModelBenchmark(args.state_dir)
        benchmark.add_prompt_variant(
            id=args.variant_id,
            name=args.variant_name,
            system_prompt=args.system_prompt,
            user_template=args.user_template,
        )
        print(f"Added variant: {args.variant_id}")


if __name__ == "__main__":
    main()
