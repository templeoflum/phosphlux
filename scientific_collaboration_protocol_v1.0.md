# Scientific Collaboration Protocol v1.0

**Version:** 1.0
**Status:** Case-study validated
**Created:** 2026-01-28
**Authors:** Claude + Hedawn
**Predecessor:** [scientific_collaboration_protocol_v0.1.md](scientific_collaboration_protocol_v0.1.md)
**Case study:** Distortion Data Experiment (Experiments 00--05, January 2026)

---

## Version History

- **v0.1** (2026-01-27): Initial draft. Eleven-phase sequential model with YAML provenance tracking. Written after Experiment 01 to codify the methodology that emerged during the [initial exploration](sources/source_conversation.txt) and first experiment.
- **v1.0** (2026-01-28): Revised after executing five experiments. Phases reduced from 11 to 6+3. Provenance simplified to Git. Two new principles added. Verification infrastructure, failure protocol, and human-AI collaboration model rewritten from empirical evidence.

---

## 1. Core Principles

### 1.1 Falsification-First

Design tests to fail. Seek disconfirmation. Agreement requires evidence.

Every experiment begins with numbered predictions, each carrying a quantitative pass/fail threshold defined before the code is written. "Does this work?" is not a hypothesis. "This mechanism will achieve similarity >= 0.999 across all tested parameters" is.

### 1.2 Isolation-Before-Integration

One tool, one purpose. Validate in isolation. Combine only after verification.

New modules do not modify existing code. Each experiment adds a self-contained module (composition pattern) so that passing tests for Experiment N remain undisturbed by Experiment N+1.

### 1.3 Defined Deliverables

Each phase has explicit outputs. No phase is complete without its deliverable.

The experiment completeness checklist (Section 4.7) is the operational expression of this principle: if any artifact is missing, the experiment is not done, regardless of whether the code runs.

### 1.4 Provenance Tracking

Every change logged. Every result traceable to its inputs. Every fix documented with before/after.

Git is the provenance system. Structured YAML logs are unnecessary when the repository history records every commit, the DEVLOG records every decision, and the metrics JSON records every measurement. The provenance layer should be as simple as possible while remaining auditable.

### 1.5 Failures Are Findings

*New in v1.0.*

Failed predictions are first-class scientific results, not blockers to be fixed. A prediction that fails with a clear explanation teaches more than one that passes unremarkably.

