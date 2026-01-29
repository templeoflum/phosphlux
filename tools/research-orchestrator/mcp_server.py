#!/usr/bin/env python3
"""
Research Orchestrator MCP Server

Document retrieval and component management tools for Claude CLI.
Provides patent, datasheet, schematic, and manual retrieval.

Note: Ollama-based tools (quick_query, research_query, generate_shader,
extract_from_image, embed_document, search_knowledge) were removed.
Claude handles research and code generation directly.
"""

import asyncio
import sys
import time
from pathlib import Path
from typing import Optional

from mcp.server import Server
from mcp.server.stdio import stdio_server
from mcp.types import Tool, TextContent

# Add retrieval tools to path
sys.path.insert(0, str(Path(__file__).parent.parent))
try:
    from retrieval import (
        PatentRetriever,
        DatasheetRetriever,
        SchematicRetriever,
        ManualRetriever,
        ComponentManager,
        ProvenanceDB,
    )
    HAS_RETRIEVAL = True
except ImportError:
    HAS_RETRIEVAL = False

server = Server("research-orchestrator")


@server.list_tools()
async def list_tools():
    """List available research tools."""
    return [
        # === Document Retrieval Tools ===
        Tool(
            name="retrieve_patent",
            description="Search and download patents. Returns patent metadata and PDF path.",
            inputSchema={
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query or patent ID (e.g., 'video synthesizer' or 'US4011557')"
                    },
                    "inventor": {
                        "type": "string",
                        "description": "Optional inventor name filter"
                    },
                    "download": {
                        "type": "boolean",
                        "description": "If true and query is a patent ID, download the patent",
                        "default": False
                    }
                },
                "required": ["query"]
            }
        ),

        Tool(
            name="retrieve_datasheet",
            description="Search and download IC datasheets. Returns datasheet path and metadata.",
            inputSchema={
                "type": "object",
                "properties": {
                    "part_number": {
                        "type": "string",
                        "description": "Part number to search (e.g., 'CA3046', 'LM1881')"
                    },
                    "download": {
                        "type": "boolean",
                        "description": "If true, download the datasheet",
                        "default": False
                    }
                },
                "required": ["part_number"]
            }
        ),

        Tool(
            name="retrieve_schematic",
            description="Search and download circuit schematics from archives.",
            inputSchema={
                "type": "object",
                "properties": {
                    "device": {
                        "type": "string",
                        "description": "Device name (e.g., 'paik-abe-synthesizer', 'sandin-ip')"
                    },
                    "download": {
                        "type": "boolean",
                        "description": "If true, download the schematic",
                        "default": False
                    }
                },
                "required": ["device"]
            }
        ),

        Tool(
            name="retrieve_manual",
            description="Search and download operation/service manuals.",
            inputSchema={
                "type": "object",
                "properties": {
                    "device": {
                        "type": "string",
                        "description": "Device name to search"
                    },
                    "manual_type": {
                        "type": "string",
                        "enum": ["service", "operation", "user", "any"],
                        "description": "Type of manual",
                        "default": "any"
                    },
                    "download": {
                        "type": "boolean",
                        "description": "If true, download the manual",
                        "default": False
                    }
                },
                "required": ["device"]
            }
        ),

        Tool(
            name="get_provenance",
            description="Get provenance information for an artifact or component.",
            inputSchema={
                "type": "object",
                "properties": {
                    "artifact_id": {
                        "type": "string",
                        "description": "Artifact ID (e.g., 'patent:US4011557', 'datasheet:CA3046')"
                    }
                },
                "required": ["artifact_id"]
            }
        ),

        Tool(
            name="list_components",
            description="List available WGSL component models with versions.",
            inputSchema={
                "type": "object",
                "properties": {
                    "category": {
                        "type": "string",
                        "enum": ["ics", "functional", "generators", "processors"],
                        "description": "Optional category filter"
                    }
                }
            }
        ),

        Tool(
            name="component_history",
            description="Get version history for a component.",
            inputSchema={
                "type": "object",
                "properties": {
                    "component": {
                        "type": "string",
                        "description": "Component path (e.g., 'ics/ca3046', 'functional/folding-amp')"
                    }
                },
                "required": ["component"]
            }
        ),
    ]


