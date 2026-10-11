# PED Colorizer integration boundary

Task T-0074 · 11 October 2026 · Decision: proposed

## Decision

Keep the existing `paik_abe_ped_colorizer` Rust device and its five semantic parameters as the integration target. A workbench export is a standalone shader snapshot, not an app node registration. First make the workbench capable of setting all five controls and reject unsupported parameter requests. Defer a general export importer, additional ports, and changes to the component demo's behavior.

The T-0073 survey identified this boundary (`RND-PIPELINE.md:71-73,120`). The source comparisons below were checked against the current checkout, based on commit `1b778c5c069dc789189b39014e8645bd09cfccd3`, rather than treating the survey as runtime proof.

## Three shader forms

| Form | Entry points and resources | Behavior and adaptation |
|---|---|---|
| Component demonstration: `research/components/ics/ca3019/v1/ped_control.wgsl` | Full standalone module: `fs_main` at lines 196–225, `vs_main` at 231–242; `VertexOutput` at 33–36. Group 0: texture binding 0, sampler 1, uniform 2 (38–40). Sixteen f32 slots in `Params` (20–31). | Active windows use smoothstep (61–65,152–155); six palettes (76–125), original RGB/colorized blend (220–224). Zero values substitute defaults (206–211), so zero softness or zero blend cannot be represented. There is no active clipping operation. A separate chroma helper exists (169–190) but `fs_main` does not call it. This is an earlier behavior variant, not an interchangeable source for the current node. |
| Workbench test: `tools/device-workbench/shaders/ped_colorizer.wgsl` | Full standalone module: `vs_main` (85–101), `fs_main` (103–137). Group 0: seven textures at bindings 0–6, sampler 7, uniform 8 (39–49). Sixteen f32 slots (20–37). Only `input0` is sampled (105). | Sigmoid diode detector, sharpness 40 (51–65), fixed blue/magenta/yellow/cyan colors (69–76), clipping (80–82), threshold sorting and four windows (118–127), clamp and opaque alpha (136). Generic workbench controls are transformed before processing (109–113). This is the closest standalone counterpart to the app. |
| App device: `apps/phosphlux/src/devices/paik_abe/ped_colorizer.wgsl` | Function module, with no vertex/fragment entry points, bind declarations or standalone parameter struct. `device_paik_abe_ped_colorizer_process(uv, time, video_in, threshold_1, threshold_2, threshold_3, softness, clipping) -> vec4<f32>` (111–120). | Rec.601 luma (122), clipping (125), fixed palette (128), sorted sigmoid windows (39–98,131–138), clamp and alpha 1 (140). UV and time are accepted but unused. Rust includes this file (mod.rs:197–198). |

The workbench renderer's layout agrees with the seven-texture form (`tools/device-workbench/workbench-app/src/renderer.rs:109-145`) and selects `vs_main`/`fs_main` (`:363-371`). The component demo's group-0 sampler and uniform at bindings 1 and 2 conflict with that layout's texture slots. A syntactically valid component demo still needs binding adaptation to that renderer; no GPU load was attempted. Naga parse/validation is present before pipeline creation (`:317-339`), but does not itself establish pipeline-layout compatibility.

The workbench and app use the same detector, palette values, clipping normalization, luma coefficients, threshold sorting, output clamp and opaque alpha. This is a source-level comparison; rendered equivalence has not been measured. The component demo differs in window formula, defaults, palette selection, blend and clipping. Its comments reference schematic page 28 (`research/components/ics/ca3019/v1/ped_control.wgsl:1-10`); that historical claim has not been verified against the schematic here.

## Parameter contract

App order, ranges and defaults come from `apps/phosphlux/src/devices/paik_abe/mod.rs:201-208`. Workbench equations come from `tools/device-workbench/shaders/ped_colorizer.wgsl:109-113`, not the misleading direct-threshold header at lines 8–13.

| App argument | App range / default | Workbench slot and equation | Reverse mapping for an app value | Workbench value at app default |
|---|---|---|---|---|
| `threshold_1` | 0–1 / 0.25 | slot 0 `gain`: t1 = gain × 0.5 | gain = 2 × threshold_1 | 0.5 |
| `threshold_2` | 0–1 / 0.50 | slot 1 `offset`: t2 = offset × 0.5 + 0.25 | offset = 2 × (threshold_2 − 0.25) | 0.5 |
| `threshold_3` | 0–1 / 0.75 | slot 2 `mix`: t3 = mix × 0.5 + 0.5 | mix = 2 × (threshold_3 − 0.5) | 0.5 |
| `softness` | 0–1 / 0.25 | slot 3 `fold`: softness = fold | fold = softness | 0.25 |
| `clipping` | 0–1 / 1.0 | slot 4 `bias`: clipping = bias + 0.5 | bias = clipping − 0.5 | 0.5 |

