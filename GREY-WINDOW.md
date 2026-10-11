# Phosphlux grey window

T-0070 · 11 October 2026 · source inspection and two bounded launches.

**Verdict: failure to show the intended startup content, not a designed empty state; the leading cause is an unstarted redraw loop, but presentation counts and the exact cause remain unproved.**

## Startup picture specified by the code

Paths below are relative to the repository root. Source was read with `nl -ba` and `rg -n`; those source-reading commands exited 0 unless noted below.

- `apps/phosphlux/src/main.rs:56` initializes `App` with `default_patch()`. Lines 72–90 add TestSignalGenerator (instance 0) and PaikAbeColorizer (instance 1), connect `video_out` to `video_in`, and designate the colorizer as output. `main.rs:433–441` has no argument parsing or saved-state loading: both launches used this unconditional initial patch. No saved-state file was read or modified by this worker.
- `apps/phosphlux/src/gpu/shader_composer.rs:246–254` copies each declared parameter's default into the device instance. `generators.rs:110–115` defaults to **mode 4, frequency 4 Hz**, with no input ports (118–120). Mode 4 is a radial grayscale gradient: `clamp(1 - 2*length(uv - 0.5), 0, 1)` (`generators.rs:85–89`). The generator's constant 50% grey fallback at 104–105 applies only outside supported modes, not to mode 4.
- `devices/paik_abe/mod.rs:324–336` loads `colorizer.wgsl` with red/green/blue gains **2 / 2.5 / 3**, fold depth **2**, saturation **1**, slew **0.3**. That shader extracts input luminance, folds each channel separately, applies saturation and a warmth bias, and returns RGB (`colorizer.wgsl:70–105`). This specifies spatially varying colored radial bands, not a uniform grey frame. The default cable is Ideal (`shader_composer.rs:177–181, 273–283`), so it adds no degradation (`824`).
- `main.rs:130–131` compiles the initial patch. A redraw ticks NTSC timing and calls `render_frame` (163–168). Rendering writes the synth texture (206–208), builds the menu/palette/graph/preview UI (232–240; layout at 297–430), and calls `render_with_egui` (265–282). The composer returns the designated output signal (`shader_composer.rs:925–934`). The GPU renderer draws the segment to the output texture (`renderer.rs:770–806`), clears the surface to dark grey `(0.08, 0.08, 0.09, 1)` (1030–1051), paints egui over it (1075–1093), submits and presents (1101–1103). That grey clear is a background, not the completed startup picture. Even an empty render plan uses color bars (`renderer.rs:49–61, 746–768`).

Thus the first completed startup frames should contain the node editor and a colored radial preview. This is a source-derived expectation; no pixels were captured or visually inspected in this run.

## Live startup evidence

Build command (repository root):

```sh
CARGO_TARGET_DIR=/tmp/phosphlux-baseline cargo build --release --locked
```

Exit **0**. Output tail captured this run:

```text
warning: `phosphlux` (bin "phosphlux") generated 66 warnings (run `cargo fix --bin "phosphlux" -p phosphlux` to apply 26 suggestions)
    Finished `release` profile [optimized] target(s) in 0.11s
warning: the following packages contain code that will be rejected by a future version of Rust: block v0.1.6
note: to see what the problems were, use the option `--future-incompat-report`, or run `cargo report future-incompatibilities --id 1`
```

Full build output: `/tmp/T-0070-build.log`. The dependency warning was not investigated or changed. `BASELINE.md` section 7 and `check.sh` were read, not executed; the launcher's documented Cargo run was read, not run. Building separately and directly launching its release executable excludes compilation time from the runtime limit and keeps build output under `/tmp`.

First launch: Python `subprocess.Popen(['/tmp/phosphlux-baseline/release/phosphlux'], cwd=repository, env=environment plus RUST_LOG=info and RUST_BACKTRACE=1)`, no arguments, stdout/stderr captured together. The worker waited 15 seconds, sent SIGTERM, and reaped the process. Wrapper exit **0**; actual app exit **-15**, elapsed **15.016 seconds**, PID **32215**, `Process reaped: True`. It did not exit on its own or report a panic. Complete app output:

```text
[2026-10-11T07:54:59Z INFO  phosphlux] Starting Phosphlux v2
[2026-10-11T07:54:59Z INFO  wgpu_core::instance] Adapter AdapterInfo { name: "Apple M5 Max", vendor: 0, device: 0, device_type: IntegratedGpu, driver: "", driver_info: "", backend: Metal }
[2026-10-11T07:54:59Z INFO  phosphlux::gpu::renderer] GPU: Apple M5 Max
[2026-10-11T07:54:59Z WARN  egui_wgpu::renderer] Detected a linear (sRGBA aware) framebuffer Bgra8UnormSrgb. egui prefers Rgba8Unorm or Bgra8Unorm
[2026-10-11T07:54:59Z INFO  phosphlux::gpu::shader_composer] Render plan: 2 reachable devices out of 2 total (output=1)
[2026-10-11T07:54:59Z INFO  phosphlux::gpu::shader_composer] Render plan: 1 segments (0 scan modulators)
[2026-10-11T07:55:00Z INFO  phosphlux] Window created with node editor layout
```

