# Swarm Agent Architecture Specification

**Version:** 0.1 (Draft)
**Date:** 2026-01-25
**Purpose:** Define a multi-agent system for parallel research, development, and documentation with continuous feedback loops.

---

## 1. Overview

### 1.1 Problem Statement

Traditional AI-assisted development is sequential:
1. Human asks question
2. AI researches
3. AI implements
4. Human reviews
5. Repeat

This leaves significant parallelism on the table. Research, development, and documentation could happen simultaneously, with each informing the others in real-time.

### 1.2 Proposed Solution

A **swarm of specialized agents** running in parallel, sharing state, and triggering each other through events. One orchestrator (Claude) monitors the swarm, resolves conflicts, and makes high-level decisions.

### 1.3 Design Principles

1. **Separation of Concerns** - Each agent type has one job
2. **Loose Coupling** - Agents communicate via events, not direct calls
3. **Shared State** - Common knowledge base all agents can read/write
4. **Eventual Consistency** - Agents may have stale data briefly; that's ok
5. **Human in the Loop** - Orchestrator can pause/redirect at any time
6. **Graceful Degradation** - System works with 1 agent or 10

---

## 2. Agent Types

### 2.1 Research Agent

**Purpose:** Gather, validate, and synthesize information about the current topic.

**Capabilities:**
- Web search and fetch
- Query local knowledge base
- Query local models (fast/deep tier)
- Extract info from images (schematics, references)
- Compare findings against implementation

**Inputs:**
- Research questions from orchestrator
- Discrepancy alerts from Development agent
- Gap notifications from Documentation agent

**Outputs:**
- Findings (structured data)
- Validation results (matches/conflicts with implementation)
- New questions discovered
- Reference materials located

**State Written:**
- `knowledge_base.findings[]`
- `knowledge_base.references[]`
- `knowledge_base.open_questions[]`

**Events Emitted:**
- `research.finding` - New information discovered
- `research.validation` - Implementation compared to reference
- `research.conflict` - Finding contradicts current implementation
- `research.gap` - Missing information identified

### 2.2 Development Agent

**Purpose:** Write code, tune parameters, and visually validate output.

**Capabilities:**
- Write/edit shader code
- Load shaders into workbench
- Set parameters
- Inject test signals
- Take screenshots
- View and evaluate screenshots
- Compare output to reference images
- Record parameter sweeps

**Inputs:**
- Implementation tasks from orchestrator
- Findings from Research agent
- Conflict alerts requiring code changes

**Outputs:**
- Code changes
- Visual output (screenshots)
- Parameter configurations
- Evaluation results ("output matches expectation")

**State Written:**
- `codebase.current_shader`
- `codebase.parameters`
- `visuals.screenshots[]`
- `visuals.evaluations[]`

**Events Emitted:**
- `dev.code_changed` - Shader code was modified
- `dev.screenshot` - New visual output captured
- `dev.evaluation` - Visual assessment completed
- `dev.milestone` - Significant progress achieved
- `dev.blocked` - Cannot proceed, needs research/input

### 2.3 Documentation Agent

**Purpose:** Record everything as it happens, maintain dev logs, capture artifacts.

**Capabilities:**
- Write markdown documents
- Capture and organize screenshots
- Record GIFs of parameter sweeps
- Generate commit messages
- Update changelogs
- Cross-reference findings with code changes

**Inputs:**
- All events from Research and Development agents
- Milestone notifications
- Orchestrator directives

**Outputs:**
- Dev log entries
- Research summaries
- Screenshot archives with metadata
- Commit-ready changesets

**State Written:**
- `docs.dev_log[]`
- `docs.research_notes[]`
- `docs.artifacts[]`
- `docs.pending_commits[]`

**Events Emitted:**
- `docs.logged` - Entry added to dev log
- `docs.gap_identified` - Documentation noticed missing info
- `docs.ready_to_commit` - Changeset ready for review

### 2.4 Orchestrator (Claude)

**Purpose:** High-level coordination, conflict resolution, human interface.

**Capabilities:**
- Start/stop/pause agents
- Set priorities and focus areas
- Resolve conflicts between agents
- Make decisions requiring judgment
- Interface with human user
- Approve commits and exports

**Inputs:**
- All events from all agents
- Human user messages
- Conflict escalations

**Outputs:**
- Directives to agents
- Responses to human
- Approvals/rejections
- Priority adjustments

**State Written:**
- `orchestrator.current_goal`
- `orchestrator.agent_status[]`
- `orchestrator.decisions[]`

