#!/usr/bin/env python3
"""
Device Workbench MCP Server

Provides Claude with tools to control the Device Emulator Workbench app
for iterative device modeling with visual feedback.

Architecture:
  Claude → MCP Tools → HTTP API → Workbench App (Rust/wgpu)

Tools:
  - Shader control (load, reload, get errors)
  - Parameter adjustment
  - Screenshot capture
  - Visual comparison (reference vs output)
  - Vision model analysis
  - Test signal injection
  - Documentation helpers
"""

import asyncio
import base64
import json
import time
from pathlib import Path
from typing import Optional
from datetime import datetime
import httpx
import ollama
from mcp.server import Server
from mcp.server.stdio import stdio_server
from mcp.types import Tool, TextContent

# Workbench app connection
WORKBENCH_URL = "http://localhost:3030"
WORKBENCH_TIMEOUT = 10.0

# Directories
BASE_DIR = Path(__file__).parent.parent.parent
SCREENSHOTS_DIR = BASE_DIR / "docs" / "screenshots"
REFERENCES_DIR = BASE_DIR / "tools" / "references"
TEST_SIGNALS_DIR = Path(__file__).parent / "test_signals"

# Vision model for analysis
VISION_MODEL = "moondream:latest"

server = Server("device-workbench")


# === Helper Functions ===

async def workbench_request(method: str, endpoint: str, **kwargs) -> dict:
    """Make request to workbench app."""
    async with httpx.AsyncClient(timeout=WORKBENCH_TIMEOUT) as client:
        url = f"{WORKBENCH_URL}{endpoint}"
        if method == "GET":
            response = await client.get(url, **kwargs)
        elif method == "POST":
            response = await client.post(url, **kwargs)
        else:
            raise ValueError(f"Unknown method: {method}")

        if response.status_code != 200:
            raise Exception(f"Workbench error {response.status_code}: {response.text}")

        # Return JSON if possible, otherwise raw content
        content_type = response.headers.get("content-type", "")
        if "application/json" in content_type:
            return response.json()
        elif "image/" in content_type:
            return {"image_data": base64.b64encode(response.content).decode()}
        else:
            return {"text": response.text}


def vision_analyze(image_path: str, prompt: str) -> str:
    """Analyze image with vision model."""
    with open(image_path, "rb") as f:
        image_data = base64.b64encode(f.read()).decode()

    response = ollama.chat(
        model=VISION_MODEL,
        messages=[{
            "role": "user",
            "content": prompt,
            "images": [image_data]
        }]
    )
    return response["message"]["content"]


def save_screenshot(image_data: bytes, name: str, subdir: str = "") -> Path:
    """Save screenshot to docs folder."""
    if subdir:
        output_dir = SCREENSHOTS_DIR / subdir
    else:
        output_dir = SCREENSHOTS_DIR
    output_dir.mkdir(parents=True, exist_ok=True)

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    filename = f"{name}_{timestamp}.png"
    output_path = output_dir / filename

    with open(output_path, "wb") as f:
        f.write(image_data)

    return output_path


# === Tool Definitions ===

