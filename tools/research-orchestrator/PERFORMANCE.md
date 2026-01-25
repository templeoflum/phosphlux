# MCP Server Performance Guide

Benchmarked: 2026-01-24

## Tier Performance Summary

| Tier | Model | Cold Start | Warm Query | Abort Threshold |
|------|-------|------------|------------|-----------------|
| Fast | phi4-mini | 3s | 4-5s | 30s |
| Deep | llama3.1 | 8-10s | 10s | 45s |
| Vision | moondream | 5s | 5-10s | 30s |
| Embed | qwen3-embedding | 3s | 0.1s | 15s |

## Key Findings

### Model Selection
- **llama3.1** chosen for deep tier over qwen3 (accurate for video synthesis domain)
- qwen3's "thinking mode" adds 20-30s overhead, making it impractical
- phi4-mini hallucinates on domain-specific queries (use only for simple lookups)

### Timing Behavior
1. **Cold start**: First query of session loads model from disk (3-10s)
2. **Warm queries**: Subsequent queries to same tier are fast
3. **Tier switching**: Changing tiers unloads old model, loads new (4-7s overhead)
4. **Model retention**: Ollama keeps models loaded for ~5 minutes of inactivity

### Error Detection
If a query exceeds the abort threshold, it indicates:
- Ollama is stuck or crashed
- Another large model is blocking memory
- System resource exhaustion

## Optimization Strategies

### 1. Batch by Tier
Group operations by tier to minimize model swapping:
```
# Good: All fast queries together
quick_query("What is a VCO?")
quick_query("Define resistor")
quick_query("Convert 1V to mV")
# Then switch to deep
research_query("Paik/Abe video synthesizer", "history")
```

### 2. Pre-warm Models
Before a work session, warm the tiers you'll use:
```bash
ollama run phi4-mini "warmup" --verbose
ollama run llama3.1 "warmup" --verbose
```

### 3. Monitor Memory
Check what's loaded before starting:
```bash
ollama ps
```

If a large model is loaded, stop it:
```bash
ollama stop deepseek-ocr
```

### 4. Interpret Timing Headers
Every response includes timing metadata:
```
[FAST] 4.2s (OK)      # Normal
[DEEP] 15.3s (SLOW)   # Exceeded warm threshold, but completed
[EMBED ERROR] 45.2s   # Failed, hit timeout
```

## Troubleshooting

### Query takes >30s
1. Check `ollama ps` - is another model loaded?
2. Check system memory - is swap being used?
3. Restart ollama: `taskkill /IM ollama.exe /F && ollama serve`

### "Model not found" error
```bash
ollama pull llama3.1
ollama pull phi4-mini
ollama pull moondream
ollama pull qwen3-embedding
```

### Vision tier failures
- Ensure image path is absolute
- Check file exists and is readable
- Supported formats: PNG, JPG, WEBP

## Hardware Requirements

Tested on system with:
- 16GB+ RAM recommended
- GPU with 6GB+ VRAM preferred
- Models use 2-5GB each when loaded

Without GPU, expect 2-3x slower inference times.
