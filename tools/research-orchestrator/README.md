# Research Orchestrator

Multi-agent research system for historical video synthesis hardware emulation.

## Philosophy

**Claude as orchestrator and quality gate.**

Local Ollama models handle the heavy lifting (research, code drafts, component extraction). Claude reviews all outputs, catches errors/inconsistencies, and makes final implementation decisions. Nothing gets committed without Claude's verification.

## Quick Start

```bash
# Install dependencies
pip install -r requirements.txt

# Verify Ollama models
ollama list

# Test the server
python mcp_server.py
```

## Model Tiers

| Tier | Model | Speed | Use For |
|------|-------|-------|---------|
| **Fast** | phi4-mini | ~4s | Quick lookups, simple questions |
| **Deep** | qwen3 | ~22s | Research synthesis, shader drafts |
| **Vision** | moondream | ~5-10s | Component extraction, OCR |
| **Embed** | qwen3-embedding | ~2s | Knowledge base search |

## Tools

| Tool | Tier | Purpose |
|------|------|---------|
| `quick_query` | Fast | Simple questions, definitions |
| `research_query` | Deep | Video synthesis research |
| `generate_shader` | Deep | WGSL shader drafts (Claude reviews) |
| `extract_from_image` | Vision | Component lists, OCR from schematics |
| `embed_document` | Embed | Add to knowledge base |
| `search_knowledge` | Embed | Search knowledge base |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                Claude CLI (Orchestrator)                     │
│                                                             │
│  - Dispatches to local models                               │
│  - Reviews all outputs                                      │
│  - Catches errors/inconsistencies                           │
│  - Makes final implementation decisions                     │
│  - Deep schematic analysis (vision)                         │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  MCP Server (mcp_server.py)                                 │
│                                                             │
│  Fast Tier (phi4-mini, ~4s)                                 │
│  └── quick_query(question)                                  │
│                                                             │
│  Deep Tier (qwen3, ~22s)                                    │
│  ├── research_query(topic, focus)                           │
│  └── generate_shader(spec)  → DRAFT for Claude review       │
│                                                             │
│  Vision Tier (moondream, ~5-10s)                            │
│  └── extract_from_image(path, type)                         │
│                                                             │
│  Embed Tier (qwen3-embedding)                               │
│  ├── embed_document(text, id)                               │
│  └── search_knowledge(query)                                │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Claude Code MCP Config

Add to your Claude Code settings:

```json
{
  "mcpServers": {
    "research": {
      "command": "python",
      "args": ["C:/path/to/tools/research-orchestrator/mcp_server.py"]
    }
  }
}
```

## Workflow Example

1. **Research phase**: Claude dispatches `research_query` for Paik/Abe folding amplifier
2. **Review**: Claude verifies the research output, flags uncertainties
3. **Code draft**: Claude dispatches `generate_shader` for the transfer function
4. **Verification**: Claude reviews shader draft, fixes errors, adapts to codebase
5. **Implementation**: Claude writes the final code

For schematics: Share image directly with Claude for deep analysis. Use `extract_from_image` only for quick component inventories.