@server.call_tool()
async def call_tool(name: str, arguments: dict):
    """Route tool calls to appropriate handlers."""

    handlers = {
        "retrieve_patent": lambda: retrieve_patent(
            arguments["query"],
            arguments.get("inventor"),
            arguments.get("download", False)
        ),
        "retrieve_datasheet": lambda: retrieve_datasheet(
            arguments["part_number"],
            arguments.get("download", False)
        ),
        "retrieve_schematic": lambda: retrieve_schematic(
            arguments["device"],
            arguments.get("download", False)
        ),
        "retrieve_manual": lambda: retrieve_manual(
            arguments["device"],
            arguments.get("manual_type", "any"),
            arguments.get("download", False)
        ),
        "get_provenance": lambda: get_provenance(arguments["artifact_id"]),
        "list_components": lambda: list_components(arguments.get("category")),
        "component_history": lambda: component_history(arguments["component"]),
    }

    if name in handlers:
        return await handlers[name]()
    return [TextContent(type="text", text=f"Unknown tool: {name}")]


# === Document Retrieval Tools ===

async def retrieve_patent(query: str, inventor: Optional[str], download: bool):
    """Search or download patents."""
    if not HAS_RETRIEVAL:
        return [TextContent(type="text", text="[ERROR] Retrieval tools not installed")]

    start = time.time()
    retriever = PatentRetriever()

    try:
        # Check if query looks like a patent ID
        is_patent_id = query.upper().startswith(("US", "EP", "WO", "JP", "CN"))

        if download and is_patent_id:
            artifact = retriever.download(query)
            if artifact:
                elapsed = time.time() - start
                return [TextContent(type="text", text=f"""[RETRIEVAL] {elapsed:.1f}s

Downloaded patent: {artifact.patent_id}
Title: {artifact.title}
Inventors: {', '.join(artifact.inventors)}
Path: {artifact.path}
Filing date: {artifact.filing_date}
""")]
            else:
                return [TextContent(type="text", text=f"[ERROR] Failed to download patent: {query}")]
        else:
            results = retriever.search(query, inventor=inventor, max_results=10)
            elapsed = time.time() - start

            output = f"[RETRIEVAL] {elapsed:.1f}s\n\nFound {len(results)} patents:\n\n"
            for r in results:
                output += f"**{r.patent_id}**: {r.title}\n"
                if r.inventors:
                    output += f"  Inventors: {', '.join(r.inventors)}\n"
                if r.filing_date:
                    output += f"  Filed: {r.filing_date}\n"
                output += f"  URL: {r.url}\n\n"

            return [TextContent(type="text", text=output)]

    except Exception as e:
        elapsed = time.time() - start
        return [TextContent(type="text", text=f"[RETRIEVAL ERROR] {elapsed:.1f}s\n{e}")]


async def retrieve_datasheet(part_number: str, download: bool):
    """Search or download datasheets."""
    if not HAS_RETRIEVAL:
        return [TextContent(type="text", text="[ERROR] Retrieval tools not installed")]

    start = time.time()
    retriever = DatasheetRetriever()

    try:
        if download:
            artifact = retriever.download(part_number)
            if artifact:
                elapsed = time.time() - start
                return [TextContent(type="text", text=f"""[RETRIEVAL] {elapsed:.1f}s

Downloaded datasheet: {artifact.part_number}
Manufacturer: {artifact.manufacturer or 'Unknown'}
Path: {artifact.path}
""")]
            else:
                return [TextContent(type="text", text=f"[ERROR] Failed to download datasheet: {part_number}")]
        else:
            results = retriever.search(part_number, max_results=10)
            elapsed = time.time() - start

            output = f"[RETRIEVAL] {elapsed:.1f}s\n\nFound {len(results)} datasheets:\n\n"
            for r in results:
                output += f"**{r.part_number}**"
                if r.manufacturer:
                    output += f" ({r.manufacturer})"
                output += f"\n  Source: {r.source}\n  URL: {r.url}\n\n"

            return [TextContent(type="text", text=output)]

    except Exception as e:
        elapsed = time.time() - start
        return [TextContent(type="text", text=f"[RETRIEVAL ERROR] {elapsed:.1f}s\n{e}")]


