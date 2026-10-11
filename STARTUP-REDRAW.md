# Startup redraw

T-0071 · 11 October 2026

An explicit startup redraw request starts the previously idle redraw chain on this Mac. The instrumented baseline presented zero frames; the candidate presented continuously on two launches. The node editor and colored radial preview have **not** been visually verified. The amended T-0071 completion criteria exclude that appearance check; it belongs to follow-up T-0072.

## Change and measurement

On `task/T-0071-startup-redraw`, `main.rs` requests a redraw after renderer and egui initialization. The existing redraw handler requests subsequent frames. Debug diagnostics count redraw events before egui filtering, render-frame calls, and successful returns from `render_with_egui`, once per second. Source inspection of `renderer.rs` confirms its success return occurs after `queue.submit` and `output.present`; this counts calls returning successfully, not proof of visible pixels. No GPU fence or pixel readback was added.

The baseline contained exactly the same diagnostics without the startup request. Only the startup request and its comment changed between the two release builds. There was no patch interaction, resize action, saved-state loading, or app argument.

Both commands ran from the repository root and exited **0**:

```sh
CARGO_TARGET_DIR=/tmp/phosphlux-baseline cargo build --release --locked
```

Logs: `/tmp/T-0071-before-build.log` and `/tmp/T-0071-after-build.log`. Their final output was respectively:

```text
warning: `phosphlux` (bin "phosphlux") generated 66 warnings (run `cargo fix --bin "phosphlux" -p phosphlux` to apply 26 suggestions)
    Finished `release` profile [optimized] target(s) in 57.23s
```

```text
warning: `phosphlux` (bin "phosphlux") generated 66 warnings (run `cargo fix --bin "phosphlux" -p phosphlux` to apply 26 suggestions)
    Finished `release` profile [optimized] target(s) in 31.53s
```

## Bounded launches

All three launches used Python `subprocess.Popen(['/tmp/phosphlux-baseline/release/phosphlux'])` with inherited environment plus `RUST_LOG=info,phosphlux=debug` and `RUST_BACKTRACE=1`, stdout/stderr joined into the logs below. Each wrapper slept for its stated bound, sent SIGTERM, and reaped the app. Wrapper commands exited **0**; app exits **-15** reflect intentional termination, not normal UI shutdown. No forced kill was needed. Termination ended each process and its window; no UI close action or screenshot was performed.

| Launch | Scheduling | PID | Seconds | Last logged redraw/render/present |
| --- | --- | --- | --- | --- |
| 1 | Original | 34627 | 8.007 | 0 / 0 / 0 |
| 2 | Startup request | 34726 | 8.016 | 396 / 396 / 396 |
| 3 | Same candidate, repeat | 34785 | 6.016 | 277 / 277 / 277 |

Counts below are the last periodic observation, not final shutdown totals. Candidate counts increased by about 60 per second. All launches identified Apple M5 Max / Metal, two reachable devices, and one render segment. No panic or runtime error appeared. All retained the warning that egui prefers a non-sRGB framebuffer. This task did not investigate or change that warning.

After all launches, `ps -p 34627,34726,34785 -o pid=,comm=` exited **1** with no output: none of the three launched processes remained.

### Baseline

Complete runtime output from `/tmp/T-0071-before-runtime.log`:

```text
[2026-10-11T08:11:01Z INFO  phosphlux] Starting Phosphlux v2
[2026-10-11T08:11:01Z INFO  wgpu_core::instance] Adapter AdapterInfo { name: "Apple M5 Max", vendor: 0, device: 0, device_type: IntegratedGpu, driver: "", driver_info: "", backend: Metal }
[2026-10-11T08:11:01Z INFO  phosphlux::gpu::renderer] GPU: Apple M5 Max
[2026-10-11T08:11:02Z WARN  egui_wgpu::renderer] Detected a linear (sRGBA aware) framebuffer Bgra8UnormSrgb. egui prefers Rgba8Unorm or Bgra8Unorm
[2026-10-11T08:11:02Z INFO  phosphlux::gpu::shader_composer] Render plan: 2 reachable devices out of 2 total (output=1)
[2026-10-11T08:11:02Z INFO  phosphlux::gpu::shader_composer] Render plan: 1 segments (0 scan modulators)
[2026-10-11T08:11:02Z INFO  phosphlux] Window created with node editor layout
[2026-10-11T08:11:02Z DEBUG phosphlux] Startup frames: redraw=0 render=0 present=0
[2026-10-11T08:11:03Z DEBUG phosphlux] Startup frames: redraw=0 render=0 present=0
[2026-10-11T08:11:04Z DEBUG phosphlux] Startup frames: redraw=0 render=0 present=0
[2026-10-11T08:11:05Z DEBUG phosphlux] Startup frames: redraw=0 render=0 present=0
[2026-10-11T08:11:06Z DEBUG phosphlux] Startup frames: redraw=0 render=0 present=0
[2026-10-11T08:11:07Z DEBUG phosphlux] Startup frames: redraw=0 render=0 present=0
[2026-10-11T08:11:08Z DEBUG phosphlux] Startup frames: redraw=0 render=0 present=0
```

### Startup request

Complete runtime output from `/tmp/T-0071-after-runtime.log`:

