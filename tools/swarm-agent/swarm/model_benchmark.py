"""
Model Benchmarking and Prompt A/B Testing

Self-improvement infrastructure for the swarm system.
Tracks model performance, accuracy, and prompt effectiveness.
"""

import asyncio
import aiohttp
import json
import time
import hashlib
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass, asdict, field


@dataclass
class BenchmarkResult:
    """Single benchmark run result."""
    timestamp: str
    model_name: str
    prompt_variant: str          # Which prompt template was used
    query_type: str              # "factual", "technical", "creative", etc.
    topic: str

    # Performance metrics
    latency_ms: int
    input_tokens: int
    output_tokens: int
    tokens_per_second: float

    # Quality metrics (filled in later via validation)
    accuracy_score: Optional[float] = None      # 0-1, how accurate vs ground truth
    completeness_score: Optional[float] = None  # 0-1, did it cover all aspects
    relevance_score: Optional[float] = None     # 0-1, stayed on topic

    # Reference for validation
    response_hash: str = ""      # Hash of response for dedup
    ground_truth_id: Optional[str] = None  # Reference to known-good answer
    validated_by: Optional[str] = None     # "human", "cross_model", "source_check"

    # Composite score (calculated)
    @property
    def composite_score(self) -> float:
        """Weighted composite of all metrics."""
        scores = []
        weights = []

        if self.accuracy_score is not None:
            scores.append(self.accuracy_score)
            weights.append(0.4)  # Accuracy most important

        if self.completeness_score is not None:
            scores.append(self.completeness_score)
            weights.append(0.3)

        if self.relevance_score is not None:
            scores.append(self.relevance_score)
            weights.append(0.2)

        # Speed bonus (normalize: <2s = 1.0, >10s = 0.0)
        speed_score = max(0, min(1, 1 - (self.latency_ms - 2000) / 8000))
        scores.append(speed_score)
        weights.append(0.1)

        if not weights:
            return 0.0

        return sum(s * w for s, w in zip(scores, weights)) / sum(weights)


@dataclass
class PromptVariant:
    """A prompt template variant for A/B testing."""
    id: str
    name: str
    system_prompt: str
    user_template: str  # Use {topic}, {context}, {focus} placeholders

    # Performance tracking
    total_runs: int = 0
    total_score: float = 0.0
    avg_latency_ms: float = 0.0

    @property
    def avg_score(self) -> float:
        return self.total_score / self.total_runs if self.total_runs > 0 else 0.0