@server.list_tools()
async def list_tools():
    """List available workbench tools."""
    return [
        # === Shader Control ===
        Tool(
            name="workbench_load_shader",
            description="Load WGSL shader code into the workbench. Hot-reloads immediately.",
            inputSchema={
                "type": "object",
                "properties": {
                    "shader_code": {
                        "type": "string",
                        "description": "WGSL shader source code"
                    },
                    "name": {
                        "type": "string",
                        "description": "Name for this shader (for display/logging)"
                    }
                },
                "required": ["shader_code"]
            }
        ),
        Tool(
            name="workbench_get_state",
            description="Get current workbench state: loaded shader, parameters, errors.",
            inputSchema={
                "type": "object",
                "properties": {},
            }
        ),

        # === Parameters ===
        Tool(
            name="workbench_set_params",
            description="Set shader uniform parameters. Values are floats 0.0-1.0 unless specified.",
            inputSchema={
                "type": "object",
                "properties": {
                    "params": {
                        "type": "object",
                        "description": "Key-value pairs of parameter names and values",
                        "additionalProperties": {"type": "number"}
                    }
                },
                "required": ["params"]
            }
        ),
        Tool(
            name="workbench_get_params",
            description="Get current parameter values and their ranges.",
            inputSchema={
                "type": "object",
                "properties": {},
            }
        ),

        # === Reference Image ===
        Tool(
            name="workbench_load_reference",
            description="Load a reference image to compare against. Can be a file path or URL.",
            inputSchema={
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to reference image file"
                    }
                },
                "required": ["path"]
            }
        ),

        # === Screenshots ===
        Tool(
            name="workbench_screenshot",
            description="Capture current output. Returns path to saved image.",
            inputSchema={
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name for the screenshot file",
                        "default": "output"
                    },
                    "include_ui": {
                        "type": "boolean",
                        "description": "Include full UI (for docs) or just output",
                        "default": False
                    }
                },
            }
        ),

        # === Visual Comparison ===
        Tool(
            name="workbench_diff",
            description="Compare current output to reference image. Returns difference metrics.",
            inputSchema={
                "type": "object",
                "properties": {
                    "save_diff_image": {
                        "type": "boolean",
                        "description": "Save visual diff image",
                        "default": False
                    }
                },
            }
        ),
        Tool(
            name="workbench_analyze",
            description="Use vision model to analyze current output. Describe what you see.",
            inputSchema={
                "type": "object",
                "properties": {
                    "prompt": {
                        "type": "string",
                        "description": "What to analyze/look for",
                        "default": "Describe the visual effect shown in this image."
                    }
                },
            }
        ),
        Tool(
            name="workbench_compare_visual",
            description="Use vision model to compare output vs reference. Describes differences.",
            inputSchema={
                "type": "object",
                "properties": {
                    "focus": {
                        "type": "string",
                        "description": "What aspect to focus comparison on",
                        "enum": ["colors", "edges", "contrast", "overall", "artifacts"]
                    }
                },
            }
        ),

        # === Test Signals ===
        Tool(
            name="workbench_inject_signal",
            description="Inject a test signal/pattern as input to the shader.",
            inputSchema={
                "type": "object",
                "properties": {
                    "signal_type": {
                        "type": "string",
                        "enum": ["ramp_h", "ramp_v", "colorbars", "sweep", "gray50", "white", "black", "checker"],
                        "description": "Type of test signal"
                    },
                    "custom_path": {
                        "type": "string",
                        "description": "Path to custom input image (overrides signal_type)"
                    }
                },
            }
        ),

        # === Documentation ===
        Tool(
            name="workbench_doc_capture",
            description="Capture screenshot specifically for documentation with metadata.",
            inputSchema={
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Descriptive name for the image"
                    },
                    "device_name": {
                        "type": "string",
                        "description": "Device being emulated (used for folder organization)"
                    },
                    "description": {
                        "type": "string",
                        "description": "Caption/description for documentation"
                    }
                },
                "required": ["name", "device_name"]
            }
        ),
        Tool(
            name="workbench_record_sweep",
            description="Record a parameter sweep as animated GIF for documentation.",
            inputSchema={
                "type": "object",
                "properties": {
                    "param_name": {
                        "type": "string",
                        "description": "Parameter to sweep"
                    },
                    "start_value": {
                        "type": "number",
                        "description": "Starting value"
                    },
                    "end_value": {
                        "type": "number",
                        "description": "Ending value"
                    },
                    "steps": {
                        "type": "integer",
                        "description": "Number of steps in sweep",
                        "default": 10
                    },
                    "name": {
                        "type": "string",
                        "description": "Name for the output GIF"
                    }
                },
                "required": ["param_name", "start_value", "end_value", "name"]
            }
        ),

        # === Export ===
        Tool(
            name="workbench_export_node",
            description="Export current shader as a node for the main Phosphlux app.",
            inputSchema={
                "type": "object",
                "properties": {
                    "node_name": {
                        "type": "string",
                        "description": "Name for the exported node"
                    },
                    "category": {
                        "type": "string",
                        "description": "Category in node palette",
                        "enum": ["generators", "processors", "modulators", "utilities"]
                    },
                    "description": {
                        "type": "string",
                        "description": "Node description for palette"
                    }
                },
                "required": ["node_name", "category"]
            }
        ),
    ]


# === Tool Handlers ===

