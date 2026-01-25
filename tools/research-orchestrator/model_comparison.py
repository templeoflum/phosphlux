#!/usr/bin/env python3
"""
Model Comparison Benchmark

Tests ALL installed models to find the best for each tier:
- Chat models (fast/deep tier)
- Vision models
- Embedding models
"""

import time
import json
from datetime import datetime
from pathlib import Path
import ollama

# Test queries
CHAT_QUERY = "What is a Paik/Abe folding amplifier? Be concise (2-3 sentences)."
VISION_QUERY = "Describe what you see in this image."
EMBED_TEXT = "analog video synthesis voltage controlled oscillator"

# All chat/reasoning models to test
CHAT_MODELS = [
    # Current selections
    "phi4-mini:latest",
    "llama3.1:latest",
    "qwen3:latest",
    # New models
    "gemma3n:latest",
    "gemma3:latest",
    "qwen2.5-coder:7b",
    "granite3.3:latest",
    "smollm2:1.7b",
    "tinyllama:latest",
    "phi4-mini-reasoning:latest",
    "lfm2.5-thinking:latest",
    "deepscaler:latest",
    "ministral-3:latest",
    "rnj-1:latest",
    "phi3:latest",
]

# Vision models
VISION_MODELS = [
    "moondream:latest",
    "llava:latest",
    "llama3.2-vision:latest",
]

# Embedding models
EMBED_MODELS = [
    "qwen3-embedding:latest",
    "bge-m3:latest",
    "mxbai-embed-large:latest",
    "nomic-embed-text:latest",
    "all-minilm:latest",
    "embeddinggemma:latest",
]


def test_chat_model(model: str) -> dict:
    """Test a chat model."""
    print(f"  Testing {model}...", end=" ", flush=True)

    start = time.time()
    try:
        response = ollama.chat(
            model=model,
            messages=[{"role": "user", "content": CHAT_QUERY}]
        )
        elapsed = time.time() - start
        content = response["message"]["content"]

        # Strip thinking tags if present
        answer = content
        if "</think>" in content:
            answer = content.split("</think>")[-1].strip()

        print(f"{elapsed:.1f}s ({len(answer)} chars)")

        return {
            "model": model,
            "time_sec": round(elapsed, 2),
            "answer_len": len(answer),
            "answer_preview": answer[:150].replace('\n', ' '),
            "success": True
        }
    except Exception as e:
        elapsed = time.time() - start
        print(f"FAILED ({e})")
        return {
            "model": model,
            "time_sec": round(elapsed, 2),
            "success": False,
            "error": str(e)
        }


def test_embed_model(model: str) -> dict:
    """Test an embedding model."""
    print(f"  Testing {model}...", end=" ", flush=True)

    start = time.time()
    try:
        response = ollama.embeddings(model=model, prompt=EMBED_TEXT)
        elapsed = time.time() - start
        dim = len(response["embedding"])

        print(f"{elapsed:.2f}s (dim={dim})")

        return {
            "model": model,
            "time_sec": round(elapsed, 3),
            "embedding_dim": dim,
            "success": True
        }
    except Exception as e:
        elapsed = time.time() - start
        print(f"FAILED ({e})")
        return {
            "model": model,
            "time_sec": round(elapsed, 2),
            "success": False,
            "error": str(e)
        }


def test_vision_model(model: str) -> dict:
    """Test a vision model with a simple prompt (no image for speed test)."""
    print(f"  Testing {model}...", end=" ", flush=True)

    start = time.time()
    try:
        # Just test response time with text query
        response = ollama.chat(
            model=model,
            messages=[{"role": "user", "content": "What is video synthesis?"}]
        )
        elapsed = time.time() - start
        content = response["message"]["content"]

        print(f"{elapsed:.1f}s ({len(content)} chars)")

        return {
            "model": model,
            "time_sec": round(elapsed, 2),
            "answer_len": len(content),
            "success": True
        }
    except Exception as e:
        elapsed = time.time() - start
        print(f"FAILED ({e})")
        return {
            "model": model,
            "time_sec": round(elapsed, 2),
            "success": False,
            "error": str(e)
        }