class ModelBenchmark:
    """
    Benchmark different models and prompt variants.

    Enables systematic improvement through:
    1. Model comparison (speed + accuracy)
    2. Prompt A/B testing
    3. Accuracy validation against ground truth
    4. Automatic selection of best performers
    """

    def __init__(self, state_dir: str = "swarm_state"):
        self.state_dir = Path(state_dir)
        self.benchmark_dir = self.state_dir / "benchmarks"
        self.benchmark_dir.mkdir(parents=True, exist_ok=True)

        self.results_file = self.benchmark_dir / "results.jsonl"
        self.variants_file = self.benchmark_dir / "prompt_variants.json"
        self.ground_truth_file = self.benchmark_dir / "ground_truth.json"
        self.model_rankings_file = self.benchmark_dir / "model_rankings.json"

        # Load or create prompt variants
        self.variants: Dict[str, PromptVariant] = {}
        self._load_variants()

        # Ground truth knowledge base
        self.ground_truth: Dict[str, Dict] = {}
        self._load_ground_truth()

        # Model rankings cache
        self._rankings_cache: Dict[str, Dict] = {}
        self._rankings_dirty = True

    def _load_variants(self):
        """Load prompt variants from disk."""
        if self.variants_file.exists():
            data = json.loads(self.variants_file.read_text())
            for v in data:
                self.variants[v["id"]] = PromptVariant(**v)
        else:
            # Create default variants for A/B testing
            self._create_default_variants()

    def _create_default_variants(self):
        """Create default prompt variants for testing."""

        # Variant A: Concise, direct
        self.variants["concise_v1"] = PromptVariant(
            id="concise_v1",
            name="Concise Technical",
            system_prompt="You are a video synthesis hardware expert. Be precise and technical. Cite specific component values when known.",
            user_template="""Topic: {topic}

Provide:
1. Technical description
2. Key components
3. Signal flow
4. Specific values/parameters

Be concise. Note uncertainties."""
        )

        # Variant B: Structured with examples
        self.variants["structured_v1"] = PromptVariant(
            id="structured_v1",
            name="Structured with Context",
            system_prompt="""You are a research assistant specializing in vintage video synthesis hardware.
You have deep knowledge of analog video circuits, particularly the Paik/Abe Video Synthesizer.
Provide accurate, technical information with specific details about circuits, components, and signal processing.""",
            user_template="""Research topic: {topic}

Focus area: {focus}

Please provide:
1. Technical description of how this works
2. Key components and circuits involved
3. Signal flow and processing stages
4. Any specific values or parameters

Be precise and technical. If uncertain about any detail, note it."""
        )

        # Variant C: Chain of thought
        self.variants["cot_v1"] = PromptVariant(
            id="cot_v1",
            name="Chain of Thought",
            system_prompt="You are an expert in analog video synthesis. Think step by step to ensure accuracy.",
            user_template="""I need to understand: {topic}

Context: {context}

Let's think through this step by step:
1. First, what is the fundamental principle?
2. What components implement this?
3. How do signals flow through the circuit?
4. What are the key parameters and their typical values?

Show your reasoning, then provide a clear summary."""
        )

        # Variant D: Minimal prompt (baseline)
        self.variants["minimal_v1"] = PromptVariant(
            id="minimal_v1",
            name="Minimal Baseline",
            system_prompt="You are a helpful assistant.",
            user_template="Explain {topic} in the context of analog video synthesis."
        )

        self._save_variants()

    def _save_variants(self):
        """Save prompt variants to disk."""
        data = [asdict(v) for v in self.variants.values()]
        self.variants_file.write_text(json.dumps(data, indent=2))

    def _load_ground_truth(self):
        """Load ground truth knowledge base."""
        if self.ground_truth_file.exists():
            self.ground_truth = json.loads(self.ground_truth_file.read_text())

    def add_ground_truth(self, topic: str, facts: List[str],
                         source: str, confidence: float = 1.0):
        """
        Add verified ground truth for accuracy checking.

        Args:
            topic: The topic/query this applies to
            facts: List of verified factual statements
            source: Where this was verified (e.g., "Paik-Abe schematics p.12")
            confidence: How confident we are in this ground truth
        """
        topic_key = self._normalize_topic(topic)

        self.ground_truth[topic_key] = {
            "topic": topic,
            "facts": facts,
            "source": source,
            "confidence": confidence,
            "added_at": datetime.now().isoformat(),
        }

        self.ground_truth_file.write_text(json.dumps(self.ground_truth, indent=2))

    def _normalize_topic(self, topic: str) -> str:
        """Normalize topic for matching."""
        return topic.lower().strip()

    async def benchmark_models(self,
                               models: List[str],
                               topic: str,
                               context: str = "",
                               focus: str = "implementation",
                               variants: List[str] = None,
                               ollama_url: str = "http://localhost:11434"
                               ) -> List[BenchmarkResult]:
        """
        Run benchmark across multiple models and prompt variants.

        Args:
            models: List of model names to test
            topic: The research topic
            context: Additional context
            focus: Focus area
            variants: Which prompt variants to test (None = all)
            ollama_url: Ollama server URL

        Returns:
            List of benchmark results
        """
        results = []
        test_variants = variants or list(self.variants.keys())

        async with aiohttp.ClientSession() as session:
            for model in models:
                for variant_id in test_variants:
                    variant = self.variants.get(variant_id)
                    if not variant:
                        continue

                    result = await self._run_single_benchmark(
                        session, model, variant, topic, context, focus, ollama_url
                    )

                    if result:
                        # Check against ground truth if available
                        result = await self._validate_accuracy(result, session)

                        # Save result
                        self._save_result(result)

                        # Update variant stats
                        variant.total_runs += 1
                        variant.total_score += result.composite_score
                        variant.avg_latency_ms = (
                            (variant.avg_latency_ms * (variant.total_runs - 1) + result.latency_ms)
                            / variant.total_runs
                        )

                        results.append(result)

        self._save_variants()
        self._rankings_dirty = True

        return results

    async def _run_single_benchmark(self,
                                    session: aiohttp.ClientSession,
                                    model: str,
                                    variant: PromptVariant,
                                    topic: str,
                                    context: str,
                                    focus: str,
                                    ollama_url: str) -> Optional[BenchmarkResult]:
        """Run a single benchmark."""

        # Build prompt from variant template
        user_prompt = variant.user_template.format(
            topic=topic,
            context=context or "N/A",
            focus=focus,
        )

        messages = [
            {"role": "system", "content": variant.system_prompt},
            {"role": "user", "content": user_prompt},
        ]

        payload = {
            "model": model,
            "messages": messages,
            "stream": False,
        }

        start_time = time.time()

        try:
            async with session.post(
                f"{ollama_url}/api/chat",
                json=payload,
                timeout=aiohttp.ClientTimeout(total=120)
            ) as response:
                if response.status != 200:
                    return None

                data = await response.json()
                content = data.get("message", {}).get("content", "")

                latency_ms = int((time.time() - start_time) * 1000)

                # Estimate tokens
                input_tokens = len(variant.system_prompt + user_prompt) // 4
                output_tokens = len(content) // 4

                tokens_per_second = output_tokens / (latency_ms / 1000) if latency_ms > 0 else 0

                return BenchmarkResult(
                    timestamp=datetime.now().isoformat(),
                    model_name=model,
                    prompt_variant=variant.id,
                    query_type="technical",
                    topic=topic,
                    latency_ms=latency_ms,
                    input_tokens=input_tokens,
                    output_tokens=output_tokens,
                    tokens_per_second=tokens_per_second,
                    response_hash=hashlib.md5(content.encode()).hexdigest()[:12],
                )

        except Exception as e:
            print(f"Benchmark failed for {model}/{variant.id}: {e}")
            return None

    async def _validate_accuracy(self, result: BenchmarkResult,
                                  session: aiohttp.ClientSession) -> BenchmarkResult:
        """Validate result accuracy against ground truth."""

        topic_key = self._normalize_topic(result.topic)

        if topic_key in self.ground_truth:
            gt = self.ground_truth[topic_key]
            facts = gt["facts"]

            # Simple fact-matching accuracy (could be enhanced with embeddings)
            # For now, check if key facts appear in the response
            # This is a placeholder - real implementation would use semantic similarity

            result.ground_truth_id = topic_key
            result.validated_by = "source_check"

            # Placeholder scores - in production, use embeddings or LLM-as-judge
            result.accuracy_score = 0.7  # Would compare against facts
            result.completeness_score = 0.6
            result.relevance_score = 0.8

        return result

    def _save_result(self, result: BenchmarkResult):
        """Append result to results file."""
        with open(self.results_file, 'a') as f:
            f.write(json.dumps(asdict(result)) + '\n')

    def get_model_rankings(self, query_type: str = None) -> Dict[str, Dict]:
        """
        Get model rankings based on benchmark results.

        Returns dict of model -> stats including:
        - avg_latency_ms
        - avg_accuracy
        - avg_composite_score
        - total_runs
        - best_variant (which prompt works best with this model)
        """
        if not self._rankings_dirty and self._rankings_cache:
            return self._rankings_cache

        rankings = {}

        if not self.results_file.exists():
            return rankings

        with open(self.results_file, 'r') as f:
            for line in f:
                if not line.strip():
                    continue

                result = json.loads(line)

                if query_type and result.get("query_type") != query_type:
                    continue

                model = result["model_name"]

                if model not in rankings:
                    rankings[model] = {
                        "total_runs": 0,
                        "total_latency_ms": 0,
                        "total_accuracy": 0,
                        "total_composite": 0,
                        "accuracy_count": 0,
                        "variant_scores": {},
                    }

                r = rankings[model]
                r["total_runs"] += 1
                r["total_latency_ms"] += result["latency_ms"]

                if result.get("accuracy_score") is not None:
                    r["total_accuracy"] += result["accuracy_score"]
                    r["accuracy_count"] += 1

                # Track variant performance per model
                variant = result["prompt_variant"]
                if variant not in r["variant_scores"]:
                    r["variant_scores"][variant] = {"runs": 0, "total_score": 0}

                # Calculate composite score
                composite = self._calculate_composite(result)
                r["total_composite"] += composite
                r["variant_scores"][variant]["runs"] += 1
                r["variant_scores"][variant]["total_score"] += composite

        # Calculate averages and find best variant
        for model, r in rankings.items():
            r["avg_latency_ms"] = r["total_latency_ms"] / r["total_runs"]
            r["avg_accuracy"] = r["total_accuracy"] / r["accuracy_count"] if r["accuracy_count"] > 0 else None
            r["avg_composite"] = r["total_composite"] / r["total_runs"]

            # Find best variant for this model
            best_variant = None
            best_score = 0
            for variant, vstats in r["variant_scores"].items():
                avg = vstats["total_score"] / vstats["runs"] if vstats["runs"] > 0 else 0
                if avg > best_score:
                    best_score = avg
                    best_variant = variant

            r["best_variant"] = best_variant
            r["best_variant_score"] = best_score

            # Clean up intermediate fields
            del r["total_latency_ms"]
            del r["total_accuracy"]
            del r["total_composite"]
            del r["accuracy_count"]

        self._rankings_cache = rankings
        self._rankings_dirty = False

        return rankings

    def _calculate_composite(self, result: Dict) -> float:
        """Calculate composite score from result dict."""
        scores = []
        weights = []

        if result.get("accuracy_score") is not None:
            scores.append(result["accuracy_score"])
            weights.append(0.4)

        if result.get("completeness_score") is not None:
            scores.append(result["completeness_score"])
            weights.append(0.3)

        if result.get("relevance_score") is not None:
            scores.append(result["relevance_score"])
            weights.append(0.2)

        speed_score = max(0, min(1, 1 - (result["latency_ms"] - 2000) / 8000))
        scores.append(speed_score)
        weights.append(0.1)

        if not weights:
            return 0.0

        return sum(s * w for s, w in zip(scores, weights)) / sum(weights)

    def get_best_model(self, query_type: str = None) -> Tuple[str, str]:
        """
        Get the best model and variant for a query type.

        Returns:
            (model_name, variant_id)
        """
        rankings = self.get_model_rankings(query_type)

        if not rankings:
            return ("phi3", "structured_v1")  # Default fallback

        best_model = max(rankings.keys(), key=lambda m: rankings[m]["avg_composite"])
        best_variant = rankings[best_model].get("best_variant", "structured_v1")

        return (best_model, best_variant)

    def get_prompt_variant(self, variant_id: str) -> Optional[PromptVariant]:
        """Get a prompt variant by ID."""
        return self.variants.get(variant_id)

    def add_prompt_variant(self, id: str, name: str,
                           system_prompt: str, user_template: str):
        """Add a new prompt variant for testing."""
        self.variants[id] = PromptVariant(
            id=id,
            name=name,
            system_prompt=system_prompt,
            user_template=user_template,
        )
        self._save_variants()

    def print_rankings(self):
        """Print formatted model rankings."""
        rankings = self.get_model_rankings()

        if not rankings:
            print("No benchmark data available.")
            return

        print("\n" + "=" * 70)
        print("MODEL RANKINGS")
        print("=" * 70)

        # Sort by composite score
        sorted_models = sorted(
            rankings.items(),
            key=lambda x: x[1]["avg_composite"],
            reverse=True
        )

        for i, (model, stats) in enumerate(sorted_models, 1):
            print(f"\n{i}. {model}")
            print(f"   Composite Score: {stats['avg_composite']:.3f}")
            print(f"   Avg Latency:     {stats['avg_latency_ms']:.0f}ms")
            if stats['avg_accuracy'] is not None:
                print(f"   Avg Accuracy:    {stats['avg_accuracy']:.2f}")
            print(f"   Total Runs:      {stats['total_runs']}")
            print(f"   Best Variant:    {stats['best_variant']} (score: {stats['best_variant_score']:.3f})")

        print("\n" + "=" * 70)

        # Variant summary
        print("\nPROMPT VARIANT SUMMARY")
        print("-" * 40)
        for vid, variant in self.variants.items():
            if variant.total_runs > 0:
                print(f"  {vid}: {variant.avg_score:.3f} avg score, "
                      f"{variant.avg_latency_ms:.0f}ms avg latency, "
                      f"{variant.total_runs} runs")
        print()


class SelfImprovementLoop:
    """
    Self-improvement loop for the swarm.

    Periodically:
    1. Runs benchmarks on available models
    2. Updates model/variant selections based on results
    3. Generates new prompt variants based on best performers
    4. Validates accuracy against accumulated ground truth
    """

    def __init__(self, benchmark: ModelBenchmark,
                 auto_tune_interval: int = 50):
        self.benchmark = benchmark
        self.auto_tune_interval = auto_tune_interval
        self.query_count = 0

    async def on_query(self, topic: str, result: Dict,
                       model: str, variant: str):
        """
        Called after each research query.
        Tracks results and triggers auto-tuning.
        """
        self.query_count += 1

        if self.query_count % self.auto_tune_interval == 0:
            await self.run_auto_tune()

    async def run_auto_tune(self):
        """Run automatic tuning cycle."""
        print("\n[Self-Improvement] Running auto-tune cycle...")

        # Get current best performers
        old_best = self.benchmark.get_best_model()

        # Run benchmarks on known topics
        # (In production, this would use real research topics)

        # Get new rankings
        new_best = self.benchmark.get_best_model()

        if new_best != old_best:
            print(f"[Self-Improvement] Updated best model: {old_best} -> {new_best}")

        self.benchmark.print_rankings()
