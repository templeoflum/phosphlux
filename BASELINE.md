# Phosphlux baseline — 2026-10-11

Task T-0063. Build/test run at 01:29 CDT: debug build passes; 8 tests pass, 0 fail; Clippy exits successfully with 106 Phosphlux warnings. GUI startup and rendering were not exercised. The 01:34 CDT correction run rechecked repository differences and revised this report; build, tests and Clippy were not rerun.

## 1. Toolchain and platform

The missing-toolchain finding applied to the initial run. In the 01:29 CDT build/test run, the commands below resolved and succeeded after Hedawn's installation. No software was installed by this worker.

Command: `pwd` — exit 0.

```text
/Users/donovan/work/0000000/analog-claude-video-synthesis-foundation-8LfZV
```

Command: `rustc --version` — exit 0.

```text
rustc 1.99.0 (b940084d7 2026-09-28)
```

Command: `cargo --version` — exit 0.

```text
cargo 1.99.0 (5f94df478 2026-08-27)
```

Command: `rustup component list --installed` — exit 0.

```text
cargo-aarch64-apple-darwin
clippy-aarch64-apple-darwin
rust-docs-aarch64-apple-darwin
rust-std-aarch64-apple-darwin
rustc-aarch64-apple-darwin
rustfmt-aarch64-apple-darwin
```

Command: `cargo clippy --version` — exit 0.

```text
clippy 0.1.99 (b940084d7e 2026-09-28)
```

Command: `sw_vers` — exit 0.

```text
ProductName:		macOS
ProductVersion:		26.5.1
BuildVersion:		25F80
```

Command: `system_profiler SPDisplaysDataType` — exit 0.

```text
Graphics/Displays:

    Apple M5 Max:

      Chipset Model: Apple M5 Max
      Type: GPU
      Bus: Built-In
      Total Number of Cores: 40
      Vendor: Apple (0x106b)
      Metal Support: Metal 4
      Displays:
        Color LCD:
          Display Type: Built-in Liquid Retina XDR Display
          Resolution: 3456 x 2234 Retina
          Main Display: Yes
          Mirror: Off
          Online: Yes
          Automatically Adjust Brightness: Yes
          Connection Type: Internal

```

README requirements (claimed): Rust 1.75 or later; GPU with Vulkan, Metal or DX12 support; Windows, macOS or Linux. No additional command-line build prerequisite is named. The Mac reports Metal 4 support; actual wgpu device initialization was not tested.

## 2. Git state

Captured in the 01:29 CDT build/test run before builds and report revision. Rechecked in the 01:34 CDT correction run: the 227 tracked modifications are line-ending differences only (CRLF in the working tree, LF in HEAD), with no content changes. Existing files were left untouched.

Command: `git branch --show-current` — exit 0.

```text
master
```

Command: `git log -1 --format='%H%n%aI%n%cI%n%s'` — exit 0.

```text
86fe8c581438a22ab74881838f0f1ab892489c2d
2026-01-30T00:46:13-06:00
2026-01-30T00:46:13-06:00
Remove phosphlux-lite from workspace (now separate repo)
```

Command: `git remote -v` — exit 0.

```text
origin	git@github.com:templeoflum/phosphlux.git (fetch)
origin	git@github.com:templeoflum/phosphlux.git (push)
```

Command: `git status --short` — exit 0.

