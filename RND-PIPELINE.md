# R&D pipeline — repository survey

Observed and claimed are separate: **Observed** identifies a file, code path, inventory result or command result read in the original survey or the correction run identified in Appendices E/F. **Claimed** identifies what a document or code comment says about history, fidelity, validation or workflow; it does not establish that claim. “Implemented” means a code body exists, not that the operation passed a runtime test.

## 1. Vocabulary

| Term | Repository definition and relation | Evidence |
|---|---|---|
| Component | Claimed: “Reusable analog component models for video synthesis emulation.” Entries contain specs, datasheets, WGSL transfer functions and demonstration shaders. Observed: CPU circuit models also use component terminology; this is not a runtime graph node class. | `research/components/README.md:3,24-32`; `apps/phosphlux/src/core/component.rs:11-14,141-144,203-205,244-247` |
| Device | Observed: `ShaderDevice`: “A device that can be composed into a shader”; exposes ID, shader code, parameters, input/output ports and processing mode. A `DeviceInstance` contains one boxed device, ID, parameter overrides and UI position. Claimed: the research devices layer describes an entire historical machine, whose app implementation can occupy several device instances. | `apps/phosphlux/src/gpu/shader_composer.rs:52-99,211-220`; `research/DISCOVERY_GUIDE.md:27` |
| Node | Observed: UI uses `selected_node`, `hovered_node` and `instance_id`; a node is the displayed device instance, not a separate circuit object. The workbench also calls a JSON shader package a node: “Export shader as node for main app.” No independent formal Node definition was found in these current graph/export files. | `apps/phosphlux/src/ui/node_editor.rs:105-124`; `tools/device-workbench/mcp_server.py:617-635` |
| Signal | Observed: CPU `AnalogSignal` is “A single-channel analog signal with electrical characteristics”; `RgbSignal` is “RGB signal with per-channel analog characteristics”; `YiqSignal` is for NTSC encoding. GPU graph `SignalType` is instead a port type tag: RGB, single-channel luma/CV, or control voltage. | `apps/phosphlux/src/core/signal.rs:91-93,161-163,208-211`; `apps/phosphlux/src/gpu/shader_composer.rs:162-170` |
| Port / socket | Observed: “A device port (input or output)” has a name and signal type. UI socket stores `(instance_id, is_output, port_index)`; socket is the UI attachment for a port. | `apps/phosphlux/src/gpu/shader_composer.rs:140-147`; `apps/phosphlux/src/ui/node_editor.rs:117` |
| Connection / cable | Observed: “A connection between device ports” stores endpoint IDs and port names plus a `CableType`. Cable is a signal-quality choice on a connection, not a device: “Cable/connector type affecting signal quality.” Types are Ideal, Composite, SVideo, Component and Rgb. “Component” here is a cable encoding rather than an IC model. | `apps/phosphlux/src/gpu/shader_composer.rs:175-206` |
| Patch | Observed: “A complete device patch” holds all device instances, connections, output device ID and next instance ID. | `apps/phosphlux/src/gpu/shader_composer.rs:223-232` |
| Plan | Observed: `RenderPlan`: “A complete render plan: ordered list of segments to execute.” This runtime plan differs from the historical emulation plan document. | `apps/phosphlux/src/gpu/shader_composer.rs:345-350`; `docs/HISTORICAL_EMULATION_PLAN.md:1` |
| Segment | Observed: “A single render segment with its compiled shader”; holds WGSL source, parameter layout, instance IDs and texture dependencies. Segment kinds are multiple composed pixel processors or one scan modulator; a plan contains segments, each representing a render pass. | `apps/phosphlux/src/gpu/shader_composer.rs:352-378` |
| PixelProcessor / ScanModulator | Observed: “Transforms pixel values: vec4 in -> vec4 out” versus “Needs to sample input at arbitrary UV coordinates”; the second requires its own render pass. | `apps/phosphlux/src/gpu/shader_composer.rs:40-47` |
| Artifact | Observed: “Base artifact with common fields”; ID, type, name, path, source URL, retrieval time and metadata. This is a retrieved-document record, distinct from device instance. | `tools/retrieval/artifacts.py:16-25` (quoted definitions in Appendix D) |
| Component version | Observed: component manager “Manages versioned WGSL component models”; folders use category/name and `vN`; provenance describes version creator and source artifacts. | `tools/retrieval/components.py:35-54,85-100`; Appendix B |
| Prototype | Claimed: “Working modules that are not authentic circuit emulations.” Distinct from production readiness. | `research/prototypes/README.md:3` |
| Validation status | Observed: registry defines Validated (“validated in workbench with documentation”), NeedsValidation, Legacy and Prototype (“Artistic interpretation - not authentic to original schematics”). These are code labels; they do not constitute fresh validation. | `apps/phosphlux/src/ui/device_registry.rs:21-31` |

Observed: the runtime containment is patch → device instances plus port connections → compiled render plan → segments; device code embeds WGSL. The research containment is historical device documents → per-part specs/models/provenance, separate from the runtime graph. Evidence: the definitions above and `apps/phosphlux/src/devices/paik_abe/mod.rs:61-71,124-134,188-198,252-262,315-325`. Claimed historical hardware relationships are recorded separately in `research/devices/paik_abe/architecture.md:7-45`.

## 2. Source documents and records

Observed: Appendix A enumerates every file under research/ and docs/, including sources, source descriptions, models, audit notes and capture artifacts. Every binary row reports its file kind by extension and byte count, not verified page contents. PDF titles/types are taken from recorded index descriptions where available, otherwise the filename. Origins are claims in the recorded source lines in Appendix C; “not recorded” means no file-specific origin was found in the accompanying text records, not that the file lacks an author. PDF page contents and embedded copyright notices were not read in this run.

Claimed: the index organizes four layers: innovators, techniques, devices and components (`research/RESEARCH_INDEX.md:7-22`). The discovery guide gives source-placement and PDF-split rules (`research/DISCOVERY_GUIDE.md:189-227`). Claimed index counts sometimes name unsplit originals; the actual inventory below counts physical files, including splits, rather than distinct publications.

Observed: rights/licence entries in accompanying text are collected in Appendix C. No per-PDF licence grant field occurs in the JSON records enumerated in Appendix B. Claimed references to open-source hardware or freely shared schematics are retained as claims, not permission determinations. The root README claims MIT for this project (`README.md:319-321`); that does not record a separate grant for every third-party PDF. Rights inside source PDFs remain unknown.

Observed: `research/`: 111 files; .db: 1, .json: 22, .md: 16, .pdf: 51, .wgsl: 21. Evidence: Appendix A inventory command (exit 0).

Observed: `docs/`: 173 files; .gif: 23, .json: 52, .md: 14, .png: 84. Evidence: Appendix A inventory command (exit 0).

Observed Git coverage: of the 284 local files in Appendix A, 277 are tracked, five original PDFs are ignored and two Rutt/Etra source/metadata files are untracked. The seven local-only files would not be supplied by a fresh clone of this Git state. Evidence: `git ls-files research docs`, `git ls-files --others --ignored --exclude-standard research docs` and `git ls-files --others --exclude-standard research docs`, each exit 0; per-file statuses are in Appendix A and real nontracked output is in Appendix F.

| Local-only file | Git status | Evidence |
|---|---|---|
| `research/components/RCA_1975_Linear_ICs.pdf` | Ignored original PDF, present locally | `.gitignore:60`; `git check-ignore -v` exit 0 |
| `research/components/RCA_1978_Linear_IC_Databook.pdf` | Ignored original PDF, present locally | `.gitignore:61`; `git check-ignore -v` exit 0 |
| `research/devices/paik_abe/Paik Abe Video Synthesizer Technical Documentation.pdf` | Ignored original PDF, present locally | `.gitignore:59`; `git check-ignore -v` exit 0 |
| `research/devices/sandin_ip/ETC_SandinIP_Manual.pdf` | Ignored original PDF, present locally | `.gitignore:57`; `git check-ignore -v` exit 0 |
| `research/devices/sandin_ip/Sandin_Distribution_Religion.pdf` | Ignored original PDF, present locally | `.gitignore:58`; `git check-ignore -v` exit 0 |
| `research/schematics/rutt-etra/metadata.json` | Untracked, present locally | `git ls-files --others --exclude-standard research docs` exit 0 |
| `research/schematics/rutt-etra/schematic.pdf` | Untracked, present locally | `git ls-files --others --exclude-standard research docs` exit 0 |

Claimed: `docs/DEVELOPMENT_ROADMAP.md:18` points to `__archive_typescript_reference/` as the archived TypeScript prototype. That archive is named here but was not surveyed; archive/v1 and phosphlux-lite also remain outside scope.

Observed: `research/provenance.db` has six tables and zero rows in each. Read-only SQL was used; no retrieval or database initialization occurred. The schema can store artifacts, versions, R&D passes, sources and relationships, but this database currently contributes no source-to-model links. Appendix B records the actual table columns and counts.

## 3. Tools

| Tool | Purpose, inputs and outputs | Requirements and observed availability |
|---|---|---|
| pdf-utils | Claimed/code: split large PDFs, extract page ranges, convert pages to images; opening docstring also claims basic compression. Observed CLI offers info, split, extract, images, with PDF path, output path/directory, page selection and DPI; no compression subcommand. Outputs are info dictionary, split PDF paths, extracted PDF or page images. | Observed requirements: pypdf ≥4, PyMuPDF ≥1.23 (`tools/pdf-utils/requirements.txt:1-3`). Help passes, exit 0; processing not run. `tools/pdf-utils/split_pdf.py:3-9,31-65,171-221`; Appendix E. |
| retrieval | Claimed/code: layer-specific patents, datasheets, schematics and manuals; SQLite provenance; component version management. Inputs are query/part number/device/URL and filters; search returns result objects; downloads return Artifact objects and files; component manager consumes spec/WGSL/source IDs and creates version folders. Observed default outputs: research/patents, datasheets, schematics, manuals; components under research/components. | Observed requirements file lists requests, aiohttp, beautifulsoup4, lxml, optional PyPDF2/numpy; SQLite stdlib. Package help and schematic CLI help pass exit 0 (CLI emits runpy warning). No search/download/constructor invoked. `tools/retrieval/__init__.py:1-10`; `tools/retrieval/base.py:60-70,84-94`; `tools/retrieval/patents.py:35`, `tools/retrieval/datasheets.py:52`, `tools/retrieval/schematics.py:91`, `tools/retrieval/manuals.py:66`; `tools/retrieval/components.py:35-100`; Appendix D/E. |
| research-orchestrator | Claimed README: multi-agent Ollama research, shader drafts, image extraction and embeddings. Observed current server explicitly says these tools were removed and exposes seven retrieval/provenance/component tools instead. Input MCP JSON schemas accept query, inventor, download, part_number, device, manual_type, artifact_id, category or component; output TextContent lists search/download paths, provenance or component histories. | Observed import requires mcp; requirements also retain ollama, aiohttp, asyncio-mqtt, pypdf, python-docx, Pillow, numpy, faiss-cpu, pydantic, dotenv. Help invocation fails exit 1: missing mcp. No retrieval run. README's model requirements and tool table disagree with current entry point. `tools/research-orchestrator/README.md:3-42,91-99`; `tools/research-orchestrator/mcp_server.py:3-10,41-181,184-213`; Appendix D/E. |
| device-workbench | Claimed/code: Rust/wgpu window with HTTP API and Python MCP bridge; accepts WGSL, params, test patterns, reference images, sweep and capture settings, node name/category. Outputs GPU image, error/state/diff data, screenshots + metadata, GIF sweeps, JSON node package. Observed export writes root `src/nodes/exported/<name>.json` from current workbench state, not a Rust ShaderDevice registration. | Observed Python imports httpx and mcp; sweep uses Pillow. No requirements file exists at the workbench tool root (Path glob check returned []). Rust manifest lists tokio, axum/tower-http, wgpu/winit/naga, image/base64, serde, anyhow, tracing, parking_lot. Requires Rust build environment and GPU window for actual rendering. Python help fails exit 1: missing httpx. Native app not launched: main has no help parser and creates a window/API server. No build or GPU validation performed. `tools/device-workbench/mcp_server.py:20-42,585-586,617-669`; `tools/device-workbench/workbench-app/src/main.rs:49-65,108-144`; `tools/device-workbench/workbench-app/Cargo.toml:7-34`; `docs/WORKBENCH_WORKFLOW.md:3-12`; Appendix E. |

Observed: neither MCP entry point implements argument parsing for --help; these checks stop on imports, before entering main. No software was installed, no network request was made, and no application window was started. Help success establishes CLI/import availability only.

## 4. Paik/Abe recorded path

1. Claimed sources: index lists technical documentation (pre-existing), schematics (pre-existing), ETC3311 schematics, ETC3310 restoration, ETC3316 budget and Vasulka pages 126–129 (`research/RESEARCH_INDEX.md:44-52`). Observed: those file families exist; the technical document exists as a Git-ignored unsplit original plus three tracked page-range files (`.gitignore:59`; Git-status inventory in Appendix A). Additional innovator/technique context and RCA/individual datasheets are inventoried in Appendix A, with claimed origins in Appendix C.
2. Claimed circuit analysis: `research/devices/paik_abe/architecture.md:3` says “Analyzed from original schematics (26 pages)”; its table links CA3019, CA3022, CA3023, CA3046, CA3054 and CA3055 to schematic pages (`:38-45`). The audit cites the technical documentation pages 1–30 and records different fidelity judgments (`research/audits/paik_abe_device_audit_20260129.md:5,11-17`). These historical/fidelity statements were not independently verified against PDF images here.
3. Observed intermediate library: per-IC `v1/spec.json`, `model.wgsl`, demonstration shaders and provenance files exist for CA3019/3022/3023/3046/3054/3080/LM311, plus functional folding_amp, wobbulator and paik_abe_chain signal_chain. Claimed specs/provenance cite source names, equations and schematic pages. Seven migration provenance records have empty source_artifacts; CA3022 and CA3023 list paths to unsplit technical documentation/databook that exist locally at those exact paths and appear in Appendix A, but are Git-ignored (`.gitignore:59,61`). Their local existence does not supply a populated database lineage record. Both provenance files also cite ETC_PaikAbe_Schematics.pdf, which is tracked. Appendix B preserves fields and provenance values. No populated database links resolve these paths.
4. Observed workbench intermediates: `tools/device-workbench/shaders/` includes input_matrix, folding_amp, ped_colorizer, output_encoder and colorizer variants; research WGSL provides transfer-function and demonstration variants. `docs/screenshots/` contains PNG/GIF outputs and JSON capture records. Claimed workflow is load → inject → adjust → screenshot/analyze → compare → sweep → document (`docs/WORKBENCH_WORKFLOW.md:150-167`). Presence of captures proves stored artifacts, not measured equivalence to hardware. Appendix A/B enumerates them.
5. Observed export artifact: `apps/phosphlux/src/nodes/exported/PaikAbeColorizer.json:2-11` records name, processors category, description, a full standalone shader, gain/offset/mix params and exported_at 2026-01-25T02:57:27.851564. Claimed description calls triangle folding/LM741 authentic (`:4`); later prototype specs and audit reject that claim. Current exporter uses root `src/nodes/exported` (`tools/device-workbench/mcp_server.py:638-644`), whereas this artifact resides inside apps/phosphlux. No export was performed here.
6. Observed app implementation: Rust module includes folding_amp.wgsl, input_matrix.wgsl, ped_colorizer.wgsl, output_encoder.wgsl and combined colorizer.wgsl; Wobulator processing follows in the same module (`apps/phosphlux/src/devices/paik_abe/mod.rs:61-71,124-134,188-198,252-262,315-325,377` onwards). PED shader explicitly references the component library file (`apps/phosphlux/src/devices/paik_abe/ped_colorizer.wgsl:18`), providing one concrete recorded library→app link. The factory constructs these Rust types; the palette marks input matrix, PED, output encoder and wobulator Validated, but wavefolder and legacy colorizer Prototype (`apps/phosphlux/src/ui/device_registry.rs:59-72,88-124`). These labels are observed code; validation success remains a claim.
7. Observed integration boundary: registry constructs devices with a match, not from exported JSON (`apps/phosphlux/src/ui/device_registry.rs:59-72`); composer embeds current app shaders and general component/cable WGSL (`apps/phosphlux/src/gpu/shader_composer.rs:23-30`). Search evidence in Appendix E finds the export artifact but no exported-node loader in current app Rust. No recorded automatic exporter→factory path was found. Exact manual adaptation history and source revision/hash for each final shader are unclear.

Claimed conflict: README describes the full Paik/Abe chain and completed component work; exported JSON and early research analysis call folding authentic/conceptually accurate; prototype specs, audit and palette call it an artistic interpretation. Evidence: `README.md:280-292`; `docs/research/paik-abe-colorizer-analysis.md:145`; `research/prototypes/paik_abe_colorizer_v1/spec.json:8-14`; `research/prototypes/paik_abe_folding_amp_v1/spec.json:8-14`; audit `:11-17`; registry `:97-123`. Claimed current device-module header itself says Input Matrix is partial (`apps/phosphlux/src/devices/paik_abe/mod.rs:24`), whereas the palette marks it Validated. The output shader claims LM741 (`apps/phosphlux/src/devices/paik_abe/output_encoder.wgsl:9`), while the audit calls the IC model wrong (`research/audits/paik_abe_device_audit_20260129.md:16`). No fidelity verdict is chosen in this survey.

## 5. Rutt/Etra recorded path

1. Observed sources: four PDFs under research/devices/rutt_etra plus schematic.pdf and metadata.json under research/schematics/rutt-etra. Claimed index origins are Vasulka and ETC2530 (`research/RESEARCH_INDEX.md:64-71`); the additional schematic metadata records ETC2731, device rutt-etra and retrieval timestamp 2026-02-02T04:12:37.263145 (`research/schematics/rutt-etra/metadata.json:2-4`). No rights, hash, revision, page count or parts-list field appears in that metadata. Observed Git coverage: the four research/devices/rutt_etra PDFs are tracked; research/schematics/rutt-etra/schematic.pdf and metadata.json are both untracked local files, so those two would not accompany a fresh clone (Git inventory commands in Appendix F).
2. Claimed analyses: technical reference discusses DC coupling and scan displacement; roadmap lists TBD op-amp/multiplier, pending schematics/patents/manual and geometry-based blocks (`docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md:149-180`; `docs/V1_EMULATION_ROADMAP.md:100-129`). Its unchecked schematic item coexists with present source files; this is a recorded status mismatch, not a quality judgment.
3. Observed intermediate boundary: inventory has no Rutt/Etra per-part spec, model, workbench shader, capture or exported node named for Rutt/Etra. Functional wobbulator spec claims a historical Rutt/Etra relation (`research/components/functional/wobbulator/v1/spec.json:56-60`), but no source-to-Rutt-specific app implementation link is recorded there.
4. Observed app boundary: `apps/phosphlux/src/devices/mod.rs:19` has commented `pub mod rutt_etra; // TODO`; factory and categories contain no Rutt/Etra entry (`apps/phosphlux/src/ui/device_registry.rs:59-127`). The recorded path stops at source records and prose plans, not a registered Rutt/Etra runtime device. Evidence: Appendix A inventory, Appendix E search and quoted module lines in Appendix D.

## 6. New-circuit steps recorded today

Claimed order below synthesizes the repository's own ordered instructions: discovery forward path (`research/DISCOVERY_GUIDE.md:30-32`), README component philosophy (`README.md:142-149`), orchestrator workflow (`tools/research-orchestrator/README.md:91-99`), workbench calibration (`docs/WORKBENCH_WORKFLOW.md:150-167`) and registry requirements (`apps/phosphlux/src/ui/device_registry.rs:6-8`). It is not an executed pipeline trace. Implementation marks are observed source bodies, with runtime limits in section 3.