The native workbench starts with only `gain=1` (range 0–2), `offset=0` (−1–1), `mix=1` (0–1); no fold or bias definitions exist (`tools/device-workbench/workbench-app/src/state.rs:79-84`). Packing supplies missing fold=0, bias=0, frequency=1, phase=0, threshold=0.5, with remaining slots zero (`:103-119`). Therefore the initial PED shader values are thresholds 0.5/0.25/1.0, softness 0, clipping 0.5; after sorting, the boundaries are 0.25/0.5/1.0. They do not match app defaults.

`set_param` clamps known keys and returns false for absent ones (`state.rs:94-100`); the API ignores that return and responds “ok” (`tools/device-workbench/workbench-app/src/api.rs:161-176`). `set_shader` does not add definitions (`state.rs:87-92`). Searches of the native source found parameter insertion only for those three defaults. Consequently fold/bias requests cannot change the current map, and an export omits them. The defect affects any workbench shader relying on absent keys, not just PED; the proposed task below remains bounded to PED.

With current ranges, t1 is reachable over 0–1, t2 over −0.25–0.75 and t3 over 0.5–1. App t2 above 0.75 and t3 below 0.5 cannot be reproduced. Sorting means parameter identity denotes the supplied control, not a permanently low/mid/high boundary. All five controls should remain explicit, including zeros. The clipping denominator floors at 0.01 in both current implementations; clipping=0 is not a separate “force black” branch (`tools/device-workbench/shaders/ped_colorizer.wgsl:80-82`; `apps/phosphlux/src/devices/paik_abe/ped_colorizer.wgsl:103-107`).

For the component demo, gain/offset/mix are direct thresholds with zero-as-default substitutions; fold defaults to 0.3 when zero; bias selects a palette; frequency blends original RGB with colorization (`research/components/ics/ca3019/v1/ped_control.wgsl:205-224`). Do not map its bias to app clipping or promise lossless conversion of its palette/blend controls. Frequency, phase and threshold are unused by the PED workbench test and have no app parameter equivalents.

## Standalone module to ShaderDevice

1. Use the current app sigmoid/clipping/fixed-palette behavior as the target. Retain its namespaced helpers and `device_paik_abe_ped_colorizer_process` function. A mechanical binding rewrite of the component demonstration would preserve the wrong behavior variant.
2. Remove standalone `VertexOutput`, vertex/fragment entry points, texture declarations, sampler and generic `Params`. Replace texture sampling with the function's `video_in` argument. Convert exported workbench values with the equations above; pass semantic values directly to the function. Do not carry a workbench uniform buffer into the app.
3. Preserve the Rust parameter and input order. The composer appends inputs followed by parameters to `device_{id}_process(uv,time,...)` (`apps/phosphlux/src/gpu/shader_composer.rs:974-1042`). PED inherits PixelProcessor, no feedback and no persistent state (`:79-97`; `apps/phosphlux/src/devices/paik_abe/mod.rs:188-224`). It needs no additional render pass.
4. Let the composer provide common definitions, entry points and bindings. It includes common/component/cable modules (`shader_composer.rs:583-590`), deduplicates device code by ID (`:657-666`), and generates `fs_main` (`:952-955`). Common WGSL supplies `vs_main` and global timing at group 0 binding 0 (`apps/phosphlux/src/shaders/common.wgsl:29-55`). App renderer selects those two entry points (`apps/phosphlux/src/gpu/renderer.rs:432-438`).
5. Device uniforms are generated at group 2 binding 0, with `{device_id}_{instance_id}_{parameter_name}` fields and 16-byte block alignment (`shader_composer.rs:1061-1105`). Parameters initialize from Rust defaults and are collected by layout offsets (`:245-259,305-330`). A lone PED contributes five floats and padding to 32 bytes; a multi-device segment shares its generated block. This differs from the workbench's fixed 64-byte generic block.