```text
 M .gitignore
 M .mcp.json
 M CLAUDE_START_PROMPT.md
 M DEVELOPMENT_PROTOCOL.md
 M DEVLOG.md
 M PLAN_authentic_paik_abe.md
 M PLAN_ui_overhaul.md
 M README.md
 M apps/phosphlux/src/core/component.rs
 M apps/phosphlux/src/core/mod.rs
 M apps/phosphlux/src/core/signal.rs
 M apps/phosphlux/src/devices/generators.rs
 M apps/phosphlux/src/devices/mod.rs
 M apps/phosphlux/src/devices/paik_abe/colorizer.wgsl
 M apps/phosphlux/src/devices/paik_abe/folding_amp.wgsl
 M apps/phosphlux/src/devices/paik_abe/input_matrix.wgsl
 M apps/phosphlux/src/devices/paik_abe/mod.rs
 M apps/phosphlux/src/devices/paik_abe/output_encoder.wgsl
 M apps/phosphlux/src/devices/paik_abe/ped_colorizer.wgsl
 M apps/phosphlux/src/gpu/mod.rs
 M apps/phosphlux/src/gpu/renderer.rs
 M apps/phosphlux/src/gpu/shader_composer.rs
 M apps/phosphlux/src/main.rs
 M apps/phosphlux/src/nodes/exported/PaikAbeColorizer.json
 M apps/phosphlux/src/shaders/cables.wgsl
 M apps/phosphlux/src/shaders/common.wgsl
 M apps/phosphlux/src/shaders/components.wgsl
 M apps/phosphlux/src/ui/device_registry.rs
 M apps/phosphlux/src/ui/mod.rs
 M apps/phosphlux/src/ui/node_editor.rs
 M apps/phosphlux/src/ui/theme.rs
 M archive/swarm-agent-concept/demo_component_workflow.py
 M archive/swarm-agent-concept/docs/dev-logs/20260125_034318_Emulate_the_Paik_Abe_colorizer.md
 M archive/swarm-agent-concept/docs/dev-logs/20260125_040908_Research_the_Paik_Abe_colorize.md
 M archive/swarm-agent-concept/docs/dev-logs/20260125_082544_Compile_all_missing_research_d.md
 M archive/swarm-agent-concept/docs/dev-logs/20260125_082919_Research_Paik_Abe_Video_Synthe.md
 M archive/swarm-agent-concept/docs/dev-logs/20260125_083047_Research_Paik_Abe_Video_Synthe.md
 M archive/swarm-agent-concept/docs/dev-logs/20260125_083603_Research_Paik_Abe_Video_Synthe.md
 M archive/swarm-agent-concept/docs/dev-logs/20260125_092714_Research_Paik_Abe_Video_Synthe.md
 M archive/swarm-agent-concept/docs/dev-logs/20260125_092750_Verify_Paik_Abe_component_mode.md
 M archive/swarm-agent-concept/docs/dev-logs/20260125_092915_Verify_Paik_Abe_Video_Synthesi.md
 M archive/swarm-agent-concept/requirements.txt
 M archive/swarm-agent-concept/run_benchmark.py
 M archive/swarm-agent-concept/run_swarm.py
 M archive/swarm-agent-concept/swarm/__init__.py
 M archive/swarm-agent-concept/swarm/base_agent.py
 M archive/swarm-agent-concept/swarm/component_agent.py
 M archive/swarm-agent-concept/swarm/development_agent.py
 M archive/swarm-agent-concept/swarm/documentation_agent.py
 M archive/swarm-agent-concept/swarm/event_bus.py
 M archive/swarm-agent-concept/swarm/model_benchmark.py
 M archive/swarm-agent-concept/swarm/orchestrator.py
 M archive/swarm-agent-concept/swarm/research_agent.py
 M archive/swarm-agent-concept/swarm/shared_state.py
 M archive/swarm-agent-concept/swarm/token_tracker.py
 M archive/swarm-agent-concept/tests/test_swarm.py
 M archive/v1/README.md
 M archive/v1/src/core/mod.rs
 M archive/v1/src/core/signal.rs
 M archive/v1/src/core/timing.rs
 M archive/v1/src/gpu/graph_compiler.rs
 M archive/v1/src/gpu/mod.rs
 M archive/v1/src/gpu/renderer.rs
 M archive/v1/src/main.rs
 M archive/v1/src/shaders/main.wgsl
 M archive/v1/src/ui/mod.rs
 M archive/v1/src/ui/node_editor.rs
 M archive/v1/src/ui/patch.rs
 M docs/CABLE_DEGRADATION_SPEC.md
 M docs/DEVELOPMENT_ROADMAP.md
 M docs/EMULATION_FIDELITY.md
 M docs/SESSION_HANDOFF_v0.15.md
 M docs/SESSION_HANDOFF_v0.16.md
 M docs/V1_EMULATION_ROADMAP.md
 M docs/WORKBENCH_WORKFLOW.md
 M docs/architecture/swarm-agent-specification.md
 M docs/dev-logs/vision-guided-development.md
 M docs/dev-logs/workbench-mcp-template.md
 M docs/research/paik-abe-colorizer-analysis.md
 M docs/screenshots/ca3019/ca3019_ped_control_colorbars_20260125_074158.json
 M docs/screenshots/ca3019/ca3019_ped_control_ramp_20260125_074148.json
 M docs/screenshots/ca3046/ca3046_colorizer_colorbars_20260125_074043.json
 M docs/screenshots/ca3046/ca3046_colorizer_ramp_20260125_074032.json
 M docs/screenshots/ca3054/ca3054_keyer_colorbars_20260125_074415.json
 M docs/screenshots/ca3054/ca3054_matrix_colorbars_20260125_074312.json
 M docs/screenshots/ca3054/ca3054_matrix_rotated_20260125_074322.json
 M docs/screenshots/ca3080/ca3080_vca_colorbars_20260125_074602.json
 M docs/screenshots/ca3080/ca3080_vca_modulated_20260125_074613.json
 M docs/screenshots/colorizer/colorizer_colorbars_20260129_230159.json
 M docs/screenshots/colorizer/colorizer_ramp_20260129_230150.json
 M docs/screenshots/folding_amp/fold_gain2_20260129_230013.json
 M docs/screenshots/folding_amp/fold_gain4_20260129_230022.json
 M docs/screenshots/folding_amp/folding_amp_ramp_3x_20260125_074504.json
 M docs/screenshots/folding_amp/folding_amp_ramp_6x_20260125_074514.json
 M docs/screenshots/lm311/lm311_keyer_window_20260125_074701.json
 M docs/screenshots/output_encoder/output_neutral_20260129_230430.json
 M docs/screenshots/output_encoder/output_warm_20260129_230440.json
 M docs/screenshots/paik-abe/ca3022_colorbars_default_20260126_044435.json
 M docs/screenshots/paik-abe/ca3023_colorbars_default_20260126_044536.json
 M docs/screenshots/paik-abe/folding_amp_default_20260126_034654.json
 M docs/screenshots/paik-abe/matrix_mixer_default_20260126_034424.json
 M docs/screenshots/paik-abe/ped_colorizer_default_20260126_034547.json
 M docs/screenshots/paik-abe/signal_chain_6stage_colorbars_20260126_044742.json
 M docs/screenshots/paik-abe/signal_chain_6stage_ramp_20260126_044725.json
 M docs/screenshots/paik-abe/signal_chain_colorbars_20260126_035333.json
 M docs/screenshots/paik-abe/signal_chain_ramp_20260126_035307.json
 M docs/screenshots/paik-abe/video_keyer_default_20260126_034808.json
 M docs/screenshots/paik-abe/wobbulator_default_20260126_034917.json
 M docs/screenshots/paik_abe/folding_amplifier_20260125_063503.json
 M docs/screenshots/paik_abe/ped_control_colorizer_20260125_061443.json
 M docs/screenshots/paik_abe/video_keyer_20260125_062533.json
 M docs/screenshots/paik_abe_folding_amp/folding_amp_default_ramp_20260129_031627.json
 M docs/screenshots/paik_abe_folding_amp/folding_amp_validated_20260129_032314.json
 M docs/screenshots/paik_abe_input_matrix/7input_default_20260129_222431.json
 M docs/screenshots/paik_abe_input_matrix/blue_isolation_20260129_054746.json
 M docs/screenshots/paik_abe_input_matrix/default_colorbars_20260129_054724.json
 M docs/screenshots/paik_abe_input_matrix/green_isolation_20260129_054739.json
 M docs/screenshots/paik_abe_input_matrix/input_matrix_validated_20260129_032536.json
 M docs/screenshots/paik_abe_input_matrix/multi_input_test_20260129_222525.json
 M docs/screenshots/paik_abe_input_matrix/ramp_default_20260129_054809.json
 M docs/screenshots/paik_abe_input_matrix/ramp_high_gain_20260129_054817.json
 M docs/screenshots/paik_abe_input_matrix/red_isolation_20260129_054731.json
 M docs/screenshots/paik_abe_input_matrix/rotation_180deg_20260129_054759.json
 M docs/screenshots/paik_abe_input_matrix/rotation_90deg_20260129_054752.json
 M docs/screenshots/paik_abe_output_encoder/output_encoder_validated_20260129_032826.json
 M docs/screenshots/paik_abe_ped_colorizer/ped_colorizer_validated_20260129_032701.json
 M docs/screenshots/ped_colorizer/ped_ramp_default_20260129_230312.json
 M docs/screenshots/ped_colorizer/ped_ramp_hard_20260129_230324.json
 M docs/screenshots/wobbulator/wobbulator_colorbars_20260125_074759.json
 M docs/screenshots/wobbulator/wobbulator_intense_20260125_074811.json
 M docs/screenshots/wobbulator/wobbulator_video_mod_20260125_074846.json
 M project_state.json
 M research/DISCOVERY_GUIDE.md
 M research/RESEARCH_INDEX.md
 M research/audits/paik_abe_device_audit_20260129.md
 M research/audits/ped_colorizer_audit_20260129.md
 M research/audits/ped_colorizer_verification_20260129.md
 M research/components/README.md
 M research/components/functional/folding_amp/v1/folder.wgsl
 M research/components/functional/folding_amp/v1/model.wgsl
 M research/components/functional/folding_amp/v1/provenance.json
 M research/components/functional/folding_amp/v1/spec.json
 M research/components/functional/paik_abe_chain/v1/signal_chain.wgsl
 M research/components/functional/wobbulator/v1/model.wgsl
 M research/components/functional/wobbulator/v1/provenance.json
 M research/components/functional/wobbulator/v1/spec.json
 M research/components/functional/wobbulator/v1/wobbulator.wgsl
 M research/components/ics/ca3019/PROVENANCE.md
 M research/components/ics/ca3019/v1/colorizer.wgsl
 M research/components/ics/ca3019/v1/model.wgsl
 M research/components/ics/ca3019/v1/ped_control.wgsl
 M research/components/ics/ca3019/v1/provenance.json
 M research/components/ics/ca3019/v1/spec.json
 M research/components/ics/ca3022/PROVENANCE.md
 M research/components/ics/ca3022/v1/model.wgsl
 M research/components/ics/ca3022/v1/provenance.json
 M research/components/ics/ca3022/v1/spec.json
 M research/components/ics/ca3022/v1/video_amp.wgsl
 M research/components/ics/ca3023/PROVENANCE.md
 M research/components/ics/ca3023/v1/model.wgsl
 M research/components/ics/ca3023/v1/provenance.json
 M research/components/ics/ca3023/v1/spec.json
 M research/components/ics/ca3023/v1/subcarrier_amp.wgsl
 M research/components/ics/ca3046/v1/colorizer.wgsl
 M research/components/ics/ca3046/v1/model.wgsl
 M research/components/ics/ca3046/v1/provenance.json
 M research/components/ics/ca3046/v1/spec.json
 M research/components/ics/ca3054/PROVENANCE.md
 M research/components/ics/ca3054/v1/matrix_mixer.wgsl
 M research/components/ics/ca3054/v1/model.wgsl
 M research/components/ics/ca3054/v1/provenance.json
 M research/components/ics/ca3054/v1/spec.json
 M research/components/ics/ca3054/v1/video_keyer.wgsl
 M research/components/ics/ca3080/v1/model.wgsl
 M research/components/ics/ca3080/v1/provenance.json
 M research/components/ics/ca3080/v1/spec.json
 M research/components/ics/ca3080/v1/vca.wgsl
 M research/components/ics/lm311/v1/keyer.wgsl
 M research/components/ics/lm311/v1/model.wgsl
 M research/components/ics/lm311/v1/provenance.json
 M research/components/ics/lm311/v1/spec.json
 M research/devices/README.md
 M research/devices/paik_abe/architecture.md
 M research/innovators/README.md
 M research/prototypes/README.md
 M research/prototypes/paik_abe_colorizer_v1/spec.json
 M research/prototypes/paik_abe_folding_amp_v1/spec.json
 M research/techniques/README.md
 M research/techniques/cable_degradation/README.md
 M research/validation/device_validation.json
 M scripts/verify_state.py
 M tools/device-workbench/API_SPEC.md
 M tools/device-workbench/generate_test_signals.py
 M tools/device-workbench/mcp_server.py
 M tools/device-workbench/shaders/colorizer.wgsl
 M tools/device-workbench/shaders/folding_amp.wgsl
 M tools/device-workbench/shaders/input_matrix_full.wgsl
 M tools/device-workbench/shaders/input_matrix_standalone.wgsl
 M tools/device-workbench/shaders/output_encoder.wgsl
 M tools/device-workbench/shaders/ped_colorizer.wgsl
 M tools/device-workbench/workbench-app/Cargo.lock
 M tools/device-workbench/workbench-app/Cargo.toml
 M tools/device-workbench/workbench-app/src/api.rs
 M tools/device-workbench/workbench-app/src/diff.rs
 M tools/device-workbench/workbench-app/src/main.rs
 M tools/device-workbench/workbench-app/src/renderer.rs
 M tools/device-workbench/workbench-app/src/state.rs
 M tools/pdf-utils/requirements.txt
 M tools/pdf-utils/split_pdf.py
 M tools/research-orchestrator/CLAUDE_CLI_GUIDE.md
 M tools/research-orchestrator/PERFORMANCE.md
 M tools/research-orchestrator/benchmark.py
 M tools/research-orchestrator/benchmark_results.json
 M tools/research-orchestrator/mcp_server.py
 M tools/research-orchestrator/model_comparison.py
 M tools/research-orchestrator/model_comparison_results.json
 M tools/retrieval/__init__.py
 M tools/retrieval/artifacts.py
 M tools/retrieval/base.py
 M tools/retrieval/components.py
 M tools/retrieval/database.py
 M tools/retrieval/datasheets.py
 M tools/retrieval/manuals.py
 M tools/retrieval/migrate_components.py
 M tools/retrieval/patents.py
 M tools/retrieval/requirements.txt
 M tools/retrieval/schematics.py
?? BASELINE.md
?? research/schematics/
```