| Step | Status | Recorded instructions and implementation evidence |
|---|---|---|
| Discover inventor/technique/device sources | Both | Claimed discovery layers and archives, `research/DISCOVERY_GUIDE.md:23-32,36-65`; observed retrieval search methods in patents/datasheets/schematics/manuals. |
| Retrieve and record documents/provenance | Both | Claimed discovery source placement `:189-192`; observed BaseRetriever/download/database bodies, tool MCP retrieval handlers. Present database has zero rows. |
| Split large PDFs / extract useful pages | Both | Claimed `research/DISCOVERY_GUIDE.md:196-227`; observed `tools/pdf-utils/split_pdf.py:55` onwards, CLI help and actual split files. |
| Identify circuit parts and topology | Documented | Claimed README component-level philosophy (`README.md:142-149`) and circuit analysis (`research/devices/paik_abe/architecture.md:38-45`); observed `tools/retrieval/schematics.py:495-518` extraction is only a placeholder that reads an existing parts_list.json or returns []. Automated extraction is not implemented there. |
| Map external physical components to normalized parameters | Documented, with implemented examples | Claimed ordered rule identifies external components, normalizes controls, documents mappings and supplies defaults (`docs/EMULATION_FIDELITY.md:51-57`). Observed specs and shader headers implement selected mappings; not every physical quantity was checked. |
| Obtain datasheets / specs / transfer functions | Both | Claimed `research/components/README.md:24-32,70-99`; observed datasheet retrieval and existing spec/model files. Some PROVENANCE.md files record failed individual downloads. |
| Draft and review WGSL component model | Both, with obsolete automation claim | Claimed orchestrator README `:93-97`; observed models and ComponentManager version creation code. Current orchestrator removes generate_shader, so README's model-dispatch step is not implemented by its current server. |
| Compose component behavior into a device shader | Both | Claimed component application examples `research/components/README.md:34-68`; observed research chain, workbench shader files and app include_str calls. No automatic circuit-to-WGSL translator was found in surveyed tools. |
| Calibrate with injected signals, screenshots, references and sweeps | Both | Claimed workflow `docs/WORKBENCH_WORKFLOW.md:150-167`; observed workbench bridge handlers, native renderer/diff code and stored captures. Runtime and historical fidelity not tested. |
| Record validation and classification | Both | Claimed schematic/behavior/visual/experiential levels `docs/HISTORICAL_EMULATION_PLAN.md:203-253`; observed validation JSON and palette statuses. Presence of status is not independent completion of those levels. |
| Export shader package as node | Both | Claimed `docs/WORKBENCH_WORKFLOW.md:140`; observed `tools/device-workbench/mcp_server.py:617-644` JSON export body and stored legacy export. |
| Automatically ingest exported node into current palette | Neither | No instruction specifying a current JSON loader, nor implementation in factory, found. Export describes “for main app” but does not specify this missing ingest mechanism. Evidence: exporter and factory source, Appendix E search. |
| Adapt/register a Rust ShaderDevice and compile patch | Both | Claimed README architecture/ShaderDevice reuse (`README.md:55,103-105,130-136`); observed device module, factory and ShaderComposer. Manual adaptation is visible in code products, but its exact authoring sequence is not recorded. |

Claimed additional completeness standard: all six fidelity tiers are required, from IC behavior through passive effects, saturation, noise, tolerance and crosstalk (`docs/EMULATION_FIDELITY.md:9-18`); per-IC checklist requires spec/model/application shader/mapping/provenance/screenshot/sweep (`:61-71`). This is a different set of requirements from the four historical verification levels; neither was certified in this run.

## 7. Missing or unclear

- Observed: provenance database is empty, seven migrated model provenance records have no source artifacts, and CA3022/CA3023 provenance points to locally present originals excluded from Git (`.gitignore:59,61`; Appendix B). Five ignored original PDFs and two untracked Rutt/Etra files would be missing from a fresh clone (section 2).
- Observed: workbench export path and current app artifact location differ; current factory does not ingest JSON (section 4; Appendix E). How exported standalone shaders become current trait implementations is not recorded in the inspected integration code.
- Claimed: orchestrator README's Ollama tool workflow conflicts with current server's removed-tool note (section 3).
- Observed: the MCP entry points cannot import their required modules with this Python; actual retrieval and GPU calibration were not exercised (Appendix E).
- Claimed: Paik/Abe folding and output-encoder fidelity classifications differ among early analysis, export description, audit, prototype records and palette. Source-page claims remain unverified here (section 4).
- Observed: JSON capture records contain parameter and timing metadata but do not supply a complete source/hash/revision lineage for every final app shader (Appendix B).
- Observed: per-source rights grants are not in the JSON fields enumerated; PDF-internal notices were not inspected. Claimed open-source/free-sharing statements do not specify a grant for each stored source (Appendix C).
- Observed/claimed: Rutt/Etra source files exist but its roadmap still requests schematics and its app module remains TODO (section 5).
- Claimed: four verification levels include measured hardware, footage, expert/blind and experiential comparisons; stored status labels/captures alone do not establish those levels (`docs/HISTORICAL_EMULATION_PLAN.md:203-253`).

## 8. What follows

Proposal grounded in the observed export/factory gap: a separate, read-only integration brief for one existing PED Colorizer shader, specifying the current workbench→ShaderDevice boundary, required shader/parameter/port adaptations and provenance fields. It should compare the stored component, workbench and app forms and produce one concrete implementation task for Hedawn to decide, without adding a loader, changing code, running the loop or fetching sources. This is a proposal, not a repository finding or approval.

## Appendix A. Complete inventory

Observed command in the original survey run: Python pathlib recursive enumeration of research/ and docs/, file sizes and reads of their text records; exit 0. This inventory includes generated captures and code models to avoid silently excluding possible source descriptions. File-specific origin claims follow each PDF row where an exact or split-family index match exists; all recorded source-description lines are in Appendix C. Non-source captures/models are classified as such.