async def retrieve_schematic(device: str, download: bool):
    """Search or download schematics."""
    if not HAS_RETRIEVAL:
        return [TextContent(type="text", text="[ERROR] Retrieval tools not installed")]

    start = time.time()
    retriever = SchematicRetriever()

    try:
        if download:
            artifact = retriever.download(device)
            if artifact:
                elapsed = time.time() - start
                return [TextContent(type="text", text=f"""[RETRIEVAL] {elapsed:.1f}s

Downloaded schematic: {artifact.device}
Designer: {artifact.designer or 'Unknown'}
Year: {artifact.year or 'Unknown'}
Path: {artifact.path}
""")]
            else:
                return [TextContent(type="text", text=f"[ERROR] Failed to download schematic: {device}")]
        else:
            results = retriever.search(device, max_results=10)
            elapsed = time.time() - start

            output = f"[RETRIEVAL] {elapsed:.1f}s\n\nFound {len(results)} schematics:\n\n"
            for r in results:
                output += f"**{r.title}**\n"
                output += f"  Archive: {r.archive}\n  URL: {r.url}\n\n"

            return [TextContent(type="text", text=output)]

    except Exception as e:
        elapsed = time.time() - start
        return [TextContent(type="text", text=f"[RETRIEVAL ERROR] {elapsed:.1f}s\n{e}")]


async def retrieve_manual(device: str, manual_type: str, download: bool):
    """Search or download manuals."""
    if not HAS_RETRIEVAL:
        return [TextContent(type="text", text="[ERROR] Retrieval tools not installed")]

    start = time.time()
    retriever = ManualRetriever()

    try:
        if download:
            artifact = retriever.download(device, manual_type=manual_type)
            if artifact:
                elapsed = time.time() - start
                return [TextContent(type="text", text=f"""[RETRIEVAL] {elapsed:.1f}s

Downloaded manual: {artifact.device}
Type: {artifact.manual_type}
Manufacturer: {artifact.manufacturer or 'Unknown'}
Pages: {artifact.page_count or 'Unknown'}
Path: {artifact.path}
""")]
            else:
                return [TextContent(type="text", text=f"[ERROR] Failed to download manual: {device}")]
        else:
            results = retriever.search(device, manual_type=manual_type, max_results=10)
            elapsed = time.time() - start

            output = f"[RETRIEVAL] {elapsed:.1f}s\n\nFound {len(results)} manuals:\n\n"
            for r in results:
                output += f"**{r.title}**\n"
                output += f"  Type: {r.manual_type}\n  Source: {r.source}\n  URL: {r.url}\n\n"

            return [TextContent(type="text", text=output)]

    except Exception as e:
        elapsed = time.time() - start
        return [TextContent(type="text", text=f"[RETRIEVAL ERROR] {elapsed:.1f}s\n{e}")]


