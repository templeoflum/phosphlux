"""
Research Agent

Gathers, validates, and synthesizes information.
Uses local models (Ollama) for queries and tracks token usage.
"""

import asyncio
import aiohttp
import json
import time
from typing import List, Optional, Dict, Any
from datetime import datetime
from pathlib import Path

from .base_agent import BaseAgent
from .shared_state import SharedState
from .event_bus import EventBus, Event
from .token_tracker import TokenTracker, TokenTimer
from .model_benchmark import ModelBenchmark, BenchmarkResult


class ResearchAgent(BaseAgent):
    """
    Research agent responsibilities:
    - Query local models for domain knowledge
    - Search web for sources and references
    - Validate implementation against documentation
    - Compare visual output to reference images
    - Track open questions and findings
    - Self-improve via model benchmarking
    """

    # Default Ollama settings
    OLLAMA_URL = "http://localhost:11434"
    OLLAMA_MODEL = "phi3"  # Default fast local model

    def __init__(self, state: SharedState, event_bus: EventBus,
                 token_tracker: Optional[TokenTracker] = None,
                 ollama_url: str = None, ollama_model: str = None,
                 benchmark: Optional[ModelBenchmark] = None,
                 auto_select_model: bool = True):
        self.ollama_url = ollama_url or self.OLLAMA_URL
        self.ollama_model = ollama_model or self.OLLAMA_MODEL
        self.token_tracker = token_tracker
        self.benchmark = benchmark
        self.auto_select_model = auto_select_model  # Use benchmark to pick best model

        self._pending_queries = []
        self._pending_validations = []
        self._pending_pdfs = []  # PDFs that need user to download
        self._current_topic = None

        # Query statistics for self-improvement
        self._query_count = 0
        self._benchmark_interval = 20  # Run benchmark every N queries

        super().__init__(state, event_bus)

    @property
    def agent_type(self) -> str:
        return "research"

    @property
    def subscriptions(self) -> List[str]:
        return [
            "dev.blocked",              # Development needs info
            "dev.screenshot",           # Validate output
            "docs.gap_identified",      # Documentation found gap
            "orch.directive",           # Orchestrator commands
            "orch.pause",
            "orch.resume",
            "user.pdf_downloaded",      # User downloaded a PDF we requested
        ]

    def request_pdf_download(self, url: str, description: str, filename: str = None):
        """
        Request user to download a PDF we found but can't access directly.

        Emits research.pdf_needed event and pauses until user confirms download.
        """
        pdf_request = {
            "id": f"pdf_{int(time.time()*1000)}",
            "url": url,
            "description": description,
            "suggested_filename": filename or f"doc_{int(time.time())}.pdf",
            "requested_at": datetime.now().isoformat(),
            "status": "pending",
        }
        self._pending_pdfs.append(pdf_request)

        # Emit event for orchestrator to pause and notify user
        self.events.emit(
            source="research",
            event_type="research.pdf_needed",
            payload={
                "request_id": pdf_request["id"],
                "url": url,
                "description": description,
                "suggested_path": f"research/devices/{pdf_request['suggested_filename']}",
                "action_required": "Please download this PDF and place it in the suggested path",
            },
            triggers=["orch.attention_needed"],
        )

        self.logger.info(f"PDF download requested: {description}")
        return pdf_request["id"]

    async def process_event(self, event: Event) -> Optional[List[Dict]]:
        """Process incoming events."""

        if event.event_type == "dev.blocked":
            # Development agent needs information
            needs = event.payload.get("needs", "")
            reason = event.payload.get("reason", "")
            self.logger.info(f"Dev blocked, needs: {needs}")

            # Queue research query
            self._pending_queries.append({
                "type": "targeted",
                "topic": needs,
                "context": reason,
                "priority": "high",
            })

        elif event.event_type == "dev.screenshot":
            # New screenshot to validate
            path = event.payload.get("path")
            params = event.payload.get("params", {})

            # Queue validation
            self._pending_validations.append({
                "screenshot_path": path,
                "params": params,
            })

        elif event.event_type == "docs.gap_identified":
            # Documentation found a gap
            question = event.payload.get("question", "")
            context = event.payload.get("context", "")

            self._pending_queries.append({
                "type": "gap_fill",
                "topic": question,
                "context": context,
                "priority": "normal",
            })

        elif event.event_type == "orch.directive":
            # Handle orchestrator directives
            task = event.payload.get("task", "")
            if "research" in task.lower():
                topic = event.payload.get("topic", task)
                self._current_topic = topic
                self._pending_queries.append({
                    "type": "comprehensive",
                    "topic": topic,
                    "context": event.payload.get("context", ""),
                    "priority": "high",
                })

        elif event.event_type == "user.pdf_downloaded":
            # User downloaded a PDF we requested
            request_id = event.payload.get("request_id")
            file_path = event.payload.get("file_path")

            # Find and update the pending request
            for pdf_req in self._pending_pdfs:
                if pdf_req["id"] == request_id:
                    pdf_req["status"] = "downloaded"
                    pdf_req["local_path"] = file_path
                    self.logger.info(f"PDF downloaded: {file_path}")

                    # Add to references
                    self.state.add_reference(
                        url=pdf_req.get("url", "local"),
                        title=pdf_req.get("description", "Downloaded PDF"),
                        ref_type="schematic",
                        notes=f"Downloaded at user request. Path: {file_path}",
                    )
                    break

        return None

    async def do_work(self) -> Optional[List[Dict]]:
        """Execute pending research tasks."""
        events_to_emit = []

        # Process queries
        if self._pending_queries:
            query = self._pending_queries.pop(0)
            result = await self._execute_query(query)
            if result:
                events_to_emit.append(result)

        # Process validations
        if self._pending_validations:
            validation = self._pending_validations.pop(0)
            result = await self._execute_validation(validation)
            if result:
                events_to_emit.append(result)

        return events_to_emit if events_to_emit else None

    async def _execute_query(self, query: Dict) -> Optional[Dict]:
        """Execute a research query using MCP tools."""
        topic = query["topic"]
        query_type = query.get("type", "targeted")

        self.log_action("Researching", {"topic": topic, "type": query_type})

        try:
            # Call research MCP server
            async with aiohttp.ClientSession() as session:
                # Try deep research first
                payload = {
                    "topic": topic,
                    "focus": "behavior" if query_type == "comprehensive" else "implementation"
                }

                # Note: In real implementation, this would use MCP protocol
                # For now, we simulate the call structure
                result = await self._call_research_tool("research_query", payload, session)

                if result:
                    confidence = self._assess_confidence(result)

                    # Store finding
                    finding_id = self.state.add_finding(
                        topic=topic,
                        finding=result.get("content", ""),
                        confidence=confidence,
                        sources=result.get("sources", []),
                        agent_id=self.agent_type,
                    )

                    return {
                        "type": "research.finding",
                        "payload": {
                            "finding_id": finding_id,
                            "topic": topic,
                            "finding": result.get("content", "")[:500],  # Summary
                            "confidence": confidence,
                            "sources": result.get("sources", []),
                        },
                        "confidence": confidence,
                    }

        except Exception as e:
            self.logger.error(f"Research query failed: {e}")

            # Add to open questions if we couldn't answer
            self.state.add_open_question(
                question=topic,
                context=f"Query failed: {e}",
                priority="normal",
            )

        return None

    async def _execute_validation(self, validation: Dict) -> Optional[Dict]:
        """Validate a screenshot against references."""
        screenshot_path = validation["screenshot_path"]
        params = validation.get("params", {})

        self.log_action("Validating screenshot", {"path": screenshot_path})

        # Get reference images for comparison
        refs = self.state._read_json(
            self.state.state_dir / "knowledge_base/references.json"
        ) or []

        reference_images = [r for r in refs if r.get("type") == "reference_image"]

        if not reference_images:
            self.logger.info("No reference images to compare against")
            return {
                "type": "research.validation",
                "payload": {
                    "screenshot": screenshot_path,
                    "result": "no_reference",
                    "message": "No reference images available for comparison",
                },
                "confidence": 0.5,
            }

        # Compare against each reference
        # In real implementation, this would use vision tools
        # For now, we emit that validation was attempted
        return {
            "type": "research.validation",
            "payload": {
                "screenshot": screenshot_path,
                "result": "pending_visual_comparison",
                "references_checked": len(reference_images),
                "params": params,
            },
            "confidence": 0.7,
        }

    async def _call_ollama(self, prompt: str, session: aiohttp.ClientSession,
                           system: str = None) -> Optional[Dict]:
        """
        Call Ollama API directly for local model inference.

        Returns dict with:
        - content: The model's response
        - input_tokens: Estimated input token count
        - output_tokens: Estimated output token count
        """
        start_time = time.time()

        try:
            # Build request payload
            messages = []
            if system:
                messages.append({"role": "system", "content": system})
            messages.append({"role": "user", "content": prompt})

            payload = {
                "model": self.ollama_model,
                "messages": messages,
                "stream": False,
            }

            async with session.post(
                f"{self.ollama_url}/api/chat",
                json=payload,
                timeout=aiohttp.ClientTimeout(total=60)
            ) as response:
                if response.status == 200:
                    data = await response.json()
                    content = data.get("message", {}).get("content", "")

                    # Estimate tokens (rough: ~4 chars per token)
                    input_tokens = len(prompt) // 4
                    if system:
                        input_tokens += len(system) // 4
                    output_tokens = len(content) // 4

                    latency_ms = int((time.time() - start_time) * 1000)

                    # Record token usage
                    if self.token_tracker:
                        self.token_tracker.record_local(
                            model_name=self.ollama_model,
                            operation="research_query",
                            input_tokens=input_tokens,
                            output_tokens=output_tokens,
                            latency_ms=latency_ms,
                            agent="research",
                        )

                    return {
                        "content": content,
                        "input_tokens": input_tokens,
                        "output_tokens": output_tokens,
                        "latency_ms": latency_ms,
                    }
                else:
                    self.logger.error(f"Ollama error: {response.status}")
                    return None

        except asyncio.TimeoutError:
            self.logger.error("Ollama request timed out")
            return None
        except aiohttp.ClientConnectorError:
            self.logger.warning("Ollama not available, using fallback")
            return None
        except Exception as e:
            self.logger.error(f"Ollama call failed: {e}")
            return None

    async def _call_research_tool(self, tool: str, payload: Dict,
                                   session: aiohttp.ClientSession) -> Optional[Dict]:
        """
        Call research tool - uses Ollama for local inference.

        If benchmark is available and auto_select_model is True,
        uses the best-performing model and prompt variant.
        """
        topic = payload.get("topic", "unknown")
        focus = payload.get("focus", "implementation")
        context = payload.get("context", "")

        # Track query count for periodic benchmarking
        self._query_count += 1

        # Select model and prompt based on benchmark data
        model = self.ollama_model
        variant_id = None

        if self.benchmark and self.auto_select_model:
            model, variant_id = self.benchmark.get_best_model(query_type="technical")
            variant = self.benchmark.get_prompt_variant(variant_id)

            if variant:
                # Use optimized prompt from benchmark
                system_prompt = variant.system_prompt
                user_prompt = variant.user_template.format(
                    topic=topic,
                    context=context or "N/A",
                    focus=focus,
                )
                self.logger.debug(f"Using optimized: {model}/{variant_id}")
            else:
                # Fallback to default prompts
                system_prompt, user_prompt = self._get_default_prompts(topic, focus)
        else:
            # Use default prompts
            system_prompt, user_prompt = self._get_default_prompts(topic, focus)

        start_time = time.time()
        result = await self._call_ollama(user_prompt, session, system_prompt)
        latency_ms = int((time.time() - start_time) * 1000)

        if result:
            # Record benchmark data for self-improvement
            if self.benchmark:
                bench_result = BenchmarkResult(
                    timestamp=datetime.now().isoformat(),
                    model_name=model,
                    prompt_variant=variant_id or "default",
                    query_type="technical",
                    topic=topic,
                    latency_ms=latency_ms,
                    input_tokens=result.get("input_tokens", 0),
                    output_tokens=result.get("output_tokens", 0),
                    tokens_per_second=result.get("output_tokens", 0) / (latency_ms / 1000) if latency_ms > 0 else 0,
                )
                self.benchmark._save_result(bench_result)

            return {
                "content": result["content"],
                "sources": [],
                "tier": "local",
                "tokens_used": result.get("input_tokens", 0) + result.get("output_tokens", 0),
                "model": model,
                "variant": variant_id,
                "latency_ms": latency_ms,
            }

        # Fallback if Ollama unavailable
        return {
            "content": f"[Ollama unavailable - research pending for: {topic}]",
            "sources": [],
            "tier": "fallback",
        }

    def _get_default_prompts(self, topic: str, focus: str) -> tuple:
        """Get default system and user prompts."""
        system_prompt = """You are a research assistant specializing in vintage video synthesis hardware.
You have deep knowledge of analog video circuits, particularly the Paik/Abe Video Synthesizer.
Provide accurate, technical information with specific details about circuits, components, and signal processing."""

        user_prompt = f"""Research topic: {topic}

Focus area: {focus}

Please provide:
1. Technical description of how this works
2. Key components and circuits involved
3. Signal flow and processing stages
4. Any specific values or parameters

Be precise and technical. If uncertain about any detail, note it."""

        return system_prompt, user_prompt

    def _assess_confidence(self, result: Dict) -> float:
        """Assess confidence in a research result."""
        # Factors that increase confidence:
        # - Multiple sources
        # - Authoritative sources (archives, academic)
        # - Consistent information

        sources = result.get("sources", [])
        content = result.get("content", "")

        confidence = 0.5  # Base confidence

        # Source count
        if len(sources) >= 3:
            confidence += 0.2
        elif len(sources) >= 1:
            confidence += 0.1

        # Check for uncertainty markers
        if "[UNCERTAIN]" in content or "[uncertain]" in content.lower():
            confidence -= 0.2
        if "may" in content.lower() or "might" in content.lower():
            confidence -= 0.05

        # Authoritative sources
        authoritative = ["archive.org", "vasulka.org", "fondation-langlois", "aic.gov"]
        for source in sources:
            if any(auth in source.lower() for auth in authoritative):
                confidence += 0.1

        return min(max(confidence, 0.1), 1.0)

    def _get_agent_state(self) -> dict:
        """Save agent-specific state."""
        return {
            "pending_queries": self._pending_queries,
            "pending_validations": self._pending_validations,
            "pending_pdfs": self._pending_pdfs,
            "current_topic": self._current_topic,
        }

    def _restore_agent_state(self, state: dict):
        """Restore agent-specific state."""
        self._pending_queries = state.get("pending_queries", [])
        self._pending_validations = state.get("pending_validations", [])
        self._pending_pdfs = state.get("pending_pdfs", [])
        self._current_topic = state.get("current_topic")
