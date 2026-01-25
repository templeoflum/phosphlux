# Research Orchestrator

Multi-agent research system for historical video synthesis hardware emulation.

## Quick Start

```bash
# Install dependencies
pip install -r requirements.txt

# Verify Ollama models
ollama list

# Run the MCP server
python mcp_server.py
```

## Available Models (from your system)

| Model | Size | Best For |
|-------|------|----------|
| `phi4-mini:latest` | 2.5GB | Fast reasoning, coordination |
| `qwen3:latest` | 5.2GB | General research, synthesis |
| `qwen2.5-coder:latest` | 4.7GB | Code generation, shader work |
| `deepseek-r1:8b` | 4.9GB | Deep analysis, verification |
| `moondream:latest` | 1.7GB | Vision - schematic analysis |
| `minicpm-v:latest` | 5.5GB | Vision - document OCR |
| `nomic-embed-text` | 274MB | Embeddings for search |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Claude CLI (Orchestrator)                 │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  MCP Server (mcp_server.py)                                 │
│  ├── research_query(topic, depth)     → Research agent      │
│  ├── analyze_schematic(image_path)    → Vision agent        │
│  ├── generate_shader(spec)            → Code agent          │
│  ├── verify_emulation(ref, output)    → Verification agent  │
│  └── embed_document(text)             → Embedding agent     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Usage with Claude CLI

Add to your Claude Code MCP config:

```json
{
  "mcpServers": {
    "research": {
      "command": "python",
      "args": ["tools/research-orchestrator/mcp_server.py"]
    }
  }
}
```

Then in Claude CLI:
```
> Research the Paik/Abe video synthesizer folding amplifier circuit
> Analyze this schematic image for component values
> Generate a WGSL shader that emulates this transfer function
```

## Directory Structure

```
research-orchestrator/
├── mcp_server.py          # MCP server exposing Ollama tools
├── agents/
│   ├── research.py        # Web/document research
│   ├── vision.py          # Schematic/image analysis
│   ├── code.py            # Shader/Rust generation
│   └── verify.py          # Emulation verification
├── prompts/
│   ├── research.txt       # Research agent system prompt
│   ├── vision.txt         # Vision agent system prompt
│   └── code.txt           # Code agent system prompt
└── cache/
    └── embeddings/        # Cached document embeddings
```