def main():
    print("=" * 60)
    print("FULL MODEL COMPARISON BENCHMARK")
    print("=" * 60)
    print(f"Started: {datetime.now().isoformat()}")

    results = {
        "timestamp": datetime.now().isoformat(),
        "chat": [],
        "vision": [],
        "embed": [],
    }

    # Test chat models
    print(f"\n{'='*60}")
    print("CHAT/REASONING MODELS")
    print(f"Query: {CHAT_QUERY}")
    print("=" * 60)

    for model in CHAT_MODELS:
        result = test_chat_model(model)
        results["chat"].append(result)

    # Test embedding models
    print(f"\n{'='*60}")
    print("EMBEDDING MODELS")
    print(f"Text: {EMBED_TEXT}")
    print("=" * 60)

    for model in EMBED_MODELS:
        result = test_embed_model(model)
        results["embed"].append(result)

    # Test vision models
    print(f"\n{'='*60}")
    print("VISION MODELS")
    print("=" * 60)

    for model in VISION_MODELS:
        result = test_vision_model(model)
        results["vision"].append(result)

    # Summary
    print("\n" + "=" * 60)
    print("SUMMARY - CHAT MODELS (sorted by speed)")
    print("=" * 60)

    successful_chat = [r for r in results["chat"] if r["success"]]
    successful_chat.sort(key=lambda x: x["time_sec"])

    print(f"\n{'Model':<30} {'Time':>8} {'Chars':>8}")
    print("-" * 48)
    for r in successful_chat:
        print(f"{r['model']:<30} {r['time_sec']:>7}s {r['answer_len']:>7}")

    print("\n" + "=" * 60)
    print("SUMMARY - EMBEDDING MODELS (sorted by speed)")
    print("=" * 60)

    successful_embed = [r for r in results["embed"] if r["success"]]
    successful_embed.sort(key=lambda x: x["time_sec"])

    print(f"\n{'Model':<30} {'Time':>8} {'Dim':>8}")
    print("-" * 48)
    for r in successful_embed:
        print(f"{r['model']:<30} {r['time_sec']:>7}s {r['embedding_dim']:>7}")

    print("\n" + "=" * 60)
    print("SUMMARY - VISION MODELS (sorted by speed)")
    print("=" * 60)

    successful_vision = [r for r in results["vision"] if r["success"]]
    successful_vision.sort(key=lambda x: x["time_sec"])

    print(f"\n{'Model':<30} {'Time':>8}")
    print("-" * 40)
    for r in successful_vision:
        print(f"{r['model']:<30} {r['time_sec']:>7}s")

    # Recommendations
    print("\n" + "=" * 60)
    print("RECOMMENDATIONS")
    print("=" * 60)

    if successful_chat:
        fastest = successful_chat[0]
        print(f"\nFAST TIER: {fastest['model']} ({fastest['time_sec']}s)")

        # Find best balance (fast but substantial answer)
        balanced = [r for r in successful_chat if r["answer_len"] > 100]
        if balanced:
            balanced.sort(key=lambda x: x["time_sec"])
            print(f"DEEP TIER: {balanced[0]['model']} ({balanced[0]['time_sec']}s, {balanced[0]['answer_len']} chars)")

    if successful_embed:
        fastest_embed = successful_embed[0]
        print(f"EMBED TIER: {fastest_embed['model']} ({fastest_embed['time_sec']}s, dim={fastest_embed['embedding_dim']})")

    if successful_vision:
        fastest_vision = successful_vision[0]
        print(f"VISION TIER: {fastest_vision['model']} ({fastest_vision['time_sec']}s)")

    # Save results
    output_path = Path(__file__).parent / "model_comparison_results.json"
    with open(output_path, "w") as f:
        json.dump(results, f, indent=2)

    print(f"\nResults saved to: {output_path}")


if __name__ == "__main__":
    main()