The log output contains commit hash, author date, committer date and subject, in that order. BASELINE.md is the report from the earlier run; the other status entries already existed at the start of the build/test run.

Repository-difference checks in the 01:34 CDT correction run (all git commands used `GIT_OPTIONAL_LOCKS=0`):

Command: `git diff --stat` — exit 0. Summary line from real output:

```text
 227 files changed, 45114 insertions(+), 45114 deletions(-)
```

Command: `git diff --ignore-cr-at-eol --stat` — exit 0; output empty.

Command: `git diff --name-only -z` — exit 0; 227 modified tracked paths. A Python byte comparison read each working file and `git show HEAD:<path>` (all 227 commands exited 0). For every path, replacing CRLF with LF in the working bytes exactly matched HEAD bytes: 227 working copies contain CRLF, 227 HEAD copies contain no CRLF, zero content differences. The comparison script exited 0.

Command: `git status --short` — exit 0; 229 lines, including 227 tracked modifications, `?? BASELINE.md` and `?? research/schematics/`. It matched the earlier recorded status.

Command: `rg --files research/schematics` — exit 0:

```text
research/schematics/rutt-etra/schematic.pdf
research/schematics/rutt-etra/metadata.json
```

Command: `git check-ignore target` — exit 0; output `target`. A Python directory listing (exit 0) found existing `target/release`, `target/doc` and `target/debug`; their age and contents were not surveyed in this correction run. They predate this correction and were not generated by its checks.

