# Phosphlux Development Protocol v1.0

**Adapted from:** Scientific Collaboration Protocol v1.0 (retired 22 Oct 2026; archived at `archive/scientific_collaboration_protocol_v1.0.md`)
**Purpose:** Machine-verifiable project state that survives AI context loss
**Created:** 2026-01-28

---

## 1. Core Problem

AI context is finite and subject to compaction. After compaction:
- Prose-based state ("we completed v0.12") must be reconstructed from memory
- Memory introduces errors, redundant work, or wrong assumptions
- The AI executes efficiently but does not audit against external standards

**Solution:** Machine-verifiable state that can be checked programmatically, independent of what anyone remembers.

---

## 2. Principles

### 2.1 Machine Verification Over Memory

Every claim about project state must be verifiable against the codebase:
- "v0.12 complete" → verified by checking that multi-pass renderer code exists
- "wobulator routing fixed" → verified by checking reachable_from_output() exists
- "LFO automation works" → verified by checking LfoState struct exists

### 2.2 State Lives in Files, Not Conversation

Project state is recorded in `project_state.json`, not in conversation history. After compaction, the AI reads this file to recover state — it does not guess from summaries.

### 2.3 Human Audit Function

The human catches when:
- Machine state is wrong or incomplete
- The AI is working on the wrong thing
- Process is being skipped under momentum pressure

The AI handles:
- Implementation
- Verification script execution
- Documentation generation
- Semantic communication

### 2.4 Gates Before Transitions

No version transition (v0.12 → v0.13) without all gates passing. Gates are defined before work begins, not after.

---

## 3. Project State Schema

File: `project_state.json`

```json
{
  "schema_version": "1.0",
  "project": "phosphlux",
  "current_version": "0.12",
  "current_phase": "complete",
  "last_verified": "2026-01-28T12:00:00Z",
  "last_session_summary": "Fixed wobulator routing bleed, added LFO automation with lo/hi range",

  "versions": {
    "0.12": {
      "status": "complete",
      "title": "Multi-Pass Renderer",
      "gates": {
        "processing_mode_enum": {
          "check": "grep 'enum ProcessingMode' src/gpu/shader_composer.rs",
          "status": "pass"
        },
        "render_segments": {
          "check": "grep 'struct RenderSegment' src/gpu/shader_composer.rs",
          "status": "pass"
        },
        "reachable_filtering": {
          "check": "grep 'fn reachable_from_output' src/gpu/shader_composer.rs",
          "status": "pass"
        },
        "lfo_automation": {
          "check": "grep 'struct LfoState' src/ui/node_editor.rs",
          "status": "pass"
        },
        "wobulator_scan_modulator": {
          "check": "grep 'ProcessingMode::ScanModulator' src/devices/paik_abe/mod.rs",
          "status": "pass"
        },
        "builds_clean": {
          "check": "cargo build --release",
          "status": "pass"
        }
      },
      "completed_at": "2026-01-28"
    },
    "0.13": {
      "status": "not_started",
      "title": "Cable Degradation",
      "gates": {
        "cable_type_enum": {
          "check": "grep 'enum CableType' src/gpu/shader_composer.rs",
          "status": "pending"
        },
        "cables_wgsl": {
          "check": "test -f src/shaders/cables.wgsl",
          "status": "pending"
        },
        "composite_degradation": {
          "check": "grep 'fn composite_degrade' src/shaders/cables.wgsl",
          "status": "pending"
        },
        "svideo_degradation": {
          "check": "grep 'fn svideo_degrade' src/shaders/cables.wgsl",
          "status": "pending"
        },
        "cable_ui": {
          "check": "grep 'CableType' src/ui/node_editor.rs",
          "status": "pending"
        },
        "builds_clean": {
          "check": "cargo build --release",
          "status": "pending"
        }
      }
    },
    "0.14": {
      "status": "not_started",
      "title": "Full Paik/Abe Chain",
      "gates": {
        "input_matrix_device": {
          "check": "grep 'struct PaikAbeInputMatrix' src/devices/paik_abe/mod.rs",
          "status": "pending"
        },
        "ped_colorizer_device": {
          "check": "grep 'struct PaikAbePedColorizer' src/devices/paik_abe/mod.rs",
          "status": "pending"
        },
        "folding_amp_device": {
          "check": "grep 'struct PaikAbeFoldingAmp' src/devices/paik_abe/mod.rs",
          "status": "pending"
        },
        "output_encoder_device": {
          "check": "grep 'struct PaikAbeOutputEncoder' src/devices/paik_abe/mod.rs",
          "status": "pending"
        },
        "patch_save_load": {
          "check": "grep 'fn save_patch' src/",
          "status": "pending"
        },
        "builds_clean": {
          "check": "cargo build --release",
          "status": "pending"
        }
      }
    },
    "0.15": {
      "status": "not_started",
      "title": "CRT Monitor",
      "gates": {
        "crt_device": {
          "check": "grep 'struct CrtMonitor' src/devices/",
          "status": "pending"
        },
        "phosphor_persistence": {
          "check": "grep 'phosphor_decay' src/devices/",
          "status": "pending"
        },
        "feedback_pair": {
          "check": "grep 'FeedbackPair' src/gpu/renderer.rs",
          "status": "pending"
        },
        "builds_clean": {
          "check": "cargo build --release",
          "status": "pending"
        }
      }
    },
    "0.16": {
      "status": "not_started",
      "title": "Camcorder + Feedback",
      "gates": {
        "camcorder_device": {
          "check": "grep 'struct Camcorder' src/devices/",
          "status": "pending"
        },
        "optical_connection": {
          "check": "grep 'ConnectionKind::Optical' src/",
          "status": "pending"
        },
        "agc": {
          "check": "grep 'agc_target' src/devices/",
          "status": "pending"
        },
        "builds_clean": {
          "check": "cargo build --release",
          "status": "pending"
        }
      }
    }
  },

  "open_issues": [],

  "next_actions": [
    {
      "action": "Begin v0.13 specification",
      "description": "Define cable degradation models, UI design, shader architecture"
    }
  ]
}
```

