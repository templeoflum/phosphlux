#!/usr/bin/env python3
"""
Research Orchestrator MCP Server

Exposes Ollama models as tools for Claude CLI to orchestrate
historical video synthesis hardware research and emulation.

Architecture:
  Claude (orchestrator) -> dispatches to local models -> reviews outputs -> implements

Tiers (benchmarked 2026-01-25):
  - Fast (phi4-mini, ~3s warm, ~9s cold): Quick lookups, simple questions
  - Deep (llama3.1, ~10s warm, ~9s cold): Research synthesis, video art domain expertise
  - Vision (moondream, ~5-10s): Extract components/text from images
  - Embed (qwen3-embedding, ~0.1s warm, ~4s cold): Document embeddings for search

Performance Notes:
  - Cold start adds 4-9s (model loading)
  - Tier switching adds 3-5s (model swap)
  - See benchmark_results.json for detailed timings
"""

import asyncio
import base64
import time
from pathlib import Path
from typing import Optional

import ollama
from mcp.server import Server
from mcp.server.stdio import stdio_server
from mcp.types import Tool, TextContent

# Tiered model assignments (benchmarked for video synthesis domain accuracy)
MODELS = {
    "fast": "phi4-mini:latest",        # Quick reasoning (~5s)
    "deep": "llama3.1:latest",         # Research - correct Paik/Abe domain (~10s)
    "vision": "moondream:latest",      # Image extraction (~5-10s)
    "embed": "qwen3-embedding:latest", # Document embeddings (~0.1s warm)
}

# Timeout thresholds (seconds) - based on benchmark_results.json
TIMEOUTS = {
    "fast": {"warm": 16, "cold": 13, "abort": 18},
    "deep": {"warm": 31, "cold": 14, "abort": 51},
    "vision": {"warm": 15, "cold": 20, "abort": 30},  # Not re-benchmarked
    "embed": {"warm": 1, "cold": 7, "abort": 9},
}

server = Server("research-orchestrator")


@server.list_tools()
async def list_tools():
    """List available research tools."""
    return [
        # Fast tier - quick lookups
        Tool(
            name="quick_query",
            description="Fast lookup for simple questions. Use for: definitions, "
                       "quick facts, format conversion, simple reasoning. ~4s response.",
            inputSchema={
                "type": "object",
                "properties": {
                    "question": {
                        "type": "string",
                        "description": "A simple, focused question"
                    }
                },
                "required": ["question"]
            }
        ),

        # Deep tier - research synthesis
        Tool(
            name="research_query",
            description="Deep research on video synthesis topics. Use for: historical "
                       "context, circuit behavior, component specs, implementation details. ~10s response.",
            inputSchema={
                "type": "object",
                "properties": {
                    "topic": {
                        "type": "string",
                        "description": "The topic to research (e.g., 'Paik/Abe folding amplifier')"
                    },
                    "focus": {
                        "type": "string",
                        "enum": ["components", "behavior", "history", "implementation"],
                        "description": "What aspect to focus on"
                    }
                },
                "required": ["topic"]
            }
        ),

        # Deep tier - code generation
        Tool(
            name="generate_shader",
            description="Generate WGSL shader code draft. Claude reviews before implementation. ~10s response.",
            inputSchema={
                "type": "object",
                "properties": {
                    "specification": {
                        "type": "string",
                        "description": "Description of the effect or component to emulate"
                    },
                    "transfer_function": {
                        "type": "string",
                        "description": "Optional: mathematical transfer function to implement"
                    },
                    "reference_code": {
                        "type": "string",
                        "description": "Optional: existing code to reference or extend"
                    }
                },
                "required": ["specification"]
            }
        ),

        # Vision tier - image extraction
        Tool(
            name="extract_from_image",
            description="Extract text or component lists from images. Use for: schematic "
                       "component lists, OCR, quick visual inventory. NOT for deep analysis. ~5-10s response.",
            inputSchema={
                "type": "object",
                "properties": {
                    "image_path": {
                        "type": "string",
                        "description": "Path to image file"
                    },
                    "extract_type": {
                        "type": "string",
                        "enum": ["components", "text", "labels"],
                        "description": "What to extract: components (part numbers, values), text (OCR), labels (annotations)"
                    }
                },
                "required": ["image_path", "extract_type"]
            }
        ),

        # Embedding tier - knowledge base
        Tool(
            name="embed_document",
            description="Add document to searchable knowledge base.",
            inputSchema={
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text content to embed"
                    },
                    "doc_id": {
                        "type": "string",
                        "description": "Identifier for this document"
                    }
                },
                "required": ["text", "doc_id"]
            }
        ),
        Tool(
            name="search_knowledge",
            description="Search the embedded knowledge base.",
            inputSchema={
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "top_k": {
                        "type": "integer",
                        "description": "Number of results (default 5)",
                        "default": 5
                    }
                },
                "required": ["query"]
            }
        ),
    ]