## 3. Layout and source claims

`cat README.md` exited 0. Claimed: real-time GPU analog video synthesis and CRT emulation using Rust/wgpu; visual node editor, Paik/Abe devices, multi-pass shader composition and 60 FPS rendering. Those behavioral/performance claims were not checked by a live app run.

`cat Cargo.toml apps/phosphlux/Cargo.toml tools/device-workbench/workbench-app/Cargo.toml` exited 0. Observed: one active workspace member, `apps/phosphlux`, package `phosphlux` 0.1.0 with edition 2024. Workspace dependencies include wgpu 23, winit 0.30 and egui 0.30. Device workbench is a separate manifest for `device-workbench` 0.1.0, edition 2021, outside the root workspace member list; it was not built or tested by these root commands.

Command: `CARGO_TARGET_DIR=/tmp/phosphlux-baseline cargo metadata --no-deps --locked --format-version 1` — exit 0. Returned one package and one binary target: `phosphlux`, source `apps/phosphlux/src/main.rs`, edition 2024, test enabled and doctest disabled. Raw JSON is in `/tmp/phosphlux-baseline/evidence.json`.

Command: `rg --files -g AGENTS.md -g CLAUDE.md -g '!archive/**' -g '!__archive_typescript_reference/**'` — exit 1.

```text
```