The protocol does not halt on failure (v0.1's STOP-DOCUMENT-ASSESS-FIX-REVALIDATE-RESUME sequence). Instead, failures are recorded as findings with root-cause analysis and added to the cross-experiment knowledge base. The experiment continues.

### 1.6 Machine Verification Over Manual Review

*New in v1.0.*

Every quantitative claim in prose must be machine-verifiable against a source-of-truth dataset. Humans write prose from memory; memory introduces errors. Machine gates catch these errors structurally, surviving context loss, session restarts, and momentum pressure.

Manual review (including AI vision inspection) remains necessary for layout, formatting, and semantic correctness --- but it is the second tier, not the first.

---

## 2. Architecture

The protocol operates at three levels:

```
Project Setup (once)
    Build the laboratory: tools, infrastructure, shared library
    |
    v
Experiment Cycle (repeated per experiment)
    Specification -> Implementation -> Execution -> Verification -> Documentation -> Commit
    |
    v  (after all experiments)
Capstone (once)
    Red Team -> Meta-Analysis -> Decision
```

### 2.1 Project Setup

The laboratory is built once and reused across experiments. It includes:
- Build system and shared library (Rust crate, Cargo.toml)
- GPU compute pipeline and CPU reference implementations
- MCP server for tool integration (build, test, verify, inspect)
- Verification scripts (claims, gates)
- Cross-experiment tracking (FINDINGS.md)

New experiments extend the shared library through composition (new modules, new workbench binaries) without modifying existing code.

### 2.2 Experiment Cycle

Six phases per experiment (Section 4). Each phase produces deliverables and must pass its exit gate before the next phase begins. In practice, Implementation and Execution often overlap: the workbench binary is both the test design and the execution engine.

### 2.3 Capstone

Three project-level phases (Section 5) executed after all experiments are complete. The dialectical structure from v0.1 is preserved:

```
Experiments 00-05: THESIS
    Build the case. Generate evidence. Document findings.
    |
    v
Red Team: ANTITHESIS
    External adversarial review. Attack methodology.
    Propose alternatives. The reviewer has no prior
    investment in the outcome.
    |
    v
Meta-Analysis: SYNTHESIS
    Evaluate the critique. Audit the process.
    Integrate valid points. Improve the protocol.
    |
    v
Decision: RESOLUTION
    Archive, iterate, or branch.
```

---

## 3. Execution Model

### 3.1 Dependencies Determine Order

```
If Phase B depends on Phase A:
    A must complete before B starts

If Phase B does not depend on Phase A:
    A and B may run concurrently
```

Within the experiment cycle, phases are numbered but not rigidly sequential. Implementation and Execution are often interleaved (the workbench binary encodes both the test design and the execution logic). Documentation begins during Implementation (the DEVLOG is written in real time).

### 3.2 The Natural Unit Is the Experiment

v0.1 imposed time bounds and gate checks at each of 11 phases. In practice, the natural unit of work is the experiment. Each experiment runs to completion in one or more sessions, producing a self-contained artifact. The gate check that matters is at the end: all verification gates pass before commit.

---

## 4. Experiment Phases

### Phase 1: Specification

**Input:** Research question, prior findings, open questions from FINDINGS.md

**Process:**
1. State the hypothesis as a testable claim
2. Define numbered predictions (P1, P2, ...) with quantitative pass/fail thresholds
3. Specify the test matrix: what will be measured, at what parameters, with what controls
4. Identify baselines (identity transforms, known-good data, SHA-256 hashes)
5. Write `spec.md`

**Deliverable:** `spec.md` containing hypothesis, predictions with thresholds, and test matrix.

**Exit gate:** Each prediction has a numeric threshold that can be mechanically evaluated as pass/fail. A stranger could read the spec and know what "success" means.

**Role division:**
- Human: provides direction, research question, constraints
- AI: proposes predictions with thresholds, identifies appropriate controls
- Both: iterate on threshold values and test coverage

### Phase 2: Implementation

**Input:** spec.md, shared library

**Process:**
1. Create new module (composition pattern --- no modification of existing code)
2. Write workbench binary that executes the test matrix
3. Write unit tests for the new module
4. The workbench IS the test design: it encodes predictions, measurements, and diagnostic output generation in executable form

**Deliverable:**
- `src/{module}.rs` (new module)
- `src/bin/{experiment}_workbench.rs` (workbench binary)
- Unit tests (in-module `#[cfg(test)]`)
- Integration tests (`tests/{experiment}.rs`)

**Exit gate:** `cargo test` passes. `cargo build --bin {workbench}` succeeds.

**Key insight:** Implementation and Test Design (v0.1 Phases 1-3) are fused. Writing the workbench binary is simultaneously writing the test specification. Separating them created false granularity in v0.1.

### Phase 3: Execution

**Input:** Working workbench binary

**Process:**
1. Run the workbench: `cargo run --bin workbench` (with experiment-specific env vars)
2. Workbench produces:
   - Metrics JSON (source of truth for all quantitative claims)
   - Diagnostic PNGs (visual evidence for each test phase)
   - Prediction evaluations (pass/fail against thresholds)
   - Test results (per-test pass/fail with measurements)

**Deliverable:**
- `output/{experiment}_metrics.json`
- `output/*.png` (diagnostic images)

**Exit gate:** Metrics JSON exists and contains evaluations for all predictions. No crashes or panics during execution.

### Phase 4: Verification

**Input:** Metrics JSON, diagnostic PNGs, claims.json

**Process:**
1. **Machine verification:** Run `python scripts/verify_claims.py experiments/{experiment}` to check all quantitative prose claims against the metrics JSON
2. **Visual inspection:** Inspect paper PDF and diagnostic PNGs for formatting, layout, and semantic correctness
3. **Test suite:** Run `cargo test` to confirm no regressions
4. **Gate check:** Run `python scripts/verify_gates.py check experiments/{experiment}` to verify both machine and inspection gates

**Deliverable:**
- `claims.json` (mapping prose claims to JSON computations --- see Section 7)
- `verification_gates.json` (declaring all gate requirements)
- Passing gate status for all machine and inspection gates

**Exit gate:** All claims pass. All inspection gates pass. No test regressions.

**Two-tier structure:**
- *Tier 1 (Machine):* Context-immune. Survives session restarts. Cannot be skipped under momentum pressure.
- *Tier 2 (Inspection):* Hash-tracked. Rebuilding an artifact invalidates its inspection, forcing re-review.

### Phase 5: Documentation

**Input:** Metrics JSON, diagnostic PNGs, DEVLOG notes, verification results

**Process:**
1. Write/update DEVLOG.md with session-level entries
2. Generate analysis figures (`reports/generate_report.py` or `scripts/generate_paper_figures.py`)
3. Write LaTeX paper (`paper/` directory with section files)
4. Compile and visually inspect paper PDF
5. Update experiment README.md with results summary
6. Update project-level FINDINGS.md with new findings (F-numbered)
7. Update project-level README.md experiment table

**Deliverable:**
- `DEVLOG.md` (session-level development log)
- `README.md` (experiment-level results summary)
- `reports/` (generated figures and analysis)
- `paper/` (LaTeX source + compiled PDF)
- Updated `FINDINGS.md` (project-level)
- Updated project `README.md`

**Exit gate:** All documents are internally consistent. Paper PDF passes visual inspection. FINDINGS.md includes all new findings with F-numbers.

### Phase 6: Commit

**Input:** All artifacts from Phases 1--5, passing verification gates

**Process:**
1. Run full gate check: `python scripts/verify_gates.py check experiments/{experiment}`
2. Run full test suite: `cargo test`
3. Stage all experiment files
4. Commit with descriptive message summarizing findings
5. Push to remote

**Exit gate:** Clean commit with all gates passing. No untracked experiment files.

### 4.7 Experiment Completeness Checklist

Every experiment must include all of the following before being considered complete:

```
[ ] spec.md               -- Formal specification with hypothesis and predictions
[ ] DEVLOG.md             -- Session-by-session development log
[ ] README.md             -- Updated with results summary
[ ] claims.json           -- Verification manifest for all quantitative claims
[ ] verification_gates.json -- Machine + inspection gate manifest
[ ] output/               -- Metrics JSON + diagnostic PNGs
[ ] reports/              -- Generated figures and analysis
[ ] paper/                -- LaTeX paper with compiled PDF
[ ] Integration tests     -- Automated correctness verification
[ ] FINDINGS.md updated   -- New findings added to project-level tracker
[ ] Project README updated -- Experiment status and summary
```

---

## 5. Project-Level Phases

These phases are executed once, after all experiments are complete. They operate on the full body of work, not on individual experiments.

### Phase 7: Red Team

**Input:** All experiment artifacts (papers, data, code, DEVLOGs). The reviewer has no prior exposure to the research process.

**Process:**
1. Review all papers and supporting data independently
2. Identify methodological weaknesses
3. Propose alternative explanations for results
4. Flag missing tests or controls
5. Check internal consistency (do the data support the claims?)
6. Check completeness (what is not addressed?)

**Deliverable:** Red team report with:
- Methodological critique (issue, severity, evidence, suggested remediation)
- Alternative explanations with distinguishing tests
- Missing elements
- Internal consistency assessment
- Strengths (honest review includes what was done well)
- Overall assessment

**Exit gate:** Critique is comprehensive, evidence-based, and honest. Not artificially harsh, not artificially gentle.

**Reviewer requirements:** Fresh perspective. No sycophancy. Critical skepticism, not bad faith. The reviewer should be a different agent or person than the one who conducted the experiments.

### Phase 8: Meta-Analysis

**Input:** All experiment artifacts, red team report, FINDINGS.md, DEVLOGs, this protocol

**Process:**

*Part A: Red Team Evaluation*
1. For each critique: valid, invalid, or partially valid?
2. Does the critique contain its own blind spots or overreach?
3. What did the reviewer miss or misrepresent?

*Part B: Process Audit*
1. Were all phases completed as specified?
2. Were deliverables actually generated or skipped?
3. Where did bias enter?
4. Did the protocol help or hinder?
5. What would change about the protocol itself?

*Part C: Synthesis*
1. Which critique points require paper revision?
2. Which points are acknowledged limitations vs. fatal flaws?
3. What protocol improvements feed forward?

**Deliverable:** Meta-analysis report covering all three parts, plus revised papers if needed.

**Exit gate:** Every red team point addressed. Process audit complete. Honest assessment of whether findings survive scrutiny.

### Phase 9: Decision

**Input:** Meta-analysis, revised papers, full body of work

**Process:**
1. Is the research question resolved?
2. What new questions emerged?
3. Should the laboratory be preserved, archived, or extended?
4. What artifacts should be kept?

**Decision options:**
- **ARCHIVE**: Experiment series complete. Seal the repository.
- **ITERATE**: Repeat with refined hypothesis or additional experiments.
- **BRANCH**: New research direction using the same infrastructure.

**Deliverable:** Decision record with reasoning and next steps.

**Exit gate:** Explicit decision recorded. No open loops.

---

## 6. Documentation Layers

### Layer 1: DEVLOG (per experiment)

**Purpose:** Capture thinking in motion. Real-time decisions, dead ends, failures, session boundaries.

**Format:** Markdown with session-level headers. Append-only within a session. Each session opens with context (what happened previously) and closes with a summary of what was accomplished.

**When:** Throughout all phases. Written during implementation, not after.

**Key distinction from v0.1:** The DEVLOG replaces the informal "Running Journal." It has more structure (sessions, findings, process notes) while remaining informal enough to capture hunches and dead ends.

### Layer 2: Experiment Artifact

**Purpose:** Self-contained, reproducible record of one experiment.

**Contents:**
- `spec.md` --- hypothesis, predictions, test matrix
- `claims.json` --- machine-verifiable claim manifest
- `verification_gates.json` --- gate requirements (machine + inspection)
- `output/` --- metrics JSON, diagnostic PNGs
- `reports/` --- generated figures and analysis
- `paper/` --- LaTeX source and compiled PDF

**Key property:** An experiment directory should be understandable in isolation. Another researcher should be able to read the spec, run the workbench, verify the claims, and read the paper without needing the rest of the repository.

### Layer 3: Git History

**Purpose:** Full audit trail. Provenance tracking.

**What it replaces:** v0.1's structured YAML provenance log. Git records every file change, every commit message, every merge. The DEVLOG adds semantic context that commit messages cannot carry. Together, they provide complete provenance without a separate logging system.

**Queryable via:** `git log`, `git diff`, `git blame`. Standard tooling, no custom infrastructure.

### Layer 4: FINDINGS.md (project-level)

**Purpose:** Cross-experiment knowledge accumulation.

**Contents:**
- Numbered findings (F1--F32 and growing) grouped by source experiment
- Disproven claims with explanations
- Cross-cutting themes
- Open questions (with resolution tracking as experiments answer them)
- Novel contributions assessment

**When:** Updated during Phase 5 (Documentation) of each experiment. The findings list grows monotonically --- findings are never deleted, only annotated if later experiments refine them.

### Layer 5: Scientific Paper (per experiment)

**Purpose:** Formal communication of findings in publication-standard format.

**Format:** LaTeX, compiled to PDF. Sections: abstract, introduction, background/methodology, results (per test phase), discussion, conclusion.

**Verification:** Machine claims check + visual inspection of compiled PDF. Both gates must pass.

---

## 7. Verification Infrastructure

This section describes the machine verification system that emerged from process failures in Experiments 02 and 03. It is the most important methodological innovation in this protocol.

### 7.1 Claims Manifest (`claims.json`)

Every quantitative claim in prose (paper, README, DEVLOG) is registered as a machine-verifiable entry:

```json
{
  "claims": [
    {
      "id": "paper_conclusion_speedup",
      "description": "Conclusion states streaming speedup is 6108x",
      "source_file": "paper/sections/conclusion.tex",
      "metric_path": "predictions[1].measured",
      "expected_value": 6107.9,
      "tolerance": 1.0,
      "comparison": "approximately_equals"
    }
  ]
}
```

Each claim specifies:
- **Where** the claim appears (source file)
- **What** metric it references (JSON path into the metrics file)
- **What** value is expected and with what tolerance
- **How** to compare (equals, approximately_equals, greater_than, less_than, etc.)

### 7.2 Claims Verification (`verify_claims.py`)

A Python script that:
1. Loads `claims.json` from the experiment directory
2. Loads the metrics JSON (source of truth)
3. Evaluates each claim against the metrics
4. Reports pass/fail for each claim
5. Exits with code 1 on any failure (functions as a build gate)

### 7.3 Verification Gates (`verification_gates.json`)

Declares two categories of gates:

**Machine gates:** Automated, context-immune, deterministic.
- Claims verification (all claims pass)
- Test suite (cargo test passes)

**Inspection gates:** Human/AI verified, hash-tracked.
- Paper PDF visual inspection
- Diagnostic PNG review
- Publication figure quality check

Each inspection gate records:
- The SHA-256 hash of the inspected artifact
- The timestamp of inspection
- Who performed the inspection

Rebuilding an artifact (e.g., recompiling the paper) changes its hash, automatically invalidating the inspection gate and requiring re-inspection.

### 7.4 Gate Checker (`verify_gates.py`)

A universal gate script that:
1. Checks all machine gates (claims, tests)
2. Checks all inspection gates (hash validity, inspection recorded)
3. Reports overall status
4. Exits with code 1 on any failure

Usage:
```bash
# Full check for one experiment
python scripts/verify_gates.py check experiments/03-transform-families

# Check all experiments
python scripts/verify_gates.py check all

# Record a visual inspection
python scripts/verify_gates.py inspect experiments/03-transform-families paper_pdf

# View gate status
python scripts/verify_gates.py status experiments/03-transform-families
```

### 7.5 Design Principle

Context thrives in planning, semantics, and communication. Machine verification must be the immutable foundation. The gate system does not rely on anyone *remembering* to verify --- it enforces verification structurally.

The two-tier system reflects a fundamental asymmetry:
- **Quantitative claims** can be verified mechanically with zero ambiguity
- **Visual/semantic quality** requires human or AI judgment and cannot be fully automated

Both tiers are necessary. Neither is sufficient alone.

---

## 8. Failure Protocol

### 8.1 Prediction Failures (Scientific)

A prediction that fails is a finding, not a problem to fix.

**Process:**
1. Record the prediction result (measured value vs. threshold)
2. Analyze the root cause
3. Assign a finding number (F-series)
4. Document in the DEVLOG, paper, and FINDINGS.md
5. Continue the experiment

**Do not:** Retroactively adjust thresholds. Discard inconvenient results. Treat failure as evidence of a bug (unless it actually is one).

The most informative experiment in the case study (Exp 05) had 3 of 5 predictions fail. All three failures traced to the same root cause (exact-match marker dependency), producing the central finding of the experiment.

### 8.2 Process Failures (Methodological)

A process failure is a gap between what should have been done and what was done.

**Process:**
1. Identify the gap (missing documentation, wrong prose, skipped verification)
2. Analyze the root cause (momentum pressure, incomplete plan, context loss)
3. Build infrastructure to prevent recurrence
4. Document the failure and the fix in the DEVLOG and FINDINGS.md

**Key insight:** Every methodology improvement in this project traces to a specific process failure. The claims verification protocol was invented after an audit found 6 prose errors. The two-tier gate system was built after a paper was committed with table overflow. The completeness checklist was created after an experiment was "completed" with only code and no documentation. Process failures are opportunities to make the infrastructure stronger.

### 8.3 What v0.1 Got Wrong

v0.1 prescribed a STOP-DOCUMENT-ASSESS-FIX-REVALIDATE-RESUME sequence for failures. This treats every failure as a crisis requiring full halt. In practice:

- Prediction failures require documentation, not halting
- Process failures require infrastructure improvements, not revalidation
- Only a genuine data integrity problem (corrupted metrics, wrong SHA-256) warrants halting execution

The revised protocol distinguishes between scientific failures (findings) and process failures (infrastructure triggers), and reserves the halt-and-revalidate sequence for integrity failures only.

---

## 9. Human-AI Collaboration Model

### 9.1 Role Division

| Function | Human | AI |
|----------|-------|-----|
| Direction setting | Primary | Proposes options |
| Hypothesis formation | Primary | Assists with structuring |
| Prediction design | Reviews and adjusts | Proposes predictions with thresholds |
| Implementation | Reviews | Primary (code, tests, workbench) |
| Execution | Triggers | Primary (runs workbench, processes output) |
| Analysis | Interprets | Primary (measurements, comparisons) |
| Writing | Reviews and edits | Primary (DEVLOG, paper, README) |
| Visual inspection | Performs or delegates | Performs (via vision capability) |
| Audit | Primary | Assists with cross-referencing |
| Quality control | Primary | Follows standards when specified |
| Process improvement | Identifies gaps | Implements infrastructure |

### 9.2 Key Asymmetry

The AI executes plans efficiently but does not flag when a plan is incomplete relative to established standards. The human's most critical function is the audit role: comparing what was done against what should have been done.

This was demonstrated in Experiment 02, where the AI implemented the code layer flawlessly (all tests passing, valid data produced) but did not flag that the experiment lacked documentation infrastructure. The human caught this by comparing the Experiment 02 directory against the Experiment 01 pattern.

### 9.3 Context Management

AI context is finite and subject to compaction. This has specific implications:

- **Machine gates survive context loss.** Claims.json and verify_claims.py work regardless of whether the AI remembers creating them.
- **Inspection obligations do not survive context loss.** If a vision pass is needed but exists only as a note in the conversation, it will be forgotten after compaction. This is why inspection gates are tracked in verification_gates.json with hash-based invalidation.
- **DEVLOGs are insurance against context loss.** Real-time documentation captures decisions that would be lost if the session is compacted before the experiment concludes.

### 9.4 The Audit Function

The human audit function operates at several levels:

1. **Completeness audit:** Does the experiment match the established pattern? (Missing artifacts, skipped steps)
2. **Consistency audit:** Do the numbers in prose match the numbers in the data? (Claims verification)
3. **Quality audit:** Does the paper look correct? Do the figures render properly? (Visual inspection)
4. **Process audit:** Is the methodology being followed? Are shortcuts being taken? (Pattern-matching against standards)

The first two are increasingly automated (completeness checklist, claims verification). The latter two remain human functions that benefit from fresh eyes and external perspective.

---

## 10. Case Study Evidence

This protocol was developed and validated through the Distortion Data Experiment, a series of six experiments (including one precursor) executed in January 2026. This section maps protocol elements to specific events in the case study.

### 10.1 Falsification-First (Principle 1.1)

**Evidence:** Every experiment from 01 onward defined numbered predictions with quantitative thresholds before code was written.

| Experiment | Predictions | Pass | Fail | Most Informative Result |
|-----------|-------------|------|------|------------------------|
| 01 | 3 | 2 | 1 | P2 fail: GPU loses at 128x128 (dispatch overhead) |
| 02 | 5 | 3 | 2 | P1 fail: GPU per-pixel cost is NOT constant |
| 03 | 5 | 3 | 2 | P2 fail: power-radius and log-polar are anticorrelated |
| 04 | 6 | 4 | 2 | P1 fail: NOT is algebraically identical to XOR(0xFF) |
| 05 | 5 | 2 | 3 | P1/P4/P5 fail: all trace to exact-match marker dependency |

Total: 24 predictions, 14 pass (58%), 10 fail (42%). The 42% failure rate indicates the predictions were genuinely falsifiable, not designed to succeed.

**Counterexample:** Experiment 00 (SPPC) lacked falsification criteria. Its "findings" included circular metrics (measuring a property guaranteed by construction), division by near-zero denominators, and claims that survived no external scrutiny. The contrast between Exp 00 and Exp 01 --- same research question, different rigor --- motivated the falsification-first principle.

### 10.2 Composition Pattern (Principle 1.2)

**Evidence:** Each experiment added a new module without modifying existing code:

- Exp 01: `encoder.rs`, `decoder.rs`, `distortion.rs`, `addressing.rs`, `gpu/`
- Exp 02: Added `encode_to_texture_sized()` (new functions, existing modules untouched)
- Exp 03: Added `src/transforms/` (6 new files), `src/gpu/transform_pipeline.rs`
- Exp 04: Added `src/value_transforms.rs`
- Exp 05: Added `src/robustness.rs`

Test counts grew monotonically: 17 -> 48 -> 55 -> 81 -> 101 -> 138. No regressions across five experiments because no existing code was modified.

### 10.3 Claims Verification (Principle 1.6)

**Origin:** Experiment 02, Session 3. After the paper was committed without visual inspection, the user triggered a cross-referencing audit. The audit checked 232 quantitative claims against `scale_metrics.json` and found 6 errors in hand-written prose:

- README contained an entire table from a pre-redesign run (30+ stale values)
- Paper rounded a threshold from 5x to 2x
- DEVLOG had wrong test counts

All mechanically generated data was correct. All errors were in prose written from memory.

**Fix:** `claims.json` manifest + `verify_claims.py` script. Every quantitative claim maps to a JSON computation. The script exits with code 1 on failure, functioning as a build gate.

**Validation:** The protocol caught an off-by-one error during its own creation, confirming the approach. By Experiment 05, the manifest contained 45 claims.

### 10.4 Two-Tier Verification Gates (Section 7)

**Origin:** Experiment 03. The paper was committed with table overflow errors that rendered text unreadable. All machine-verified data was correct; the visual layer failed because the vision pass obligation existed only in conversation context, which was lost during compaction.

**Fix:** `verification_gates.json` + `verify_gates.py`. Machine gates (claims, tests) are automated. Inspection gates (paper PDF, figures) are hash-tracked: rebuilding the artifact changes the hash, invalidating the previous inspection and forcing re-review.

**Design principle:** The gate system does not rely on anyone *remembering* to verify. It enforces verification structurally.

### 10.5 Failures as Findings (Principle 1.5)

**Evidence from Experiment 05:** Three of five predictions failed, and these failures were the most informative results:

- **P1 (dual encoding survives corruption):** Failed. Both P and 1/P copies occupy the same raster-order rows. Top-half corruption kills both simultaneously. This revealed that P/1/P duality changes radial mapping but not spatial data placement (Finding F27).

- **P4 (structured redundancy beats random):** Failed. Inverse power (1/P) has 6.87% address overlap with P, but random power has 11.31%. Lower overlap does not translate to better recovery (Finding F30).

- **P5 (graceful degradation under noise):** Failed. The degradation curve is a cliff, not a slope: perfect at sigma=0, zero at sigma=1. The exact-match validity marker creates a binary gate with no intermediate state (Finding F31).

All three failures trace to the same root cause --- the ordered-decode marker system requires exact byte values --- producing the experiment's central finding (F32): the mechanism is fundamentally lossless-channel-dependent.

Under v0.1's failure protocol, each of these would have triggered STOP-DOCUMENT-ASSESS-FIX-REVALIDATE-RESUME. Under v1.0, they are documented as findings and the experiment continues.

### 10.6 Stubs Are Not Plans (Phase 1 Exit Gate)

**Evidence from Experiment 02:** A stub README created during Exp 01's wrap-up was treated as the implementation plan for Exp 02. The stub covered the code (binary, encoder, tests) but omitted all documentation infrastructure (spec, DEVLOG, reports, paper).

The code ran correctly --- all tests passing, valid metrics produced. But the experiment-as-research-artifact was only 2% complete when audited. The human caught this by comparing experiment directories.

**Protocol response:** The completeness checklist (Section 4.7) was created. The Phase 1 exit gate requires that a spec exists with falsifiable predictions, not just a stub with vague directions.

### 10.7 Human Audit Function (Section 9.2)

**Evidence:** Every process improvement traces to a human audit:

| What the Human Caught | When | Infrastructure Built |
|----------------------|------|---------------------|
| Experiment 02 missing documentation | After Exp 02 code complete | Completeness checklist |
| 6 prose errors in Exp 01-02 papers | Cross-referencing audit, Exp 02 Session 3 | Claims verification protocol |
| Paper committed without visual inspection | After Exp 02 Session 2 commit | Verification gates system |
| Table overflow in Exp 03 paper | Post-commit review | Hash-based inspection invalidation |
| Figure text clipping in Exp 01 | Verification gates pass | Programmatic layout guards |

The AI did not independently flag any of these issues. This is the key asymmetry: the AI executes efficiently but does not spontaneously audit against external standards. The human audit function is irreplaceable.

### 10.8 AI Vision Capabilities and Limits (Section 7.3)

**Evidence from verification gates pass:** AI vision inspection reliably detected character corruption (backslash artifacts in matplotlib figures: `\%`, `vs.\`). Five instances found across Experiments 01 and 02.

However, AI vision missed text clipping in Experiment 01's Figure 18 (bar annotations extending above the y-axis maximum, cut off by the chart boundary). The text was partially readable and the affected values were visually unremarkable.

**Implication:** AI vision has high reliability for character-level corruption (unambiguous errors) and lower reliability for layout violations (require understanding intended visual structure). Programmatic layout guards (e.g., `ylim` headroom) should be the first line of defense; vision inspection is a second layer.

### 10.9 Context Loss and Infrastructure (Section 9.3)

**Evidence from Experiment 03:** The vision pass obligation existed only in conversation context. After context compaction, the obligation was forgotten. The paper was committed with table overflow errors.

**Protocol response:** Inspection gates are tracked in `verification_gates.json` with SHA-256 hash-based invalidation. The obligation to inspect is encoded in a file, not in memory. The gate script enforces it regardless of what anyone remembers.

This is the central insight of the verification infrastructure: machine verification must be context-immune. Anything that depends on someone remembering to do it will eventually be forgotten.

---

## 11. Appendix: What Changed from v0.1

### Phases

| v0.1 | v1.0 | Change |
|------|------|--------|
| Phase 0: Hypothesis | Phase 1: Specification | Expanded to include predictions with thresholds |
| Phase 1: Laboratory | Project Setup (one-time) | Removed as per-experiment phase; laboratory is built once |
| Phase 2: Controls | Merged into Phase 1 (Specification) | Controls are part of test design, not a separate phase |
| Phase 3: Test Design | Merged into Phase 2 (Implementation) | Code IS the test design; separating them created false granularity |
| Phase 4: Execution | Phase 3: Execution | Simplified; workbench binary handles all execution |
| Phase 5: Verification | Phase 4: Verification | Expanded with two-tier gate system |
| Phase 6: Interpretation | Merged into Phase 5 (Documentation) | Interpretation happens during documentation, not as separate phase |
| Phase 7: Documentation | Phase 5: Documentation | Added completeness checklist, FINDINGS.md updates |
| Phase 8: Red Team | Phase 7: Red Team (project-level) | Moved from per-experiment to capstone |
| Phase 9: Meta-Analysis | Phase 8: Meta-Analysis (project-level) | Moved from per-experiment to capstone |
| Phase 10: Review | Phase 9: Decision (project-level) | Moved from per-experiment to capstone |
| --- | Phase 6: Commit (new) | Added as explicit experiment-closing gate |

**Net change:** 11 per-experiment phases -> 6 per-experiment + 3 project-level = 9 total.

### Principles

| v0.1 | v1.0 | Change |
|------|------|--------|
| Falsification-First | Falsification-First | Kept, unchanged |
| Isolation-Before-Integration | Isolation-Before-Integration | Kept, refined to include composition pattern |
| Defined Deliverables | Defined Deliverables | Kept, operationalized as completeness checklist |
| Provenance Tracking | Provenance Tracking | Kept, simplified from YAML to Git |
| --- | Failures Are Findings (new) | Failed predictions are results, not blockers |
| --- | Machine Verification Over Manual Review (new) | Claims.json, not proofreading |

### Documentation Layers

| v0.1 | v1.0 | Change |
|------|------|--------|
| Running Journal | DEVLOG | Formalized with session structure |
| Phase Deliverables | Experiment Artifact | Packaged as self-contained directory |
| Provenance Log (YAML) | Git History | Simplified; structured YAML dropped |
| Scientific Paper | Scientific Paper | Unchanged |
| --- | FINDINGS.md (new) | Cross-experiment knowledge tracker |

### New Sections (not in v0.1)

- **Verification Infrastructure** (Section 7): Claims manifest, gate checker, two-tier design
- **Human-AI Collaboration Model** (Section 9): Role division, key asymmetry, context management
- **Case Study Evidence** (Section 10): Specific examples from Experiments 00--05

### Removed from v0.1

- **Structured YAML provenance specification** (Section "Provenance Tracking Specification"): Replaced by Git history + DEVLOG. The 40+ lines of YAML schema were never implemented; Git provides equivalent queryability with standard tooling.
- **Detailed YAML deliverable templates** (per-phase): Replaced by concrete file artifacts (spec.md, claims.json, metrics JSON). Structured YAML for hypothesis, controls, tests, execution logs, etc. was theoretical; the actual deliverables are simpler and more useful.
- **Journal prompts** (per-phase): Useful for learning but not used in practice. The DEVLOG captures the same reflections organically.

### Open Questions (from v0.1) --- Now Answered

| Question | v0.1 Status | v1.0 Answer |
|----------|-------------|-------------|
| Should provenance integrate with git? | Open | Yes. Git IS the provenance system. |
| Which parts can be automated? | Open | Machine verification for quantitative claims. Human/AI for visual inspection. Two tiers. |
| How to handle multiple hypotheses? | Open | Multiple predictions per experiment, each with independent pass/fail. |
| How do collaboration responsibilities divide? | Open | Human: direction + audit. AI: implementation + analysis. See Section 9. |
| Should phases have time limits? | Open | No. The natural unit is the experiment, not the phase. |

---

## Document Metadata

**Version:** 1.0
**Status:** Case-study validated
**Created:** 2026-01-28
**Authors:** Claude + Hedawn
**Case study:** Distortion Data Experiment, 6 experiments, January 2026
**License:** Open for modification and use