```text
[2026-10-11T08:11:52Z INFO  phosphlux] Starting Phosphlux v2
[2026-10-11T08:11:53Z INFO  wgpu_core::instance] Adapter AdapterInfo { name: "Apple M5 Max", vendor: 0, device: 0, device_type: IntegratedGpu, driver: "", driver_info: "", backend: Metal }
[2026-10-11T08:11:53Z INFO  phosphlux::gpu::renderer] GPU: Apple M5 Max
[2026-10-11T08:11:53Z WARN  egui_wgpu::renderer] Detected a linear (sRGBA aware) framebuffer Bgra8UnormSrgb. egui prefers Rgba8Unorm or Bgra8Unorm
[2026-10-11T08:11:53Z INFO  phosphlux::gpu::shader_composer] Render plan: 2 reachable devices out of 2 total (output=1)
[2026-10-11T08:11:53Z INFO  phosphlux::gpu::shader_composer] Render plan: 1 segments (0 scan modulators)
[2026-10-11T08:11:53Z INFO  phosphlux] Window created with node editor layout
[2026-10-11T08:11:53Z DEBUG phosphlux] First RedrawRequested received
[2026-10-11T08:11:53Z DEBUG phosphlux] First render_with_egui success (submit and present returned)
[2026-10-11T08:11:53Z DEBUG phosphlux] Startup frames: redraw=33 render=33 present=33
[2026-10-11T08:11:54Z DEBUG phosphlux] Startup frames: redraw=94 render=94 present=94
[2026-10-11T08:11:55Z DEBUG phosphlux] Startup frames: redraw=154 render=154 present=154
[2026-10-11T08:11:56Z DEBUG phosphlux] Startup frames: redraw=215 render=215 present=215
[2026-10-11T08:11:57Z DEBUG phosphlux] Startup frames: redraw=276 render=276 present=276
[2026-10-11T08:11:58Z DEBUG phosphlux] Startup frames: redraw=336 render=336 present=336
[2026-10-11T08:11:59Z DEBUG phosphlux] Startup frames: redraw=396 render=396 present=396
```

### Repeat candidate

Complete runtime output from `/tmp/T-0071-confirm-runtime.log`:

```text
[2026-10-11T08:12:14Z INFO  phosphlux] Starting Phosphlux v2
[2026-10-11T08:12:14Z INFO  wgpu_core::instance] Adapter AdapterInfo { name: "Apple M5 Max", vendor: 0, device: 0, device_type: IntegratedGpu, driver: "", driver_info: "", backend: Metal }
[2026-10-11T08:12:14Z INFO  phosphlux::gpu::renderer] GPU: Apple M5 Max
[2026-10-11T08:12:14Z WARN  egui_wgpu::renderer] Detected a linear (sRGBA aware) framebuffer Bgra8UnormSrgb. egui prefers Rgba8Unorm or Bgra8Unorm
[2026-10-11T08:12:14Z INFO  phosphlux::gpu::shader_composer] Render plan: 2 reachable devices out of 2 total (output=1)
[2026-10-11T08:12:14Z INFO  phosphlux::gpu::shader_composer] Render plan: 1 segments (0 scan modulators)
[2026-10-11T08:12:14Z INFO  phosphlux] Window created with node editor layout
[2026-10-11T08:12:14Z DEBUG phosphlux] First RedrawRequested received
[2026-10-11T08:12:14Z DEBUG phosphlux] First render_with_egui success (submit and present returned)
[2026-10-11T08:12:15Z DEBUG phosphlux] Startup frames: redraw=35 render=35 present=35
[2026-10-11T08:12:16Z DEBUG phosphlux] Startup frames: redraw=96 render=96 present=96
[2026-10-11T08:12:17Z DEBUG phosphlux] Startup frames: redraw=157 render=157 present=157
[2026-10-11T08:12:18Z DEBUG phosphlux] Startup frames: redraw=217 render=217 present=217
[2026-10-11T08:12:19Z DEBUG phosphlux] Startup frames: redraw=277 render=277 present=277
```

## Checks and limits

`./check.sh` ran after the final code change and exited **0**. Actual output:

```text
Phosphlux check (cargo build/test/clippy --locked)
build: PASS (exit 0, 112 warnings)
test: PASS (exit 0, 106 warnings)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
clippy: PASS (exit 0, 152 warnings)
Logs: /tmp/phosphlux-check/run.ovD0WI
Overall: PASS
```

`git diff --check` exited **0** with no output. No new tests were written: bounded runtime comparison directly tests the scheduling behavior. The third launch used the final changed code without further edits.

Existing modified `Cargo.toml` / `Cargo.lock` and untracked `BASELINE.md`, `BLOCK_COMPATIBILITY_VALIDATION.md`, `research/schematics/`, and `vendor/` were present before this task and were excluded from its commit. Builds used that existing dependency working tree. No dependency, shader, asset or feature changes were made by this worker. No merge, push, installation, screenshot or screen recording occurred. Build output remained under `/tmp`.

## Remaining check

No available tool in this session provides direct visual inspection without a screenshot, and the task prohibits screenshots. Continuous successful presentation establishes the scheduling fix, but does not establish that the running window visibly contains the node editor and default colored radial preview. That appearance check remains for Hedawn or an authorized observer; it must not be inferred from passing checks or source expectations.

The existing double-click launcher is `/Users/donovan/work/studio-operations/launchers/Open Phosphlux.command`. A proposed follow-up asks Hedawn to observe the default window, close it, and record the result on the board. All three launches allowed to this worker are exhausted. No further app launches were attempted.

## Scope amendment

On resumption, the amended task packet moved visual confirmation to T-0072 and required no new launches. The retained runtime logs were reread and compared against the complete output quoted above; all three matched exactly. This resumed run did not repeat builds, tests, launches, or shutdown checks. The code still matches scheduling-fix commit `4d113dd`; only the report wording changed to reflect the amended scope.