@server.call_tool()
async def call_tool(name: str, arguments: dict):
    """Route tool calls to handlers."""

    try:
        if name == "workbench_load_shader":
            return await handle_load_shader(arguments)
        elif name == "workbench_get_state":
            return await handle_get_state()
        elif name == "workbench_set_params":
            return await handle_set_params(arguments)
        elif name == "workbench_get_params":
            return await handle_get_params()
        elif name == "workbench_load_reference":
            return await handle_load_reference(arguments)
        elif name == "workbench_screenshot":
            return await handle_screenshot(arguments)
        elif name == "workbench_diff":
            return await handle_diff(arguments)
        elif name == "workbench_analyze":
            return await handle_analyze(arguments)
        elif name == "workbench_compare_visual":
            return await handle_compare_visual(arguments)
        elif name == "workbench_inject_signal":
            return await handle_inject_signal(arguments)
        elif name == "workbench_doc_capture":
            return await handle_doc_capture(arguments)
        elif name == "workbench_record_sweep":
            return await handle_record_sweep(arguments)
        elif name == "workbench_export_node":
            return await handle_export_node(arguments)
        else:
            return [TextContent(type="text", text=f"Unknown tool: {name}")]
    except httpx.ConnectError:
        return [TextContent(
            type="text",
            text=f"[ERROR] Cannot connect to workbench app at {WORKBENCH_URL}\n"
                 f"Make sure the Device Workbench app is running."
        )]
    except Exception as e:
        return [TextContent(type="text", text=f"[ERROR] {str(e)}")]


# === Handler Implementations ===

async def handle_load_shader(args: dict):
    """Load shader into workbench."""
    shader_code = args["shader_code"]
    name = args.get("name", "unnamed")

    result = await workbench_request(
        "POST", "/shader",
        json={"code": shader_code, "name": name}
    )

    if result.get("error"):
        return [TextContent(
            type="text",
            text=f"[SHADER ERROR]\n{result['error']}"
        )]

    return [TextContent(
        type="text",
        text=f"[OK] Shader '{name}' loaded successfully.\n"
             f"Compile time: {result.get('compile_time_ms', '?')}ms"
    )]


async def handle_get_state():
    """Get current workbench state."""
    result = await workbench_request("GET", "/state")

    output = "[WORKBENCH STATE]\n"
    output += f"Shader: {result.get('shader_name', 'none')}\n"
    output += f"Status: {result.get('status', 'unknown')}\n"
    output += f"FPS: {result.get('fps', '?')}\n"
    output += f"Reference loaded: {result.get('has_reference', False)}\n"

    if result.get("params"):
        output += "\nParameters:\n"
        for name, value in result["params"].items():
            output += f"  {name}: {value}\n"

    if result.get("errors"):
        output += f"\nErrors:\n{result['errors']}"

    return [TextContent(type="text", text=output)]


async def handle_set_params(args: dict):
    """Set shader parameters."""
    params = args["params"]

    result = await workbench_request(
        "POST", "/params",
        json=params
    )

    output = "[OK] Parameters updated:\n"
    for name, value in params.items():
        output += f"  {name}: {value}\n"

    return [TextContent(type="text", text=output)]


async def handle_get_params():
    """Get current parameters."""
    result = await workbench_request("GET", "/params")

    output = "[PARAMETERS]\n"
    for name, info in result.get("params", {}).items():
        if isinstance(info, dict):
            output += f"  {name}: {info.get('value', '?')} (range: {info.get('min', 0)}-{info.get('max', 1)})\n"
        else:
            output += f"  {name}: {info}\n"

    return [TextContent(type="text", text=output)]


async def handle_load_reference(args: dict):
    """Load reference image."""
    path = args["path"]

    # Read image and send to workbench
    with open(path, "rb") as f:
        image_data = base64.b64encode(f.read()).decode()

    result = await workbench_request(
        "POST", "/reference",
        json={"image": image_data, "path": path}
    )

    return [TextContent(
        type="text",
        text=f"[OK] Reference image loaded: {path}\n"
             f"Size: {result.get('width', '?')}x{result.get('height', '?')}"
    )]


async def handle_screenshot(args: dict):
    """Capture screenshot."""
    name = args.get("name", "output")
    include_ui = args.get("include_ui", False)

    endpoint = "/screenshot/ui" if include_ui else "/screenshot"
    result = await workbench_request("GET", endpoint)

    # Decode and save
    image_data = base64.b64decode(result["image_data"])
    output_path = save_screenshot(image_data, name)

    return [TextContent(
        type="text",
        text=f"[OK] Screenshot saved: {output_path}"
    )]


