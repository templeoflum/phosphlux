# Device Workbench HTTP API Specification

The Rust workbench app exposes these endpoints on `http://localhost:3030`.

## State & Info

### GET /state
Returns current workbench state.

**Response:**
```json
{
  "shader_name": "colorizer_v1",
  "shader_code": "// WGSL source...",
  "status": "ok",
  "fps": 60.0,
  "has_reference": true,
  "params": {
    "gain": 1.0,
    "fold": 0.5,
    "bias": 0.0
  },
  "errors": null
}
```

### GET /params
Returns parameter definitions with ranges.

**Response:**
```json
{
  "params": {
    "gain": {"value": 1.0, "min": 0.0, "max": 2.0, "default": 1.0},
    "fold": {"value": 0.5, "min": 0.0, "max": 1.0, "default": 0.5}
  }
}
```

## Shader Control

### POST /shader
Load/hot-reload a shader.

**Request:**
```json
{
  "code": "// WGSL source code",
  "name": "my_shader"
}
```

**Response (success):**
```json
{
  "status": "ok",
  "compile_time_ms": 12
}
```

**Response (error):**
```json
{
  "status": "error",
  "error": "line 23: expected ';' after expression"
}
```

### POST /params
Set parameter values.

**Request:**
```json
{
  "gain": 1.5,
  "fold": 0.7
}
```

**Response:**
```json
{
  "status": "ok",
  "params": {"gain": 1.5, "fold": 0.7}
}
```

## Images

### POST /reference
Load reference image for comparison.

**Request:**
```json
{
  "image": "<base64 encoded PNG>",
  "path": "/path/to/original.png"
}
```

**Response:**
```json
{
  "status": "ok",
  "width": 512,
  "height": 512
}
```

### GET /reference
Get currently loaded reference image.

**Response:**
- Content-Type: `image/png` (raw PNG bytes)
- Or JSON with `{"image_data": "<base64>"}` if Accept header requests JSON

### POST /input
Inject input image/signal to process.

**Request:**
```json
{
  "image": "<base64 encoded PNG>"
}
```

**Response:**
```json
{
  "status": "ok"
}
```

## Screenshots

### GET /screenshot
Capture current shader output only.

**Response:**
- Content-Type: `image/png`
- Raw PNG bytes of the output render

### GET /screenshot/ui
Capture full window including UI.

**Response:**
- Content-Type: `image/png`
- Raw PNG bytes of entire window

## Comparison

### GET /diff
Compare current output to reference image.

**Response:**
```json
{
  "similarity": 0.92,
  "mae": 0.034,
  "peak_error": 0.21,
  "diff_regions": [
    "top-right corner: high contrast difference",
    "center: slight color shift"
  ],
  "diff_image": "<base64 encoded diff visualization>"
}
```

## Metrics

### GET /metrics
Performance and render statistics.

**Response:**
```json
{
  "fps": 60.0,
  "frame_time_ms": 2.3,
  "gpu_time_ms": 1.8,
  "shader_compile_ms": 12,
  "memory_mb": 45.2
}
```

---

## Implementation Notes

### CORS
Enable CORS for local development:
```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, OPTIONS
Access-Control-Allow-Headers: Content-Type
```

### Error Handling
All endpoints return appropriate HTTP status codes:
- 200: Success
- 400: Bad request (invalid JSON, missing params)
- 500: Internal error (shader compile fail, etc.)

Error response format:
```json
{
  "status": "error",
  "error": "description of what went wrong"
}
```

### Recommended Libraries (Rust)
- `warp` or `axum` for HTTP server
- `wgpu` for rendering
- `image` crate for PNG encode/decode
- `base64` crate for encoding
- `serde_json` for JSON
