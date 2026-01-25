#!/usr/bin/env python3
"""
Research Orchestrator MCP Server

Exposes Ollama models as tools for Claude CLI to orchestrate
historical video synthesis hardware research and emulation.
"""

import asyncio
import json
import base64
from pathlib import Path
from typing import Optional

import ollama
from mcp.server import Server
from mcp.server.stdio import stdio_server
from mcp.types import Tool, TextContent, ImageContent

# Model assignments based on capabilities
MODELS = {
    "fast": "phi4-mini:latest",       # Quick reasoning, coordination
    "research": "qwen3:latest",        # Deep research, synthesis
    "code": "qwen2.5-coder:latest",    # Shader/Rust generation
    "analysis": "deepseek-r1:8b",      # Verification, deep analysis
    "vision_fast": "moondream:latest", # Quick schematic glance
    "vision_ocr": "minicpm-v:latest",  # Document OCR, detailed analysis
    "embed": "nomic-embed-text",       # Document embeddings
}

# Initialize MCP server
server = Server("research-orchestrator")


@server.list_tools()
async def list_tools():
    """List available research tools."""
    return [
        Tool(
            name="research_query",
            description="Research a video synthesis topic using the research agent. "
                       "Good for: component specs, circuit behavior, historical context.",
            inputSchema={
                "type": "object",
                "properties": {
                    "topic": {
                        "type": "string",
                        "description": "The topic to research (e.g., 'Paik/Abe folding amplifier')"
                    },
                    "depth": {
                        "type": "string",
                        "enum": ["quick", "standard", "deep"],
                        "description": "Research depth - quick for overview, deep for comprehensive"
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
        Tool(
            name="analyze_schematic",
            description="Analyze a schematic image using vision models. "
                       "Extracts component values, signal flow, and circuit topology.",
            inputSchema={
                "type": "object",
                "properties": {
                    "image_path": {
                        "type": "string",
                        "description": "Path to schematic image file"
                    },
                    "analysis_type": {
                        "type": "string",
                        "enum": ["components", "signal_flow", "full"],
                        "description": "Type of analysis to perform"
                    }
                },
                "required": ["image_path"]
            }
        ),
        Tool(
            name="generate_shader",
            description="Generate WGSL shader code for a video effect or component emulation.",
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
        Tool(
            name="verify_emulation",
            description="Verify emulation accuracy by comparing output characteristics.",
            inputSchema={
                "type": "object",
                "properties": {
                    "expected_behavior": {
                        "type": "string",
                        "description": "Description of expected behavior from documentation"
                    },
                    "actual_output": {
                        "type": "string",
                        "description": "Description or data from actual emulation output"
                    },
                    "tolerance": {
                        "type": "string",
                        "enum": ["strict", "moderate", "loose"],
                        "description": "How strict the comparison should be"
                    }
                },
                "required": ["expected_behavior", "actual_output"]
            }
        ),
        Tool(
            name="embed_document",
            description="Create embeddings for a document to enable semantic search.",
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
            name="search_documents",
            description="Search embedded documents for relevant information.",
            inputSchema={
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "top_k": {
                        "type": "integer",
                        "description": "Number of results to return",
                        "default": 5
                    }
                },
                "required": ["query"]
            }
        ),
    ]


@server.call_tool()
async def call_tool(name: str, arguments: dict):
    """Handle tool calls."""

    if name == "research_query":
        return await research_query(
            arguments["topic"],
            arguments.get("depth", "standard"),
            arguments.get("focus", "behavior")
        )

    elif name == "analyze_schematic":
        return await analyze_schematic(
            arguments["image_path"],
            arguments.get("analysis_type", "full")
        )

    elif name == "generate_shader":
        return await generate_shader(
            arguments["specification"],
            arguments.get("transfer_function"),
            arguments.get("reference_code")
        )

    elif name == "verify_emulation":
        return await verify_emulation(
            arguments["expected_behavior"],
            arguments["actual_output"],
            arguments.get("tolerance", "moderate")
        )

    elif name == "embed_document":
        return await embed_document(
            arguments["text"],
            arguments["doc_id"]
        )

    elif name == "search_documents":
        return await search_documents(
            arguments["query"],
            arguments.get("top_k", 5)
        )

    else:
        return [TextContent(type="text", text=f"Unknown tool: {name}")]


async def research_query(topic: str, depth: str, focus: str):
    """Execute research query using the research model."""

    system_prompt = """You are a video synthesis hardware research specialist.
Your knowledge spans analog video circuits from 1950s oscilloscopes to modern LZX modules.

When researching, provide:
1. Historical context and key figures
2. Technical specifications and component values
3. Signal flow and circuit topology
4. Behavioral characteristics (transfer functions, nonlinearities)
5. Known quirks and "happy accidents"

Always cite sources when possible. Flag uncertain information.
Format output as structured markdown."""

    depth_instructions = {
        "quick": "Provide a brief 2-3 paragraph overview.",
        "standard": "Provide comprehensive coverage with key technical details.",
        "deep": "Provide exhaustive analysis including edge cases and implementation notes."
    }

    focus_instructions = {
        "components": "Focus on specific component values, ICs, and their datasheets.",
        "behavior": "Focus on input/output behavior, transfer functions, and signal characteristics.",
        "history": "Focus on historical development, key figures, and evolution.",
        "implementation": "Focus on practical implementation details for software emulation."
    }

    prompt = f"""Research topic: {topic}

{depth_instructions[depth]}
{focus_instructions[focus]}

Structure your response with clear sections. Include any relevant equations or transfer functions."""

    try:
        response = ollama.chat(
            model=MODELS["research"],
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": prompt}
            ]
        )
        return [TextContent(type="text", text=response["message"]["content"])]
    except Exception as e:
        return [TextContent(type="text", text=f"Research error: {str(e)}")]


async def analyze_schematic(image_path: str, analysis_type: str):
    """Analyze schematic image using vision model."""

    path = Path(image_path)
    if not path.exists():
        return [TextContent(type="text", text=f"Image not found: {image_path}")]

    # Read and encode image
    with open(path, "rb") as f:
        image_data = base64.b64encode(f.read()).decode()

    analysis_prompts = {
        "components": """Identify all components in this schematic:
- Resistor values (with tolerances if shown)
- Capacitor values and types
- IC part numbers
- Transistor types
- Any other components

List each with its reference designator.""",

        "signal_flow": """Trace the signal flow through this circuit:
- Identify input and output points
- Describe the signal path
- Note any feedback loops
- Identify mixing/summing points
- Describe any nonlinear stages""",

        "full": """Provide complete analysis of this schematic:

1. COMPONENTS: List all components with values
2. SIGNAL FLOW: Trace input to output
3. FUNCTION: What does this circuit do?
4. TRANSFER FUNCTION: Mathematical description if determinable
5. QUIRKS: Any unusual design choices or expected behaviors
6. EMULATION NOTES: Key characteristics to capture in software"""
    }

    try:
        response = ollama.chat(
            model=MODELS["vision_ocr"],
            messages=[{
                "role": "user",
                "content": analysis_prompts[analysis_type],
                "images": [image_data]
            }]
        )
        return [TextContent(type="text", text=response["message"]["content"])]
    except Exception as e:
        return [TextContent(type="text", text=f"Vision analysis error: {str(e)}")]


async def generate_shader(specification: str, transfer_function: Optional[str], reference_code: Optional[str]):
    """Generate WGSL shader code."""

    system_prompt = """You are a GPU shader specialist for video synthesis.
You write WGSL shaders for the Phosphlux analog video synthesizer emulator.

Shader requirements:
- WGSL syntax (not GLSL)
- Video signal range is 0.0 to 1.0
- Use vec3<f32> for RGB, f32 for luma
- Include proper clamping/saturation
- Comment nonlinear transfer functions
- Optimize for real-time performance

The codebase uses a node-graph architecture where each node has:
- Input textures (previous nodes' outputs)
- Output texture
- Uniform parameters"""

    prompt = f"""Generate WGSL shader code for: {specification}"""

    if transfer_function:
        prompt += f"\n\nTransfer function to implement:\n{transfer_function}"

    if reference_code:
        prompt += f"\n\nReference code to extend:\n```wgsl\n{reference_code}\n```"

    prompt += "\n\nProvide complete, working WGSL code with comments explaining the analog behavior being emulated."

    try:
        response = ollama.chat(
            model=MODELS["code"],
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": prompt}
            ]
        )
        return [TextContent(type="text", text=response["message"]["content"])]
    except Exception as e:
        return [TextContent(type="text", text=f"Shader generation error: {str(e)}")]


async def verify_emulation(expected: str, actual: str, tolerance: str):
    """Verify emulation accuracy."""

    system_prompt = """You are a verification specialist for analog circuit emulation.
Your job is to compare expected behavior (from documentation/hardware) with
actual emulation output and identify discrepancies.

Consider:
- Transfer function accuracy
- Frequency response
- Nonlinear behavior (clipping, saturation, folding)
- Timing characteristics
- Edge cases and boundary conditions

Rate accuracy and provide specific improvement suggestions."""

    tolerance_context = {
        "strict": "This is a high-fidelity emulation. Any deviation from documented behavior is a bug.",
        "moderate": "Reasonable approximations are acceptable if they capture the essential character.",
        "loose": "Focus on perceptual similarity rather than exact numerical accuracy."
    }

    prompt = f"""Compare expected vs actual behavior:

EXPECTED (from documentation):
{expected}

ACTUAL (from emulation):
{actual}

Tolerance level: {tolerance}
{tolerance_context[tolerance]}

Provide:
1. ACCURACY RATING: X/10
2. MATCHES: What behaviors are correctly captured
3. DISCREPANCIES: Specific differences found
4. RECOMMENDATIONS: How to improve accuracy"""

    try:
        response = ollama.chat(
            model=MODELS["analysis"],
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": prompt}
            ]
        )
        return [TextContent(type="text", text=response["message"]["content"])]
    except Exception as e:
        return [TextContent(type="text", text=f"Verification error: {str(e)}")]


# Simple in-memory embedding store (replace with FAISS for production)
embedding_store = {}

async def embed_document(text: str, doc_id: str):
    """Create and store document embedding."""
    try:
        response = ollama.embeddings(
            model=MODELS["embed"],
            prompt=text
        )
        embedding_store[doc_id] = {
            "embedding": response["embedding"],
            "text": text[:500]  # Store preview
        }
        return [TextContent(type="text", text=f"Embedded document: {doc_id} ({len(text)} chars)")]
    except Exception as e:
        return [TextContent(type="text", text=f"Embedding error: {str(e)}")]


async def search_documents(query: str, top_k: int):
    """Search embedded documents."""
    if not embedding_store:
        return [TextContent(type="text", text="No documents embedded yet.")]

    try:
        # Get query embedding
        response = ollama.embeddings(
            model=MODELS["embed"],
            prompt=query
        )
        query_embedding = response["embedding"]

        # Simple cosine similarity search
        import numpy as np

        results = []
        for doc_id, data in embedding_store.items():
            similarity = np.dot(query_embedding, data["embedding"]) / (
                np.linalg.norm(query_embedding) * np.linalg.norm(data["embedding"])
            )
            results.append((doc_id, similarity, data["text"]))

        results.sort(key=lambda x: x[1], reverse=True)

        output = "Search results:\n\n"
        for doc_id, score, preview in results[:top_k]:
            output += f"**{doc_id}** (score: {score:.3f})\n{preview}...\n\n"

        return [TextContent(type="text", text=output)]
    except Exception as e:
        return [TextContent(type="text", text=f"Search error: {str(e)}")]


async def main():
    """Run the MCP server."""
    async with stdio_server() as (read_stream, write_stream):
        await server.run(read_stream, write_stream, server.create_initialization_options())


if __name__ == "__main__":
    asyncio.run(main())