---

## 4. Verification Script

File: `scripts/verify_state.py`

The verification script:
1. Reads `project_state.json`
2. Executes each gate check against the codebase
3. Reports pass/fail for each gate
4. Updates gate statuses in the JSON
5. Exits with code 1 if any claimed "pass" is actually failing

```python
#!/usr/bin/env python3
"""
Verify project state against codebase.
Run at session start to establish ground truth.
"""

import json
import subprocess
import sys
from pathlib import Path
from datetime import datetime

def run_check(check_cmd: str, project_root: Path) -> bool:
    """Execute a gate check command, return True if passes."""
    try:
        # Handle different check types
        if check_cmd.startswith("grep "):
            # grep returns 0 if found, 1 if not found
            result = subprocess.run(
                check_cmd,
                shell=True,
                cwd=project_root,
                capture_output=True
            )
            return result.returncode == 0
        elif check_cmd.startswith("test -f "):
            filepath = check_cmd.replace("test -f ", "")
            return (project_root / filepath).exists()
        elif check_cmd.startswith("cargo "):
            result = subprocess.run(
                check_cmd,
                shell=True,
                cwd=project_root,
                capture_output=True
            )
            return result.returncode == 0
        else:
            print(f"  Unknown check type: {check_cmd}")
            return False
    except Exception as e:
        print(f"  Check failed with error: {e}")
        return False

def verify_version(version: str, version_data: dict, project_root: Path) -> tuple[int, int]:
    """Verify all gates for a version. Returns (passed, failed) counts."""
    passed = 0
    failed = 0

    print(f"\n{'='*60}")
    print(f"Version {version}: {version_data.get('title', 'Untitled')}")
    print(f"Claimed status: {version_data.get('status', 'unknown')}")
    print(f"{'='*60}")

    for gate_name, gate_data in version_data.get("gates", {}).items():
        check_cmd = gate_data.get("check", "")
        claimed_status = gate_data.get("status", "pending")

        actual_pass = run_check(check_cmd, project_root)
        actual_status = "pass" if actual_pass else "fail"

        # Check for mismatches
        if claimed_status == "pass" and not actual_pass:
            status_icon = "MISMATCH"
            failed += 1
        elif actual_pass:
            status_icon = "PASS"
            passed += 1
        else:
            status_icon = "pending"

        print(f"  [{status_icon:^10}] {gate_name}")
        if claimed_status == "pass" and not actual_pass:
            print(f"             ^ Claimed PASS but actually FAILING")
            print(f"             Check: {check_cmd}")

    return passed, failed

def main():
    project_root = Path(__file__).parent.parent
    state_file = project_root / "project_state.json"

    if not state_file.exists():
        print("ERROR: project_state.json not found")
        sys.exit(1)

    with open(state_file) as f:
        state = json.load(f)

    print(f"Phosphlux State Verification")
    print(f"Current version: {state.get('current_version', 'unknown')}")
    print(f"Current phase: {state.get('current_phase', 'unknown')}")
    print(f"Last verified: {state.get('last_verified', 'never')}")

    total_passed = 0
    total_failed = 0

    for version, version_data in state.get("versions", {}).items():
        passed, failed = verify_version(version, version_data, project_root)
        total_passed += passed
        total_failed += failed

    print(f"\n{'='*60}")
    print(f"SUMMARY: {total_passed} passed, {total_failed} failed/mismatched")
    print(f"{'='*60}")

    if total_failed > 0:
        print("\nERROR: State claims do not match codebase!")
        print("Update project_state.json to reflect actual state.")
        sys.exit(1)
    else:
        # Update last_verified timestamp
        state["last_verified"] = datetime.now().isoformat()
        with open(state_file, "w") as f:
            json.dump(state, f, indent=2)
        print("\nState verified successfully.")
        sys.exit(0)

if __name__ == "__main__":
    main()
```