| File | Kind / recorded title | Bytes | Origin record | Git status |
|---|---|---:|---|---|
| Observed `docs/ANALOG_ROUTING_ARCHITECTURE.md` | Local documentation / source description: Analog Signal Routing Architecture | 10186 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/CABLE_DEGRADATION_SPEC.md` | Local documentation / source description: Cable Degradation Specification v0.13 | 13239 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/DEVELOPMENT_ROADMAP.md` | Local documentation / source description: Development Roadmap | 9893 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/EMULATION_FIDELITY.md` | Local documentation / source description: Emulation Fidelity Standards | 6433 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/HISTORICAL_EMULATION_PLAN.md` | Local documentation / source description: Historical Video Synthesis Emulation Plan | 13305 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/SESSION_HANDOFF_v0.15.md` | Local documentation / source description: Session Handoff: v0.16 Planning | 2702 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/SESSION_HANDOFF_v0.16.md` | Local documentation / source description: Session Handoff: v0.17 Planning | 3106 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/V1_EMULATION_ROADMAP.md` | Local documentation / source description: Phosphlux v1 Emulation Roadmap | 8947 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md` | Local documentation / source description: Video Synthesis: A Complete Technical History for Digital Implementation | 30116 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/WORKBENCH_WORKFLOW.md` | Local documentation / source description: Device Workbench Development Workflow | 7299 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/architecture/swarm-agent-specification.md` | Local documentation / source description: Swarm Agent Architecture Specification | 27588 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/dev-logs/vision-guided-development.md` | Local documentation / source description: Vision-Guided Development Workflow | 10216 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/dev-logs/workbench-mcp-template.md` | Local documentation / source description: Visual Workbench MCP Template | 6223 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/research/paik-abe-colorizer-analysis.md` | Local documentation / source description: Paik/Abe Colorizer - Research Analysis | 5925 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3019/ca3019_ped_control_colorbars_20260125_074158.json` | Structured metadata / specification | 306 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3019/ca3019_ped_control_colorbars_20260125_074158.png` | Generated screenshot | 33366 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3019/ca3019_ped_control_ramp_20260125_074148.json` | Structured metadata / specification | 298 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3019/ca3019_ped_control_ramp_20260125_074148.png` | Generated screenshot | 33633 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3022_colorbars_moderate_20260126_044358.png` | Generated screenshot | 3088718 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3023_colorbars_moderate_20260126_044527.png` | Generated screenshot | 33412 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3046/ca3046_colorizer_colorbars_20260125_074043.json` | Structured metadata / specification | 296 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3046/ca3046_colorizer_colorbars_20260125_074043.png` | Generated screenshot | 33390 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3046/ca3046_colorizer_ramp_20260125_074032.json` | Structured metadata / specification | 303 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3046/ca3046_colorizer_ramp_20260125_074032.png` | Generated screenshot | 34251 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3054/ca3054_keyer_colorbars_20260125_074415.json` | Structured metadata / specification | 291 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3054/ca3054_keyer_colorbars_20260125_074415.png` | Generated screenshot | 33403 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3054/ca3054_matrix_colorbars_20260125_074312.json` | Structured metadata / specification | 294 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3054/ca3054_matrix_colorbars_20260125_074312.png` | Generated screenshot | 33473 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3054/ca3054_matrix_rotated_20260125_074322.json` | Structured metadata / specification | 288 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3054/ca3054_matrix_rotated_20260125_074322.png` | Generated screenshot | 33473 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3080/ca3080_vca_colorbars_20260125_074602.json` | Structured metadata / specification | 278 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3080/ca3080_vca_colorbars_20260125_074602.png` | Generated screenshot | 33328 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3080/ca3080_vca_modulated_20260125_074613.json` | Structured metadata / specification | 261 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ca3080/ca3080_vca_modulated_20260125_074613.png` | Generated screenshot | 33328 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/colorizer/colorizer_colorbars_20260129_230159.json` | Structured metadata / specification | 245 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/colorizer/colorizer_colorbars_20260129_230159.png` | Generated screenshot | 33405 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/colorizer/colorizer_ramp_20260129_230150.json` | Structured metadata / specification | 243 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/colorizer/colorizer_ramp_20260129_230150.png` | Generated screenshot | 34215 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/component_colorbars_test_20260128_205352.png` | Generated screenshot | 33397 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/composite_colorbars_test_20260128_205155.png` | Generated screenshot | 35602 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/folding_amp/fold_gain2_20260129_230013.json` | Structured metadata / specification | 221 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/folding_amp/fold_gain2_20260129_230013.png` | Generated screenshot | 34388 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/folding_amp/fold_gain4_20260129_230022.json` | Structured metadata / specification | 216 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/folding_amp/fold_gain4_20260129_230022.png` | Generated screenshot | 34257 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/folding_amp/folding_amp_ramp_3x_20260125_074504.json` | Structured metadata / specification | 264 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/folding_amp/folding_amp_ramp_3x_20260125_074504.png` | Generated screenshot | 34602 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/folding_amp/folding_amp_ramp_6x_20260125_074514.json` | Structured metadata / specification | 266 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/folding_amp/folding_amp_ramp_6x_20260125_074514.png` | Generated screenshot | 34602 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/folding_amp_check_20260129_031833.png` | Generated screenshot | 34109 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/folding_amp_gain4_20260129_031857.png` | Generated screenshot | 34109 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/folding_amp_gain8_20260129_032059.png` | Generated screenshot | 34109 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/folding_amp_max_20260129_032254.png` | Generated screenshot | 34967 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/folding_amp_mid_gain_20260129_032209.png` | Generated screenshot | 34815 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/folding_amp_min_20260129_032233.png` | Generated screenshot | 34006 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/folding_amp_ramp_h_20260126_034619.png` | Generated screenshot | 34440 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/input_matrix_colorbars_default_20260129_032356.png` | Generated screenshot | 33402 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/input_matrix_red_only_20260129_032516.png` | Generated screenshot | 33377 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/input_matrix_rot90_20260129_032447.png` | Generated screenshot | 33402 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/input_matrix_rotated_20260129_032419.png` | Generated screenshot | 33402 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/lm311/lm311_keyer_window_20260125_074701.json` | Structured metadata / specification | 266 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/lm311/lm311_keyer_window_20260125_074701.png` | Generated screenshot | 33398 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/matrix_mixer_colorbars_20260126_034347.png` | Generated screenshot | 33473 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/output_encoder/output_neutral_20260129_230430.json` | Structured metadata / specification | 251 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/output_encoder/output_neutral_20260129_230430.png` | Generated screenshot | 33390 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/output_encoder/output_warm_20260129_230440.json` | Structured metadata / specification | 242 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/output_encoder/output_warm_20260129_230440.png` | Generated screenshot | 33389 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/output_encoder_default_20260129_032741.png` | Generated screenshot | 33390 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/output_encoder_warm_20260129_032804.png` | Generated screenshot | 33389 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/ca3022_colorbars_default_20260126_044435.json` | Structured metadata / specification | 345 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/ca3022_colorbars_default_20260126_044435.png` | Generated screenshot | 3089133 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/ca3023_colorbars_default_20260126_044536.json` | Structured metadata / specification | 324 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/ca3023_colorbars_default_20260126_044536.png` | Generated screenshot | 33412 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/folding_amp_default_20260126_034654.json` | Structured metadata / specification | 358 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/folding_amp_default_20260126_034654.png` | Generated screenshot | 34576 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/matrix_mixer_default_20260126_034424.json` | Structured metadata / specification | 326 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/matrix_mixer_default_20260126_034424.png` | Generated screenshot | 33473 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/ped_colorizer_default_20260126_034547.json` | Structured metadata / specification | 314 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/ped_colorizer_default_20260126_034547.png` | Generated screenshot | 33543 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/signal_chain_6stage_colorbars_20260126_044742.json` | Structured metadata / specification | 319 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/signal_chain_6stage_colorbars_20260126_044742.png` | Generated screenshot | 2368767 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/signal_chain_6stage_ramp_20260126_044725.json` | Structured metadata / specification | 356 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/signal_chain_6stage_ramp_20260126_044725.png` | Generated screenshot | 2383467 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/signal_chain_colorbars_20260126_035333.json` | Structured metadata / specification | 375 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/signal_chain_colorbars_20260126_035333.png` | Generated screenshot | 91165 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/signal_chain_ramp_20260126_035307.json` | Structured metadata / specification | 379 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/signal_chain_ramp_20260126_035307.png` | Generated screenshot | 678131 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/video_keyer_default_20260126_034808.json` | Structured metadata / specification | 349 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/video_keyer_default_20260126_034808.png` | Generated screenshot | 33395 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/wobbulator_default_20260126_034917.json` | Structured metadata / specification | 353 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik-abe/wobbulator_default_20260126_034917.png` | Generated screenshot | 1867662 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe/folding_amplifier_20260125_063503.json` | Structured metadata / specification | 274 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe/folding_amplifier_20260125_063503.png` | Generated screenshot | 61819 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe/ped_control_colorizer_20260125_061443.json` | Structured metadata / specification | 282 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe/ped_control_colorizer_20260125_061443.png` | Generated screenshot | 73908 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe/video_keyer_20260125_062533.json` | Structured metadata / specification | 250 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe/video_keyer_20260125_062533.png` | Generated screenshot | 73992 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_folding_amp/folding_amp_default_ramp_20260129_031627.json` | Structured metadata / specification | 320 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_folding_amp/folding_amp_default_ramp_20260129_031627.png` | Generated screenshot | 34109 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_folding_amp/folding_amp_validated_20260129_032314.json` | Structured metadata / specification | 379 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_folding_amp/folding_amp_validated_20260129_032314.png` | Generated screenshot | 34967 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/7input_default_20260129_222431.json` | Structured metadata / specification | 243 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/7input_default_20260129_222431.png` | Generated screenshot | 33402 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/blue_isolation_20260129_054746.json` | Structured metadata / specification | 237 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/blue_isolation_20260129_054746.png` | Generated screenshot | 33387 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/default_colorbars_20260129_054724.json` | Structured metadata / specification | 255 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/default_colorbars_20260129_054724.png` | Generated screenshot | 33402 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/green_isolation_20260129_054739.json` | Structured metadata / specification | 240 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/green_isolation_20260129_054739.png` | Generated screenshot | 33372 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/input_matrix_validated_20260129_032536.json` | Structured metadata / specification | 333 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/input_matrix_validated_20260129_032536.png` | Generated screenshot | 33377 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/multi_input_test_20260129_222525.json` | Structured metadata / specification | 295 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/multi_input_test_20260129_222525.png` | Generated screenshot | 33376 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/ramp_default_20260129_054809.json` | Structured metadata / specification | 260 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/ramp_default_20260129_054809.png` | Generated screenshot | 33777 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/ramp_high_gain_20260129_054817.json` | Structured metadata / specification | 292 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/ramp_high_gain_20260129_054817.png` | Generated screenshot | 33946 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/red_isolation_20260129_054731.json` | Structured metadata / specification | 234 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/red_isolation_20260129_054731.png` | Generated screenshot | 33377 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/rotation_180deg_20260129_054759.json` | Structured metadata / specification | 242 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/rotation_180deg_20260129_054759.png` | Generated screenshot | 33402 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/rotation_90deg_20260129_054752.json` | Structured metadata / specification | 217 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_input_matrix/rotation_90deg_20260129_054752.png` | Generated screenshot | 33402 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_output_encoder/output_encoder_validated_20260129_032826.json` | Structured metadata / specification | 355 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_output_encoder/output_encoder_validated_20260129_032826.png` | Generated screenshot | 33389 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_ped_colorizer/ped_colorizer_validated_20260129_032701.json` | Structured metadata / specification | 383 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/paik_abe_ped_colorizer/ped_colorizer_validated_20260129_032701.png` | Generated screenshot | 33768 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ped_ca3019_clipped_20260129_044957.png` | Generated screenshot | 33448 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ped_ca3019_soft_20260129_044937.png` | Generated screenshot | 33761 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ped_ca3019_test_20260129_044917.png` | Generated screenshot | 33478 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ped_colorizer/ped_ramp_default_20260129_230312.json` | Structured metadata / specification | 256 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ped_colorizer/ped_ramp_default_20260129_230312.png` | Generated screenshot | 33378 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ped_colorizer/ped_ramp_hard_20260129_230324.json` | Structured metadata / specification | 244 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ped_colorizer/ped_ramp_hard_20260129_230324.png` | Generated screenshot | 33378 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ped_colorizer_hard_20260129_032641.png` | Generated screenshot | 33768 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ped_colorizer_ramp_20260129_032618.png` | Generated screenshot | 33768 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/ped_colorizer_ramp_v_20260126_034516.png` | Generated screenshot | 33555 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/rgb_colorbars_test_20260128_205433.png` | Generated screenshot | 33398 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/signal_chain_6stage_colorbars_20260126_044742.png` | Generated screenshot | 2368767 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/signal_chain_6stage_ramp_20260126_044713.png` | Generated screenshot | 2383467 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/signal_chain_colorbars_20260126_035139.png` | Generated screenshot | 83012 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/signal_chain_ramp_h_20260126_035211.png` | Generated screenshot | 472328 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/svideo_colorbars_test_20260128_205309.png` | Generated screenshot | 33397 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/ca3019_softness_sweep.gif` | Generated parameter sweep | 22267 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/ca3019_threshold_sweep.gif` | Generated parameter sweep | 92869 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/ca3022_bandwidth_sweep.gif` | Generated parameter sweep | 158675 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/ca3022_gain_sweep.gif` | Generated parameter sweep | 937102 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/ca3023_gain_sweep.gif` | Generated parameter sweep | 200945 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/ca3023_phase_sweep.gif` | Generated parameter sweep | 31621 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/ca3046_gain_sweep.gif` | Generated parameter sweep | 1223680 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/ca3054_keyclip_sweep.gif` | Generated parameter sweep | 78270 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/ca3054_rotation_sweep.gif` | Generated parameter sweep | 35603 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/ca3080_gain_sweep.gif` | Generated parameter sweep | 3132 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/folding_amp_gain_sweep.gif` | Generated parameter sweep | 172436 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/input_matrix_rotation_sweep.gif` | Generated parameter sweep | 32763 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/lm311_threshold_sweep.gif` | Generated parameter sweep | 86484 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/matrix_mixer_rotation_sweep.gif` | Generated parameter sweep | 35603 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/multi_input_rotation_sweep.gif` | Generated parameter sweep | 16005 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/ped_colorizer_threshold1_sweep.gif` | Generated parameter sweep | 36292 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/signal_chain_fold_sweep.gif` | Generated parameter sweep | 1151188 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/signal_chain_vidamp_gain_sweep.gif` | Generated parameter sweep | 86770 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/signal_chain_wobbulator_sweep.gif` | Generated parameter sweep | 32516 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/video_keyer_clip_sweep.gif` | Generated parameter sweep | 68663 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/wobbulator_amplitude_sweep.gif` | Generated parameter sweep | 443318 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/wobbulator_animation.gif` | Generated parameter sweep | 38007 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/sweeps/wobbulator_phase_sweep.gif` | Generated parameter sweep | 204310 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/video_keyer_colorbars_20260126_034735.png` | Generated screenshot | 33403 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/wobbulator/wobbulator_colorbars_20260125_074759.json` | Structured metadata / specification | 299 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/wobbulator/wobbulator_colorbars_20260125_074759.png` | Generated screenshot | 98070 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/wobbulator/wobbulator_intense_20260125_074811.json` | Structured metadata / specification | 254 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/wobbulator/wobbulator_intense_20260125_074811.png` | Generated screenshot | 106614 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/wobbulator/wobbulator_video_mod_20260125_074846.json` | Structured metadata / specification | 282 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/wobbulator/wobbulator_video_mod_20260125_074846.png` | Generated screenshot | 105963 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `docs/screenshots/wobbulator_checker_20260126_034843.png` | Generated screenshot | 1867662 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/DISCOVERY_GUIDE.md` | Local documentation / source description: Research Discovery Guide | 12534 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/RESEARCH_INDEX.md` | Local documentation / source description: Research Index | 12143 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/audits/paik_abe_device_audit_20260129.md` | Local documentation / source description: Paik/Abe Device Audit | 6229 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/audits/ped_colorizer_audit_20260129.md` | Local documentation / source description: PED Colorizer Detailed Audit | 5499 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/audits/ped_colorizer_verification_20260129.md` | Local documentation / source description: PED Colorizer CA3019 Verification Document | 4510 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/RCA_1975_Linear_ICs.pdf` | PDF source: 1975 RCA Linear ICs | 48180891 | Source origin not recorded in this index row; it records title/coverage. Appendix C contains source-family references. | Ignored (local only) |
| Observed `research/components/RCA_1975_Linear_ICs_pages_1-248.pdf` | PDF source: 1975 RCA Linear ICs | 15445827 | Source origin not recorded in this index row; it records title/coverage. Appendix C contains source-family references. | Tracked |
| Observed `research/components/RCA_1975_Linear_ICs_pages_249-496.pdf` | PDF source: 1975 RCA Linear ICs | 14906436 | Source origin not recorded in this index row; it records title/coverage. Appendix C contains source-family references. | Tracked |
| Observed `research/components/RCA_1975_Linear_ICs_pages_497-744.pdf` | PDF source: 1975 RCA Linear ICs | 17412601 | Source origin not recorded in this index row; it records title/coverage. Appendix C contains source-family references. | Tracked |
| Observed `research/components/RCA_1975_Linear_ICs_pages_745-761.pdf` | PDF source: 1975 RCA Linear ICs | 1124083 | Source origin not recorded in this index row; it records title/coverage. Appendix C contains source-family references. | Tracked |
| Observed `research/components/RCA_1978_Linear_IC_Databook.pdf` | PDF source: 1978 RCA Linear IC Databook | 34068255 | Source origin not recorded in this index row; it records title/coverage. Appendix C contains source-family references. | Ignored (local only) |
| Observed `research/components/RCA_1978_Linear_IC_Databook_pages_1-261.pdf` | PDF source: 1978 RCA Linear IC Databook | 15886857 | Source origin not recorded in this index row; it records title/coverage. Appendix C contains source-family references. | Tracked |
| Observed `research/components/RCA_1978_Linear_IC_Databook_pages_262-522.pdf` | PDF source: 1978 RCA Linear IC Databook | 14355935 | Source origin not recorded in this index row; it records title/coverage. Appendix C contains source-family references. | Tracked |
| Observed `research/components/RCA_1978_Linear_IC_Databook_pages_523-567.pdf` | PDF source: 1978 RCA Linear IC Databook | 3014976 | Source origin not recorded in this index row; it records title/coverage. Appendix C contains source-family references. | Tracked |
| Observed `research/components/README.md` | Local documentation / source description: Component Library | 4459 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/functional/folding_amp/v1/folder.wgsl` | WGSL model / demonstration shader | 4630 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/functional/folding_amp/v1/model.wgsl` | WGSL model / demonstration shader | 5745 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/functional/folding_amp/v1/provenance.json` | Structured metadata / specification | 203 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/functional/folding_amp/v1/spec.json` | Structured metadata / specification | 1824 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/functional/paik_abe_chain/v1/signal_chain.wgsl` | WGSL model / demonstration shader | 12622 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/functional/wobbulator/v1/model.wgsl` | WGSL model / demonstration shader | 6523 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/functional/wobbulator/v1/provenance.json` | Structured metadata / specification | 202 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/functional/wobbulator/v1/spec.json` | Structured metadata / specification | 2624 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/functional/wobbulator/v1/wobbulator.wgsl` | WGSL model / demonstration shader | 5325 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3019/PROVENANCE.md` | Local documentation / source description: CA3019 - Ultra-Fast Low-Capacitance Matched Diodes | 901 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3019/v1/colorizer.wgsl` | WGSL model / demonstration shader | 6843 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3019/v1/model.wgsl` | WGSL model / demonstration shader | 6484 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3019/v1/ped_control.wgsl` | WGSL model / demonstration shader | 9184 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3019/v1/provenance.json` | Structured metadata / specification | 191 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3019/v1/spec.json` | Structured metadata / specification | 1743 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3022/PROVENANCE.md` | Local documentation / source description: CA3022 - Low Power Wideband Amplifier | 851 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3022/v1/model.wgsl` | WGSL model / demonstration shader | 3698 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3022/v1/provenance.json` | Structured metadata / specification | 629 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3022/v1/spec.json` | Structured metadata / specification | 2947 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3022/v1/video_amp.wgsl` | WGSL model / demonstration shader | 5326 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3023/PROVENANCE.md` | Local documentation / source description: CA3023 - Low Power Wideband Amplifier | 895 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3023/v1/model.wgsl` | WGSL model / demonstration shader | 4111 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3023/v1/provenance.json` | Structured metadata / specification | 733 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3023/v1/spec.json` | Structured metadata / specification | 3626 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3023/v1/subcarrier_amp.wgsl` | WGSL model / demonstration shader | 6142 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3046/CA3046_Intersil_datasheet.pdf` | PDF source: CA3046 | 121465 | Claimed `research/RESEARCH_INDEX.md:103`: ivytechengineering.com | Tracked |
| Observed `research/components/ics/ca3046/v1/colorizer.wgsl` | WGSL model / demonstration shader | 3729 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3046/v1/model.wgsl` | WGSL model / demonstration shader | 3333 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3046/v1/provenance.json` | Structured metadata / specification | 191 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3046/v1/spec.json` | Structured metadata / specification | 2643 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3054/PROVENANCE.md` | Local documentation / source description: CA3054 - Dual Independent Differential Amplifier | 1058 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3054/v1/matrix_mixer.wgsl` | WGSL model / demonstration shader | 7039 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3054/v1/model.wgsl` | WGSL model / demonstration shader | 8566 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3054/v1/provenance.json` | Structured metadata / specification | 191 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3054/v1/spec.json` | Structured metadata / specification | 1705 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3054/v1/video_keyer.wgsl` | WGSL model / demonstration shader | 5322 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3080/CA3080_Application_Note_AN6668.pdf` | PDF source: CA3080 | 1978248 | Claimed `research/RESEARCH_INDEX.md:105`: intersil.com | Tracked |
| Observed `research/components/ics/ca3080/CA3080_Intersil_datasheet.pdf` | PDF source: CA3080 | 215969 | Claimed `research/RESEARCH_INDEX.md:104`: octopart.com | Tracked |
| Observed `research/components/ics/ca3080/v1/model.wgsl` | WGSL model / demonstration shader | 603 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3080/v1/provenance.json` | Structured metadata / specification | 191 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3080/v1/spec.json` | Structured metadata / specification | 265 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/ca3080/v1/vca.wgsl` | WGSL model / demonstration shader | 3136 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/lm311/LM311_TI_datasheet.pdf` | PDF source: LM311 | 2392958 | Claimed `research/RESEARCH_INDEX.md:106`: ti.com | Tracked |
| Observed `research/components/ics/lm311/v1/keyer.wgsl` | WGSL model / demonstration shader | 3727 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/lm311/v1/model.wgsl` | WGSL model / demonstration shader | 675 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/lm311/v1/provenance.json` | Structured metadata / specification | 190 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/components/ics/lm311/v1/spec.json` | Structured metadata / specification | 216 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/devices/README.md` | Local documentation / source description: Devices | 2015 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/devices/jones_colorizer/Jones_MVIP_Instructions.pdf` | PDF source: MVIP Instructions | 235134 | Claimed `research/RESEARCH_INDEX.md:77`: jonesvideo.com | Tracked |
| Observed `research/devices/paik_abe/ETC_PaikAbe_Budget_1972.pdf` | PDF source: Budget Document (1972) | 9541957 | Claimed `research/RESEARCH_INDEX.md:50`: archive.org/details/ETC3316 | Tracked |
| Observed `research/devices/paik_abe/ETC_PaikAbe_Restoration_2011.pdf` | PDF source: Restoration Project (2011) | 792782 | Claimed `research/RESEARCH_INDEX.md:49`: archive.org/details/ETC3310 | Tracked |
| Observed `research/devices/paik_abe/ETC_PaikAbe_Schematics.pdf` | PDF source: ETC Schematics | 1114235 | Claimed `research/RESEARCH_INDEX.md:48`: archive.org/details/ETC3311 | Tracked |
| Observed `research/devices/paik_abe/Paik Abe Video Synthesizer Technical Documentation.pdf` | PDF source: Technical Documentation (87 pages) | 53171994 | Claimed `research/RESEARCH_INDEX.md:46`: Pre-existing | Ignored (local only) |
| Observed `research/devices/paik_abe/Paik Abe Video Synthesizer Technical Documentation_pages_1-30.pdf` | PDF source: Technical Documentation (87 pages) | 18646515 | Claimed `research/RESEARCH_INDEX.md:46`: Pre-existing | Tracked |
| Observed `research/devices/paik_abe/Paik Abe Video Synthesizer Technical Documentation_pages_31-60.pdf` | PDF source: Technical Documentation (87 pages) | 17992891 | Claimed `research/RESEARCH_INDEX.md:46`: Pre-existing | Tracked |
| Observed `research/devices/paik_abe/Paik Abe Video Synthesizer Technical Documentation_pages_61-87.pdf` | PDF source: Technical Documentation (87 pages) | 16513601 | Claimed `research/RESEARCH_INDEX.md:46`: Pre-existing | Tracked |
| Observed `research/devices/paik_abe/Paik-Abe Video Synthesizer - Schematics.pdf` | PDF source: Schematics (26 pages) | 1114235 | Claimed `research/RESEARCH_INDEX.md:47`: Pre-existing | Tracked |
| Observed `research/devices/paik_abe/Vasulka_Eigenwelt_PaikAbe_126-129.pdf` | PDF source: Vasulka Eigenwelt pp.126-129 | 582400 | Claimed `research/RESEARCH_INDEX.md:51`: vasulka.org | Tracked |
| Observed `research/devices/paik_abe/architecture.md` | Local documentation / source description: Paik/Abe Video Synthesizer Architecture | 6664 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/devices/rutt_etra/ETC_RuttEtra_Panel.pdf` | PDF source: ETC Panel Documentation | 242250 | Claimed `research/RESEARCH_INDEX.md:71`: archive.org/details/ETC2530 | Tracked |
| Observed `research/devices/rutt_etra/Vasulka_Eigenwelt_RuttEtra_136-139.pdf` | PDF source: Eigenwelt pp.136-139 | 393868 | Claimed `research/RESEARCH_INDEX.md:70`: vasulka.org | Tracked |
| Observed `research/devices/rutt_etra/Vasulka_RuttEtra_General.pdf` | PDF source: General Documentation | 2607999 | Claimed `research/RESEARCH_INDEX.md:68`: vasulka.org | Tracked |
| Observed `research/devices/rutt_etra/Vasulka_RuttEtra_VidSynthCont.pdf` | PDF source: Video Synth Continuation | 2756779 | Claimed `research/RESEARCH_INDEX.md:69`: vasulka.org | Tracked |
| Observed `research/devices/sandin_ip/Connolly_Guide_SandinIP_screen.pdf` | PDF source: Connolly Guide (reformatted) | 7963126 | Claimed `research/RESEARCH_INDEX.md:62`: jameshconnolly.com | Tracked |
| Observed `research/devices/sandin_ip/ETC_SandinIP_Manual.pdf` | PDF source: ETC Manual (full schematics) | 111581323 | Claimed `research/RESEARCH_INDEX.md:61`: archive.org/details/ETC1365 | Ignored (local only) |
| Observed `research/devices/sandin_ip/ETC_SandinIP_Manual_pages_1-8.pdf` | PDF source: ETC Manual (full schematics) | 15175196 | Claimed `research/RESEARCH_INDEX.md:61`: archive.org/details/ETC1365 | Tracked |
| Observed `research/devices/sandin_ip/ETC_SandinIP_Manual_pages_17-24.pdf` | PDF source: ETC Manual (full schematics) | 13975739 | Claimed `research/RESEARCH_INDEX.md:61`: archive.org/details/ETC1365 | Tracked |
| Observed `research/devices/sandin_ip/ETC_SandinIP_Manual_pages_25-32.pdf` | PDF source: ETC Manual (full schematics) | 13876156 | Claimed `research/RESEARCH_INDEX.md:61`: archive.org/details/ETC1365 | Tracked |
| Observed `research/devices/sandin_ip/ETC_SandinIP_Manual_pages_33-40.pdf` | PDF source: ETC Manual (full schematics) | 14057836 | Claimed `research/RESEARCH_INDEX.md:61`: archive.org/details/ETC1365 | Tracked |
| Observed `research/devices/sandin_ip/ETC_SandinIP_Manual_pages_41-48.pdf` | PDF source: ETC Manual (full schematics) | 14356851 | Claimed `research/RESEARCH_INDEX.md:61`: archive.org/details/ETC1365 | Tracked |
| Observed `research/devices/sandin_ip/ETC_SandinIP_Manual_pages_49-56.pdf` | PDF source: ETC Manual (full schematics) | 13999973 | Claimed `research/RESEARCH_INDEX.md:61`: archive.org/details/ETC1365 | Tracked |
| Observed `research/devices/sandin_ip/ETC_SandinIP_Manual_pages_57-63.pdf` | PDF source: ETC Manual (full schematics) | 11844550 | Claimed `research/RESEARCH_INDEX.md:61`: archive.org/details/ETC1365 | Tracked |
| Observed `research/devices/sandin_ip/ETC_SandinIP_Manual_pages_9-16.pdf` | PDF source: ETC Manual (full schematics) | 14279741 | Claimed `research/RESEARCH_INDEX.md:61`: archive.org/details/ETC1365 | Tracked |
| Observed `research/devices/sandin_ip/Sandin_Distribution_Religion.pdf` | PDF source: Distribution Religion Manual | 34746828 | Claimed `research/RESEARCH_INDEX.md:60`: criticalartware.net | Ignored (local only) |
| Observed `research/devices/sandin_ip/Sandin_Distribution_Religion_pages_1-52.pdf` | PDF source: Distribution Religion Manual | 14505167 | Claimed `research/RESEARCH_INDEX.md:60`: criticalartware.net | Tracked |
| Observed `research/devices/sandin_ip/Sandin_Distribution_Religion_pages_105-117.pdf` | PDF source: Distribution Religion Manual | 4416860 | Claimed `research/RESEARCH_INDEX.md:60`: criticalartware.net | Tracked |
| Observed `research/devices/sandin_ip/Sandin_Distribution_Religion_pages_53-104.pdf` | PDF source: Distribution Religion Manual | 15736691 | Claimed `research/RESEARCH_INDEX.md:60`: criticalartware.net | Tracked |
| Observed `research/innovators/README.md` | Local documentation / source description: Innovators | 3498 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/innovators/beck_stephen/Beck_Image_Processing_Video_Synthesis.pdf` | PDF source: Beck: Image Processing and Video Synthesis | 1408311 | Claimed `research/RESEARCH_INDEX.md:155`: vasulka.org | Tracked |
| Observed `research/innovators/paik_abe/Meigh-Andrews_Development_Functions_Legacy_PaikAbe.pdf` | PDF source: Meigh-Andrews: Development, Functions & Legacy of Paik-Abe | 198236 | Claimed `research/RESEARCH_INDEX.md:152`: NJP Art Center | Tracked |
| Observed `research/innovators/paik_abe/Vasulka_Eigenwelt_ArsElectronica_AE043.pdf` | PDF source: Vasulka Eigenwelt: Ars Electronica catalogue | 723686 | Claimed `research/RESEARCH_INDEX.md:153`: vasulka.org | Tracked |
| Observed `research/innovators/paik_abe/Vasulka_Eigenwelt_Intro_001-009.pdf` | PDF source: Vasulka Eigenwelt: Introduction pp.1-9 | 290034 | Claimed `research/RESEARCH_INDEX.md:154`: vasulka.org | Tracked |
| Observed `research/prototypes/README.md` | Local documentation / source description: Prototypes | 1359 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/prototypes/paik_abe_colorizer_v1/spec.json` | Structured metadata / specification | 2610 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/prototypes/paik_abe_folding_amp_v1/spec.json` | Structured metadata / specification | 2183 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/provenance.db` | SQLite provenance records | 86016 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/schematics/rutt-etra/metadata.json` | Structured metadata / specification | 134 | Local project record; creator/date where present in Appendix B/C | Untracked (local only) |
| Observed `research/schematics/rutt-etra/schematic.pdf` | PDF source (filename title) | 954013 | Claimed research/schematics/rutt-etra/metadata.json:3, ETC2731 | Untracked (local only) |
| Observed `research/techniques/DAFx17_Buchla259_Wavefolder.pdf` | PDF source: DAFx-17: Virtual Analog Buchla 259 Wavefolder | 1563941 | Source origin not recorded in this index row; it records title/coverage. Appendix C contains source-family references. | Tracked |
| Observed `research/techniques/Kane_Synthetic_Color_Video_Synthesis_1969.pdf` | PDF source: Kane: Synthetic Color & Video Synthesis Circa 1969 | 1016425 | Source origin not recorded in this index row; it records title/coverage. Appendix C contains source-family references. | Tracked |
| Observed `research/techniques/README.md` | Local documentation / source description: Techniques | 3215 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/techniques/SMC17_Lockhart_Wavefolder.pdf` | PDF source: SMC-17: Virtual Analog Lockhart Wavefolder | 346584 | Source origin not recorded in this index row; it records title/coverage. Appendix C contains source-family references. | Tracked |
| Observed `research/techniques/Stromeko_OTA_Overview.pdf` | PDF source: Stromeko: OTA Overview for Synth DIY | 419260 | Source origin not recorded in this index row; it records title/coverage. Appendix C contains source-family references. | Tracked |
| Observed `research/techniques/TI_Demystifying_OTA_sboa117a.pdf` | PDF source: TI: Demystifying the OTA | 266159 | Source origin not recorded in this index row; it records title/coverage. Appendix C contains source-family references. | Tracked |
| Observed `research/techniques/cable_degradation/README.md` | Local documentation / source description: Cable Degradation Research | 1764 | Local project record; creator/date where present in Appendix B/C | Tracked |
| Observed `research/validation/device_validation.json` | Structured metadata / specification | 4728 | Local project record; creator/date where present in Appendix B/C | Tracked |

## Appendix B. Metadata fields and provenance values

Observed: each JSON file below was parsed; nested object paths and array-item fields are listed, preserving the fields rather than inferring a schema. For source/provenance/spec records, source-bearing values follow. Capture JSONs are local workbench records; no file-specific third-party origin is inferred. Evidence is the named JSON file and key; line positions of source-bearing keys are included.


Observed `docs/screenshots/ca3019/ca3019_ped_control_colorbars_20260125_074158.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/ca3019/ca3019_ped_control_ramp_20260125_074148.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/ca3046/ca3046_colorizer_colorbars_20260125_074043.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/ca3046/ca3046_colorizer_ramp_20260125_074032.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/ca3054/ca3054_keyer_colorbars_20260125_074415.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/ca3054/ca3054_matrix_colorbars_20260125_074312.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/ca3054/ca3054_matrix_rotated_20260125_074322.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/ca3080/ca3080_vca_colorbars_20260125_074602.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/ca3080/ca3080_vca_modulated_20260125_074613.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/colorizer/colorizer_colorbars_20260129_230159.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/colorizer/colorizer_ramp_20260129_230150.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/folding_amp/fold_gain2_20260129_230013.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/folding_amp/fold_gain4_20260129_230022.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/folding_amp/folding_amp_ramp_3x_20260125_074504.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/folding_amp/folding_amp_ramp_6x_20260125_074514.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/lm311/lm311_keyer_window_20260125_074701.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/output_encoder/output_neutral_20260129_230430.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/output_encoder/output_warm_20260129_230440.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik-abe/ca3022_colorbars_default_20260126_044435.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik-abe/ca3023_colorbars_default_20260126_044536.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik-abe/folding_amp_default_20260126_034654.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik-abe/matrix_mixer_default_20260126_034424.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik-abe/ped_colorizer_default_20260126_034547.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik-abe/signal_chain_6stage_colorbars_20260126_044742.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik-abe/signal_chain_6stage_ramp_20260126_044725.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik-abe/signal_chain_colorbars_20260126_035333.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik-abe/signal_chain_ramp_20260126_035307.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik-abe/video_keyer_default_20260126_034808.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik-abe/wobbulator_default_20260126_034917.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik_abe/folding_amplifier_20260125_063503.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik_abe/ped_control_colorizer_20260125_061443.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik_abe/video_keyer_20260125_062533.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik_abe_folding_amp/folding_amp_default_ramp_20260129_031627.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik_abe_folding_amp/folding_amp_validated_20260129_032314.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik_abe_input_matrix/7input_default_20260129_222431.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik_abe_input_matrix/blue_isolation_20260129_054746.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik_abe_input_matrix/default_colorbars_20260129_054724.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik_abe_input_matrix/green_isolation_20260129_054739.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik_abe_input_matrix/input_matrix_validated_20260129_032536.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik_abe_input_matrix/multi_input_test_20260129_222525.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik_abe_input_matrix/ramp_default_20260129_054809.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik_abe_input_matrix/ramp_high_gain_20260129_054817.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik_abe_input_matrix/red_isolation_20260129_054731.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik_abe_input_matrix/rotation_180deg_20260129_054759.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik_abe_input_matrix/rotation_90deg_20260129_054752.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik_abe_output_encoder/output_encoder_validated_20260129_032826.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/paik_abe_ped_colorizer/ped_colorizer_validated_20260129_032701.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/ped_colorizer/ped_ramp_default_20260129_230312.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/ped_colorizer/ped_ramp_hard_20260129_230324.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/wobbulator/wobbulator_colorbars_20260125_074759.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/wobbulator/wobbulator_intense_20260125_074811.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `docs/screenshots/wobbulator/wobbulator_video_mod_20260125_074846.json`: fields `name`, `device`, `description`, `timestamp`, `image_file`.