Same-segment input is a `signal_{instance_id}` value. Cross-segment input is sampled by the composer using group 1 textures/samplers, with the primary pair at bindings 0/1 and further pairs at 2i/2i+1 (`shader_composer.rs:614-643,974-1008`). Disconnected inputs pass zero vec4 (`:1013-1015`); the PED transformation may still output a shadow color, so disconnected input does not promise black output.

## Ports and registration

| Boundary | Existing mechanism | Required action for current PED |
|---|---|---|
| Workbench textures → graph inputs | Seven workbench textures are renderer resources; this shader only samples input0. | Map input0 to `video_in`, not seven node sockets. |
| Graph ports | One RGB `video_in`, one RGB `video_out` (`apps/phosphlux/src/devices/paik_abe/mod.rs:211-223`). | Preserve these names/types and one output. CHROMA IN and BLK IN in the component comments are not current ports. Adding either is separate historical-emulation work. |
| Rust type and WGSL inclusion | `PaikAbePedColorizer`, stable ID `paik_abe_ped_colorizer`, constructor and include_str (`mod.rs:170-198`). | Already present. No duplicate type or changed ID. |
| Module export | `pub mod paik_abe`; re-export of `PaikAbePedColorizer` (`apps/phosphlux/src/devices/mod.rs:16,23-30`). | Already present. |
| Factory | Registry imports the type and matches stable ID to constructor (`apps/phosphlux/src/ui/device_registry.rs:11-19,59-72`). | Already present. Export name is not a constructor key. |
| Palette | Paik/Abe category, PED Colorizer display label, Validated enum (`device_registry.rs:88-106`). | Already present. Export category “processors” does not determine palette placement. The label is a recorded status, not fresh verification. |
| UI insertion | Enumerates registry, creates device, adds it to patch (`apps/phosphlux/src/ui/node_editor.rs:1016,1044-1047`). | No export-directory discovery in this path. |

## Export and provenance

The Python exporter reads `/state` and writes exactly `name`, `category`, `description`, `shader`, `params`, `exported_at` (`tools/device-workbench/mcp_server.py:617-644`). `/state` returns values only, whereas `/params` returns definitions (`tools/device-workbench/workbench-app/src/api.rs:134-159`). The export therefore carries no parameter ranges, defaults, display labels, order, units or port schema.

`BASE_DIR` is the repository root (`mcp_server.py:36-40`); the current output is root `src/nodes/exported/<name>.json` (`:639-644`), outside the app's source directory. The one stored app export is `apps/phosphlux/src/nodes/exported/PaikAbeColorizer.json:2-11`, a legacy folding/LM741 colorizer, not a PED export. Its description's authenticity is a source claim. Its six fields match the exporter shape, but its location does not establish that the current exporter wrote it there.

No JSON import mechanism was found in current app Rust source: the targeted `rg` search for exported/export_node/nodes/exported/PaikAbeColorizer.json/serde_json returned exit 1 and empty stdout. The inspected palette and factory are static Rust mechanisms. No automatic standalone-to-function adapter was found in the exporter or those registration paths. These are scoped negative findings, not claims about every archived file.

| Record | Existing fields / source evidence | Missing link or qualification |
|---|---|---|
| Component version | `component`, `version`, `created_at`, `created_by`, `notes`, `source_artifacts` (`research/components/ics/ca3019/v1/provenance.json:2-7`). | source_artifacts is empty; created_by is “migration”, not a shader-author attribution. |
| Component spec | source titles, used_in/page references and transfer parameters (`research/components/ics/ca3019/v1/spec.json:33-60`). | Source-title strings do not identify retrieved artifact IDs, file hashes or exact datasheet pages. |
| Source description | Databook paths, failed individual retrieval claim and known URLs (`research/components/ics/ca3019/PROVENANCE.md:6-19`). | Claims were not checked against PDFs or the network. Package descriptions disagree: spec says TO-99-10 (spec.json:6), description says 14-pin DIP / TO-5 (PROVENANCE.md:12); preserve the discrepancy. |
| App link | Comment references the component demo (`apps/phosphlux/src/devices/paik_abe/ped_colorizer.wgsl:18`). | Reference path does not record behavior differences, transformation or a hash. |
| Validation record | validated, validation_date, validation_notes, reference_doc, screenshots (`research/validation/device_validation.json:69-76`). | Stored claim; no exact shader hash, parameter snapshot or measured current workbench/app parity. |
| Capture metadata | name, device, description, timestamp, image_file (`tools/device-workbench/mcp_server.py:539-546`). | No source version, shader hash, effective params or input hash. |