Command: `rg --files -g Cargo.toml -g Cargo.lock -g package.json -g requirements.txt -g pyproject.toml -g "!archive/**" -g "!__archive_typescript_reference/**"` — exit 0.

```text
apps/phosphlux/Cargo.toml
Cargo.toml
Cargo.lock
tools/retrieval/requirements.txt
tools/device-workbench/workbench-app/Cargo.lock
tools/device-workbench/workbench-app/Cargo.toml
tools/pdf-utils/requirements.txt
tools/research-orchestrator/requirements.txt
```

Command: `rg --files apps/phosphlux/src | xargs wc -lc` — exit 0.

```text
     129    3661 apps/phosphlux/src/devices/generators.rs
      30     980 apps/phosphlux/src/devices/mod.rs
     464   13579 apps/phosphlux/src/devices/paik_abe/mod.rs
     118    4357 apps/phosphlux/src/devices/paik_abe/input_matrix.wgsl
      84    3069 apps/phosphlux/src/devices/paik_abe/folding_amp.wgsl
     141    5517 apps/phosphlux/src/devices/paik_abe/ped_colorizer.wgsl
      75    2726 apps/phosphlux/src/devices/paik_abe/output_encoder.wgsl
     106    3821 apps/phosphlux/src/devices/paik_abe/colorizer.wgsl
     442   15531 apps/phosphlux/src/main.rs
     352   10445 apps/phosphlux/src/shaders/components.wgsl
     249   11509 apps/phosphlux/src/shaders/cables.wgsl
     222    6404 apps/phosphlux/src/shaders/common.wgsl
      11    3459 apps/phosphlux/src/nodes/exported/PaikAbeColorizer.json
     174    4997 apps/phosphlux/src/core/timing.rs
     442   13419 apps/phosphlux/src/core/component.rs
      14     595 apps/phosphlux/src/core/mod.rs
     326   10118 apps/phosphlux/src/core/signal.rs
    1558   57678 apps/phosphlux/src/gpu/shader_composer.rs
      11     394 apps/phosphlux/src/gpu/mod.rs
    1107   43698 apps/phosphlux/src/gpu/renderer.rs
    1366   57743 apps/phosphlux/src/ui/node_editor.rs
     128    4924 apps/phosphlux/src/ui/device_registry.rs
       7     140 apps/phosphlux/src/ui/mod.rs
     175    6955 apps/phosphlux/src/ui/theme.rs
    7731  285719 total
```