Observed adapter **Apple M5 Max**, backend **Metal**. One runtime warning: egui prefers a non-sRGB framebuffer over the chosen **Bgra8UnormSrgb**. No runtime error was logged. This does not establish that the warning causes the blank window. The successful render-plan logs establish two reachable devices and one segment, not presented frames.

Second launch: the same executable, no arguments, `RUST_LOG=info`; PID **32247**. After two seconds the worker ran:

```sh
/usr/bin/sample 32247 2 1 -file /tmp/T-0070-process-sample.txt
```

Sample command exit **0**, real output:

```text
Sampling process 32247 for 2 seconds with 1 millisecond of run time between samples
Sampling completed, processing symbols...
Sample analysis of process 32247 written to file /tmp/T-0070-process-sample.txt
```

Wrapper exit **0**; app terminated by SIGTERM and reaped, actual exit **-15**, elapsed **4.575 seconds**. Second startup output (`/tmp/T-0070-sample-startup.log`) repeated the same adapter, warning, render plan and window-created messages, dated 07:55:38–07:55:39 UTC, with no further messages or errors.

Sample excerpt:

```text
    1662 Thread_1771219   DispatchQueue_1: com.apple.main-thread  (serial)
    + !       1661 -[NSApplication run]  (in AppKit) + 368
    + !         1661 -[NSApplication(NSEventRouting) nextEventMatchingMask:untilDate:inMode:dequeue:]  (in AppKit) + 72
```

A Python string-count check of the entire sample exited 0:

```text
process sample occurrences render_frame: 0
process sample occurrences render_with_egui: 0
process sample occurrences render_to_texture: 0
process sample occurrences present: 0
process sample occurrences RedrawRequested: 0
process sample occurrences wgpu: 0
```

**Frame presentation: no presentation was observed in the logs or sample; continuous rendering appears inactive during the sample.** Info logging has no per-frame message, and a release stack sample can miss or inline calls, so these observations do not prove zero frames were ever presented. No GPU frame counter, breakpoint, screen recording or screenshot was used. Hedawn's prior grey-window observation is context, not a visual observation by this worker.

After both launches, `ps -p 32215,32247 -o pid=,comm=` produced no output and exited **1**: neither launched process remained. Ending each process closed its app window; no UI close action was performed. Exactly two launches were made, each under 30 seconds.

## README expectation (claimed)

`README.md:87` says: **“The application uses a 3-panel layout:”**. Its Interface section (85–101) describes the node palette and inspector on the left, node graph in the center, and video preview on the right. These are README claims. The README supplies no explicit first-launch pattern or empty-state description. Its patch save/load feature remains listed as pending (302). The initial radial/colorizer patch is established by code, not by that README claim.

## Smallest candidate action or change

No patch-loading action is needed: a picture-producing patch is already constructed. File → Default Patch recreates it (`main.rs:306–309`), but cannot help if the UI never renders.

The smallest code candidate is **one `window.request_redraw()` after startup initialization**, to seed the existing redraw chain. `rg -n 'request_redraw|about_to_wait|RedrawRequested' apps/phosphlux/src` exited 0 and found only:

```text
apps/phosphlux/src/main.rs:163:            WindowEvent::RedrawRequested => {
apps/phosphlux/src/main.rs:168:                    window.request_redraw();
```

There is no explicit startup redraw request or `about_to_wait` handler. `ControlFlow::Poll` (438) polls events but does not itself render. This, together with the event-loop sample, makes redraw scheduling the leading hypothesis. Inspection of installed egui-winit 0.30.0 also shows RedrawRequested is returned with `consumed: false`, so the early return at `main.rs:152–154` is not evidence that egui swallows redraw events. Native winit can also originate redraws; therefore the absence of an explicit seed is not by itself a proven cause. No candidate change or menu action was made.

## What follows

Propose one bounded task: **make startup redraw reliably and verify the default picture on this Mac**. First establish redraw/render/present counts, test the one-line startup redraw candidate, and keep only a fix supported by those observations. Completion requires visible node editor plus the default colored radial preview after a no-argument launch, continuing frame presentation, and shutdown with no runtime errors. Confine changes to startup redraw scheduling and diagnostic evidence; leave dependency warnings, device shaders and new features out of scope. If the candidate fails, record the measured cause rather than claiming a fix.

## Completion checks

The report covers the default patch and parameter inputs, intended first completed frame, live adapter/backend/warning output, presentation uncertainty, README claims, verdict, smallest candidate change, and one next task. Two launches were bounded and reaped. Only this report was added on `task/T-0070-grey-window`; no code, configuration, assets, merges or pushes. Existing untracked `BASELINE.md` and `research/schematics/` were left untouched. Build/test success is not used as proof of the visual result; tests and Clippy were not rerun for this report-only task. Visual appearance and the redraw fix remain for the follow-up and Hedawn's judgment.