For a future reviewed PED package, proposed metadata fields are: schema_version; device_id and processing_mode; source_form and source_path; source_sha256 and target_path/target_sha256; ordered parameter definitions and conversion equations; input/output names and signal types; component/version and provenance_record; source_artifacts with local path, hash, recorded title/origin and page locator; exported_at; adaptation_author/date; validation status with shader hash, effective parameter values, input identity/hash, captures, method and reviewer. Unknown origin/page/author fields must remain explicitly unknown. These fields do not exist in the current six-field export and are not an approved loader schema. Hashes establish file identity, not authenticity. A separate implementation task would be needed to add package provenance storage or ingestion.

## Proposed implementation task

**Title:** Make existing PED workbench controls reproduce the app parameter domain.

**Objective:** Add an explicit PED parameter profile for the existing workbench shader while retaining the current generic profile for other shaders. Use the current app's five-parameter behavior as the target; keep WGSL processing and palette registration unchanged.

**Bounded changes:** Profile selection must be explicit, not inferred from arbitrary shader text. Supply gain range 0–2/default 0.5, offset −0.5–1.5/default 0.5, mix −1–1/default 0.5, fold 0–1/default 0.25 and bias −0.5–0.5/default 0.5. These inverse-mapped domains cover all five app ranges. Replace silent unsupported-key success with a clear rejection, validating the whole update before mutation. Ensure the existing state/export snapshot includes fold/bias when PED is selected. No general importer, export-path migration, additional graph ports, physics changes or validation-label promotion.

**Completion checks:** CPU/offline checks demonstrate the selected profile's defaults map to 0.25/0.50/0.75/0.25/1.0; endpoints and interior values round-trip for each control, including zeros; fold/bias affect packed slots 3/4; unknown keys reject without partial updates; non-PED defaults remain unchanged; the export snapshot contains all five selected values and remains a standalone package. Parse/validate both existing WGSL forms and a composed PED patch using installed tooling without launching a GPU app. Run relevant Rust/Python checks with no downloads or installs and record unavailable checks as blocked. Do not claim visual parity from CPU checks. Commit only that bounded change locally on its own branch. Suggested worker: codex.

**Hedawn's decision:** Approve this parameter-profile repair or defer it. Rendered parity, circuit fidelity, additional historical inputs, export ingestion and richer provenance remain separate decisions.

## Checks and limits

- Read the project README, development protocol, state record, T-0073 survey, three shaders, native workbench state/API/renderer, exporter, Rust device/composer/registry/UI and provenance/validation records. Current state claims v0.13 complete; source already contains the PED device. No version gates were updated.
- The project's full state verifier and check.sh were not run: they are broader than this one-document task, may build or update state, and cannot establish the source boundary or visual equivalence. No synthesizer/workbench app, GPU pipeline, export operation or R&D loop was run; no network or installation occurred.
- Source inspection commands returned exit 0 where used above. One exploratory search included nonexistent shader.rs and returned an error; subsequent targeted reads used the actual state.rs/renderer.rs and the importer search explicitly reported exit 1/empty output. Instruction-file glob found no AGENTS.md or CLAUDE.md in this checkout.
- The task branch was created from the existing T-0073 survey branch. Pre-existing changes in Cargo.toml, Cargo.lock, BASELINE.md, BLOCK_COMPATIBILITY_VALIDATION.md, research/schematics and vendor were left outside the commit. Only this brief is authorized as a new tracked artifact.
- Completion is a source-backed implementation proposal. GPU output, historical accuracy and creator preference remain unchecked; the registry's Validated label has not been reassessed.

### Document verification

The read-only Python check exited 0 and printed “PASS: 27 explicit file/line references resolve within source bounds”, “PASS: 11 shader, Rust parameter and packing anchors match source”, and “PASS: all seven completion sections present”. It also checked all five conversion equations at semantic values 0, 0.25, 0.5, 0.75 and 1, reporting roundtrip PASS for every control. These arithmetic checks validate the documented inverse mappings, not native API behavior or GPU output. The complete Markdown was reread for source consistency and decision clarity. `git diff --check` exited 0 with no output before staging; the staged-document check and committed-file inventory are recorded in the board submission.