---

## 5. Session Protocol

### 5.1 Session Start

**AI actions (in order):**

1. **Read `project_state.json`** — Do not rely on conversation summary
2. **Run verification** — `python scripts/verify_state.py`
3. **Report state to human:**
   - Current version and phase
   - Gates status for current/next version
   - Open issues
   - Next actions from state file
4. **Confirm direction with human** — "State shows v0.12 complete, v0.13 not started. Ready to begin v0.13?"

**If verification fails:**
- Report the mismatch
- Do NOT proceed with work
- Update state file to reflect actual codebase state
- Re-verify

### 5.2 During Session

1. **Update DEVLOG.md in real-time** — Decisions, dead ends, findings
2. **After completing a gate** — Run that specific check, update state file
3. **On blockers/issues** — Add to `open_issues` array in state file
4. **On direction changes** — Confirm with human, update `next_actions`

### 5.3 Session End

**AI actions (in order):**

1. **Run full verification** — `python scripts/verify_state.py`
2. **Update `project_state.json`:**
   - `last_session_summary` — One line describing what was done
   - Gate statuses — Based on verification results
   - `next_actions` — What should happen next session
3. **Update DEVLOG.md** — Session summary
3. **Commit state file with code changes**
4. **Report to human:**
   - What was accomplished
   - What gates now pass
   - What's next

---

## 6. Version Lifecycle

```
Research → Specification → Implementation → Verification → Documentation → Release
    │            │               │                │               │            │
    │            │               │                │               │            │
    v            v               v                v               v            v
 sources     spec.md        code exists      gates pass      docs updated   version++
 gathered    in docs/       tests pass       state updated   DEVLOG done    tag commit
```

### 6.0 Research Phase (NEW - v1.1)

**Purpose:** Gather source-of-truth documentation before writing code.

**Why this matters:** Without research, shader values are made up ("chroma_blur = 0.003") instead of derived from actual specifications (NTSC I channel bandwidth = 1.3 MHz).

**Steps:**

1. **Gather primary sources** using research MCP tools:
   - `mcp__research__research_query` - Deep research on topics
   - `mcp__research__retrieve_datasheet` - IC specifications
   - `mcp__research__retrieve_patent` - Original circuit designs
   - Web search for standards documents

2. **Check existing documentation:**
   - `docs/VIDEO_SYNTHESIS_TECHNICAL_REFERENCE.md`
   - `docs/EMULATION_FIDELITY.md`
   - `research/components/` directory

3. **Build knowledge base entries:**
   - `mcp__research__embed_document` - Add to searchable KB
   - Record sources in spec document

4. **Create specification document** with:
   - Source citations
   - Transfer functions as equations
   - Quantitative values (bandwidth in MHz, not arbitrary floats)
   - Expected visual characteristics
   - Acceptance criteria

**Research gate (must pass before implementation):**
```json
{
  "spec_complete": {
    "type": "inspection",
    "description": "Specification document exists with cited values",
    "expected": "docs/FEATURE_SPEC.md contains transfer functions and source citations",
    "status": "pending"
  }
}
```

### 6.1 Specification Phase

- Define gates for the version (what code must exist)
- Write spec in DEVLOG under version header
- Update `project_state.json` with gate definitions
- Get human approval on scope

### 6.2 Implementation Phase

- Write code to satisfy gates
- **For shader code:** Follow Workbench Development Workflow (Section 8)
  - Load shader into workbench
  - Inject test signal
  - Verify visual output before marking gate as pass
- Run individual gate checks as features complete
- Update state file incrementally
- Keep DEVLOG current with decisions

### 6.3 Verification Phase

- Run full `verify_state.py`
- All gates must pass
- Manual testing of features
- Fix any issues, re-verify

### 6.4 Documentation Phase

- Update README.md with new features
- Update DEVELOPMENT_ROADMAP.md
- Ensure DEVLOG has complete session history
- Update version status to "complete" in state file