Command: `rg --files tools/device-workbench/workbench-app/src | xargs wc -lc` — exit 0.

```text
     402   11264 tools/device-workbench/workbench-app/src/api.rs
     159    5164 tools/device-workbench/workbench-app/src/diff.rs
     144    4591 tools/device-workbench/workbench-app/src/main.rs
     206    5930 tools/device-workbench/workbench-app/src/state.rs
     739   28801 tools/device-workbench/workbench-app/src/renderer.rs
    1650   55750 total
```

Command: `rg -n '^fn main|ApplicationHandler|EventLoop|Renderer::new|pollster' apps/phosphlux/src/main.rs` — exit 0.

```text
21:    application::ApplicationHandler,
23:    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
94:impl ApplicationHandler for App {
95:    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
104:            let mut renderer = pollster::block_on(Renderer::new(Arc::clone(&window)));
116:            let mut egui_renderer = egui_wgpu::Renderer::new(
144:        event_loop: &ActiveEventLoop,
195:    fn render_frame(&mut self, event_loop: &ActiveEventLoop) {
433:fn main() {
437:    let event_loop = EventLoop::new().unwrap();
```

No AGENTS.md or CLAUDE.md was found by the search above; README was read for build/run requirements. The main app has 24 source/asset files (7,731 newline-delimited lines, 285,719 bytes); workbench has 5 source files (1,650 lines, 55,750 bytes). Counts include WGSL/JSON under the app source tree. Main initializes a winit event loop and GPU/egui renderers. Supporting folders include docs, research, scripts and Python tools; they were not executed.

`archive/v1` was encountered in git status only, and the last commit names phosphlux-lite as a separate repo. Neither was surveyed. The TypeScript reference archive was also excluded from the layout search.

## 4. Build

README documents `cargo build` and `cargo build --release` (release recommended). The debug command was selected. `--locked` was added to prevent Cargo.lock modification; CARGO_TARGET_DIR keeps generated files out of the repository. Release build was not run.

Command: `CARGO_TARGET_DIR=/tmp/phosphlux-baseline cargo build --locked` — exit 0. Full output: `/tmp/phosphlux-baseline/build.log`.

```text
warning: `phosphlux` (bin "phosphlux") generated 66 warnings (run `cargo fix --bin "phosphlux" -p phosphlux` to apply 26 suggestions)
    Finished `dev` profile [optimized + debuginfo] target(s) in 28.71s
warning: the following packages contain code that will be rejected by a future version of Rust: block v0.1.6
note: to see what the problems were, use the option `--future-incompat-report`, or run `cargo report future-incompatibilities --id 1`
```

## 5. Tests

Eight unit tests exist in the active app: three analog-component tests, three signal tests, and two shader-composition tests. They exercise op-amp saturation, RC response, tolerance, voltage normalization, impedance loading, YIQ roundtrip, simple shader composition and a single render-plan segment. No GUI/GPU rendering or performance behavior is exercised by these tests. The separate workbench has a test marker in diff.rs, but that crate and Python/research tooling were not tested.

Command: `CARGO_TARGET_DIR=/tmp/phosphlux-baseline cargo test --locked` — exit 0. Full output: `/tmp/phosphlux-baseline/test.log`.