async def handle_diff(args: dict):
    """Compare output to reference."""
    save_image = args.get("save_diff_image", False)

    result = await workbench_request("GET", "/diff")

    output = "[DIFF ANALYSIS]\n"
    output += f"Similarity: {result.get('similarity', 0) * 100:.1f}%\n"
    output += f"Mean absolute error: {result.get('mae', '?')}\n"
    output += f"Peak error: {result.get('peak_error', '?')}\n"

    regions = result.get("diff_regions", [])
    if regions:
        output += f"\nHigh-diff regions:\n"
        for region in regions[:5]:
            output += f"  - {region}\n"

    if save_image and result.get("diff_image"):
        image_data = base64.b64decode(result["diff_image"])
        diff_path = save_screenshot(image_data, "diff")
        output += f"\nDiff image saved: {diff_path}"

    return [TextContent(type="text", text=output)]


async def handle_analyze(args: dict):
    """Analyze output with vision model."""
    prompt = args.get("prompt", "Describe the visual effect shown in this image.")

    # Get screenshot
    result = await workbench_request("GET", "/screenshot")
    image_data = base64.b64decode(result["image_data"])

    # Save temporarily for vision model
    temp_path = Path(__file__).parent / "temp_screenshot.png"
    with open(temp_path, "wb") as f:
        f.write(image_data)

    # Analyze with vision model
    analysis = vision_analyze(str(temp_path), prompt)

    # Cleanup
    temp_path.unlink(missing_ok=True)

    return [TextContent(
        type="text",
        text=f"[VISION ANALYSIS]\n{analysis}"
    )]


async def handle_compare_visual(args: dict):
    """Visual comparison of output vs reference."""
    focus = args.get("focus", "overall")

    # Get both images
    output_result = await workbench_request("GET", "/screenshot")

    # For comparison, we need reference too - get from state
    state = await workbench_request("GET", "/state")
    if not state.get("has_reference"):
        return [TextContent(type="text", text="[ERROR] No reference image loaded")]

    ref_result = await workbench_request("GET", "/reference")

    # Save both temporarily
    temp_output = Path(__file__).parent / "temp_output.png"
    temp_ref = Path(__file__).parent / "temp_reference.png"

    with open(temp_output, "wb") as f:
        f.write(base64.b64decode(output_result["image_data"]))
    with open(temp_ref, "wb") as f:
        f.write(base64.b64decode(ref_result["image_data"]))

    focus_prompts = {
        "colors": "Compare the colors between these two images. What color differences do you see?",
        "edges": "Compare the edges and transitions. Are they sharper or softer in one image?",
        "contrast": "Compare the contrast and dynamic range. Which has more/less contrast?",
        "overall": "Compare these two images overall. What are the main differences?",
        "artifacts": "Look for any visual artifacts, banding, or errors in the second image compared to the first."
    }

    prompt = f"""I'm comparing a reference image (first) to my shader output (second).
{focus_prompts.get(focus, focus_prompts['overall'])}
Be specific about what needs to change to make the output match the reference."""

    # Analyze - note: for true comparison we'd need multi-image support
    # For now, analyze output and ask about differences
    analysis = vision_analyze(str(temp_output), prompt)

    # Cleanup
    temp_output.unlink(missing_ok=True)
    temp_ref.unlink(missing_ok=True)

    return [TextContent(
        type="text",
        text=f"[VISUAL COMPARISON - {focus.upper()}]\n{analysis}"
    )]


async def handle_inject_signal(args: dict):
    """Inject test signal."""
    signal_type = args.get("signal_type", "ramp_h")
    custom_path = args.get("custom_path")

    if custom_path:
        with open(custom_path, "rb") as f:
            image_data = base64.b64encode(f.read()).decode()
        source = custom_path
    else:
        # Load from test_signals directory
        signal_path = TEST_SIGNALS_DIR / f"{signal_type}.png"
        if not signal_path.exists():
            # Generate programmatically if not exists
            return [TextContent(
                type="text",
                text=f"[ERROR] Test signal '{signal_type}' not found at {signal_path}"
            )]
        with open(signal_path, "rb") as f:
            image_data = base64.b64encode(f.read()).decode()
        source = signal_type

    result = await workbench_request(
        "POST", "/input",
        json={"image": image_data}
    )

    return [TextContent(
        type="text",
        text=f"[OK] Injected test signal: {source}"
    )]