**Events Emitted:**
- `orch.directive` - Task assigned to agent
- `orch.priority_change` - Focus shift
- `orch.approval` - Human/orchestrator approved something
- `orch.pause` - Halt agent activity
- `orch.resume` - Continue agent activity

---

## 3. Shared State Architecture

### 3.1 State Store Structure

```
shared_state/
├── knowledge_base/
│   ├── findings.json        # Research discoveries
│   ├── references.json      # URLs, docs, images
│   ├── open_questions.json  # Unanswered questions
│   └── embeddings/          # Vector store for semantic search
│
├── codebase/
│   ├── current_shader.wgsl  # Active shader code
│   ├── parameters.json      # Current param values
│   ├── history/             # Previous versions
│   └── exports/             # Exported nodes
│
├── visuals/
│   ├── screenshots/         # Captured outputs
│   ├── references/          # Target images to match
│   ├── diffs/               # Comparison results
│   └── sweeps/              # Recorded GIFs
│
├── docs/
│   ├── dev_log.md           # Running development log
│   ├── research_notes.md    # Compiled research
│   ├── artifacts.json       # Metadata for all artifacts
│   └── pending_commits/     # Staged changesets
│
└── orchestrator/
    ├── current_goal.json    # What we're trying to achieve
    ├── agent_status.json    # Health/progress of each agent
    ├── event_log.json       # All events (append-only)
    └── decisions.json       # Orchestrator decisions with rationale
```

### 3.2 State Access Patterns

| Agent | Reads | Writes |
|-------|-------|--------|
| Research | knowledge_base, codebase (code only) | knowledge_base |
| Development | knowledge_base, codebase, visuals | codebase, visuals |
| Documentation | ALL | docs |
| Orchestrator | ALL | orchestrator, can write directives anywhere |

### 3.3 Concurrency Model

- **File-level locking** - Only one agent writes to a file at a time
- **Append-only logs** - Events and history are append-only (no conflicts)
- **Last-write-wins** - For mutable state (parameters, current_shader)
- **Orchestrator arbitration** - Conflicts escalate to orchestrator

---

## 4. Event System

### 4.1 Event Structure

```json
{
  "id": "evt_20260125_143052_a1b2c3",
  "timestamp": "2026-01-25T14:30:52.123Z",
  "source": "research",
  "type": "research.finding",
  "payload": {
    "topic": "Paik/Abe op-amp",
    "finding": "Historical sources suggest CA3130, not LM741",
    "confidence": 0.7,
    "sources": ["https://archive.org/..."]
  },
  "triggers": ["dev.review_requested"]
}
```

### 4.2 Event Flow

```
┌─────────┐    event     ┌─────────────┐    route    ┌─────────┐
│  Agent  │─────────────►│ Event Bus   │────────────►│ Agents  │
└─────────┘              └──────┬──────┘             └─────────┘
                                │
                                │ log
                                ▼
                         ┌─────────────┐
                         │ Event Store │
                         │ (append)    │
                         └─────────────┘
```

### 4.3 Event Subscriptions

| Agent | Subscribes To |
|-------|---------------|
| Research | `dev.blocked`, `docs.gap_identified`, `orch.directive` |
| Development | `research.finding`, `research.conflict`, `orch.directive` |
| Documentation | `research.*`, `dev.*`, `orch.*` |
| Orchestrator | `*` (all events) |

### 4.4 Trigger Chains

Events can trigger other events, creating chains:

```
research.finding (CA3130 op-amp)
    └─► dev.review_requested
            └─► dev.code_changed (updated saturation model)
                    └─► dev.screenshot
                            └─► docs.logged
                            └─► research.validation (compare to ref)
```

---

## 5. Feedback Loops

### 5.1 Research → Development Loop

```
Research finds information
    │
    ▼
Emits research.finding
    │
    ▼
Development receives, evaluates relevance
    │
    ├─► Relevant: Adjust implementation
    │       └─► Emit dev.code_changed
    │
    └─► Conflicts: Request clarification
            └─► Emit dev.blocked (needs_research)
                    └─► Research investigates further
```

### 5.2 Development → Research Loop (Validation)

```
Development produces screenshot
    │
    ▼
Emits dev.screenshot
    │
    ▼
Research compares to reference images
    │
    ├─► Match: Emit research.validation (success)
    │
    └─► Mismatch: Emit research.conflict
            └─► Development adjusts
```

### 5.3 Documentation → Research Loop (Gap Detection)