```text
warning: `phosphlux` (bin "phosphlux" test) generated 60 warnings (run `cargo fix --bin "phosphlux" -p phosphlux --tests` to apply 26 suggestions)
    Finished `test` profile [optimized + debuginfo] target(s) in 1.10s
warning: the following packages contain code that will be rejected by a future version of Rust: block v0.1.6
note: to see what the problems were, use the option `--future-incompat-report`, or run `cargo report future-incompatibilities --id 1`
     Running unittests src/main.rs (/tmp/phosphlux-baseline/debug/deps/phosphlux-de0bc5fd6f5baa3a)

running 8 tests
test core::component::tests::test_opamp_saturation ... ok
test core::component::tests::test_rc_lowpass ... ok
test core::component::tests::test_tolerance_variation ... ok
test core::signal::tests::test_loading_effect ... ok
test core::signal::tests::test_voltage_range_normalize ... ok
test core::signal::tests::test_yiq_roundtrip ... ok
test gpu::shader_composer::tests::test_compose_plan_single_segment ... ok
test gpu::shader_composer::tests::test_compose_simple ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

## 6. Lint

Clippy was already installed. The default active binary target was checked, without `--fix` or warning-as-error flags. Cargo reports **106 Phosphlux warnings**; this is the compiler summary count, including Rust warnings emitted during Clippy. The separate `block` future-compatibility notice is not counted as a Phosphlux lint warning.

Command: `CARGO_TARGET_DIR=/tmp/phosphlux-baseline cargo clippy --locked` — exit 0. Full output: `/tmp/phosphlux-baseline/clippy.log`.

```text
warning: `phosphlux` (bin "phosphlux") generated 106 warnings (run `cargo clippy --fix --bin "phosphlux" -p phosphlux -- ` to apply 59 suggestions)
    Finished `dev` profile [optimized + debuginfo] target(s) in 6.22s