Observed `research/components/functional/folding_amp/v1/provenance.json`: fields `component`, `version`, `created_at`, `created_by`, `notes`, `source_artifacts`.
Observed stored values (claims by the record author):
```json
{
  "component": "functional/folding_amp",
  "version": 1,
  "created_at": "2026-01-25T09:19:26.518354",
  "created_by": "migration",
  "notes": "Migrated from flat structure",
  "source_artifacts": []
}
```

Observed `research/components/functional/folding_amp/v1/spec.json`: fields `name`, `type`, `description`, `behavior`, `behavior.summary`, `behavior.visual_effect`, `behavior.audio_analog`, `transfer_function`, `transfer_function.type`, `transfer_function.basic_equation`, `transfer_function.with_gain`, `transfer_function.soft_fold`, `transfer_function.parameters`, `transfer_function.parameters.gain`, `transfer_function.parameters.symmetry`, `transfer_function.parameters.softness`, `implementations`, `implementations.paik_abe`, `implementations.paik_abe.description`, `implementations.paik_abe.components`, `implementations.paik_abe.characteristics`, `implementations.serge`, `implementations.serge.description`, `implementations.serge.characteristics`, `implementations.buchla`, `implementations.buchla.description`, `implementations.buchla.characteristics`, `applications`, `used_in`.

Observed `research/components/functional/wobbulator/v1/provenance.json`: fields `component`, `version`, `created_at`, `created_by`, `notes`, `source_artifacts`.
Observed stored values (claims by the record author):
```json
{
  "component": "functional/wobbulator",
  "version": 1,
  "created_at": "2026-01-25T09:19:26.527016",
  "created_by": "migration",
  "notes": "Migrated from flat structure",
  "source_artifacts": []
}
```

Observed `research/components/functional/wobbulator/v1/spec.json`: fields `name`, `type`, `description`, `origin`, `behavior`, `behavior.summary`, `behavior.horizontal`, `behavior.vertical`, `behavior.video_rate`, `modulation_sources`, `modulation_sources.internal_oscillator`, `modulation_sources.internal_oscillator.description`, `modulation_sources.internal_oscillator.frequency_range_hz`, `modulation_sources.internal_oscillator.waveforms`, `modulation_sources.video_signal`, `modulation_sources.video_signal.description`, `modulation_sources.video_signal.effect`, `modulation_sources.external_cv`, `modulation_sources.external_cv.description`, `modulation_sources.external_cv.range`, `parameters`, `parameters.h_frequency`, `parameters.h_amplitude`, `parameters.h_phase`, `parameters.v_frequency`, `parameters.v_amplitude`, `parameters.v_phase`, `parameters.video_mod_h`, `parameters.video_mod_v`, `transfer_function`, `transfer_function.type`, `transfer_function.equation_h`, `transfer_function.equation_v`, `transfer_function.notes`, `visual_effects`, `historical_notes`, `used_in`.

Observed `research/components/ics/ca3019/v1/provenance.json`: fields `component`, `version`, `created_at`, `created_by`, `notes`, `source_artifacts`.
Observed stored values (claims by the record author):
```json
{
  "component": "ics/ca3019",
  "version": 1,
  "created_at": "2026-01-25T09:19:26.537334",
  "created_by": "migration",
  "notes": "Migrated from flat structure",
  "source_artifacts": []
}
```

Observed `research/components/ics/ca3019/v1/spec.json`: fields `name`, `type`, `manufacturer`, `description`, `package`, `specs`, `specs.diode_count`, `specs.configuration`, `specs.vf_typical_v`, `specs.vf_match_mv`, `specs.capacitance_pf`, `specs.reverse_recovery_ns`, `specs.ir_max_na`, `specs.if_max_ma`, `specs.vr_max_v`, `pinout`, `pinout.1`, `pinout.2`, `pinout.3`, `pinout.4`, `pinout.5`, `pinout.6`, `pinout.7`, `pinout.8`, `pinout.9`, `pinout.10`, `transfer_function`, `transfer_function.type`, `transfer_function.equation`, `transfer_function.parameters`, `transfer_function.parameters.Is`, `transfer_function.parameters.n`, `transfer_function.parameters.Vt`, `transfer_function.parameters.Vf`, `transfer_function.notes`, `amplitude_window_function`, `amplitude_window_function.description`, `amplitude_window_function.equation`, `amplitude_window_function.notes`, `used_in`, `sources`.
Claimed `research/components/ics/ca3019/v1/spec.json:56`: `"sources": [`

Observed `research/components/ics/ca3022/v1/provenance.json`: fields `component`, `version`, `created_at`, `created_by`, `notes`, `source_artifacts`.
Observed stored values (claims by the record author):
```json
{
  "component": "ics/ca3022",
  "version": 1,
  "created_at": "2026-01-26T00:00:00.000000",
  "created_by": "claude-opus-4.5",
  "notes": "CA3022 wideband video amplifier model. Single-stage broadband amp from RCA CA3020 family. Models gain, bandwidth rolloff, slew limiting, soft saturation, and noise floor. Used in Paik/Abe as chroma key video output amplifier (schematic pages 10, 20).",
  "source_artifacts": [
    "research/devices/paik_abe/Paik Abe Video Synthesizer Technical Documentation.pdf",
    "research/devices/paik_abe/ETC_PaikAbe_Schematics.pdf",
    "research/components/RCA_1978_Linear_IC_Databook.pdf"
  ]
}
```

Observed `research/components/ics/ca3022/v1/spec.json`: fields `name`, `type`, `manufacturer`, `description`, `package`, `era`, `family`, `specs`, `specs.supply_voltage_max_v`, `specs.supply_voltage_typ_v`, `specs.supply_voltage_min_v`, `specs.quiescent_current_ma`, `specs.power_dissipation_mw`, `specs.voltage_gain_min_db`, `specs.voltage_gain_max_db`, `specs.bandwidth_mhz`, `specs.slew_rate_v_us`, `specs.input_impedance_kohm`, `specs.output_impedance_ohm`, `specs.output_swing_v_pp`, `specs.noise_figure_db`, `specs.operating_temp_min_c`, `specs.operating_temp_max_c`, `pinout`, `pinout.1`, `pinout.2`, `pinout.3`, `pinout.4`, `pinout.5`, `pinout.6`, `pinout.7`, `pinout.8`, `transfer_function`, `transfer_function.type`, `transfer_function.equation`, `transfer_function.bandwidth_model`, `transfer_function.saturation_model`, `transfer_function.slew_rate_model`, `transfer_function.parameters`, `transfer_function.parameters.gain_min`, `transfer_function.parameters.gain_max`, `transfer_function.parameters.f_3dB_mhz`, `transfer_function.parameters.slew_rate_v_us`, `transfer_function.parameters.headroom_v`, `external_components`, `external_components.R1`, `external_components.R2`, `external_components.C1`, `external_components.C2`, `external_components.C3`, `applications`, `used_in`, `notes`, `substitutes`, `substitutes[].part`, `substitutes[].notes`, `datasheet_source`, `sources`.
Claimed `research/components/ics/ca3022/v1/spec.json:90`: `"datasheet_source": "1978 RCA Linear IC Databook",`
Claimed `research/components/ics/ca3022/v1/spec.json:91`: `"sources": [`

Observed `research/components/ics/ca3023/v1/provenance.json`: fields `component`, `version`, `created_at`, `created_by`, `notes`, `source_artifacts`.
Observed stored values (claims by the record author):
```json
{
  "component": "ics/ca3023",
  "version": 1,
  "created_at": "2026-01-26T00:00:00.000000",
  "created_by": "claude-opus-4.5",
  "notes": "CA3023 high-gain wideband amplifier model. Two-stage broadband amp from RCA CA3020 family with bandpass characteristic peaked around NTSC 3.58 MHz subcarrier. Models cascaded gain stages, bandpass filtering, phase rotation (hue control), black level clamping, and drive saturation. Used in Paik/Abe as subcarrier amplifier (schematic pages 3, 4, 26, 30).",
  "source_artifacts": [
    "research/devices/paik_abe/Paik Abe Video Synthesizer Technical Documentation.pdf",
    "research/devices/paik_abe/ETC_PaikAbe_Schematics.pdf",
    "research/components/RCA_1978_Linear_IC_Databook.pdf"
  ]
}
```

Observed `research/components/ics/ca3023/v1/spec.json`: fields `name`, `type`, `manufacturer`, `description`, `package`, `era`, `family`, `specs`, `specs.supply_voltage_max_v`, `specs.supply_voltage_typ_v`, `specs.supply_voltage_min_v`, `specs.quiescent_current_ma`, `specs.power_dissipation_mw`, `specs.voltage_gain_min_db`, `specs.voltage_gain_max_db`, `specs.bandwidth_low_mhz`, `specs.bandwidth_high_mhz`, `specs.center_frequency_mhz`, `specs.slew_rate_v_us`, `specs.input_impedance_kohm`, `specs.output_impedance_ohm`, `specs.output_swing_v_pp`, `specs.noise_figure_db`, `specs.phase_shift_range_deg`, `specs.operating_temp_min_c`, `specs.operating_temp_max_c`, `pinout`, `pinout.1`, `pinout.2`, `pinout.3`, `pinout.4`, `pinout.5`, `pinout.6`, `pinout.7`, `pinout.8`, `pinout.9`, `pinout.10`, `pinout.11`, `pinout.12`, `internal_structure`, `internal_structure.stages`, `internal_structure.stage1`, `internal_structure.stage2`, `internal_structure.total_transistors`, `internal_structure.description`, `transfer_function`, `transfer_function.type`, `transfer_function.equation`, `transfer_function.bandpass_model`, `transfer_function.phase_model`, `transfer_function.clamp_model`, `transfer_function.parameters`, `transfer_function.parameters.gain1_typical`, `transfer_function.parameters.gain2_typical`, `transfer_function.parameters.f_center_mhz`, `transfer_function.parameters.Q_typical`, `transfer_function.parameters.phase_range_deg`, `transfer_function.parameters.clamp_level_v`, `external_components`, `external_components.R1`, `external_components.R2`, `external_components.C1`, `external_components.C2`, `external_components.L1`, `applications`, `used_in`, `notes`, `substitutes`, `substitutes[].part`, `substitutes[].notes`, `datasheet_source`, `sources`.
Claimed `research/components/ics/ca3023/v1/spec.json:106`: `"datasheet_source": "1978 RCA Linear IC Databook",`
Claimed `research/components/ics/ca3023/v1/spec.json:107`: `"sources": [`

Observed `research/components/ics/ca3046/v1/provenance.json`: fields `component`, `version`, `created_at`, `created_by`, `notes`, `source_artifacts`.
Observed stored values (claims by the record author):
```json
{
  "component": "ics/ca3046",
  "version": 1,
  "created_at": "2026-01-25T09:19:26.545064",
  "created_by": "migration",
  "notes": "Migrated from flat structure",
  "source_artifacts": []
}
```

Observed `research/components/ics/ca3046/v1/spec.json`: fields `name`, `aliases`, `type`, `manufacturer`, `description`, `package`, `era`, `specs`, `specs.vce_max_v`, `specs.vceo_max_v`, `specs.ic_max_ma`, `specs.power_max_mw`, `specs.hfe_min`, `specs.hfe_typical`, `specs.hfe_max`, `specs.ft_mhz`, `specs.vbe_typical_v`, `specs.thermal_resistance_jc`, `specs.matching_vbe_mv`, `pinout`, `pinout.1`, `pinout.2`, `pinout.3`, `pinout.4`, `pinout.5`, `pinout.6`, `pinout.7`, `pinout.8`, `pinout.9`, `pinout.10`, `pinout.11`, `pinout.12`, `pinout.13`, `transistor_model`, `transistor_model.type`, `transistor_model.is`, `transistor_model.bf`, `transistor_model.nf`, `transistor_model.vaf`, `transistor_model.ikf`, `transistor_model.ise`, `transistor_model.ne`, `transistor_model.br`, `transistor_model.nr`, `transistor_model.var`, `transistor_model.ikr`, `transistor_model.isc`, `transistor_model.nc`, `transistor_model.rb`, `transistor_model.irb`, `transistor_model.rbm`, `transistor_model.re`, `transistor_model.rc`, `transistor_model.cje`, `transistor_model.vje`, `transistor_model.mje`, `transistor_model.tf`, `transistor_model.xtf`, `transistor_model.vtf`, `transistor_model.itf`, `transistor_model.ptf`, `transistor_model.cjc`, `transistor_model.vjc`, `transistor_model.mjc`, `transistor_model.xcjc`, `transistor_model.tr`, `transistor_model.cjs`, `transistor_model.vjs`, `transistor_model.mjs`, `transistor_model.xtb`, `transistor_model.eg`, `transistor_model.xti`, `transistor_model.kf`, `transistor_model.af`, `transistor_model.fc`, `transfer_function`, `transfer_function.description`, `transfer_function.equation`, `transfer_function.simplified_wgsl`, `transfer_function.parameters`, `transfer_function.parameters.Is`, `transfer_function.parameters.Vt`, `transfer_function.parameters.Vaf`, `common_uses`, `used_in_devices`, `substitutes`, `substitutes[].part`, `substitutes[].notes`, `datasheet_source`, `spice_model_source`.
Claimed `research/components/ics/ca3046/v1/spec.json:117`: `"datasheet_source": "1978 RCA Linear IC Databook",`

Observed `research/components/ics/ca3054/v1/provenance.json`: fields `component`, `version`, `created_at`, `created_by`, `notes`, `source_artifacts`.
Observed stored values (claims by the record author):
```json
{
  "component": "ics/ca3054",
  "version": 1,
  "created_at": "2026-01-25T09:19:26.554088",
  "created_by": "migration",
  "notes": "Migrated from flat structure",
  "source_artifacts": []
}
```

Observed `research/components/ics/ca3054/v1/spec.json`: fields `name`, `type`, `manufacturer`, `description`, `package`, `specs`, `specs.supply_voltage_max`, `specs.supply_voltage_min`, `specs.input_offset_voltage_mv`, `specs.input_bias_current_ua`, `specs.open_loop_gain_db`, `specs.bandwidth_mhz`, `specs.slew_rate_v_us`, `specs.cmrr_db`, `specs.output_swing_v`, `pinout`, `pinout.1`, `pinout.2`, `pinout.3`, `pinout.4`, `pinout.5`, `pinout.6`, `pinout.7`, `pinout.8`, `internal_structure`, `internal_structure.amplifiers`, `internal_structure.transistors_per_amp`, `internal_structure.current_sources`, `internal_structure.description`, `transfer_function`, `transfer_function.type`, `transfer_function.equation`, `transfer_function.diff_pair_transfer`, `transfer_function.parameters`, `transfer_function.parameters.Vt`, `transfer_function.parameters.gm_typical`, `transfer_function.parameters.Itail_typical_ma`, `applications`, `used_in`, `notes`.

Observed `research/components/ics/ca3080/v1/provenance.json`: fields `component`, `version`, `created_at`, `created_by`, `notes`, `source_artifacts`.
Observed stored values (claims by the record author):
```json
{
  "component": "ics/ca3080",
  "version": 1,
  "created_at": "2026-01-25T09:19:26.562017",
  "created_by": "migration",
  "notes": "Migrated from flat structure",
  "source_artifacts": []
}
```

Observed `research/components/ics/ca3080/v1/spec.json`: fields `name`, `type`, `manufacturer`, `description`, `specs`, `specs.supply_v`, `specs.gm_max`, `specs.ibias_max_ma`, `datasheet_source`.
Claimed `research/components/ics/ca3080/v1/spec.json:11`: `"datasheet_source": "1978 RCA Linear IC Databook"`

Observed `research/components/ics/lm311/v1/provenance.json`: fields `component`, `version`, `created_at`, `created_by`, `notes`, `source_artifacts`.
Observed stored values (claims by the record author):
```json
{
  "component": "ics/lm311",
  "version": 1,
  "created_at": "2026-01-25T09:19:26.570997",
  "created_by": "migration",
  "notes": "Migrated from flat structure",
  "source_artifacts": []
}
```

Observed `research/components/ics/lm311/v1/spec.json`: fields `name`, `type`, `manufacturer`, `description`, `specs`, `specs.supply_v`, `specs.response_ns`, `datasheet_source`.
Claimed `research/components/ics/lm311/v1/spec.json:10`: `"datasheet_source": "TI Datasheet"`

Observed `research/prototypes/paik_abe_colorizer_v1/spec.json`: fields `name`, `version`, `type`, `status`, `created_date`, `classification_date`, `description`, `authenticity`, `authenticity.is_authentic`, `authenticity.reason`, `authenticity.schematic_basis`, `authenticity.notes`, `technical`, `technical.approach`, `technical.transfer_function`, `technical.claimed_components`, `technical.actual_paik_abe_components`, `parameters`, `parameters.gain_r`, `parameters.gain_r.range`, `parameters.gain_r.default`, `parameters.gain_r.description`, `parameters.gain_g`, `parameters.gain_g.range`, `parameters.gain_g.default`, `parameters.gain_g.description`, `parameters.gain_b`, `parameters.gain_b.range`, `parameters.gain_b.default`, `parameters.gain_b.description`, `parameters.fold_depth`, `parameters.fold_depth.range`, `parameters.fold_depth.default`, `parameters.fold_depth.description`, `parameters.saturation`, `parameters.saturation.range`, `parameters.saturation.default`, `parameters.saturation.description`, `parameters.slew`, `parameters.slew.range`, `parameters.slew.default`, `parameters.slew.description`, `artistic_value`, `artistic_value.visual_character`, `artistic_value.use_cases`, `artistic_value.comparison_to_authentic`, `files`, `files.shader`, `files.device`, `files.screenshots`, `future`, `future.keep_available`, `future.reason`, `future.potential_evolution`.
Claimed `research/prototypes/paik_abe_colorizer_v1/spec.json:14`: `"schematic_basis": "None - creative interpretation",`

Observed `research/prototypes/paik_abe_folding_amp_v1/spec.json`: fields `name`, `version`, `type`, `status`, `created_date`, `classification_date`, `description`, `authenticity`, `authenticity.is_authentic`, `authenticity.reason`, `authenticity.schematic_basis`, `authenticity.notes`, `technical`, `technical.approach`, `technical.transfer_function`, `technical.features`, `parameters`, `parameters.gain`, `parameters.gain.range`, `parameters.gain.default`, `parameters.gain.description`, `parameters.depth`, `parameters.depth.range`, `parameters.depth.default`, `parameters.depth.description`, `parameters.bias`, `parameters.bias.range`, `parameters.bias.default`, `parameters.bias.description`, `artistic_value`, `artistic_value.visual_character`, `artistic_value.use_cases`, `artistic_value.note`, `files`, `files.shader`, `files.device_struct`, `future`, `future.keep_available`, `future.reason`, `future.potential_rename`.
Claimed `research/prototypes/paik_abe_folding_amp_v1/spec.json:14`: `"schematic_basis": "None - creative interpretation based on audio synthesis concepts",`