```
Documentation logging progress
    │
    ▼
Notices missing context ("why this value?")
    │
    ▼
Emits docs.gap_identified
    │
    ▼
Research investigates
    │
    ▼
Emits research.finding
    │
    ▼
Documentation updates with context
```

### 5.4 Full Cycle Example

```
Goal: Implement Sandin IP differencing amplifier

T+0s:   Orchestrator emits orch.directive (implement Sandin diff amp)

T+1s:   Research starts: web search, local model queries [PARALLEL]
        Development starts: scaffold shader structure
        Documentation starts: create dev log entry

T+5s:   Research emits research.finding (diff amp uses 741, subtracts signals)

T+6s:   Development receives finding, implements subtraction
        Documentation logs the finding

T+10s:  Development emits dev.code_changed, dev.screenshot

T+11s:  Research validates screenshot against reference
        Documentation captures screenshot with metadata

T+15s:  Research emits research.conflict (output polarity inverted)

T+16s:  Development adjusts, new screenshot
        Documentation logs the iteration

T+20s:  Research emits research.validation (matches reference)

T+21s:  Orchestrator emits orch.milestone
        Documentation prepares commit

T+25s:  Orchestrator approves, commit made
```

---

## 6. Implementation Approach

### 6.1 Phase 1: Single-Process Simulation

Before building distributed agents, simulate the architecture in a single process:

```python
class AgentSimulator:
    def __init__(self):
        self.state = SharedState()
        self.event_bus = EventBus()
        self.agents = {
            'research': ResearchAgent(self.state, self.event_bus),
            'development': DevelopmentAgent(self.state, self.event_bus),
            'documentation': DocumentationAgent(self.state, self.event_bus),
        }

    async def run_cycle(self, goal: str):
        """Run one cycle of all agents."""
        # Orchestrator sets goal
        self.state.set_goal(goal)

        # Agents run in parallel
        await asyncio.gather(
            self.agents['research'].cycle(),
            self.agents['development'].cycle(),
            self.agents['documentation'].cycle(),
        )

        # Process events
        self.event_bus.process_pending()

        # Check for completion/conflicts
        return self.evaluate_progress()
```

### 6.2 Phase 2: MCP-Based Agents

Each agent becomes an MCP server that Claude can spawn:

```
claude spawn research-agent --goal "Paik/Abe colorizer circuit"
claude spawn dev-agent --workbench localhost:3030
claude spawn docs-agent --output docs/dev-logs/
```

Agents run as background processes, communicate via shared filesystem + event log.

### 6.3 Phase 3: Distributed Agents

For heavy workloads, agents can run on separate machines:

- Research agents on CPU-heavy machine (running local models)
- Development agent on GPU machine (running workbench)
- Documentation agent anywhere (just file I/O)
- Event bus becomes networked (Redis, NATS, or similar)

### 6.4 Technology Choices

| Component | Option A | Option B | Recommendation |
|-----------|----------|----------|----------------|
| Event Bus | File-based (JSON append) | Redis pub/sub | File for Phase 1-2, Redis for Phase 3 |
| State Store | Filesystem | SQLite | Filesystem (simpler, git-friendly) |
| Agent Runtime | Python async | Node.js | Python (matches existing MCP servers) |
| Inter-agent Protocol | JSON events | Protocol Buffers | JSON (human-readable, debuggable) |

---

## 7. MCP Server Extensions

### 7.1 New Tools for Orchestration

```python
# Swarm management
@server.tool()
def spawn_agent(agent_type: str, config: dict) -> str:
    """Spawn a new agent of the given type."""

@server.tool()
def stop_agent(agent_id: str) -> bool:
    """Stop a running agent."""

@server.tool()
def list_agents() -> list:
    """List all running agents and their status."""

@server.tool()
def send_directive(agent_id: str, directive: dict) -> bool:
    """Send a directive to a specific agent."""

# Event system
@server.tool()
def emit_event(event_type: str, payload: dict) -> str:
    """Emit an event to the event bus."""

@server.tool()
def subscribe_events(patterns: list) -> str:
    """Subscribe to events matching patterns."""

@server.tool()
def get_recent_events(since: str, types: list = None) -> list:
    """Get events since timestamp, optionally filtered by type."""

# Shared state
@server.tool()
def read_state(path: str) -> dict:
    """Read from shared state."""

@server.tool()
def write_state(path: str, data: dict) -> bool:
    """Write to shared state."""

@server.tool()
def watch_state(path: str) -> str:
    """Watch for changes to state path."""
```

### 7.2 Agent-Specific Tools

Each agent type gets specialized tools beyond the base set:

