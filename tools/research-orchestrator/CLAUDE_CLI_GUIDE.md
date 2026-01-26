# Claude CLI Research Orchestration Guide

This guide explains how to use Claude CLI to orchestrate multi-agent research for historical video synthesis hardware emulation.

## Quick Start

```bash
# 1. Navigate to project
cd C:\Users\User\Documents\0000000\analog-claude-video-synthesis-foundation-8LfZV

# 2. Install MCP server dependencies
cd tools/research-orchestrator
pip install -r requirements.txt

# 3. Add to Claude CLI config (~/.claude/config.json or equivalent)
```

Add this to your Claude CLI MCP configuration:

```json
{
  "mcpServers": {
    "research": {
      "command": "python",
      "args": ["C:\\Users\\User\\Documents\\0000000\\analog-claude-video-synthesis-foundation-8LfZV\\tools\\research-orchestrator\\mcp_server.py"]
    }
  }
}
```

## Available Tools

Once configured, these tools become available in Claude CLI:

### 1. `research_query`
Deep research on video synthesis topics.

```
> Research the Paik/Abe video synthesizer's folding amplifier circuit

Uses: qwen3:latest (5.2GB)
Output: Structured markdown with components, behavior, history
```

### 2. `analyze_schematic`
Vision-based schematic analysis.

```
> Analyze the schematic at research/devices/sandin_ip_differentiator.png

Uses: minicpm-v:latest (5.5GB)
Output: Component list, signal flow, transfer function
```

### 3. `generate_shader`
WGSL shader code generation.

```
> Generate a shader for Paik/Abe style signal folding with gain control

Uses: qwen2.5-coder:latest (4.7GB)
Output: Complete WGSL code with comments
```

### 4. `verify_emulation`
Compare emulation against documented behavior.

```
> Verify that my folding implementation matches the expected triangle wave output

Uses: deepseek-r1:8b (4.9GB)
Output: Accuracy rating, discrepancies, recommendations
```

### 5. `embed_document` / `search_documents`
Build searchable knowledge base.

```
> Embed this datasheet for the MC1445 gain-controlled amplifier
> Search for "video rate op-amp bandwidth"

Uses: nomic-embed-text (274MB)
Output: Semantic search results
```

---

## Research Workflows

### Workflow 1: Research a New Device

```
Step 1: Initial Research
> research_query("Rutt/Etra video synthesizer", depth="deep", focus="components")

Step 2: Find Schematics
> [Use web search or scan documents]
> analyze_schematic("research/devices/rutt_etra_deflection.png")

Step 3: Extract Key Behaviors
> research_query("Rutt/Etra Z-axis displacement", focus="behavior")

Step 4: Generate Implementation
> generate_shader("Rutt/Etra scan displacement with Z-axis luminance offset")

Step 5: Verify
> verify_emulation(
    expected="Documentation says displacement should scale linearly with luma",
    actual="Current implementation shows exponential response",
    tolerance="strict"
  )
```

### Workflow 2: Build Knowledge Base

```
# Embed key documents as you find them
> embed_document(text="[paste datasheet content]", doc_id="MC1445_datasheet")
> embed_document(text="[paste forum thread]", doc_id="scanlines_paik_abe_discussion")

# Later, search for relevant info
> search_documents("gain controlled amplifier video rate")
```

### Workflow 3: Parallel Research (Multi-Agent)

For complex topics, run multiple queries:

```
# In separate terminals or using Claude CLI's parallel execution:

Terminal 1: research_query("Paik/Abe wobbulator", focus="components")
Terminal 2: research_query("CRT horizontal deflection coils", focus="behavior")
Terminal 3: research_query("video rate oscillator circuits", focus="implementation")

# Then synthesize results
> "Combine the research on wobbulator, deflection coils, and oscillators
   to design a complete scan modulation system"
```

---

## Model Selection Guide

Your system has these Ollama models available:

| Model | VRAM | Speed | Best For |
|-------|------|-------|----------|
| `phi4-mini` | ~2GB | Fast | Quick questions, coordination |
| `qwen3` | ~4GB | Medium | Deep research, synthesis |
| `qwen2.5-coder` | ~4GB | Medium | Shader/Rust code generation |
| `deepseek-r1:8b` | ~4GB | Slow | Complex analysis, verification |
| `moondream` | ~1.5GB | Fast | Quick image glance |
| `minicpm-v` | ~5GB | Slow | Detailed schematic OCR |
| `nomic-embed-text` | ~0.3GB | Fast | Document embeddings |

**Memory Strategy** (32GB RAM, 6GB VRAM):
- Keep `nomic-embed-text` loaded (tiny)
- Load task-specific model on demand
- Vision models are heavy - use `moondream` for quick looks, `minicpm-v` for detailed analysis

---

## Directory Structure

```
tools/research-orchestrator/
├── mcp_server.py              # MCP server (run this)
├── requirements.txt           # Python dependencies
├── prompts/
│   ├── research.txt          # Research agent system prompt
│   ├── vision.txt            # Vision agent system prompt
│   └── code.txt              # Code agent system prompt
├── cache/
│   └── embeddings/           # Cached document embeddings
└── CLAUDE_CLI_GUIDE.md       # This file

research/                      # Create this for collected materials
├── schematics/               # Schematic images
├── datasheets/               # Component datasheets
├── patents/                  # Patent PDFs
├── manuals/                  # Service manuals
└── notes/                    # Research notes
```

---

## Example Session

```
User: I want to emulate the Sandin Image Processor's amplitude classifier

Claude: Let me research that component.

> research_query("Sandin Image Processor amplitude classifier",
                 depth="deep", focus="implementation")

[Model returns detailed analysis of the amplitude classifier circuit,
 including the MC1445 IC, threshold comparators, and step function output]

Claude: The amplitude classifier uses an MC1445 gain-controlled amp
with multiple threshold comparators. Let me generate the shader.

> generate_shader(
    specification="Sandin IP amplitude classifier with 8 quantization levels",
    transfer_function="output = floor(input * 8) / 7"
  )

[Model returns WGSL shader code]

Claude: Here's the implementation. Let me verify it matches
the documented behavior.

> verify_emulation(
    expected="8 discrete output levels, hard transitions at each threshold",
    actual="[describe your implementation's behavior]",
    tolerance="moderate"
  )

[Model returns accuracy analysis and suggestions]
```

---

## Troubleshooting

### "Model not found"
Ensure the model is downloaded:
```bash
ollama pull qwen3:latest
```

### "CUDA out of memory"
Your RTX 2060 has 6GB. Unload other models:
```bash
ollama stop qwen3
ollama run phi4-mini  # Use smaller model
```

### "MCP server not connecting"
Check the server is running:
```bash
python tools/research-orchestrator/mcp_server.py
```

### Vision model slow
`minicpm-v` is thorough but slow. For quick checks:
- Edit `mcp_server.py` line 18: change `"vision_ocr"` to use `"moondream:latest"`

---

## Next Steps

1. **Set up research directory**: Create `research/` folder structure
2. **Collect first device**: Start with Paik/Abe (most documented)
3. **Build knowledge base**: Embed key documents as you find them
4. **Iterate**: Research → Implement → Verify → Refine

The goal is to follow the inventors, not the interpreters. Each emulation should be traceable back to primary sources.