warning: the following packages contain code that will be rejected by a future version of Rust: block v0.1.6
note: to see what the problems were, use the option `--future-incompat-report`, or run `cargo report future-incompatibilities --id 1`
```

Warning headlines (real output; repeated headlines retained):

```text
warning: unused imports: `Add`, `Mul`, and `Sub`
warning: unused imports: `BendPoint`, `BendType`, `BjTransistor`, `Diode`, `OpAmp`, `RcFilter`, and `Tolerance`
warning: unused imports: `AnalogSignal`, `Impedance`, `ProcessingRate`, `RgbSignal`, `VoltageRange`, and `YiqSignal`
warning: unused imports: `SyncGenerator`, `TimingUniforms`, and `VideoStandard`
warning: unused imports: `Connection`, `ParamLayout`, `RenderPlan`, `SegmentKind`, and `SegmentShader`
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
warning: struct `OpAmp` is never constructed
warning: multiple associated items are never used
warning: struct `BjTransistor` is never constructed
warning: associated items `n2n3904`, `n2n3906`, and `collector_current` are never used
warning: struct `Diode` is never constructed
warning: associated items `n1n4148`, `led_red`, and `forward_current` are never used
warning: struct `RcFilter` is never constructed
warning: associated items `new`, `tau`, `f_cutoff`, `lowpass`, `highpass`, and `reset` are never used
warning: struct `Tolerance` is never constructed
warning: associated items `resistor_5pct`, `capacitor_10pct`, `actual_value`, and `at_temperature` are never used
warning: enum `BendType` is never used
warning: struct `BendPoint` is never constructed
warning: associated functions `removable` and `body_contact` are never used
warning: enum `VoltageRange` is never used
warning: methods `min`, `max`, `normalize`, and `denormalize` are never used
warning: struct `Impedance` is never constructed
warning: associated items `VIDEO`, `HIGH_Z`, `LOW_Z_OUT`, and `loading_factor` are never used
warning: struct `AnalogSignal` is never constructed
warning: associated items `video`, `cv_bipolar`, `cv_unipolar`, `normalized`, `loaded_by`, and `soft_clip` are never used
warning: struct `RgbSignal` is never constructed
warning: associated items `from_rgb`, `from_luma`, `to_luma`, `as_tuple`, and `soft_clip` are never used
warning: struct `YiqSignal` is never constructed
warning: associated items `from_rgb`, `to_rgb`, `rotate_hue`, `chroma`, and `hue` are never used
warning: enum `ProcessingRate` is never used
warning: methods `hz_ntsc` and `dt` are never used
warning: fields `fallback_params_buffer`, `empty_bg_layout`, `texture_input_bg_layout`, and `black_bind_group` are never read
warning: methods `needs_feedback`, `needs_state`, and `state_struct` are never used
warning: field `signal_type` is never read
warning: struct `InputBinding` is never constructed
warning: variants `Mono` and `Cv` are never constructed
warning: fields `needs_input_texture` and `device_instance_ids` are never read
warning: fields `wgsl_source`, `param_layout`, `texture_bindings`, and `state_bindings` are never read
warning: fields `device_id` and `binding_type` are never read
warning: variants `Feedback` and `Input` are never constructed
warning: fields `device_id`, `struct_name`, and `size` are never read
warning: field `cache` is never read
warning: multiple methods are never used
warning: variant `InvalidConnection` is never constructed
warning: variants `NeedsValidation` and `Legacy` are never constructed
warning: fields `panel_front`, `case_body`, and `text_disabled` are never read
warning: name `TTL` contains a capitalized acronym
warning: methods with the following characteristics: (`to_*` and `self` type is `Copy`) usually take `self` by value
warning: methods with the following characteristics: (`to_*` and `self` type is `Copy`) usually take `self` by value
warning: all variants have the same prefix: `Per`
warning: name `NTSC` contains a capitalized acronym
warning: name `PAL` contains a capitalized acronym
warning: manual implementation of `.is_multiple_of()`
warning: manual arithmetic check found
warning: calling `push_str()` using a single-character string literal
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: calling `push_str()` using a single-character string literal
warning: calling `push_str()` using a single-character string literal
warning: manual implementation of `.is_multiple_of()`
warning: manual implementation of `.is_multiple_of()`
warning: this `if` statement can be collapsed
warning: calling `push_str()` using a single-character string literal
warning: calling `push_str()` using a single-character string literal
warning: the loop variable `i` is used to index `in_degree`
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: this `if` statement can be collapsed
warning: this `if` can be collapsed into the outer `match`
warning: this `if` statement can be collapsed
```

## 7. Startup

README documents `RUST_LOG=info cargo run --release`. From this repository root, Hedawn can run it with the task's output-directory constraint:

```sh
CARGO_TARGET_DIR=/tmp/phosphlux-baseline RUST_LOG=info cargo run --release --locked
```

Command: `cat run.bat run-debug.bat` — exit 0 in the correction run. The Windows launchers change to their script directory and invoke `cargo run --release` and `cargo run`, respectively, followed by `pause`. They were read, not executed.

The startup command above was not executed. No app window was opened or app process left running. The successful debug build produced `/tmp/phosphlux-baseline/debug/phosphlux`; release executable generation and app startup remain unchecked.

## 8. Broken, missing or unclear

- **Observed:** debug build emits 66 Phosphlux warnings; test compilation emits 60; Clippy emits 106. Diagnostics include unused imports/code, collapsible conditionals and other lint findings. None was fixed.
- **Observed:** Cargo reports `block v0.1.6` contains code that a future Rust version will reject. This did not fail the 01:29 CDT build/tests/lint; the detailed future-incompatibility report was not requested.
- **Observed (correction run):** 227 tracked files show as modified, but the only difference is line endings (CRLF in the working tree, LF in HEAD); no content changes. Untracked: `research/schematics/` contains rutt-etra research files, and `BASELINE.md` is this task output. The earlier description of substantial modifications of unknown correctness was too broad and has been corrected by the comparisons in §2.
- **Observed:** workspace edition is 2024. **Claimed:** README minimum Rust is 1.75. Compatibility with that claimed minimum was not tested; only installed Rust 1.99.0 was used.
- **Claimed:** README marks CRT monitor, camcorder/feedback, patch save/load, additional device families, MIDI/OSC and video input as pending. Source implementation completeness was not verified.
- **Observed:** eight active-workspace unit tests pass; they do not establish runtime shader validation, GUI interactions, visual fidelity or 60 FPS. These behaviors remain unchecked under the no-launch constraint.
- **Observed:** root commands cover Phosphlux only. Separate device-workbench and Python/research tools have no build/test result from this run.

## Completion checks

Tool versions/platform, git evidence, package/entry/size inventory, documented debug build result, test names/counts, installed Clippy warning count, unexecuted startup command and observed/claimed issues are recorded above. Exact command exit codes or output are included. Raw command capture and build/test/lint logs are under `/tmp/phosphlux-baseline`.

Existing-file SHA-256 comparisons excluded BASELINE.md and covered all files returned by `git ls-files -co --exclude-standard -z` at the start of the 01:29 CDT build/test run. No existing-file hash changed through the build/test/lint steps. The correction run also compares existing-file SHA-256 values and git status after report revision; it does not rerun the build/test/lint commands. No fixes, commits, branch changes or installations were performed.