@server.call_tool()
async def call_tool(name: str, arguments: dict):
    """Route tool calls to appropriate handlers."""

    handlers = {
        "quick_query": lambda: quick_query(arguments["question"]),
        "research_query": lambda: research_query(
            arguments["topic"],
            arguments.get("focus", "behavior")
        ),
        "generate_shader": lambda: generate_shader(
            arguments["specification"],
            arguments.get("transfer_function"),
            arguments.get("reference_code")
        ),
        "extract_from_image": lambda: extract_from_image(
            arguments["image_path"],
            arguments["extract_type"]
        ),
        "embed_document": lambda: embed_document(
            arguments["text"],
            arguments["doc_id"]
        ),
        "search_knowledge": lambda: search_knowledge(
            arguments["query"],
            arguments.get("top_k", 5)
        ),
    }

    if name in handlers:
        return await handlers[name]()
    return [TextContent(type="text", text=f"Unknown tool: {name}")]


# === Helpers ===

def timed_response(tier: str, content: str, elapsed: float) -> list:
    """Format response with timing metadata."""
    threshold = TIMEOUTS[tier]["warm"]
    status = "OK" if elapsed < threshold else "SLOW"
    header = f"[{tier.upper()}] {elapsed:.1f}s ({status})\n\n"
    return [TextContent(type="text", text=header + content)]


def error_response(tier: str, error: str, elapsed: float) -> list:
    """Format error response with timing."""
    return [TextContent(
        type="text",
        text=f"[{tier.upper()} ERROR] {elapsed:.1f}s\n{error}"
    )]


# === Fast Tier ===

async def quick_query(question: str):
    """Fast lookup using phi4-mini."""
    start = time.time()
    try:
        response = ollama.chat(
            model=MODELS["fast"],
            messages=[{
                "role": "user",
                "content": question
            }]
        )
        elapsed = time.time() - start
        return timed_response("fast", response["message"]["content"], elapsed)
    except Exception as e:
        elapsed = time.time() - start
        return error_response("fast", str(e), elapsed)


# === Deep Tier ===

async def research_query(topic: str, focus: str):
    """Deep research using llama3.1 (accurate for video synthesis domain)."""
    start = time.time()

    system_prompt = """You are a video synthesis hardware research specialist.
Your knowledge spans analog video circuits from 1950s oscilloscopes to modern LZX modules.
Key figures: Nam June Paik, Shuya Abe, Dan Sandin, Bill Hearn, Eric Siegel.

Provide:
1. Technical specifications and component values
2. Signal flow and circuit topology
3. Behavioral characteristics (transfer functions, nonlinearities)
4. Known quirks and "happy accidents"

Flag uncertain information with [UNCERTAIN].
Format as structured markdown. Be concise but complete."""

    focus_instructions = {
        "components": "Focus on component values, ICs, part numbers.",
        "behavior": "Focus on transfer functions, signal characteristics.",
        "history": "Focus on historical development, key figures.",
        "implementation": "Focus on software emulation approach."
    }

    prompt = f"""Research: {topic}

{focus_instructions.get(focus, focus_instructions["behavior"])}

Be specific and technical."""

    try:
        response = ollama.chat(
            model=MODELS["deep"],
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": prompt}
            ]
        )
        elapsed = time.time() - start
        return timed_response("deep", response["message"]["content"], elapsed)
    except Exception as e:
        elapsed = time.time() - start
        return error_response("deep", str(e), elapsed)


