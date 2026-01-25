# Visual Workbench MCP Template

A minimal template for creating vision-guided development environments.

## Required MCP Tools

```python
# Core tools - minimum viable workbench
@server.tool()
def load_code(code: str, name: str = "unnamed"):
    """Hot-reload code into the workbench"""

@server.tool()
def set_params(params: dict):
    """Set runtime parameters"""

@server.tool()
def get_state():
    """Get current workbench state (loaded code, params, errors)"""

@server.tool()
def screenshot(name: str = "output"):
    """Capture current output to file, return path"""

# Test input tools
@server.tool()
def inject_input(input_type: str):
    """Load standard test input (ramp, grid, noise, etc.)"""

@server.tool()
def load_custom_input(path: str):
    """Load custom input file"""

# Analysis tools
@server.tool()
def analyze(prompt: str):
    """Vision model analyzes current output"""

@server.tool()
def diff(reference_path: str):
    """Compare output to reference, return metrics"""

# Documentation tools
@server.tool()
def record_sweep(param: str, start: float, end: float, steps: int):
    """Record parameter sweep as GIF"""

@server.tool()
def export(name: str, category: str):
    """Export current code as reusable component"""
```

## Workbench App Requirements

### Minimal Architecture
```
┌─────────────────────────────────────┐
│  Workbench App                      │
├─────────────────────────────────────┤
│  ┌─────────┐    ┌────────────────┐  │
│  │ Renderer │◄───│ Code Hot-Reload│  │
│  └────┬────┘    └────────────────┘  │
│       │                              │
│       ▼                              │
│  ┌─────────┐    ┌────────────────┐  │
│  │ Output  │───►│ Screenshot API │  │
│  └─────────┘    └────────────────┘  │
│                                      │
│  ┌──────────────────────────────┐   │
│  │ HTTP API (localhost:PORT)    │   │
│  │ - POST /code                 │   │
│  │ - POST /params               │   │
│  │ - GET  /screenshot           │   │
│  │ - GET  /state                │   │
│  └──────────────────────────────┘   │
└─────────────────────────────────────┘
         ▲
         │ HTTP
         ▼
┌─────────────────────────────────────┐
│  MCP Server                         │
│  - Translates MCP calls to HTTP     │
│  - Handles file I/O for screenshots │
│  - Provides vision analysis         │
└─────────────────────────────────────┘
```

### Key Design Decisions

1. **Separate processes**: Workbench app and MCP server are separate
   - Workbench can crash without killing MCP
   - Can restart workbench without reconnecting MCP

2. **HTTP API**: Simple REST, not complex IPC
   - Easy to debug with curl
   - Language-agnostic
   - Can add other clients later

3. **File-based screenshots**: Write to disk, return path
   - Claude can read files directly
   - Avoids base64 in responses
   - Screenshots persist for review

4. **Stateless code loading**: Full replacement, not incremental
   - Simpler mental model
   - No hidden state bugs
   - "What you send is what runs"

## Standard Test Inputs by Domain

### Graphics/Shaders
- `ramp_h` - Horizontal grayscale gradient
- `ramp_v` - Vertical grayscale gradient
- `colorbars` - SMPTE color bars
- `checker` - Checkerboard pattern
- `uv_grid` - UV coordinate visualization

### UI Development
- `lorem` - Standard lorem ipsum content
- `edge_cases` - Very long text, empty states
- `breakpoints` - Content at each responsive size

### Data Visualization
- `linear` - Linear growth data
- `exponential` - Exponential growth
- `categorical` - Discrete categories
- `sparse` - Data with gaps
- `outliers` - Data with extreme values

### Image Processing
- `gradient` - Smooth gradient
- `noise` - Random noise
- `edges` - High contrast edges
- `photo` - Natural photograph

## Workflow Checklist

```markdown
## Starting a tuning session

- [ ] Workbench app running
- [ ] MCP server connected
- [ ] Test inputs available
- [ ] Screenshot directory exists and is writable
- [ ] Reference images loaded (if comparing)

## During iteration

- [ ] Load code → check for compile errors
- [ ] Inject appropriate test input
- [ ] Take screenshot after each change
- [ ] VIEW screenshot (don't just save it)
- [ ] Describe what you see before making changes
- [ ] Make ONE change at a time
- [ ] Record parameter sweeps for documentation

## Before export

- [ ] Test with multiple input types
- [ ] Verify edge cases (0, 1, extremes)
- [ ] Record demonstration sweeps
- [ ] Document parameter ranges and meanings
```

## Example: Adding to New Project

1. **Create workbench app** (use wgpu/winit or web canvas)
2. **Add HTTP endpoints** for code, params, screenshot
3. **Create MCP server** that calls HTTP endpoints
4. **Add to .mcp.json**:
```json
{
  "mcpServers": {
    "my-workbench": {
      "command": "python",
      "args": ["tools/my-workbench/mcp_server.py"]
    }
  }
}
```
5. **Start workbench app** before MCP server
6. **Begin vision-guided development**

## Anti-patterns

❌ **Don't** mix workbench with full application
❌ **Don't** require human to describe visual output
❌ **Don't** skip viewing screenshots ("I'm sure it's fine")
❌ **Don't** change multiple things between screenshots
❌ **Don't** use complex IPC when HTTP works
❌ **Don't** embed images in responses (use file paths)