async def get_provenance(artifact_id: str):
    """Get provenance information for an artifact."""
    if not HAS_RETRIEVAL:
        return [TextContent(type="text", text="[ERROR] Retrieval tools not installed")]

    start = time.time()

    try:
        db = ProvenanceDB()
        provenance = db.export_provenance(artifact_id)
        elapsed = time.time() - start

        if not provenance:
            return [TextContent(type="text", text=f"[PROVENANCE] Artifact not found: {artifact_id}")]

        output = f"[PROVENANCE] {elapsed:.1f}s\n\n"
        output += f"**Artifact**: {artifact_id}\n\n"

        if provenance.get("artifact"):
            artifact = provenance["artifact"]
            output += f"Type: {artifact.get('type', 'Unknown')}\n"
            output += f"Name: {artifact.get('name', 'Unknown')}\n"
            output += f"Path: {artifact.get('path', 'Unknown')}\n"
            output += f"Source: {artifact.get('source_url', 'Unknown')}\n"
            output += f"Retrieved: {artifact.get('retrieved_at', 'Unknown')}\n"
            output += f"Hash: {artifact.get('hash', 'Unknown')[:16]}...\n\n"

        if provenance.get("relationships"):
            output += "**Relationships**:\n"
            for rel in provenance["relationships"]:
                output += f"  - {rel['relationship']} -> {rel['related_id']}\n"
            output += "\n"

        if provenance.get("versions"):
            output += "**Versions**:\n"
            for v in provenance["versions"]:
                output += f"  - v{v['version']}: {v.get('notes', 'No notes')}\n"

        return [TextContent(type="text", text=output)]

    except Exception as e:
        elapsed = time.time() - start
        return [TextContent(type="text", text=f"[PROVENANCE ERROR] {elapsed:.1f}s\n{e}")]


async def list_components(category: Optional[str]):
    """List available component models."""
    if not HAS_RETRIEVAL:
        return [TextContent(type="text", text="[ERROR] Retrieval tools not installed")]

    start = time.time()

    try:
        manager = ComponentManager()
        components = manager.list_components(category=category)
        elapsed = time.time() - start

        output = f"[COMPONENTS] {elapsed:.1f}s\n\n"
        output += f"Found {len(components)} components"
        if category:
            output += f" in category '{category}'"
        output += ":\n\n"

        for comp in components:
            output += f"**{comp['component']}**\n"
            output += f"  Versions: {comp['versions'] or 'None'}\n"
            if comp.get('latest_spec', {}).get('description'):
                output += f"  Description: {comp['latest_spec']['description'][:80]}\n"
            output += "\n"

        if not components:
            output += "(No components found. Create components in research/components/)\n"

        return [TextContent(type="text", text=output)]

    except Exception as e:
        elapsed = time.time() - start
        return [TextContent(type="text", text=f"[COMPONENTS ERROR] {elapsed:.1f}s\n{e}")]


async def component_history(component: str):
    """Get version history for a component."""
    if not HAS_RETRIEVAL:
        return [TextContent(type="text", text="[ERROR] Retrieval tools not installed")]

    start = time.time()

    try:
        manager = ComponentManager()
        history = manager.get_history(component)
        elapsed = time.time() - start

        output = f"[COMPONENT HISTORY] {elapsed:.1f}s\n\n"
        output += f"History for **{component}**:\n\n"

        if history:
            for h in history:
                output += f"**v{h['version']}**\n"
                if h.get('created_at'):
                    output += f"  Created: {h['created_at']}\n"
                if h.get('created_by'):
                    output += f"  By: {h['created_by']}\n"
                if h.get('commit_hash'):
                    output += f"  Commit: {h['commit_hash']}\n"
                if h.get('notes'):
                    output += f"  Notes: {h['notes']}\n"
                output += "\n"
        else:
            output += "(No version history found)\n"

        return [TextContent(type="text", text=output)]

    except Exception as e:
        elapsed = time.time() - start
        return [TextContent(type="text", text=f"[COMPONENT HISTORY ERROR] {elapsed:.1f}s\n{e}")]


async def main():
    """Run the MCP server."""
    async with stdio_server() as (read_stream, write_stream):
        await server.run(read_stream, write_stream, server.create_initialization_options())


if __name__ == "__main__":
    asyncio.run(main())