async def generate_shader(spec: str, transfer_function: Optional[str], reference: Optional[str]):
    """Generate WGSL shader draft using llama3.1."""
    start = time.time()

    system_prompt = """You are a GPU shader specialist for video synthesis.
Write WGSL shaders for the Phosphlux analog video synthesizer.

Requirements:
- WGSL syntax (not GLSL)
- Video signal range 0.0 to 1.0
- vec3<f32> for RGB, f32 for luma
- Include clamping/saturation
- Comment transfer functions
- Optimize for real-time

This is a DRAFT for Claude to review before implementation."""

    prompt = f"Generate WGSL shader for: {spec}"

    if transfer_function:
        prompt += f"\n\nTransfer function:\n{transfer_function}"
    if reference:
        prompt += f"\n\nReference:\n```wgsl\n{reference}\n```"

    try:
        response = ollama.chat(
            model=MODELS["deep"],
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": prompt}
            ]
        )
        elapsed = time.time() - start
        return timed_response("deep", response["message"]["content"], elapsed)
    except Exception as e:
        elapsed = time.time() - start
        return error_response("deep", str(e), elapsed)


# === Vision Tier ===

async def extract_from_image(image_path: str, extract_type: str):
    """Extract components/text from image using moondream."""
    start = time.time()

    path = Path(image_path)
    if not path.exists():
        return error_response("vision", f"Image not found: {image_path}", 0)

    with open(path, "rb") as f:
        image_data = base64.b64encode(f.read()).decode()

    prompts = {
        "components": "Describe all electronic components in this schematic. "
                     "Include resistor values, capacitor values, IC part numbers, "
                     "transistor types, and any other parts you can identify.",
        "text": "Read and transcribe all text visible in this image.",
        "labels": "What labels, titles, and reference designators (like R1, C1, U1) are shown?"
    }

    try:
        response = ollama.chat(
            model=MODELS["vision"],
            messages=[{
                "role": "user",
                "content": prompts[extract_type],
                "images": [image_data]
            }]
        )
        elapsed = time.time() - start
        return timed_response("vision", response["message"]["content"], elapsed)
    except Exception as e:
        elapsed = time.time() - start
        return error_response("vision", str(e), elapsed)


# === Embedding Tier ===

embedding_store = {}  # In-memory; replace with FAISS for persistence

async def embed_document(text: str, doc_id: str):
    """Create and store document embedding."""
    start = time.time()
    try:
        response = ollama.embeddings(
            model=MODELS["embed"],
            prompt=text
        )
        embedding_store[doc_id] = {
            "embedding": response["embedding"],
            "text": text[:500]
        }
        elapsed = time.time() - start
        return timed_response("embed", f"Embedded: {doc_id} ({len(text)} chars)", elapsed)
    except Exception as e:
        elapsed = time.time() - start
        return error_response("embed", str(e), elapsed)


async def search_knowledge(query: str, top_k: int):
    """Search embedded documents."""
    start = time.time()

    if not embedding_store:
        return [TextContent(type="text", text="[EMBED] No documents embedded yet.")]

    try:
        import numpy as np

        response = ollama.embeddings(model=MODELS["embed"], prompt=query)
        query_emb = response["embedding"]

        results = []
        for doc_id, data in embedding_store.items():
            sim = np.dot(query_emb, data["embedding"]) / (
                np.linalg.norm(query_emb) * np.linalg.norm(data["embedding"])
            )
            results.append((doc_id, sim, data["text"]))

        results.sort(key=lambda x: x[1], reverse=True)

        output = f"Found {len(results)} documents:\n\n"
        for doc_id, score, preview in results[:top_k]:
            output += f"**{doc_id}** ({score:.3f})\n{preview}...\n\n"

        elapsed = time.time() - start
        return timed_response("embed", output, elapsed)
    except Exception as e:
        elapsed = time.time() - start
        return error_response("embed", str(e), elapsed)


async def main():
    """Run the MCP server."""
    async with stdio_server() as (read_stream, write_stream):
        await server.run(read_stream, write_stream, server.create_initialization_options())


if __name__ == "__main__":
    asyncio.run(main())