Observed `research/schematics/rutt-etra/metadata.json`: fields `device`, `source_url`, `retrieved_at`.
Observed stored values (claims by the record author):
```json
{
  "device": "rutt-etra",
  "source_url": "https://archive.org/details/ETC2731",
  "retrieved_at": "2026-02-02T04:12:37.263145"
}
```

Observed `research/validation/device_validation.json`: fields `schema_version`, `description`, `devices`, `devices.test_signal`, `devices.test_signal.validated`, `devices.test_signal.validation_date`, `devices.test_signal.validation_notes`, `devices.test_signal.reference_doc`, `devices.test_signal.screenshots`, `devices.paik_abe_colorizer`, `devices.paik_abe_colorizer.validated`, `devices.paik_abe_colorizer.validation_date`, `devices.paik_abe_colorizer.validation_notes`, `devices.paik_abe_colorizer.reference_doc`, `devices.paik_abe_colorizer.prototype_spec`, `devices.paik_abe_colorizer.screenshots`, `devices.paik_abe_colorizer.authenticity`, `devices.paik_abe_colorizer.authenticity.is_authentic`, `devices.paik_abe_colorizer.authenticity.classification`, `devices.paik_abe_colorizer.authenticity.authentic_alternative`, `devices.paik_abe_wobulator`, `devices.paik_abe_wobulator.validated`, `devices.paik_abe_wobulator.validation_date`, `devices.paik_abe_wobulator.validation_notes`, `devices.paik_abe_wobulator.reference_doc`, `devices.paik_abe_wobulator.screenshots`, `devices.paik_abe_input_matrix`, `devices.paik_abe_input_matrix.validated`, `devices.paik_abe_input_matrix.validation_date`, `devices.paik_abe_input_matrix.validation_notes`, `devices.paik_abe_input_matrix.reference_doc`, `devices.paik_abe_input_matrix.screenshots`, `devices.paik_abe_folding_amp`, `devices.paik_abe_folding_amp.validated`, `devices.paik_abe_folding_amp.validation_date`, `devices.paik_abe_folding_amp.validation_notes`, `devices.paik_abe_folding_amp.reference_doc`, `devices.paik_abe_folding_amp.prototype_spec`, `devices.paik_abe_folding_amp.screenshots`, `devices.paik_abe_folding_amp.authenticity`, `devices.paik_abe_folding_amp.authenticity.is_authentic`, `devices.paik_abe_folding_amp.authenticity.classification`, `devices.paik_abe_folding_amp.authenticity.notes`, `devices.paik_abe_ped_colorizer`, `devices.paik_abe_ped_colorizer.validated`, `devices.paik_abe_ped_colorizer.validation_date`, `devices.paik_abe_ped_colorizer.validation_notes`, `devices.paik_abe_ped_colorizer.reference_doc`, `devices.paik_abe_ped_colorizer.screenshots`, `devices.paik_abe_output_encoder`, `devices.paik_abe_output_encoder.validated`, `devices.paik_abe_output_encoder.validation_date`, `devices.paik_abe_output_encoder.validation_notes`, `devices.paik_abe_output_encoder.reference_doc`, `devices.paik_abe_output_encoder.screenshots`, `validation_requirements`, `validation_requirements.minimum_tests`, `validation_requirements.required_artifacts`, `validation_requirements.documentation`.
Claimed `research/validation/device_validation.json:10`: `"reference_doc": null,`
Claimed `research/validation/device_validation.json:18`: `"reference_doc": null,`
Claimed `research/validation/device_validation.json:35`: `"reference_doc": "research/components/functional/wobbulator/v1/spec.json",`
Claimed `research/validation/device_validation.json:47`: `"reference_doc": "research/components/ics/ca3054/v1/spec.json",`
Claimed `research/validation/device_validation.json:57`: `"reference_doc": null,`
Claimed `research/validation/device_validation.json:73`: `"reference_doc": "research/components/ics/ca3019/v1/spec.json",`
Claimed `research/validation/device_validation.json:83`: `"reference_doc": null,`

Observed command: `sqlite3.connect("file:research/provenance.db?mode=ro&immutable=1", uri=True)` then sqlite_master, PRAGMA table_info and SELECT count(*) for each table; exit 0. Real output:
- Observed `artifacts`: 0 rows; columns `id`, `type`, `name`, `path`, `hash`, `source_url`, `retrieved_at`, `metadata`.
- Observed `versions`: 0 rows; columns `id`, `artifact_id`, `version`, `created_at`, `created_by`, `commit_hash`, `notes`.
- Observed `rd_passes`: 0 rows; columns `id`, `name`, `device`, `started_at`, `completed_at`, `goals`, `outcomes`.
- Observed `pass_artifacts`: 0 rows; columns `pass_id`, `artifact_id`, `role`.
- Observed `sources`: 0 rows; columns `id`, `url`, `name`, `type`, `reliability`, `last_checked`, `notes`.
- Observed `relationships`: 0 rows; columns `from_id`, `to_id`, `relationship`.


### Source-bearing JSON values

Observed stored values below are claims by each record author; each key is qualified by its JSON file.
- Claimed `research/components/functional/folding_amp/v1/provenance.json` key `source_artifacts`: []
- Claimed `research/components/functional/wobbulator/v1/provenance.json` key `source_artifacts`: []
- Claimed `research/components/functional/wobbulator/v1/spec.json` key `modulation_sources`: {"internal_oscillator": {"description": "Built-in sine/triangle oscillators", "frequency_range_hz": [0.1, 1000], "waveforms": ["sine", "triangle", "square", "sawtooth"]}, "video_signal": {"description": "Luminance of input video drives displacement", "effect": "Bright areas push outward, dark areas pull inward (or inverse)"}, "external_cv": {"description": "Control voltage input for modular patching", "range": "0-1V typical"}}
- Claimed `research/components/ics/ca3019/v1/provenance.json` key `source_artifacts`: []
- Claimed `research/components/ics/ca3019/v1/spec.json` key `sources`: ["Paik-Abe Video Synthesizer Technical Documentation", "Intersil CA3019 Datasheet", "1978 RCA Linear IC Databook"]
- Claimed `research/components/ics/ca3022/v1/provenance.json` key `source_artifacts`: ["research/devices/paik_abe/Paik Abe Video Synthesizer Technical Documentation.pdf", "research/devices/paik_abe/ETC_PaikAbe_Schematics.pdf", "research/components/RCA_1978_Linear_IC_Databook.pdf"]
- Claimed `research/components/ics/ca3022/v1/spec.json` key `datasheet_source`: "1978 RCA Linear IC Databook"
- Claimed `research/components/ics/ca3022/v1/spec.json` key `sources`: ["RCA Linear IC Databook 1978", "RCA Application Note AN-5250", "Paik/Abe Synthesizer Schematics (Pages 10, 20)"]
- Claimed `research/components/ics/ca3023/v1/provenance.json` key `source_artifacts`: ["research/devices/paik_abe/Paik Abe Video Synthesizer Technical Documentation.pdf", "research/devices/paik_abe/ETC_PaikAbe_Schematics.pdf", "research/components/RCA_1978_Linear_IC_Databook.pdf"]
- Claimed `research/components/ics/ca3023/v1/spec.json` key `datasheet_source`: "1978 RCA Linear IC Databook"
- Claimed `research/components/ics/ca3023/v1/spec.json` key `sources`: ["RCA Linear IC Databook 1978", "NTSC Standard (RS-170A) for 3.58 MHz subcarrier reference", "Paik/Abe Synthesizer Schematics (Pages 3, 4, 26, 30)"]
- Claimed `research/components/ics/ca3046/v1/provenance.json` key `source_artifacts`: []
- Claimed `research/components/ics/ca3046/v1/spec.json` key `datasheet_source`: "1978 RCA Linear IC Databook"
- Claimed `research/components/ics/ca3046/v1/spec.json` key `spice_model_source`: "Intersil Application Note"
- Claimed `research/components/ics/ca3054/v1/provenance.json` key `source_artifacts`: []
- Claimed `research/components/ics/ca3054/v1/spec.json` key `internal_structure.current_sources`: 2
- Claimed `research/components/ics/ca3080/v1/provenance.json` key `source_artifacts`: []
- Claimed `research/components/ics/ca3080/v1/spec.json` key `datasheet_source`: "1978 RCA Linear IC Databook"
- Claimed `research/components/ics/lm311/v1/provenance.json` key `source_artifacts`: []
- Claimed `research/components/ics/lm311/v1/spec.json` key `datasheet_source`: "TI Datasheet"
- Claimed `research/prototypes/paik_abe_colorizer_v1/spec.json` key `authenticity.schematic_basis`: "None - creative interpretation"
- Claimed `research/prototypes/paik_abe_folding_amp_v1/spec.json` key `authenticity.schematic_basis`: "None - creative interpretation based on audio synthesis concepts"
- Claimed `research/schematics/rutt-etra/metadata.json` key `source_url`: "https://archive.org/details/ETC2731"
- Claimed `research/validation/device_validation.json` key `devices.test_signal.reference_doc`: null
- Claimed `research/validation/device_validation.json` key `devices.paik_abe_colorizer.reference_doc`: null
- Claimed `research/validation/device_validation.json` key `devices.paik_abe_wobulator.reference_doc`: "research/components/functional/wobbulator/v1/spec.json"
- Claimed `research/validation/device_validation.json` key `devices.paik_abe_input_matrix.reference_doc`: "research/components/ics/ca3054/v1/spec.json"
- Claimed `research/validation/device_validation.json` key `devices.paik_abe_folding_amp.reference_doc`: null
- Claimed `research/validation/device_validation.json` key `devices.paik_abe_ped_colorizer.reference_doc`: "research/components/ics/ca3019/v1/spec.json"
- Claimed `research/validation/device_validation.json` key `devices.paik_abe_output_encoder.reference_doc`: null

## Appendix C. Recorded source descriptions and rights statements

Claimed: the following are source, reference, retrieval or rights statements from every Markdown record under research/ and docs/ that matches these categories. Each row quotes the recorded line; historical and licence claims were not checked externally. URLs were read as text only. Source-code examples and planned sources remain claims, not evidence of a successful download.


### docs/ANALOG_ROUTING_ARCHITECTURE.md
- Claimed `docs/ANALOG_ROUTING_ARCHITECTURE.md:57`: ### Sources (Outputs Only)
- Claimed `docs/ANALOG_ROUTING_ARCHITECTURE.md:346`: - **External video input** - Just another source node
- Claimed `docs/ANALOG_ROUTING_ARCHITECTURE.md:347`: - **Video file playback** - Source node with transport controls
- Claimed `docs/ANALOG_ROUTING_ARCHITECTURE.md:349`: - **Network streaming** - Source/destination nodes for NDI/Syphon

