"""
Component Research Agent

Specializes in:
- Finding datasheets for analog ICs
- Extracting transfer functions and key specs
- Building WGSL shader models from component behavior
- Tracking component substitutes and era-appropriate parts
"""

import asyncio
import aiohttp
import json
import re
from typing import List, Optional, Dict, Any
from datetime import datetime
from pathlib import Path

from .base_agent import BaseAgent
from .shared_state import SharedState
from .event_bus import EventBus, Event


class ComponentAgent(BaseAgent):
    """
    Component research agent for analog hardware emulation.

    Responsibilities:
    - Search for component datasheets (AllDatasheet, archive.org, etc.)
    - Extract key specs: transfer function, voltage/current limits, pinout
    - Create component spec.json files
    - Generate WGSL shader models from transfer functions
    - Track which devices use which components
    - Find substitutes for obsolete parts
    """

    # Datasheet sources
    DATASHEET_SOURCES = [
        "https://www.alldatasheet.com",
        "https://www.datasheetcatalog.com",
        "https://archive.org",
        "https://www.analog.com",
    ]

    # Common video synthesis IC families
    VIDEO_IC_FAMILIES = {
        "CA30xx": "RCA Linear ICs (video amps, arrays)",
        "CA31xx": "RCA Video/RF ICs",
        "LM3xx": "National general purpose",
        "AD8xx": "Analog Devices video",
        "EL2xxx": "Elantec/Intersil video",
        "LT1xxx": "Linear Tech precision",
    }

    def __init__(self, state: SharedState, event_bus: EventBus,
                 components_dir: str = None):
        self.components_dir = Path(components_dir) if components_dir else None
        self._pending_lookups = []
        self._pending_models = []
        super().__init__(state, event_bus)

    @property
    def agent_type(self) -> str:
        return "component"

    @property
    def subscriptions(self) -> List[str]:
        return [
            "dev.component_needed",    # Development needs a component model
            "research.schematic_parsed",  # Schematic analyzed, extract parts
            "orch.directive",
        ]

    async def process_event(self, event: Event) -> Optional[List[Dict]]:
        """Process incoming events."""

        if event.event_type == "dev.component_needed":
            part_number = event.payload.get("part_number", "")
            device = event.payload.get("device", "unknown")
            self._pending_lookups.append({
                "part_number": part_number,
                "device": device,
                "priority": "high",
            })

        elif event.event_type == "research.schematic_parsed":
            # Extract component list from schematic analysis
            components = event.payload.get("components", [])
            for comp in components:
                if comp not in [p["part_number"] for p in self._pending_lookups]:
                    self._pending_lookups.append({
                        "part_number": comp,
                        "device": event.payload.get("device", "unknown"),
                        "priority": "normal",
                    })

        elif event.event_type == "orch.directive":
            task = event.payload.get("task", "")
            if "component" in task.lower() or "datasheet" in task.lower():
                # Parse part numbers from task
                parts = re.findall(r'\b(CA\d{4}|LM\d{3,4}|AD\d{3,4}|TL\d{3})\b',
                                   task.upper())
                for part in parts:
                    self._pending_lookups.append({
                        "part_number": part,
                        "device": "unknown",
                        "priority": "high",
                    })

        return None

    async def do_work(self) -> Optional[List[Dict]]:
        """Execute pending component research tasks."""
        events = []

        # Process lookups
        if self._pending_lookups:
            lookup = self._pending_lookups.pop(0)
            result = await self._lookup_component(lookup)
            if result:
                events.append(result)

        # Process model generation
        if self._pending_models:
            model = self._pending_models.pop(0)
            result = await self._generate_model(model)
            if result:
                events.append(result)

        return events if events else None

    async def _lookup_component(self, lookup: Dict) -> Optional[Dict]:
        """Look up a component and create spec file."""
        part_number = lookup["part_number"]
        device = lookup["device"]

        self.log_action(f"Looking up {part_number}", {"device": device})

        # Check if we already have this component
        if self.components_dir:
            part_dir = self.components_dir / "ics" / part_number.lower()
            if (part_dir / "spec.json").exists():
                self.logger.info(f"Component {part_number} already documented")
                return None

        # Try to find datasheet info
        spec = await self._search_datasheet(part_number)

        if spec:
            # Save spec
            if self.components_dir:
                part_dir = self.components_dir / "ics" / part_number.lower()
                part_dir.mkdir(parents=True, exist_ok=True)
                with open(part_dir / "spec.json", 'w') as f:
                    json.dump(spec, f, indent=2)

            # Queue model generation
            self._pending_models.append({
                "part_number": part_number,
                "spec": spec,
            })

            return {
                "type": "component.found",
                "payload": {
                    "part_number": part_number,
                    "type": spec.get("type", "unknown"),
                    "description": spec.get("description", ""),
                    "has_datasheet": spec.get("datasheet_source") is not None,
                },
            }
        else:
            # Request user to find datasheet
            self.state.add_document_request(
                category="datasheets",
                description=f"Datasheet for {part_number}",
                url=f"https://www.alldatasheet.com/view.jsp?Searchword={part_number}",
                device=device,
            )

            return {
                "type": "component.datasheet_needed",
                "payload": {
                    "part_number": part_number,
                    "device": device,
                    "search_url": f"https://www.alldatasheet.com/view.jsp?Searchword={part_number}",
                },
            }

    async def _search_datasheet(self, part_number: str) -> Optional[Dict]:
        """Search for datasheet info. Returns basic spec if found."""
        # For now, return known components from our database
        # In production, would scrape datasheet sites

        known_components = {
            "CA3046": {
                "name": "CA3046",
                "type": "transistor_array",
                "manufacturer": "RCA/Intersil",
                "description": "NPN Transistor Array - 5 matched transistors",
                "specs": {
                    "vce_max_v": 15,
                    "ic_max_ma": 50,
                    "hfe_typical": 100,
                },
                "datasheet_source": "1978 RCA Linear IC Databook",
            },
            "CA3080": {
                "name": "CA3080",
                "type": "ota",
                "manufacturer": "RCA/Intersil",
                "description": "Operational Transconductance Amplifier",
                "specs": {
                    "supply_v": 15,
                    "gm_max": 9600,  # uS
                    "ibias_max_ma": 2,
                },
                "datasheet_source": "1978 RCA Linear IC Databook",
            },
            "LM311": {
                "name": "LM311",
                "type": "comparator",
                "manufacturer": "National/TI",
                "description": "Voltage Comparator",
                "specs": {
                    "supply_v": 36,
                    "response_ns": 200,
                },
                "datasheet_source": "TI Datasheet",
            },
        }

        return known_components.get(part_number.upper())

    async def _generate_model(self, model_info: Dict) -> Optional[Dict]:
        """Generate WGSL shader model from component spec."""
        part_number = model_info["part_number"]
        spec = model_info["spec"]

        self.log_action(f"Generating model for {part_number}")

        comp_type = spec.get("type", "unknown")

        if comp_type == "transistor_array":
            wgsl = self._generate_transistor_array_model(spec)
        elif comp_type == "ota":
            wgsl = self._generate_ota_model(spec)
        elif comp_type == "comparator":
            wgsl = self._generate_comparator_model(spec)
        else:
            wgsl = f"// TODO: Model for {part_number} ({comp_type})\n"

        # Save model
        if self.components_dir and wgsl:
            part_dir = self.components_dir / "ics" / part_number.lower()
            part_dir.mkdir(parents=True, exist_ok=True)
            with open(part_dir / "model.wgsl", 'w') as f:
                f.write(wgsl)

            return {
                "type": "component.model_generated",
                "payload": {
                    "part_number": part_number,
                    "type": comp_type,
                    "path": str(part_dir / "model.wgsl"),
                },
            }

        return None

    def _generate_transistor_array_model(self, spec: Dict) -> str:
        """Generate WGSL for transistor array."""
        name = spec.get("name", "TRANSISTOR_ARRAY")
        hfe = spec.get("specs", {}).get("hfe_typical", 100)

        return f"""// {name} Transistor Array Model
// Generated from spec

const {name}_BETA: f32 = {hfe}.0;
const {name}_VT: f32 = 0.026;  // Thermal voltage

fn {name.lower()}_bjt(vbe: f32, vce: f32) -> f32 {{
    let ic = exp(clamp(vbe, -0.5, 0.8) / {name}_VT);
    let sat = tanh(max(vce, 0.0) * 10.0);
    return ic * sat;
}}

fn {name.lower()}_diff_pair(v1: f32, v2: f32) -> f32 {{
    return tanh((v1 - v2) / (2.0 * {name}_VT));
}}
"""

    def _generate_ota_model(self, spec: Dict) -> str:
        """Generate WGSL for OTA."""
        name = spec.get("name", "OTA")
        gm_max = spec.get("specs", {}).get("gm_max", 9600)

        return f"""// {name} Operational Transconductance Amplifier Model
// Generated from spec

const {name}_GM_MAX: f32 = {gm_max}.0;  // uS

// OTA: Iout = gm * (V+ - V-)
// gm is controlled by Iabc (amplifier bias current)
fn {name.lower()}_ota(v_plus: f32, v_minus: f32, i_abc: f32) -> f32 {{
    let gm = i_abc * {name}_GM_MAX / 1000.0;  // Scale bias to gm
    let v_diff = v_plus - v_minus;
    // Soft limiting for large signals
    return tanh(v_diff * gm) * gm;
}}

// VCA application: voltage controlled amplifier
fn {name.lower()}_vca(signal: f32, cv: f32) -> f32 {{
    let gain = clamp(cv, 0.0, 1.0);
    return signal * gain;
}}
"""

    def _generate_comparator_model(self, spec: Dict) -> str:
        """Generate WGSL for comparator."""
        name = spec.get("name", "COMPARATOR")

        return f"""// {name} Comparator Model
// Generated from spec

// Hard comparator: output high when V+ > V-
fn {name.lower()}_compare(v_plus: f32, v_minus: f32) -> f32 {{
    return select(0.0, 1.0, v_plus > v_minus);
}}

// Soft comparator with adjustable sharpness
fn {name.lower()}_compare_soft(v_plus: f32, v_minus: f32, sharpness: f32) -> f32 {{
    let diff = v_plus - v_minus;
    return clamp(diff * sharpness + 0.5, 0.0, 1.0);
}}

// Window comparator: output high when signal is between thresholds
fn {name.lower()}_window(signal: f32, low: f32, high: f32) -> f32 {{
    let above_low = select(0.0, 1.0, signal > low);
    let below_high = select(0.0, 1.0, signal < high);
    return above_low * below_high;
}}
"""

    def _get_agent_state(self) -> dict:
        """Save agent-specific state."""
        return {
            "pending_lookups": self._pending_lookups,
            "pending_models": self._pending_models,
        }

    def _restore_agent_state(self, state: dict):
        """Restore agent-specific state."""
        self._pending_lookups = state.get("pending_lookups", [])
        self._pending_models = state.get("pending_models", [])