### 6.5 Release Phase

- Final verification pass
- Commit with version tag
- Push to remote
- Increment `current_version` for next cycle

---

## 7. Two-Tier Verification System

Phosphlux requires both code existence AND visual correctness verification. A shader function that exists but produces wrong output is not complete.

### 7.1 Tier 1: Machine Gates (Automated)

Machine gates verify code existence. They are:
- **Context-immune** — survive session restarts
- **Deterministic** — same input always produces same result
- **Fast** — run in milliseconds

Examples:
- `grep 'enum CableType' src/` → code exists
- `cargo build --release` → code compiles
- `dir src/shaders/cables.wgsl` → file exists

### 7.2 Tier 2: Visual Inspection Gates (Workbench)

Visual gates verify shader output correctness. They require:
- Loading shader into device workbench
- Injecting appropriate test signal
- Capturing screenshot
- AI vision analysis OR human inspection
- Recording verification in state file

Visual gates have additional fields:
```json
{
  "composite_visual_verified": {
    "type": "inspection",
    "description": "Composite degradation shows visible chroma blur, dot crawl on colorbars",
    "test_signal": "colorbars",
    "expected": "Chroma bleeding on color transitions, dot crawl pattern visible",
    "screenshot": "output/v0.13_composite_colorbars.png",
    "status": "pending",
    "verified_by": null,
    "verified_at": null
  }
}
```

### 7.3 Gate Type Summary

| Type | Checks | Method | Survives Context Loss |
|------|--------|--------|----------------------|
| Machine | Code exists | grep/file check | Yes (automated) |
| Build | Code compiles | cargo build | Yes (automated) |
| Inspection | Output correct | Workbench + vision | Yes (recorded in state) |

### 7.4 Gates Must Be Defined Before Implementation

Write gates when specifying the version, not after implementing it. This prevents post-hoc adjustment of success criteria.

### 7.5 Gates Accumulate

Gates for v0.12 remain checked even when working on v0.13. Regression = gate that previously passed now fails.

---

## 8. Workbench Development Workflow

**For all shader/visual code, the development cycle is:**

```
┌─────────────────────────────────────────────────────────────┐
│  1. WRITE CODE                                              │
│     Write shader function in .wgsl file                     │
├─────────────────────────────────────────────────────────────┤
│  2. LOAD INTO WORKBENCH                                     │
│     mcp__workbench__workbench_load_shader                   │
├─────────────────────────────────────────────────────────────┤
│  3. INJECT TEST SIGNAL                                      │
│     mcp__workbench__workbench_inject_signal                 │
│     - colorbars: for color processing                       │
│     - ramp_h/ramp_v: for spatial effects                    │
│     - gray50: for gain/level effects                        │
├─────────────────────────────────────────────────────────────┤
│  4. CAPTURE & ANALYZE                                       │
│     mcp__workbench__workbench_screenshot                    │
│     mcp__workbench__workbench_analyze                       │
├─────────────────────────────────────────────────────────────┤
│  5. COMPARE TO EXPECTED                                     │
│     Does output show expected visual characteristics?       │
│     - If NO: iterate on code, return to step 1              │
│     - If YES: proceed to step 6                             │
├─────────────────────────────────────────────────────────────┤
│  6. RECORD VERIFICATION                                     │
│     Update state file with:                                 │
│     - screenshot path                                       │
│     - verification timestamp                                │
│     - verified_by (AI vision / human)                       │
└─────────────────────────────────────────────────────────────┘
```

### 8.1 Test Signal Selection

| Effect Type | Recommended Test Signal | What to Look For |
|-------------|------------------------|------------------|
| Color processing | colorbars | Color shifts, saturation changes |
| Chroma bandwidth | colorbars | Bleeding on sharp color transitions |
| Spatial displacement | ramp_h, ramp_v | Visible warping/displacement |
| Scan effects | checker | Line-level artifacts |
| Gain/contrast | gray50 | Level shifts, clipping |
| Overall quality | sweep | Frequency response |

### 8.2 Workbench MCP Tools

| Tool | Purpose |
|------|---------|
| `workbench_load_shader` | Hot-reload WGSL into workbench |
| `workbench_set_params` | Adjust shader uniforms |
| `workbench_inject_signal` | Load test pattern |
| `workbench_screenshot` | Capture current output |
| `workbench_analyze` | AI vision analysis of output |
| `workbench_compare_visual` | Compare output vs reference |
| `workbench_record_sweep` | Create parameter sweep GIF |
| `workbench_doc_capture` | Screenshot with metadata for docs |

### 8.3 When Workbench Verification is Required