### docs/CABLE_DEGRADATION_SPEC.md
- Claimed `docs/CABLE_DEGRADATION_SPEC.md:9`: ## 1. Source References
- Claimed `docs/CABLE_DEGRADATION_SPEC.md:11`: This specification is derived from the following sources:
- Claimed `docs/CABLE_DEGRADATION_SPEC.md:13`: | Source | Used For |
- Claimed `docs/CABLE_DEGRADATION_SPEC.md:15`: | [NTSC Wikipedia](https://en.wikipedia.org/wiki/NTSC) | Subcarrier frequency, signal structure |
- Claimed `docs/CABLE_DEGRADATION_SPEC.md:16`: | [YIQ Wikipedia](https://en.wikipedia.org/wiki/YIQ) | Color space conversion matrices |
- Claimed `docs/CABLE_DEGRADATION_SPEC.md:17`: | [Dot Crawl Wikipedia](https://en.wikipedia.org/wiki/Dot_crawl) | Artifact characteristics |
- Claimed `docs/CABLE_DEGRADATION_SPEC.md:18`: | [S-Video Wikipedia](https://en.wikipedia.org/wiki/S-Video) | Y/C separation benefits |
- Claimed `docs/CABLE_DEGRADATION_SPEC.md:19`: | [RG-59 Wikipedia](https://en.wikipedia.org/wiki/RG-59) | Cable attenuation values |
- Claimed `docs/CABLE_DEGRADATION_SPEC.md:20`: | [MATLAB rgb2ntsc](https://www.mathworks.com/help/images/ref/rgb2ntsc.html) | YIQ matrix coefficients |
- Claimed `docs/CABLE_DEGRADATION_SPEC.md:222`: - [ ] Nearly indistinguishable from source on short cables
- Claimed `docs/CABLE_DEGRADATION_SPEC.md:351`: | RGB | Cleanest, virtually identical to source |

### docs/DEVELOPMENT_ROADMAP.md
- Claimed `docs/DEVELOPMENT_ROADMAP.md:36`: - [x] Patent/datasheet/schematic/manual retrieval system
- Claimed `docs/DEVELOPMENT_ROADMAP.md:90`: - [x] Clipping parameter (CLP pin 8 from Page 28 schematic)
- Claimed `docs/DEVELOPMENT_ROADMAP.md:122`: - [x] input_source_segments: Vec<usize> in SegmentShader
- Claimed `docs/DEVELOPMENT_ROADMAP.md:243`: 1. Research the original analog implementation (patents, schematics, service manuals)
- Claimed `docs/DEVELOPMENT_ROADMAP.md:244`: 2. Identify the ICs and extract transfer functions from datasheets

### docs/EMULATION_FIDELITY.md
- Claimed `docs/EMULATION_FIDELITY.md:13`: | 1 | IC Behavioral Models | Transfer function of every IC on the schematic | CA3019 diode windows, CA3054 diff amp tanh curve, CA3080 OTA transconductance |
- Claimed `docs/EMULATION_FIDELITY.md:22`: **Tier 1: IC Behavioral Models** are the foundation. Each IC gets a model.wgsl with pure transfer functions derived from datasheet curves. No shader boilerplate - just math.
- Claimed `docs/EMULATION_FIDELITY.md:65`: - [ ] spec.json - Full electrical parameters from datasheet (voltage limits, gain, bandwidth, pinout, transfer function)
- Claimed `docs/EMULATION_FIDELITY.md:69`: - [ ] provenance.json - Version metadata and source artifact references

### docs/HISTORICAL_EMULATION_PLAN.md
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:93`: - Cadet series (schematics publicly available)
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:108`: ### Data Sources (Priority Order)
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:110`: **Primary Sources (Most Reliable):**
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:113`: 3. Official schematics and blueprints
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:117`: **Secondary Sources (Verify Against Primary):**
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:124`: **Tertiary Sources (Context Only):**
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:130`: ### Source Repositories to Monitor
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:134`: - lzxindustries.net (schematics, manuals)
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:145`: - github.com/lzxindustries (Cadet schematics)
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:156`: - alldatasheet.com
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:157`: - datasheetcatalog.com
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:175`: │     ├─ Download PDFs, images, schematics                    │
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:178`: │     └─ Store with source attribution + date                 │
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:182`: │     ├─ Map signal flow from schematics                      │
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:187`: │     ├─ Compare multiple sources for same device             │
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:189`: │     ├─ Weight by source reliability                         │
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:194`: │     ├─ Check component behavior against datasheets          │
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:207`: **Level 1: Schematic Accuracy**
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:234`: - [ ] Schematic verified against multiple sources
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:235`: - [ ] Component datasheets collected
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:245`: - [ ] Level 1: Schematic review passed
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:251`: - Links to sources
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:271`: - Load component schematic view
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:308`: ├── schematics/              # Collected schematics
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:320`: 4. Identify component datasheet sources
- Claimed `docs/HISTORICAL_EMULATION_PLAN.md:349`: 3. **Validation Footage Sources**: Do you have access to hardware for capture? Known good reference footage?

### docs/SESSION_HANDOFF_v0.16.md
- Claimed `docs/SESSION_HANDOFF_v0.16.md:21`: - SegmentShader.input_source_segments: Vec<usize> — maps texture slot to intermediate texture index
- Claimed `docs/SESSION_HANDOFF_v0.16.md:23`: - count_cross_segment_sources() returns source device IDs for multi-input binding
- Claimed `docs/SESSION_HANDOFF_v0.16.md:28`: - create_segment_input_bind_group() — builds bind group from input_source_segments
- Claimed `docs/SESSION_HANDOFF_v0.16.md:29`: - SegmentPipeline now stores input_texture_count and input_source_segments for resize rebuild

### docs/V1_EMULATION_ROADMAP.md
- Claimed `docs/V1_EMULATION_ROADMAP.md:30`: - [x] Schematics (26 pages)
- Claimed `docs/V1_EMULATION_ROADMAP.md:55`: - ✅ CA3019 diode array modeled from schematic page 28 (PED Control)
- Claimed `docs/V1_EMULATION_ROADMAP.md:68`: *Dan Sandin — Open-source modular video processor*
- Claimed `docs/V1_EMULATION_ROADMAP.md:71`: - Modular "Distribution Religion" — schematics freely shared
- Claimed `docs/V1_EMULATION_ROADMAP.md:78`: - [ ] Full schematics from Archive.org
- Claimed `docs/V1_EMULATION_ROADMAP.md:87`: | MC1445 | Gain Block | Signal amplification | ❌ Need datasheet |
- Claimed `docs/V1_EMULATION_ROADMAP.md:111`: - [ ] Schematics (search Archive.org, Vasulka)
- Claimed `docs/V1_EMULATION_ROADMAP.md:144`: - [ ] Schematics (Jones published openly)
- Claimed `docs/V1_EMULATION_ROADMAP.md:169`: - Open-source schematics (GitHub)
- Claimed `docs/V1_EMULATION_ROADMAP.md:174`: - [x] Full schematics on GitHub (lzxindustries/lzxcadet)
- Claimed `docs/V1_EMULATION_ROADMAP.md:214`: ### Known Sources
- Claimed `docs/V1_EMULATION_ROADMAP.md:215`: - [1978 RCA Linear IC Databook](https://www.smcelectronics.com/DOWNLOADS/1978-RCA%20LINEAR%20IC.PDF) — All CA-30xx parts
- Claimed `docs/V1_EMULATION_ROADMAP.md:216`: - [LZX Cadet GitHub](https://github.com/lzxindustries/lzxcadet) — Modern schematics + BOMs
- Claimed `docs/V1_EMULATION_ROADMAP.md:217`: - [Archive.org](https://archive.org) — Sandin IP documentation
- Claimed `docs/V1_EMULATION_ROADMAP.md:255`: 1. **Download CA-30xx datasheets** from RCA Linear IC Databook

### docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md
- Claimed `docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md:41`: The **Moog synthesizer** (1964) provided the crucial paradigm: voltage control. Robert Moog's 1V/octave standard, modular patch cord architecture, and separation of oscillators, filters, amplifiers, and envelope generators became the organizational model for video synthesis. Don Buchla's alternative "West Coast" approach—waveshaping over subtractive synthesis, touch plates over keyboards, "sources of uncertainty" for controlled randomness—also influenced video module design philosophy.
- Claimed `docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md:85`: Dan Sandin's Image Processor at University of Illinois Chicago established the **modular paradigm for video**. His "Distribution Religion" philosophy made all schematics freely available—more IPs were built than any other video synthesizer of the era.
- Claimed `docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md:127`: - Q3: Constant current source (value set by R1)
- Claimed `docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md:404`: **Open-source Cadet Series** (GitHub: lzxindustries/lzxcadet): Complete schematics, PCB gerbers, front panel artwork for DIY builds.
- Claimed `docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md:418`: **Key resources**:
- Claimed `docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md:577`: 6. **Sync is foundational**: All timing derives from master clock; genlock enables multi-source work

### docs/WORKBENCH_WORKFLOW.md
- Claimed `docs/WORKBENCH_WORKFLOW.md:12`: The workbench opens a GPU-accelerated window and starts an HTTP server on http://localhost:3030.
- Claimed `docs/WORKBENCH_WORKFLOW.md:176`: | Video Keyer | Clean transitions between two virtual sources at key threshold |

### docs/architecture/swarm-agent-specification.md
- Claimed `docs/architecture/swarm-agent-specification.md:44`: ║    • Document says X → cite the source                          ║
- Claimed `docs/architecture/swarm-agent-specification.md:55`: | Research | Every finding has a source | URL, document path, or "local model said [X] - NEEDS CONFIRMATION" |
- Claimed `docs/architecture/swarm-agent-specification.md:57`: | Documentation | Every claim is traceable | Links to events, screenshots, sources |
- Claimed `docs/architecture/swarm-agent-specification.md:80`: - Extract info from images (schematics, references)
- Claimed `docs/architecture/swarm-agent-specification.md:277`: "source": "research",
- Claimed `docs/architecture/swarm-agent-specification.md:281`: "finding": "Historical sources suggest CA3130, not LM741",
- Claimed `docs/architecture/swarm-agent-specification.md:283`: "sources": ["https://archive.org/..."]
- Claimed `docs/architecture/swarm-agent-specification.md:618`: - Archival research with verifiable sources
- Claimed `docs/architecture/swarm-agent-specification.md:625`: │ • Every finding must have a source                          │
- Claimed `docs/architecture/swarm-agent-specification.md:634`: │ • Conflicting sources (can't determine which is correct)    │
- Claimed `docs/architecture/swarm-agent-specification.md:685`: ### 9.3 Resource Management
- Claimed `docs/architecture/swarm-agent-specification.md:692`: class ResourceManager:
- Claimed `docs/architecture/swarm-agent-specification.md:816`: | Event | Source | Payload | Typical Triggers |
- Claimed `docs/architecture/swarm-agent-specification.md:818`: | research.finding | Research | topic, finding, confidence, sources | dev.review_requested |

### docs/dev-logs/vision-guided-development.md
- Claimed `docs/dev-logs/vision-guided-development.md:75`: │ Definitions  │ │ Code drafts  │ │ Schematics   │
- Claimed `docs/dev-logs/vision-guided-development.md:108`: - extract_from_image("sandin_schematic.png", "components")

### docs/research/paik-abe-colorizer-analysis.md
- Claimed `docs/research/paik-abe-colorizer-analysis.md:6`: ## Sources Consulted
- Claimed `docs/research/paik-abe-colorizer-analysis.md:8`: 1. [The Early Video Project - Paik/Abe Synthesizer](http://davidsonsfiles.org/PaikAbeSythesizer.html)
- Claimed `docs/research/paik-abe-colorizer-analysis.md:9`: 2. [Internet Archive - Paik-Abe Schematics](https://archive.org/details/ETC3311) (PDF not accessible via fetch)
- Claimed `docs/research/paik-abe-colorizer-analysis.md:10`: 3. [Video History Project - Colorizers Come of Age](https://www.videohistoryproject.org/colorizers-come-age)
- Claimed `docs/research/paik-abe-colorizer-analysis.md:11`: 4. [Vasulka Archive](https://www.vasulka.org/archive/eigenwelt/pdf/126-129.pdf)
- Claimed `docs/research/paik-abe-colorizer-analysis.md:12`: 5. [Fondation Langlois - Paik/Abe Documentation](https://www.fondation-langlois.org/html/e/page.php?NumPage=452)
- Claimed `docs/research/paik-abe-colorizer-analysis.md:97`: | **Color source** | Multiple cameras assigned to R/G/B | Luminance folded 3 ways |
- Claimed `docs/research/paik-abe-colorizer-analysis.md:132`: 1. Access to actual schematics from Internet Archive (ETC3311, ETC2370)
- Claimed `docs/research/paik-abe-colorizer-analysis.md:145`: Our Paik/Abe Colorizer is a **conceptually accurate** emulation that captures the key insight (nonlinear folding creates color from luminance) but uses a **simplified circuit model**. For artistic use, this is perfectly valid. For museum-quality historical accuracy, we'd need the actual schematics.

### research/DISCOVERY_GUIDE.md
- Claimed `research/DISCOVERY_GUIDE.md:3`: How to find primary source documents for faithful 1:1 emulation of historical video synthesis hardware.
- Claimed `research/DISCOVERY_GUIDE.md:27`: | **Devices** | The actual circuit to recreate: topology, values, rails, calibration | Schematics, service manuals, patents, calibration data, operation guides |
- Claimed `research/DISCOVERY_GUIDE.md:28`: | **Components** | The physics of each part: transfer functions, limits, thermal drift, power draw | Vendor datasheets, SPICE parameters, spec.json, WGSL models |
- Claimed `research/DISCOVERY_GUIDE.md:32`: - **Backward:** Have a component datasheet → understand its role in a device → see which technique it implements → attribute to innovator
- Claimed `research/DISCOVERY_GUIDE.md:36`: ## Source Discovery by Layer
- Claimed `research/DISCOVERY_GUIDE.md:41`: | Source Type | Where to Find |
- Claimed `research/DISCOVERY_GUIDE.md:49`: - [Vasulka Archive](http://vasulka.org) - Comprehensive video art history
- Claimed `research/DISCOVERY_GUIDE.md:50`: - [Experimental Television Center](http://experimentaltvcenter.org) - Equipment and artists
- Claimed `research/DISCOVERY_GUIDE.md:51`: - [Video Data Bank](http://www.vdb.org) - Artist interviews and works
- Claimed `research/DISCOVERY_GUIDE.md:52`: - [Fondation Langlois](http://fondation-langlois.org) - Daniel Langlois collection
- Claimed `research/DISCOVERY_GUIDE.md:59`: | Source Type | Where to Find |
- Claimed `research/DISCOVERY_GUIDE.md:83`: | **Schematics** | Circuit topology, component values, power rails | Archives, patents, service manuals |
- Claimed `research/DISCOVERY_GUIDE.md:84`: | **Service manuals** | Schematics + calibration voltages + test points | Archive.org, eBay, HiFi Engine |
- Claimed `research/DISCOVERY_GUIDE.md:104`: site:archive.org "[device name]" (schematic OR manual OR documentation)
- Claimed `research/DISCOVERY_GUIDE.md:107`: 4. **GitHub (for modern/open-source devices):**
- Claimed `research/DISCOVERY_GUIDE.md:109`: site:github.com "[device name]" schematic
- Claimed `research/DISCOVERY_GUIDE.md:120`: **For Open-Source Devices:**
- Claimed `research/DISCOVERY_GUIDE.md:121`: - [LZX Industries GitHub](https://github.com/lzxindustries) - Cadet, Castle, etc.
- Claimed `research/DISCOVERY_GUIDE.md:122`: - [Syntonie DIY](https://github.com/syntonie) - Video synth circuits
- Claimed `research/DISCOVERY_GUIDE.md:125`: **Goal:** Get vendor datasheets and transfer functions for every IC/transistor/part
- Claimed `research/DISCOVERY_GUIDE.md:128`: - **Vendor datasheet PDF** - the authoritative source
- Claimed `research/DISCOVERY_GUIDE.md:133`: **Primary Datasheet Sources:**
- Claimed `research/DISCOVERY_GUIDE.md:134`: - [AllDatasheet](https://www.alldatasheet.com)
- Claimed `research/DISCOVERY_GUIDE.md:135`: - [DatasheetCatalog](https://www.datasheetcatalog.com)
- Claimed `research/DISCOVERY_GUIDE.md:136`: - [Datasheet Archive](https://www.datasheetarchive.com)
- Claimed `research/DISCOVERY_GUIDE.md:137`: - [BITSAVERS](http://bitsavers.org) - Historical datasheets
- Claimed `research/DISCOVERY_GUIDE.md:140`: - [1978 RCA Linear IC Databook](https://www.smcelectronics.com/DOWNLOADS/1978-RCA%20LINEAR%20IC.PDF) - CA-30xx series
- Claimed `research/DISCOVERY_GUIDE.md:142`: - eBay "NOS" (new old stock) listings often have datasheets
- Claimed `research/DISCOVERY_GUIDE.md:166`: │   ├── paik_abe/       # Schematics, patents, manuals, calibration
- Claimed `research/DISCOVERY_GUIDE.md:171`: ├── components/         # Per-part: datasheets + models
- Claimed `research/DISCOVERY_GUIDE.md:172`: │   ├── RCA_1975_Linear_ICs*.pdf   # Comprehensive databook (splits)
- Claimed `research/DISCOVERY_GUIDE.md:173`: │   ├── RCA_1978_Linear_IC_Databook*.pdf
- Claimed `research/DISCOVERY_GUIDE.md:176`: │   │   │   ├── CA3046_*.pdf      # Vendor datasheet (permanent ref)
- Claimed `research/DISCOVERY_GUIDE.md:177`: │   │   │   ├── PROVENANCE.md     # Source URLs, retrieval notes
- Claimed `research/DISCOVERY_GUIDE.md:192`: - **PROVENANCE.md** at component root documents source URLs and retrieval status
- Claimed `research/DISCOVERY_GUIDE.md:204`: python tools/pdf-utils/split_pdf.py info <path.pdf>
- Claimed `research/DISCOVERY_GUIDE.md:207`: python tools/pdf-utils/split_pdf.py split <path.pdf>
- Claimed `research/DISCOVERY_GUIDE.md:210`: python tools/pdf-utils/split_pdf.py extract <path.pdf> 10 25
- Claimed `research/DISCOVERY_GUIDE.md:216`: - **Splits (_pages_N-M.pdf)**: Tracked in git, all under 20MB
- Claimed `research/DISCOVERY_GUIDE.md:217`: - To regenerate splits from originals: python tools/pdf-utils/split_pdf.py split <original.pdf>
- Claimed `research/DISCOVERY_GUIDE.md:223`: | ETC_SandinIP_Manual.pdf | 106MB | 8 parts |
- Claimed `research/DISCOVERY_GUIDE.md:224`: | Paik Abe Video Synthesizer Technical Documentation.pdf | 51MB | 3 parts |
- Claimed `research/DISCOVERY_GUIDE.md:225`: | RCA_1975_Linear_ICs.pdf | 46MB | 4 parts |
- Claimed `research/DISCOVERY_GUIDE.md:226`: | Sandin_Distribution_Religion.pdf | 33MB | 3 parts |
- Claimed `research/DISCOVERY_GUIDE.md:227`: | RCA_1978_Linear_IC_Databook.pdf | 33MB | 3 parts |
- Claimed `research/DISCOVERY_GUIDE.md:255`: | Original source? | Avoid transcription errors |
- Claimed `research/DISCOVERY_GUIDE.md:256`: | Complete? | Partial schematics miss crucial details |
- Claimed `research/DISCOVERY_GUIDE.md:259`: | Verified? | Cross-reference with other sources |
- Claimed `research/DISCOVERY_GUIDE.md:263`: ## Discovered Sources Log
- Claimed `research/DISCOVERY_GUIDE.md:268`: - [x] Schematics (26 pages) - ETC3311 + pre-existing
- Claimed `research/DISCOVERY_GUIDE.md:275`: - [x] ETC Manual with schematics (63 pages, split to 8 parts) - archive.org/details/ETC1365
- Claimed `research/DISCOVERY_GUIDE.md:284`: - [ ] Full schematics - NOT publicly available (contact Dave Jones Design)
- Claimed `research/DISCOVERY_GUIDE.md:287`: - [x] Schematics, BOMs, firmware — GitHub (github.com/lzxindustries/lzxcadet)
- Claimed `research/DISCOVERY_GUIDE.md:298`: | retrieve_datasheet | Search/download datasheets → store in component dir |
- Claimed `research/DISCOVERY_GUIDE.md:299`: | retrieve_schematic | Search/download schematics → store in device dir |
- Claimed `research/DISCOVERY_GUIDE.md:315`: *This guide should be updated as new sources are discovered.*

### research/RESEARCH_INDEX.md
- Claimed `research/RESEARCH_INDEX.md:13`: ├── devices/        # Per-device: schematics, patents, manuals, calibration
- Claimed `research/RESEARCH_INDEX.md:18`: ├── components/     # Per-part: vendor datasheets, spec.json, model.wgsl
- Claimed `research/RESEARCH_INDEX.md:24`: **See [DISCOVERY_GUIDE.md](DISCOVERY_GUIDE.md) for how to find primary sources.**
- Claimed `research/RESEARCH_INDEX.md:32`: | 1 | **Paik/Abe Video Synthesizer** | 1969 | Comprehensive | 8 PDFs (schematics, tech docs, budget, restoration, Vasulka) |
- Claimed `research/RESEARCH_INDEX.md:35`: | 4 | **Jones Colorizer** | 1974 | Partial | 1 PDF (MVIP instructions); full schematics not publicly available |
- Claimed `research/RESEARCH_INDEX.md:36`: | 5 | **LZX Cadet Series** | 2012 | Available | Open source on GitHub (github.com/lzxindustries/lzxcadet) |
- Claimed `research/RESEARCH_INDEX.md:44`: | Document | File | Size | Source |
- Claimed `research/RESEARCH_INDEX.md:46`: | Technical Documentation (87 pages) | devices/paik_abe/Paik Abe Video Synthesizer Technical Documentation.pdf | 53MB | Pre-existing |
- Claimed `research/RESEARCH_INDEX.md:47`: | Schematics (26 pages) | devices/paik_abe/Paik-Abe Video Synthesizer - Schematics.pdf | 1.1MB | Pre-existing |
- Claimed `research/RESEARCH_INDEX.md:48`: | ETC Schematics | devices/paik_abe/ETC_PaikAbe_Schematics.pdf | 1.1MB | archive.org/details/ETC3311 |
- Claimed `research/RESEARCH_INDEX.md:49`: | Restoration Project (2011) | devices/paik_abe/ETC_PaikAbe_Restoration_2011.pdf | 793KB | archive.org/details/ETC3310 |
- Claimed `research/RESEARCH_INDEX.md:50`: | Budget Document (1972) | devices/paik_abe/ETC_PaikAbe_Budget_1972.pdf | 9.5MB | archive.org/details/ETC3316 |
- Claimed `research/RESEARCH_INDEX.md:51`: | Vasulka Eigenwelt pp.126-129 | devices/paik_abe/Vasulka_Eigenwelt_PaikAbe_126-129.pdf | 582KB | vasulka.org |
- Claimed `research/RESEARCH_INDEX.md:58`: | Document | File | Size | Source |
- Claimed `research/RESEARCH_INDEX.md:60`: | Distribution Religion Manual | devices/sandin_ip/Sandin_Distribution_Religion.pdf | 34.7MB | criticalartware.net |
- Claimed `research/RESEARCH_INDEX.md:61`: | ETC Manual (full schematics) | devices/sandin_ip/ETC_SandinIP_Manual.pdf | 111MB | archive.org/details/ETC1365 |
- Claimed `research/RESEARCH_INDEX.md:62`: | Connolly Guide (reformatted) | devices/sandin_ip/Connolly_Guide_SandinIP_screen.pdf | 8MB | jameshconnolly.com |
- Claimed `research/RESEARCH_INDEX.md:66`: | Document | File | Size | Source |
- Claimed `research/RESEARCH_INDEX.md:68`: | General Documentation | devices/rutt_etra/Vasulka_RuttEtra_General.pdf | 2.6MB | vasulka.org |
- Claimed `research/RESEARCH_INDEX.md:69`: | Video Synth Continuation | devices/rutt_etra/Vasulka_RuttEtra_VidSynthCont.pdf | 2.8MB | vasulka.org |
- Claimed `research/RESEARCH_INDEX.md:70`: | Eigenwelt pp.136-139 | devices/rutt_etra/Vasulka_Eigenwelt_RuttEtra_136-139.pdf | 394KB | vasulka.org |
- Claimed `research/RESEARCH_INDEX.md:71`: | ETC Panel Documentation | devices/rutt_etra/ETC_RuttEtra_Panel.pdf | 242KB | archive.org/details/ETC2530 |
- Claimed `research/RESEARCH_INDEX.md:75`: | Document | File | Size | Source |
- Claimed `research/RESEARCH_INDEX.md:77`: | MVIP Instructions | devices/jones_colorizer/Jones_MVIP_Instructions.pdf | 235KB | jonesvideo.com |
- Claimed `research/RESEARCH_INDEX.md:79`: **Note:** Full colorizer schematics are not publicly available. Contact Dave Jones Design (djdesign.com) or check ETC archives. Video History Project has detailed design history: videohistoryproject.org/jones-colorizer-history-design
- Claimed `research/RESEARCH_INDEX.md:83`: Full open-source hardware at: https://github.com/lzxindustries/lzxcadet
- Claimed `research/RESEARCH_INDEX.md:84`: Includes KiCad schematics, PCB layouts, BOMs, and owner's manuals.
- Claimed `research/RESEARCH_INDEX.md:94`: | 1975 RCA Linear ICs | components/RCA_1975_Linear_ICs.pdf | 48MB | All CA30xx series |
- Claimed `research/RESEARCH_INDEX.md:95`: | 1978 RCA Linear IC Databook | components/RCA_1978_Linear_IC_Databook.pdf | 34MB | All CA30xx series |
- Claimed `research/RESEARCH_INDEX.md:101`: | Component | Type | File | Size | Source |
- Claimed `research/RESEARCH_INDEX.md:103`: | CA3046 | Transistor Array | components/ics/ca3046/CA3046_Intersil_datasheet.pdf | 121KB | ivytechengineering.com |
- Claimed `research/RESEARCH_INDEX.md:104`: | CA3080 | OTA | components/ics/ca3080/CA3080_Intersil_datasheet.pdf | 216KB | octopart.com |
- Claimed `research/RESEARCH_INDEX.md:105`: | CA3080 | OTA App Note | components/ics/ca3080/CA3080_Application_Note_AN6668.pdf | 1.9MB | intersil.com |
- Claimed `research/RESEARCH_INDEX.md:106`: | LM311 | Comparator | components/ics/lm311/LM311_TI_datasheet.pdf | 2.4MB | ti.com |
- Claimed `research/RESEARCH_INDEX.md:110`: These datasheets could not be retrieved programmatically due to aggregator site JavaScript gating. Full specs are available in the RCA databooks above. Provenance notes with known download URLs are in each component directory.
- Claimed `research/RESEARCH_INDEX.md:112`: | Component | Type | Known Browser Sources |
- Claimed `research/RESEARCH_INDEX.md:114`: | CA3019 | Diode Array | semiee.com, alldatasheet.com |
- Claimed `research/RESEARCH_INDEX.md:115`: | CA3054 | Dual Diff Amp | mouser.com (fn388.pdf), ibselectronics.com |
- Claimed `research/RESEARCH_INDEX.md:116`: | CA3022 | Video Amp | datasheet4u.com |
- Claimed `research/RESEARCH_INDEX.md:117`: | CA3023 | Video Amp | datasheet4u.com, datasheetspdf.com |
- Claimed `research/RESEARCH_INDEX.md:150`: | Document | File | Size | Source |
- Claimed `research/RESEARCH_INDEX.md:152`: | Meigh-Andrews: Development, Functions & Legacy of Paik-Abe | innovators/paik_abe/Meigh-Andrews_Development_Functions_Legacy_PaikAbe.pdf | 198KB | NJP Art Center |
- Claimed `research/RESEARCH_INDEX.md:153`: | Vasulka Eigenwelt: Ars Electronica catalogue | innovators/paik_abe/Vasulka_Eigenwelt_ArsElectronica_AE043.pdf | 724KB | vasulka.org |
- Claimed `research/RESEARCH_INDEX.md:154`: | Vasulka Eigenwelt: Introduction pp.1-9 | innovators/paik_abe/Vasulka_Eigenwelt_Intro_001-009.pdf | 290KB | vasulka.org |
- Claimed `research/RESEARCH_INDEX.md:155`: | Beck: Image Processing and Video Synthesis | innovators/beck_stephen/Beck_Image_Processing_Video_Synthesis.pdf | 1.4MB | vasulka.org |
- Claimed `research/RESEARCH_INDEX.md:163`: | Kane: Synthetic Color & Video Synthesis Circa 1969 | techniques/Kane_Synthetic_Color_Video_Synthesis_1969.pdf | 1MB | Historical survey, Paik/Abe colorizer |
- Claimed `research/RESEARCH_INDEX.md:164`: | DAFx-17: Virtual Analog Buchla 259 Wavefolder | techniques/DAFx17_Buchla259_Wavefolder.pdf | 1.5MB | Wavefolding theory, parallel folding stages |
- Claimed `research/RESEARCH_INDEX.md:165`: | SMC-17: Virtual Analog Lockhart Wavefolder | techniques/SMC17_Lockhart_Wavefolder.pdf | 347KB | Lockhart/Serge wavefolder, Lambert-W function |
- Claimed `research/RESEARCH_INDEX.md:166`: | TI: Demystifying the OTA | techniques/TI_Demystifying_OTA_sboa117a.pdf | 266KB | OTA theory, video-rate applications |
- Claimed `research/RESEARCH_INDEX.md:167`: | Stromeko: OTA Overview for Synth DIY | techniques/Stromeko_OTA_Overview.pdf | 419KB | Practical OTA circuits for synthesis |
- Claimed `research/RESEARCH_INDEX.md:177`: ## Source Archives
- Claimed `research/RESEARCH_INDEX.md:180`: - [Vasulka Archive](http://vasulka.org) - Historical documentation (multiple PDFs retrieved)
- Claimed `research/RESEARCH_INDEX.md:181`: - [Internet Archive / ETC](https://archive.org) - Experimental TV Center collection (ETC3310, ETC3311, ETC3316, ETC1365, ETC2530)
- Claimed `research/RESEARCH_INDEX.md:182`: - [LZX Industries GitHub](https://github.com/lzxindustries) - Cadet schematics (open source)
- Claimed `research/RESEARCH_INDEX.md:183`: - [NJP Art Center](https://njpart.ggcf.kr) - Paik/Abe academic papers
- Claimed `research/RESEARCH_INDEX.md:184`: - [criticalartware.net](http://criticalartware.net) - Sandin Distribution Religion
- Claimed `research/RESEARCH_INDEX.md:186`: ### Datasheet Sources
- Claimed `research/RESEARCH_INDEX.md:188`: - [AllDatasheet](https://www.alldatasheet.com) - General search (browser only)
- Claimed `research/RESEARCH_INDEX.md:189`: - [Datasheet Archive](https://www.datasheetarchive.com) - Obsolete parts (browser only)
- Claimed `research/RESEARCH_INDEX.md:192`: - [Scanlines.xyz](https://scanlines.xyz) - Video synthesis forum
- Claimed `research/RESEARCH_INDEX.md:193`: - [ModWiggler Video Forum](https://modwiggler.com/forum) - Eurorack video
- Claimed `research/RESEARCH_INDEX.md:194`: - [Video History Project](https://www.videohistoryproject.org) - Historical documentation
- Claimed `research/RESEARCH_INDEX.md:203`: - Moved datasheets and PROVENANCE.md from v1/ to component root
- Claimed `research/RESEARCH_INDEX.md:204`: - Reference docs (datasheets) are permanent, shared across model versions
- Claimed `research/RESEARCH_INDEX.md:207`: - Component structure now: ics/part_name/{datasheet.pdf, PROVENANCE.md, v1/{model.wgsl, spec.json}}
- Claimed `research/RESEARCH_INDEX.md:212`: - Paik/Abe: 7 PDFs (schematics, tech docs, budget, restoration, Vasulka)
- Claimed `research/RESEARCH_INDEX.md:217`: - **Components**: 6 PDFs (2 RCA databooks + 4 individual datasheets)
- Claimed `research/RESEARCH_INDEX.md:218`: - CA3046, CA3080, CA3080 App Note, LM311 - individual datasheets
- Claimed `research/RESEARCH_INDEX.md:228`: - Patents/manuals/schematics consolidated under Devices (per-device)
- Claimed `research/RESEARCH_INDEX.md:229`: - Vendor datasheets consolidated under Components (per-part)
- Claimed `research/RESEARCH_INDEX.md:239`: Automated pipeline: Extract ICs -> lookup datasheets -> spec.json -> model.wgsl -> test

### research/audits/paik_abe_device_audit_20260129.md
- Claimed `research/audits/paik_abe_device_audit_20260129.md:5`: **Reference:** Paik Abe Video Synthesizer Technical Documentation.pdf (Pages 1-30)
- Claimed `research/audits/paik_abe_device_audit_20260129.md:11`: | Device | Schematic Ref | Inputs | Authentic? | Action |
- Claimed `research/audits/paik_abe_device_audit_20260129.md:28`: ### Schematic Evidence
- Claimed `research/audits/paik_abe_device_audit_20260129.md:29`: **NONE.** There is no "folding amplifier" circuit in the Paik/Abe schematics.
- Claimed `research/audits/paik_abe_device_audit_20260129.md:31`: The schematics show:
- Claimed `research/audits/paik_abe_device_audit_20260129.md:50`: ### Schematic Evidence
- Claimed `research/audits/paik_abe_device_audit_20260129.md:61`: 3. **Weight matrix** - Values are speculative, not from schematic
- Claimed `research/audits/paik_abe_device_audit_20260129.md:81`: ### Schematic Evidence (Page 28 - PED.CONT.)
- Claimed `research/audits/paik_abe_device_audit_20260129.md:127`: ### Schematic Evidence (Pages 22-23, 9-10)
- Claimed `research/audits/paik_abe_device_audit_20260129.md:158`: **NEEDS UPDATE** - These claims are not supported by schematics.
- Claimed `research/audits/paik_abe_device_audit_20260129.md:181`: ## Appendix: Schematic Page Reference
- Claimed `research/audits/paik_abe_device_audit_20260129.md:194`: | 22-23 | Encoder (6-12) | Full encoder schematic |

### research/audits/ped_colorizer_audit_20260129.md
- Claimed `research/audits/ped_colorizer_audit_20260129.md:9`: ## Schematic Analysis (Page 28)
- Claimed `research/audits/ped_colorizer_audit_20260129.md:38`: | Schematic | Our Parameter | Status |
- Claimed `research/audits/ped_colorizer_audit_20260129.md:48`: | Schematic | Our Input | Status |
- Claimed `research/audits/ped_colorizer_audit_20260129.md:103`: 1. **CA3019-based amplitude window architecture** - Core concept matches schematic
- Claimed `research/audits/ped_colorizer_audit_20260129.md:164`: **Confidence Level:** 85% authentic to schematic (up from 75% after fixes)

### research/components/README.md
- Claimed `research/components/README.md:28`: - datasheet.pdf - Original datasheet (if available)
- Claimed `research/components/README.md:102`: ## Priority Components (from Paik/Abe schematics)
- Claimed `research/components/README.md:107`: | CA3022 | Video Amplifier | Signal conditioning | Need datasheet |
- Claimed `research/components/README.md:108`: | CA3023 | Video Amplifier | Signal conditioning | Need datasheet |
- Claimed `research/components/README.md:111`: | CA3055 | Differential Amp | Matrix mixing | Need datasheet |
- Claimed `research/components/README.md:115`: ## Sources
- Claimed `research/components/README.md:117`: - [1978 RCA Linear IC Databook](https://www.smcelectronics.com/DOWNLOADS/1978-RCA%20LINEAR%20IC.PDF)
- Claimed `research/components/README.md:118`: - [LZX Cadet Schematics](https://github.com/lzxindustries/lzxcadet)
- Claimed `research/components/README.md:119`: - [Sandin IP Schematics](https://archive.org/details/sandin-image-processor-schematics_2)
- Claimed `research/components/README.md:120`: - [AllDatasheet](https://www.alldatasheet.com)

### research/components/ics/ca3019/PROVENANCE.md
- Claimed `research/components/ics/ca3019/PROVENANCE.md:7`: Individual vendor datasheet could not be retrieved programmatically (all aggregator sites require browser JavaScript). Full specifications are available in:
- Claimed `research/components/ics/ca3019/PROVENANCE.md:8`: - research/components/RCA_1975_Linear_ICs.pdf (48MB)
- Claimed `research/components/ics/ca3019/PROVENANCE.md:9`: - research/components/RCA_1978_Linear_IC_Databook.pdf (34MB)
- Claimed `research/components/ics/ca3019/PROVENANCE.md:17`: ## Known Download Sources (require browser)
- Claimed `research/components/ics/ca3019/PROVENANCE.md:18`: - https://www.alldatasheet.com/datasheet-pdf/pdf/87264/INTERSIL/CA3019.html
- Claimed `research/components/ics/ca3019/PROVENANCE.md:19`: - https://www.semiee.com/file/EOL2/Intersil-CA3019.pdf

### research/components/ics/ca3022/PROVENANCE.md
- Claimed `research/components/ics/ca3022/PROVENANCE.md:7`: Individual vendor datasheet could not be retrieved programmatically (all aggregator sites require browser JavaScript). Full specifications are available in:
- Claimed `research/components/ics/ca3022/PROVENANCE.md:8`: - research/components/RCA_1975_Linear_ICs.pdf (48MB)
- Claimed `research/components/ics/ca3022/PROVENANCE.md:9`: - research/components/RCA_1978_Linear_IC_Databook.pdf (34MB)
- Claimed `research/components/ics/ca3022/PROVENANCE.md:17`: ## Known Download Sources (require browser)
- Claimed `research/components/ics/ca3022/PROVENANCE.md:18`: - https://datasheet4u.com/datasheet-pdf/RCASolidState/CA3022/pdf.php?id=559232

### research/components/ics/ca3023/PROVENANCE.md
- Claimed `research/components/ics/ca3023/PROVENANCE.md:7`: Individual vendor datasheet could not be retrieved programmatically (all aggregator sites require browser JavaScript). Full specifications are available in:
- Claimed `research/components/ics/ca3023/PROVENANCE.md:8`: - research/components/RCA_1975_Linear_ICs.pdf (48MB)
- Claimed `research/components/ics/ca3023/PROVENANCE.md:9`: - research/components/RCA_1978_Linear_IC_Databook.pdf (34MB)
- Claimed `research/components/ics/ca3023/PROVENANCE.md:17`: ## Known Download Sources (require browser)
- Claimed `research/components/ics/ca3023/PROVENANCE.md:18`: - https://datasheet4u.com/datasheet-pdf/RCASolidState/CA3023/pdf.php?id=559230
- Claimed `research/components/ics/ca3023/PROVENANCE.md:19`: - https://datasheetspdf.com/datasheet/CA3023.html

### research/components/ics/ca3054/PROVENANCE.md
- Claimed `research/components/ics/ca3054/PROVENANCE.md:7`: Individual vendor datasheet could not be retrieved programmatically (all aggregator sites require browser JavaScript). Full specifications are available in:
- Claimed `research/components/ics/ca3054/PROVENANCE.md:8`: - research/components/RCA_1975_Linear_ICs.pdf (48MB)
- Claimed `research/components/ics/ca3054/PROVENANCE.md:9`: - research/components/RCA_1978_Linear_IC_Databook.pdf (34MB)
- Claimed `research/components/ics/ca3054/PROVENANCE.md:18`: ## Known Download Sources (require browser)
- Claimed `research/components/ics/ca3054/PROVENANCE.md:19`: - https://www.mouser.com/catalog/specsheets/intersil_fn388.pdf
- Claimed `research/components/ics/ca3054/PROVENANCE.md:20`: - https://www.ibselectronics.com/ibsstore/datasheet/Intersil/ca3054.pdf
- Claimed `research/components/ics/ca3054/PROVENANCE.md:21`: - https://www.alldatasheet.com/datasheet-pdf/pdf/66311/INTERSIL/CA3054.html

### research/devices/README.md
- Claimed `research/devices/README.md:5`: Each device directory is the **single source of truth** for all documentation needed
- Claimed `research/devices/README.md:6`: to faithfully emulate that machine: schematics, patents, service manuals, calibration
- Claimed `research/devices/README.md:23`: | Schematics | Circuit topology, component values, power rails |
- Claimed `research/devices/README.md:25`: | Service manuals | Schematics + calibration voltages + test points |
- Claimed `research/devices/README.md:37`: │   ├── *.pdf               # All device-specific documents

### research/devices/paik_abe/architecture.md
- Claimed `research/devices/paik_abe/architecture.md:3`: Analyzed from original schematics (26 pages).
- Claimed `research/devices/paik_abe/architecture.md:38`: | IC | Type | Function | Schematic Page | Model Status |
- Claimed `research/devices/paik_abe/architecture.md:146`: ## Notes from Schematics
- Claimed `research/devices/paik_abe/architecture.md:148`: - Hand-drawn original schematics with component values

### research/innovators/README.md
- Claimed `research/innovators/README.md:20`: **Archive:** [Smithsonian American Art Museum](https://americanart.si.edu/artist/nam-june-paik-3670)
- Claimed `research/innovators/README.md:37`: **Archive:** [Experimental Television Center](http://experimentaltvcenter.org/shuya-abe)
- Claimed `research/innovators/README.md:46`: Created the Distribution Religion - freely sharing schematics for video processing equipment.
- Claimed `research/innovators/README.md:49`: - Open-source hardware before open-source existed
- Claimed `research/innovators/README.md:54`: **Archive:** [SAIC](https://www.saic.edu/profiles/faculty/dan-sandin)
- Claimed `research/innovators/README.md:91`: Modernized video synthesis with Eurorack-format modules and open-source designs.
- Claimed `research/innovators/README.md:95`: - Open-source Cadet series
- Claimed `research/innovators/README.md:98`: **Archive:** [LZX Industries GitHub](https://github.com/lzxindustries)

### research/prototypes/README.md
- Claimed `research/prototypes/README.md:21`: Triangle wave folding colorizer. Originally thought to be authentic Paik/Abe emulation based on online descriptions mentioning "folding" behavior. Schematic analysis revealed actual Paik/Abe uses CA3019 amplitude windows (PED Control), not triangle wave folding.
- Claimed `research/prototypes/README.md:38`: 1. They are validated against actual schematics

### research/techniques/README.md
- Claimed `research/techniques/README.md:85`: ## Research Sources
- Claimed `research/techniques/README.md:90`: 3. **Schematics** - Show specific circuit realizations

### research/techniques/cable_degradation/README.md
- Claimed `research/techniques/cable_degradation/README.md:5`: ## Source Documents
- Claimed `research/techniques/cable_degradation/README.md:7`: | Source | Type | Used For |
- Claimed `research/techniques/cable_degradation/README.md:9`: | [NTSC Wikipedia](https://en.wikipedia.org/wiki/NTSC) | Web | Subcarrier frequency (3.579545 MHz), bandwidth specs |
- Claimed `research/techniques/cable_degradation/README.md:10`: | [YIQ Wikipedia](https://en.wikipedia.org/wiki/YIQ) | Web | Color space conversion matrices |
- Claimed `research/techniques/cable_degradation/README.md:11`: | [Dot Crawl Wikipedia](https://en.wikipedia.org/wiki/Dot_crawl) | Web | Y/C crosstalk artifact characteristics |
- Claimed `research/techniques/cable_degradation/README.md:12`: | [S-Video Wikipedia](https://en.wikipedia.org/wiki/S-Video) | Web | Y/C separation benefits |
- Claimed `research/techniques/cable_degradation/README.md:13`: | [RG-59 Wikipedia](https://en.wikipedia.org/wiki/RG-59) | Web | Cable attenuation values |

## Appendix D. Tool and integration source excerpts

Observed: named files were read at these line ranges. Comments/docstrings are claims; executable bodies show implementation only.


Observed `tools/retrieval/artifacts.py:1-115`:
```text
1: """
2: Artifact dataclasses for the document retrieval system.
3:
4: Each artifact type represents a specific kind of research document
5: with appropriate metadata fields.
6: """
7:
8: from dataclasses import dataclass, field
9: from datetime import datetime
10: from pathlib import Path
11: from typing import Optional, List, Dict, Any
12: import hashlib
13: import json
14:
15:
16: @dataclass
17: class Artifact:
18:     """Base artifact with common fields."""
19:     id: str
20:     type: str  # 'patent', 'datasheet', 'schematic', 'manual', 'component'
21:     name: str
22:     path: Path
23:     source_url: Optional[str] = None
24:     retrieved_at: Optional[datetime] = None
25:     metadata: Dict[str, Any] = field(default_factory=dict)
26:
27:     def compute_hash(self) -> str:
28:         """Compute SHA256 hash of the artifact file."""
29:         if not self.path.exists():
30:             return ""
31:         sha256 = hashlib.sha256()
32:         with open(self.path, "rb") as f:
33:             for chunk in iter(lambda: f.read(8192), b""):
34:                 sha256.update(chunk)
35:         return sha256.hexdigest()
36:
37:     def to_dict(self) -> Dict[str, Any]:
38:         """Convert to dictionary for JSON serialization."""
39:         return {
40:             "id": self.id,
41:             "type": self.type,
42:             "name": self.name,
43:             "path": str(self.path),
44:             "source_url": self.source_url,
45:             "retrieved_at": self.retrieved_at.isoformat() if self.retrieved_at else None,
46:             "metadata": self.metadata,
47:         }
48:
49:     @classmethod
50:     def from_dict(cls, data: Dict[str, Any]) -> "Artifact":
51:         """Create from dictionary."""
52:         data = data.copy()
53:         data["path"] = Path(data["path"])
54:         if data.get("retrieved_at"):
55:             data["retrieved_at"] = datetime.fromisoformat(data["retrieved_at"])
56:         return cls(**data)
57:
58:
59: @dataclass
60: class PatentArtifact(Artifact):
61:     """Patent document artifact."""
62:     patent_id: str = ""
63:     title: str = ""
64:     inventors: List[str] = field(default_factory=list)
65:     assignee: Optional[str] = None
66:     filing_date: Optional[str] = None
67:     publication_date: Optional[str] = None
68:     abstract: str = ""
69:     claims: List[str] = field(default_factory=list)
70:     figures_dir: Optional[Path] = None
71:
72:     def __post_init__(self):
73:         self.type = "patent"
74:         if not self.id:
75:             self.id = f"patent:{self.patent_id}"
76:
77:     def to_dict(self) -> Dict[str, Any]:
78:         """Convert to dictionary for JSON serialization."""
79:         data = super().to_dict()
80:         data.update({
81:             "patent_id": self.patent_id,
82:             "title": self.title,
83:             "inventors": self.inventors,
84:             "assignee": self.assignee,
85:             "filing_date": self.filing_date,
86:             "publication_date": self.publication_date,
87:             "abstract": self.abstract,
88:             "claims": self.claims,
89:             "figures_dir": str(self.figures_dir) if self.figures_dir else None,
90:         })
91:         return data
92:
93:
94: @dataclass
95: class DatasheetArtifact(Artifact):
96:     """IC datasheet artifact."""
97:     part_number: str = ""
98:     manufacturer: str = ""
99:     description: str = ""
100:     package_types: List[str] = field(default_factory=list)
101:     specs: Dict[str, Any] = field(default_factory=dict)
102:
103:     def __post_init__(self):
104:         self.type = "datasheet"
105:         if not self.id:
106:             self.id = f"datasheet:{self.part_number}"
107:
108:     def to_dict(self) -> Dict[str, Any]:
109:         """Convert to dictionary for JSON serialization."""
110:         data = super().to_dict()
111:         data.update({
112:             "part_number": self.part_number,
113:             "manufacturer": self.manufacturer,
114:             "description": self.description,
115:             "package_types": self.package_types,
```

Observed `tools/retrieval/components.py:170-267`:
```text
170:             "provenance": provenance,
171:             "wgsl_files": wgsl_files,
172:         }
173:
174:     def create_version(
175:         self,
176:         component: str,
177:         spec: Dict[str, Any],
178:         source_artifacts: Optional[List[str]] = None,
179:         created_by: str = "manual",
180:         notes: str = "",
181:         wgsl_files: Optional[Dict[str, str]] = None,
182:     ) -> ComponentVersion:
183:         """
184:         Create a new version of a component.
185:
186:         Args:
187:             component: Component path (e.g., "ics/ca3046")
188:             spec: Specification dictionary
189:             source_artifacts: List of artifact IDs this version is based on
190:             created_by: Who/what created this version
191:             notes: Version notes
192:             wgsl_files: Dict of filename -> WGSL code
193:
194:         Returns:
195:             ComponentVersion object
196:         """
197:         comp_dir = self._get_component_dir(component)
198:         comp_dir.mkdir(parents=True, exist_ok=True)
199:
200:         # Determine new version number
201:         existing = sorted([
202:             int(v.name[1:])
203:             for v in comp_dir.iterdir()
204:             if v.is_dir() and v.name.startswith("v")
205:         ])
206:         new_version = (existing[-1] + 1) if existing else 1
207:
208:         version_dir = comp_dir / f"v{new_version}"
209:         version_dir.mkdir()
210:
211:         logger.info(f"Creating {component} v{new_version}")
212:
213:         # Save spec
214:         spec_path = version_dir / "spec.json"
215:         with open(spec_path, "w") as f:
216:             json.dump(spec, f, indent=2, default=str)
217:
218:         # Save WGSL files
219:         if wgsl_files:
220:             for filename, code in wgsl_files.items():
221:                 wgsl_path = version_dir / filename
222:                 with open(wgsl_path, "w") as f:
223:                     f.write(code)
224:
225:         # Get git commit hash if in a repo
226:         commit_hash = self._get_git_commit()
227:
228:         # Create provenance record
229:         provenance = {
230:             "component": component,
231:             "version": new_version,
232:             "created_at": datetime.now().isoformat(),
233:             "created_by": created_by,
234:             "commit_hash": commit_hash,
235:             "notes": notes,
236:             "source_artifacts": source_artifacts or [],
237:         }
238:
239:         prov_path = version_dir / "provenance.json"
240:         with open(prov_path, "w") as f:
241:             json.dump(provenance, f, indent=2)
242:
243:         # Create ComponentVersion object
244:         version_obj = ComponentVersion(
245:             id=f"component:{component}:v{new_version}",
246:             component=component,
247:             version=new_version,
248:             path=version_dir,
249:             created_at=datetime.now(),
250:             created_by=created_by,
251:             commit_hash=commit_hash,
252:             notes=notes,
253:             spec=spec,
254:             source_artifacts=source_artifacts or [],
255:         )
256:
257:         # Track in database
258:         self.db.add_version(version_obj)
259:
260:         # Create relationships to source artifacts
261:         if source_artifacts:
262:             for artifact_id in source_artifacts:
263:                 self.db.add_relationship(
264:                     f"component:{component}",
265:                     artifact_id,
266:                     "derived_from"
267:                 )
```

Observed `tools/research-orchestrator/mcp_server.py:41-181`:
```text
41: @server.list_tools()
42: async def list_tools():
43:     """List available research tools."""
44:     return [
45:         # === Document Retrieval Tools ===
46:         Tool(
47:             name="retrieve_patent",
48:             description="Search and download patents. Returns patent metadata and PDF path.",
49:             inputSchema={
50:                 "type": "object",
51:                 "properties": {
52:                     "query": {
53:                         "type": "string",
54:                         "description": "Search query or patent ID (e.g., 'video synthesizer' or 'US4011557')"
55:                     },
56:                     "inventor": {
57:                         "type": "string",
58:                         "description": "Optional inventor name filter"
59:                     },
60:                     "download": {
61:                         "type": "boolean",
62:                         "description": "If true and query is a patent ID, download the patent",
63:                         "default": False
64:                     }
65:                 },
66:                 "required": ["query"]
67:             }
68:         ),
69:
70:         Tool(
71:             name="retrieve_datasheet",
72:             description="Search and download IC datasheets. Returns datasheet path and metadata.",
73:             inputSchema={
74:                 "type": "object",
75:                 "properties": {
76:                     "part_number": {
77:                         "type": "string",
78:                         "description": "Part number to search (e.g., 'CA3046', 'LM1881')"
79:                     },
80:                     "download": {
81:                         "type": "boolean",
82:                         "description": "If true, download the datasheet",
83:                         "default": False
84:                     }
85:                 },
86:                 "required": ["part_number"]
87:             }
88:         ),
89:
90:         Tool(
91:             name="retrieve_schematic",
92:             description="Search and download circuit schematics from archives.",
93:             inputSchema={
94:                 "type": "object",
95:                 "properties": {
96:                     "device": {
97:                         "type": "string",
98:                         "description": "Device name (e.g., 'paik-abe-synthesizer', 'sandin-ip')"
99:                     },
100:                     "download": {
101:                         "type": "boolean",
102:                         "description": "If true, download the schematic",
103:                         "default": False
104:                     }
105:                 },
106:                 "required": ["device"]
107:             }
108:         ),
109:
110:         Tool(
111:             name="retrieve_manual",
112:             description="Search and download operation/service manuals.",
113:             inputSchema={
114:                 "type": "object",
115:                 "properties": {
116:                     "device": {
117:                         "type": "string",
118:                         "description": "Device name to search"
119:                     },
120:                     "manual_type": {
121:                         "type": "string",
122:                         "enum": ["service", "operation", "user", "any"],
123:                         "description": "Type of manual",
124:                         "default": "any"
125:                     },
126:                     "download": {
127:                         "type": "boolean",
128:                         "description": "If true, download the manual",
129:                         "default": False
130:                     }
131:                 },
132:                 "required": ["device"]
133:             }
134:         ),
135:
136:         Tool(
137:             name="get_provenance",
138:             description="Get provenance information for an artifact or component.",
139:             inputSchema={
140:                 "type": "object",
141:                 "properties": {
142:                     "artifact_id": {
143:                         "type": "string",
144:                         "description": "Artifact ID (e.g., 'patent:US4011557', 'datasheet:CA3046')"
145:                     }
146:                 },
147:                 "required": ["artifact_id"]
148:             }
149:         ),
150:
151:         Tool(
152:             name="list_components",
153:             description="List available WGSL component models with versions.",
154:             inputSchema={
155:                 "type": "object",
156:                 "properties": {
157:                     "category": {
158:                         "type": "string",
159:                         "enum": ["ics", "functional", "generators", "processors"],
160:                         "description": "Optional category filter"
161:                     }
162:                 }
163:             }
164:         ),
165:
166:         Tool(
167:             name="component_history",
168:             description="Get version history for a component.",
169:             inputSchema={
170:                 "type": "object",
171:                 "properties": {
172:                     "component": {
173:                         "type": "string",
174:                         "description": "Component path (e.g., 'ics/ca3046', 'functional/folding-amp')"
175:                     }
176:                 },
177:                 "required": ["component"]
178:             }
179:         ),
180:     ]
181:
```

Observed `tools/research-orchestrator/mcp_server.py:218-264`:
```text
218: async def retrieve_patent(query: str, inventor: Optional[str], download: bool):
219:     """Search or download patents."""
220:     if not HAS_RETRIEVAL:
221:         return [TextContent(type="text", text="[ERROR] Retrieval tools not installed")]
222:
223:     start = time.time()
224:     retriever = PatentRetriever()
225:
226:     try:
227:         # Check if query looks like a patent ID
228:         is_patent_id = query.upper().startswith(("US", "EP", "WO", "JP", "CN"))
229:
230:         if download and is_patent_id:
231:             artifact = retriever.download(query)
232:             if artifact:
233:                 elapsed = time.time() - start
234:                 return [TextContent(type="text", text=f"""[RETRIEVAL] {elapsed:.1f}s
235:
236: Downloaded patent: {artifact.patent_id}
237: Title: {artifact.title}
238: Inventors: {', '.join(artifact.inventors)}
239: Path: {artifact.path}
240: Filing date: {artifact.filing_date}
241: """)]
242:             else:
243:                 return [TextContent(type="text", text=f"[ERROR] Failed to download patent: {query}")]
244:         else:
245:             results = retriever.search(query, inventor=inventor, max_results=10)
246:             elapsed = time.time() - start
247:
248:             output = f"[RETRIEVAL] {elapsed:.1f}s\n\nFound {len(results)} patents:\n\n"
249:             for r in results:
250:                 output += f"**{r.patent_id}**: {r.title}\n"
251:                 if r.inventors:
252:                     output += f"  Inventors: {', '.join(r.inventors)}\n"
253:                 if r.filing_date:
254:                     output += f"  Filed: {r.filing_date}\n"
255:                 output += f"  URL: {r.url}\n\n"
256:
257:             return [TextContent(type="text", text=output)]
258:
259:     except Exception as e:
260:         elapsed = time.time() - start
261:         return [TextContent(type="text", text=f"[RETRIEVAL ERROR] {elapsed:.1f}s\n{e}")]
262:
263:
264: async def retrieve_datasheet(part_number: str, download: bool):
```

Observed `tools/device-workbench/mcp_server.py:617-669`:
```text
617: async def handle_export_node(args: dict):
618:     """Export shader as node for main app."""
619:     node_name = args["node_name"]
620:     category = args["category"]
621:     description = args.get("description", "")
622:
623:     # Get current state
624:     state = await workbench_request("GET", "/state")
625:     shader_code = state.get("shader_code", "")
626:     params = state.get("params", {})
627:
628:     # Generate node definition
629:     node_def = {
630:         "name": node_name,
631:         "category": category,
632:         "description": description,
633:         "shader": shader_code,
634:         "params": params,
635:         "exported_at": datetime.now().isoformat()
636:     }
637:
638:     # Save to exports directory
639:     exports_dir = BASE_DIR / "src" / "nodes" / "exported"
640:     exports_dir.mkdir(parents=True, exist_ok=True)
641:
642:     output_path = exports_dir / f"{node_name}.json"
643:     with open(output_path, "w") as f:
644:         json.dump(node_def, f, indent=2)
645:
646:     return [TextContent(
647:         type="text",
648:         text=f"[OK] Node exported: {output_path}\n"
649:              f"  Name: {node_name}\n"
650:              f"  Category: {category}\n"
651:              f"  Parameters: {len(params)}"
652:     )]
653:
654:
655: # === Main ===
656:
657: async def main():
658:     """Run the MCP server."""
659:     # Ensure directories exist
660:     SCREENSHOTS_DIR.mkdir(parents=True, exist_ok=True)
661:     REFERENCES_DIR.mkdir(parents=True, exist_ok=True)
662:     TEST_SIGNALS_DIR.mkdir(parents=True, exist_ok=True)
663:
664:     async with stdio_server() as (read_stream, write_stream):
665:         await server.run(read_stream, write_stream, server.create_initialization_options())
666:
667:
668: if __name__ == "__main__":
669:     asyncio.run(main())
```

Observed `tools/device-workbench/workbench-app/src/main.rs:108-144`:
```text
108:     }
109: }
110:
111: fn main() -> Result<()> {
112:     // Initialize logging
113:     let subscriber = FmtSubscriber::builder()
114:         .with_max_level(Level::INFO)
115:         .finish();
116:     tracing::subscriber::set_global_default(subscriber)?;
117:
118:     info!("Device Workbench starting...");
119:
120:     // Create shared state
121:     let state = Arc::new(RwLock::new(AppState::new()));
122:
123:     // Spawn HTTP API server in background thread
124:     let api_state = state.clone();
125:     std::thread::spawn(move || {
126:         let rt = tokio::runtime::Runtime::new().unwrap();
127:         rt.block_on(async {
128:             if let Err(e) = api::run_server(api_state, API_PORT).await {
129:                 tracing::error!("API server error: {}", e);
130:             }
131:         });
132:     });
133:
134:     info!("API server running on http://localhost:{}", API_PORT);
135:
136:     // Create event loop and run
137:     let event_loop = EventLoop::new()?;
138:     event_loop.set_control_flow(ControlFlow::Poll);
139:
140:     let mut app = App::new(state);
141:     event_loop.run_app(&mut app)?;
142:
143:     Ok(())
144: }
```

Observed `apps/phosphlux/src/devices/mod.rs:1-30`:
```text
1: //! Device Modules
2: //!
3: //! Each device is an authentic emulation of historical video synthesis hardware.
4: //! Devices expose their circuit behavior through WGSL shaders and can be
5: //! composed into patches using the shader composer.
6: //!
7: //! Device categories:
8: //! - **generators**: Test signal generators
9: //! - **paik_abe**: Paik/Abe Video Synthesizer (1969-1970)
10: //! - **sandin_ip**: Sandin Image Processor (1973)
11: //! - **jones**: Dave Jones Colorizers (1970s-1990s)
12: //! - **rutt_etra**: Rutt/Etra Video Synthesizer (1973)
13: //! - **lzx**: LZX Industries modules (2010s-present)
14:
15: pub mod generators;
16: pub mod paik_abe;
17: // pub mod sandin_ip;  // TODO
18: // pub mod jones;       // TODO
19: // pub mod rutt_etra;   // TODO
20: // pub mod lzx;         // TODO
21:
22: pub use generators::TestSignalGenerator;
23: pub use paik_abe::{
24:     PaikAbeColorizer,
25:     PaikAbeFoldingAmp,
26:     PaikAbeInputMatrix,
27:     PaikAbeOutputEncoder,
28:     PaikAbePedColorizer,
29:     PaikAbeWobulator,
30: };
```

Observed `tools/pdf-utils/requirements.txt`:
```text
1: # PDF Processing Utilities
2: pypdf>=4.0.0      # For splitting/extracting (pure Python, fast)
3: pymupdf>=1.23.0   # For page-to-image conversion (needs system libs)
```

Observed `tools/retrieval/requirements.txt`:
```text
1: # Document Retrieval System Requirements
2: # Install with: pip install -r requirements.txt
3:
4: # Core HTTP functionality
5: requests>=2.28.0
6: aiohttp>=3.8.0
7:
8: # HTML parsing
9: beautifulsoup4>=4.11.0
10: lxml>=4.9.0
11:
12: # PDF handling (optional, for page count and extraction)
13: PyPDF2>=3.0.0
14:
15: # Database (built into Python stdlib, but listing for reference)
16: # sqlite3 - included in Python
17:
18: # For numpy operations in embedding search (optional)
19: numpy>=1.21.0
```

Observed `tools/research-orchestrator/requirements.txt`:
```text
1: # MCP Server
2: mcp>=1.0.0
3:
4: # Ollama client
5: ollama>=0.3.0
6:
7: # Async support
8: aiohttp>=3.9.0
9: asyncio-mqtt>=0.16.0
10:
11: # Document processing
12: pypdf>=4.0.0
13: python-docx>=1.1.0
14:
15: # Image handling
16: Pillow>=10.0.0
17:
18: # Embeddings and search
19: numpy>=1.26.0
20: faiss-cpu>=1.7.0
21:
22: # Utilities
23: pydantic>=2.0.0
24: python-dotenv>=1.0.0
```

Observed `tools/device-workbench/workbench-app/Cargo.toml`:
```text
1: [package]
2: name = "device-workbench"
3: version = "0.1.0"
4: edition = "2021"
5: description = "Lightweight device emulation workbench with Claude integration"
6:
7: [dependencies]
8: # Async runtime
9: tokio = { version = "1", features = ["full"] }
10:
11: # HTTP API
12: axum = { version = "0.7", features = ["json"] }
13: tower-http = { version = "0.5", features = ["cors"] }
14:
15: # GPU rendering
16: wgpu = "23"
17: winit = "0.30"
18: pollster = "0.4"
19: bytemuck = { version = "1", features = ["derive"] }
20: naga = { version = "23", features = ["wgsl-in"] }
21:
22: # Image handling
23: image = "0.25"
24: base64 = "0.22"
25:
26: # Serialization
27: serde = { version = "1", features = ["derive"] }
28: serde_json = "1"
29:
30: # Utilities
31: anyhow = "1"
32: tracing = "0.1"
33: tracing-subscriber = "0.3"
34: parking_lot = "0.12"
35:
36: # UI (optional, for parameter controls)
37: # egui = "0.27"
38: # egui-wgpu = "0.27"
39: # egui-winit = "0.27"
40:
41: [profile.release]
42: opt-level = 3
43: lto = true
```

## Appendix E. Command checks and limits

Observed commands in the original survey run (2026-10-11, before this correction):

- `python3 -B tools/pdf-utils/split_pdf.py --help` — exit 0. Real output begins `usage: split_pdf.py [-h] {info,split,extract,images} ...`, followed by info/split/extract/images descriptions and the --help option.
- `python3 -B -c 'import sys; sys.path.insert(0,"tools"); import retrieval; help(retrieval)'` — exit 0. Real output begins `Help on package retrieval:` and `NAME retrieval - Document Retrieval System for Analog Video Synthesis Research`; lists Artifact, BaseRetriever, ComponentManager, retrieval classes and package contents. No object constructor was called.
- `PYTHONPATH=tools python3 -B -m retrieval.schematics --help` — exit 0. Real output includes RuntimeWarning that retrieval.schematics was already in sys.modules; usage `python3.14 -m retrieval.schematics [-h] {search,download,list-known} ...`, description `Schematic Retriever`, and subcommand/help listings. --help exits before SchematicRetriever construction (`tools/retrieval/schematics.py:542-544`).
- `python3 -B tools/research-orchestrator/mcp_server.py --help` — exit 1. Real output: traceback at mcp_server.py line 19, `from mcp.server import Server`; `ModuleNotFoundError: No module named 'mcp'`.
- `python3 -B tools/device-workbench/mcp_server.py --help` — exit 1. Real output: traceback at mcp_server.py line 27, `import httpx`; `ModuleNotFoundError: No module named 'httpx'`.
- `rg -n 'rutt|Rutt|Etra|ETC2731|exported' research docs apps/phosphlux/src tools/retrieval/schematics.py -g '*.md' -g '*.rs' -g '*.py' -g '*.json'` (displayed through head) — exit 0; output includes `apps/phosphlux/src/devices/mod.rs:19:// pub mod rutt_etra; // TODO`, schematic metadata ETC2731, index source records and `apps/phosphlux/src/nodes/exported/PaikAbeColorizer.json:11` exported_at. The broader inventory and inspected factory delimit the missing-path finding.
- `rg -n 'exported|PaikAbeColorizer.json|nodes/export' apps/phosphlux/src -g '*.rs'` — exit 1, no output (run within a multi-command shell; the same search was then rerun separately with exit 1). No exported-node reference matched the current app Rust sources.
- Initial `git status --short` — exit 0; pre-existing modifications: Cargo.lock, Cargo.toml; untracked BASELINE.md, BLOCK_COMPATIBILITY_VALIDATION.md, research/schematics/, vendor/. Initial branch: task/T-0071-startup-redraw. These files were left intact.
- `git switch -c task/T-0073-rnd-survey` — exit 0, `Switched to a new branch 'task/T-0073-rnd-survey'`.

Observed completion checks: all 284 inventory paths are present in Appendix A; 443 explicit file/range citations resolve to existing files and valid line ranges; all eight required sections and 74 JSON field inventories are present. A first citation check found abbreviated paths and an overlong excerpt range; these were corrected and the check rerun with zero errors. A first staged whitespace check found spaces after blank excerpt-line labels; these were removed throughout.

Observed limits: no build, shader load, signal injection, screenshot capture, app launch, network request, installation, source retrieval, hardware comparison, archive/v1 survey, TypeScript reference archive survey or phosphlux-lite survey was run. Stored screenshots were inventoried with their metadata; their rendered appearance was not used as fresh validation. PDF pages and embedded notices were not inspected. BASELINE.md was read as prior-run claims only, not current evidence.

## Appendix F. Correction-run evidence — 2026-10-11

Observed: the two original paths previously described as absent are present: Python Path.is_file()/stat returned `True`, 53171994 bytes for research/devices/paik_abe/Paik Abe Video Synthesizer Technical Documentation.pdf and `True`, 34068255 bytes for research/components/RCA_1978_Linear_IC_Databook.pdf. Sections 4 and 7 now distinguish local existence from Git coverage.

Observed command: `git ls-files --others --exclude-standard research docs` — exit 0; real output:
```text
research/schematics/rutt-etra/metadata.json
research/schematics/rutt-etra/schematic.pdf
```

Observed command: `git ls-files --others --ignored --exclude-standard research docs` — exit 0; real output:
```text
research/components/RCA_1975_Linear_ICs.pdf
research/components/RCA_1978_Linear_IC_Databook.pdf
research/devices/paik_abe/Paik Abe Video Synthesizer Technical Documentation.pdf
research/devices/sandin_ip/ETC_SandinIP_Manual.pdf
research/devices/sandin_ip/Sandin_Distribution_Religion.pdf
```

Observed command: `git check-ignore -v` with the five paths above — exit 0; rules returned .gitignore lines 60, 61, 59, 57 and 58 respectively. The source rows in section 2 and Appendix A record those statuses; source files were not added or modified.

Observed branch ancestry: before the correction, `git rev-parse HEAD^ task/T-0071-startup-redraw` exited 0 and returned `36148b8473458f6d13eedee7034ce2ff34237549` twice. Thus the survey commit was based on the existing startup-redraw task branch and inherits its earlier unmerged commits. `git log -8 --oneline` showed earlier redraw/diagnosis/build-check commits, not changes made by this survey. T-0070 to T-0072 work remains separate; the survey did not rebase, merge, change or verify that earlier work.

Observed limits: the correction rechecked file existence, inventory Git coverage, byte sizes, source citations, branch ancestry and the report diff. It did not rerun the earlier tool help checks, database queries or application-source integration search; their original-run observations remain in Appendix E. No build, app launch, network, installation or source retrieval was performed.

Observed correction validation command: Python compared every Appendix A row with pathlib inventory/stat and Git tracked/ignored/untracked lists, checked all complete file:line lists including comma-separated ranges, and asserted the requested replacements and eight section headings. Exit 0; real output: `PASS inventory: 284 rows, five columns, exact paths and byte sizes; Git statuses {Tracked: 277, Ignored (local only): 5, Untracked (local only): 2}`; `PASS citations: 455 complete file/line lists, all ranges in bounds; ignore rules match.` The seven excluded-file byte sizes were printed and matched Appendix A. `git diff --check -- RND-PIPELINE.md` exited 0 with no output.