**Research Agent:**
- `search_web`, `fetch_url`, `query_local_model`
- `embed_document`, `search_embeddings`
- `compare_to_reference`, `validate_implementation`

**Development Agent:**
- `load_shader`, `set_params`, `screenshot`
- `inject_signal`, `record_sweep`
- `analyze_visual`, `diff_images`

**Documentation Agent:**
- `write_log`, `capture_artifact`
- `generate_commit_message`, `stage_changes`
- `cross_reference`, `generate_summary`

---

## 8. Example Session

### 8.1 Goal: Implement Jones Colorizer

```
USER: Implement the Dave Jones colorizer from the 1970s

ORCHESTRATOR: Starting swarm for "Jones Colorizer implementation"
  - Spawning research-agent
  - Spawning dev-agent (workbench: localhost:3030)
  - Spawning docs-agent

RESEARCH [T+2s]: Searching for "Dave Jones colorizer circuit"...
DEV [T+2s]: Creating scaffold shader...
DOCS [T+2s]: Initialized dev log: docs/dev-logs/jones-colorizer.md

RESEARCH [T+8s]: FINDING - Jones colorizer uses 4-channel voltage-controlled
                 colorizer with gray level keyers. Created 1974.

DEV [T+10s]: Received finding. Implementing 4-channel structure.

RESEARCH [T+12s]: FINDING - Key feature is "gray level keying" -
                  thresholds that switch color on/off at luminance levels

DEV [T+15s]: Implementing threshold-based keying. Screenshot taken.

RESEARCH [T+18s]: Comparing screenshot to reference...
RESEARCH [T+20s]: CONFLICT - Reference shows hard edges at thresholds,
                  our output has soft transitions

DEV [T+22s]: Adjusting - using step() instead of smoothstep()

DEV [T+25s]: New screenshot taken.

RESEARCH [T+28s]: VALIDATION SUCCESS - Output matches reference character

DOCS [T+30s]: Logged 3 iterations, 2 findings, 1 conflict resolution.
              Commit ready: "Implement Jones colorizer with gray-level keying"

ORCHESTRATOR: Milestone reached. Approve commit? [Y/n]

USER: y

DOCS [T+35s]: Committed. Exported node to src/nodes/exported/JonesColorizer.json
```

---

## 9. Open Questions

1. **Agent Autonomy Level** - How much can agents do without orchestrator approval?
2. **Conflict Resolution** - When research and development disagree, who wins?
3. **Resource Limits** - How many parallel agents? Token/compute budgets?
4. **Failure Handling** - What if an agent crashes mid-task?
5. **Human Override** - How does user interrupt/redirect the swarm?
6. **Persistence** - Should agent state survive restarts?
7. **Multi-User** - Can multiple humans orchestrate the same swarm?

---

## 10. Next Steps

1. [ ] Review and refine this specification
2. [ ] Decide on Phase 1 implementation scope
3. [ ] Build SharedState and EventBus classes
4. [ ] Implement ResearchAgent with existing MCP tools
5. [ ] Implement DevelopmentAgent with workbench integration
6. [ ] Implement DocumentationAgent
7. [ ] Build orchestration harness
8. [ ] Test with real device implementation task
9. [ ] Iterate based on findings

---

## Appendix A: Event Type Reference

| Event | Source | Payload | Typical Triggers |
|-------|--------|---------|------------------|
| `research.finding` | Research | topic, finding, confidence, sources | dev.review_requested |
| `research.validation` | Research | target, result, similarity_score | docs.logged |
| `research.conflict` | Research | expected, actual, severity | dev.code_changed |
| `research.gap` | Research | topic, question | (none) |
| `dev.code_changed` | Development | file, diff_summary | docs.logged, research.validation |
| `dev.screenshot` | Development | path, params, signal | docs.logged, research.validation |
| `dev.evaluation` | Development | assessment, confidence | docs.logged |
| `dev.milestone` | Development | description | docs.ready_to_commit |
| `dev.blocked` | Development | reason, needs | research.* (targeted query) |
| `docs.logged` | Documentation | entry_id, summary | (none) |
| `docs.gap_identified` | Documentation | context, question | research.* |
| `docs.ready_to_commit` | Documentation | files, message | orch.approval |
| `orch.directive` | Orchestrator | agent, task, priority | (agent-specific) |
| `orch.approval` | Orchestrator | item, decision | docs.commit |
| `orch.milestone` | Orchestrator | description | docs.logged |
| `orch.pause` | Orchestrator | agent, reason | (agent stops) |
| `orch.resume` | Orchestrator | agent | (agent continues) |