async def handle_doc_capture(args: dict):
    """Capture for documentation with metadata."""
    name = args["name"]
    device_name = args["device_name"]
    description = args.get("description", "")

    # Capture with UI
    result = await workbench_request("GET", "/screenshot/ui")
    image_data = base64.b64decode(result["image_data"])

    # Save to device-specific folder
    output_path = save_screenshot(image_data, name, subdir=device_name)

    # Also save metadata
    meta_path = output_path.with_suffix(".json")
    metadata = {
        "name": name,
        "device": device_name,
        "description": description,
        "timestamp": datetime.now().isoformat(),
        "image_file": output_path.name
    }
    with open(meta_path, "w") as f:
        json.dump(metadata, f, indent=2)

    return [TextContent(
        type="text",
        text=f"[OK] Documentation capture saved:\n"
             f"  Image: {output_path}\n"
             f"  Metadata: {meta_path}"
    )]


async def handle_record_sweep(args: dict):
    """Record parameter sweep as GIF."""
    param_name = args["param_name"]
    start_value = args["start_value"]
    end_value = args["end_value"]
    steps = args.get("steps", 10)
    name = args["name"]

    frames = []
    step_size = (end_value - start_value) / (steps - 1)

    for i in range(steps):
        value = start_value + (i * step_size)

        # Set parameter
        await workbench_request("POST", "/params", json={param_name: value})

        # Small delay for render
        await asyncio.sleep(0.1)

        # Capture frame
        result = await workbench_request("GET", "/screenshot")
        frames.append(base64.b64decode(result["image_data"]))

    # Combine into GIF (requires pillow)
    try:
        from PIL import Image
        import io

        images = [Image.open(io.BytesIO(f)) for f in frames]

        output_dir = SCREENSHOTS_DIR / "sweeps"
        output_dir.mkdir(parents=True, exist_ok=True)
        output_path = output_dir / f"{name}.gif"

        images[0].save(
            output_path,
            save_all=True,
            append_images=images[1:],
            duration=100,
            loop=0
        )

        return [TextContent(
            type="text",
            text=f"[OK] Sweep recorded: {output_path}\n"
                 f"  Parameter: {param_name}\n"
                 f"  Range: {start_value} → {end_value}\n"
                 f"  Frames: {steps}"
        )]
    except ImportError:
        return [TextContent(
            type="text",
            text="[ERROR] PIL/Pillow not installed. Cannot create GIF.\n"
                 "Install with: pip install Pillow"
        )]


async def handle_export_node(args: dict):
    """Export shader as node for main app."""
    node_name = args["node_name"]
    category = args["category"]
    description = args.get("description", "")

    # Get current state
    state = await workbench_request("GET", "/state")
    shader_code = state.get("shader_code", "")
    params = state.get("params", {})

    # Generate node definition
    node_def = {
        "name": node_name,
        "category": category,
        "description": description,
        "shader": shader_code,
        "params": params,
        "exported_at": datetime.now().isoformat()
    }

    # Save to exports directory
    exports_dir = BASE_DIR / "src" / "nodes" / "exported"
    exports_dir.mkdir(parents=True, exist_ok=True)

    output_path = exports_dir / f"{node_name}.json"
    with open(output_path, "w") as f:
        json.dump(node_def, f, indent=2)

    return [TextContent(
        type="text",
        text=f"[OK] Node exported: {output_path}\n"
             f"  Name: {node_name}\n"
             f"  Category: {category}\n"
             f"  Parameters: {len(params)}"
    )]


# === Main ===

async def main():
    """Run the MCP server."""
    # Ensure directories exist
    SCREENSHOTS_DIR.mkdir(parents=True, exist_ok=True)
    REFERENCES_DIR.mkdir(parents=True, exist_ok=True)
    TEST_SIGNALS_DIR.mkdir(parents=True, exist_ok=True)

    async with stdio_server() as (read_stream, write_stream):
        await server.run(read_stream, write_stream, server.create_initialization_options())


if __name__ == "__main__":
    asyncio.run(main())
