# Phosphlux Research Session

I'm working on Phosphlux - a real-time GPU emulation of analog video synthesis hardware and CRT displays.

The goal is physical modeling of historical equipment: oscilloscopes, video synthesizers (Paik/Abe, Sandin IP, Rutt/Etra), mixers (Edirol V4), and display technology (CRT phosphors, electron beam behavior).

First, help me set up and test the MCP server at tools/research-orchestrator/mcp_server.py

Then I want to understand my Ollama models and their best use cases:
- phi4-mini:latest (2.5GB)
- qwen3:latest (5.2GB)
- qwen2.5-coder:latest (4.7GB)
- deepseek-r1:8b (4.9GB)
- moondream:latest (1.7GB)
- minicpm-v:latest (5.5GB)
- nomic-embed-text (274MB)

Key constraints: 32GB RAM, RTX 2060 6GB VRAM

Read docs/HISTORICAL_EMULATION_PLAN.md and tools/research-orchestrator/CLAUDE_CLI_GUIDE.md for context.

Test each model with a simple query to understand speed/quality tradeoffs, then plan how to use them for researching the actual circuits and physics we need to model.