**Always required for:**
- New shader functions (cables, effects, devices)
- Changes to existing shader logic
- Parameter range adjustments that affect visual output

**Not required for:**
- Rust code changes (UI, data structures)
- Non-visual features (save/load, MIDI)
- Documentation updates

---

## 9. Gate Design Principles

### 9.1 Every Shader Function Needs THREE Gates

The research-first workflow requires three tiers of verification:

```json
{
  "composite_spec_complete": {
    "type": "inspection",
    "description": "Specification exists with NTSC bandwidth values and sources",
    "expected": "docs/CABLE_DEGRADATION_SPEC.md contains composite section with cited values",
    "status": "pending"
  },
  "composite_code_exists": {
    "type": "machine",
    "check": "grep 'fn composite_degrade' src/shaders/cables.wgsl",
    "status": "pending"
  },
  "composite_visual_verified": {
    "type": "inspection",
    "test_signal": "colorbars",
    "expected": "Visible chroma blur, dot crawl artifacts",
    "status": "pending"
  }
}
```

**Gate ordering (must pass in sequence):**
1. **spec_complete** - Research done, spec document exists
2. **code_exists** - Implementation based on spec
3. **visual_verified** - Output matches expected behavior

### 9.2 Inspection Gates Record Evidence

When a visual inspection passes, record:
- `screenshot`: path to captured image
- `verified_by`: "ai_vision" or "human"
- `verified_at`: ISO timestamp
- `notes`: any observations

### 9.3 Stubs Do Not Pass Visual Gates

A stub function like `fn svideo_degrade() { return color; }` will:
- PASS the machine gate (function exists)
- FAIL the visual gate (no visible degradation effect)

This is the correct behavior — machine gates verify existence, visual gates verify correctness.

---

## 10. Human Responsibilities

The human audit function cannot be automated:

1. **Direction setting** — What version/feature to work on next
2. **Scope decisions** — What's in/out for a version
3. **Quality review** — Does the code actually work as intended?
4. **Process audit** — Is the protocol being followed?
5. **Gate accuracy** — Are the gates actually checking what matters?
6. **Mismatch resolution** — When state doesn't match reality, what's the fix?

---

## 11. AI Responsibilities

The AI handles execution within the framework:

1. **State verification** — Run checks, report results
2. **Implementation** — Write code to satisfy gates
3. **Documentation** — Update DEVLOG, README, state file
4. **Communication** — Report progress, ask clarifying questions
5. **Following protocol** — Session start/end procedures

**The AI does NOT:**
- Decide what to work on (human direction)
- Adjust gates after implementation (that's cheating)
- Skip verification under momentum pressure
- Assume state from conversation history after compaction

---

## 12. Recovery After Context Loss

When a session starts after compaction:

1. **DO:** Read `project_state.json` first
2. **DO:** Run `verify_state.py` to confirm state accuracy
3. **DO:** Ask human to confirm direction if unclear
4. **DO NOT:** Assume state from conversation summary
5. **DO NOT:** Repeat completed work
6. **DO NOT:** Start new work without verifying current state

The state file is the source of truth, not the conversation history.

---

## 13. Bootstrapping

To initialize this protocol for Phosphlux:

1. Create `project_state.json` with current state (v0.12 complete)
2. Create `scripts/verify_state.py`
3. Run verification to confirm accuracy
4. Commit both files
5. Begin using protocol for v0.13

---

## 14. Verification Checklist for Shader Features

Before marking any shader feature complete, verify all items:

### Research Phase
- [ ] Primary sources identified (standards, datasheets, patents)
- [ ] Existing documentation checked (`docs/`, `research/`)
- [ ] Specification document created with transfer functions
- [ ] Quantitative values cited (not made up)

### Implementation Phase
- [ ] Code based on spec values (not arbitrary constants)
- [ ] Constants documented with source reference
- [ ] Function signature matches spec

### Verification Phase
- [ ] Workbench test with appropriate signal
- [ ] Screenshot captured as evidence
- [ ] Vision analysis confirms expected behavior
- [ ] `project_state.json` updated with verification evidence

### Documentation Phase
- [ ] Spec document linked to implementation
- [ ] DEVLOG updated with decisions
- [ ] Committed with screenshot in `output/`

---

## Document Metadata

**Version:** 1.1
**Adapted from:** Scientific Collaboration Protocol v1.0 (retired 22 Oct 2026; archived at `archive/scientific_collaboration_protocol_v1.0.md`)
**Created:** 2026-01-28
**Updated:** 2026-01-28 - Added Research Phase (Section 6.0), three-tier gates (Section 9.1), verification checklist (Section 14)
**For project:** Phosphlux (analog video synthesizer)
