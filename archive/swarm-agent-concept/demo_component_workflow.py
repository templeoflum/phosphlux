#!/usr/bin/env python3
"""
Demo: Component lookup and WGSL model generation workflow.

This demonstrates the ComponentAgent's ability to:
1. Look up a component by part number
2. Find its datasheet info (from internal database or request download)
3. Generate a WGSL shader model from the transfer function
4. Save both spec.json and model.wgsl to the component library
"""

import asyncio
import sys
import io
from pathlib import Path

# Fix Windows encoding for Unicode
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')

# Add swarm to path
sys.path.insert(0, str(Path(__file__).parent))

from swarm import SharedState, EventBus, Event, ComponentAgent


async def demo_component_lookup():
    """Demo the component lookup workflow."""

    # Use actual research directories
    project_root = Path(__file__).parent.parent.parent
    research_dir = project_root / "research"
    components_dir = research_dir / "components"
    state_dir = project_root / ".swarm"

    print(f"Project root: {project_root}")
    print(f"Components dir: {components_dir}")
    print()

    # Initialize
    state = SharedState(str(state_dir), research_dir=str(research_dir))
    bus = EventBus(str(state_dir))

    agent = ComponentAgent(state, bus, components_dir=str(components_dir))

    # Test components - some known, some unknown
    test_components = [
        ("CA3046", "Paik/Abe"),  # Should find in database
        ("CA3080", "Sandin IP"),  # Should find in database (OTA)
        ("LM311", "Generic"),     # Should find in database (comparator)
        ("CA3019", "Paik/Abe"),   # NOT in database - should request download
    ]

    print("=" * 60)
    print("COMPONENT LOOKUP DEMO")
    print("=" * 60)

    for part_number, device in test_components:
        print(f"\n--- Looking up: {part_number} (used in {device}) ---")

        # Check if already exists
        spec_path = components_dir / "ics" / part_number.lower() / "spec.json"
        model_path = components_dir / "ics" / part_number.lower() / "model.wgsl"

        if spec_path.exists():
            print(f"  [CACHED] Component already in library")
            print(f"    Spec: {spec_path}")
            if model_path.exists():
                print(f"    Model: {model_path}")
            continue

        # Create event
        event = Event(
            id=f"demo-{part_number}",
            source="demo",
            event_type="dev.component_needed",
            payload={"part_number": part_number, "device": device},
            timestamp="2024-01-01T00:00:00",
        )

        # Process event
        await agent.process_event(event)

        # Run do_work to process
        result_events = await agent.do_work()

        if result_events:
            for evt in result_events:
                if evt["type"] == "component.found":
                    print(f"  [NEW] Component found: {evt['payload']['description']}")
                    print(f"    Spec: {spec_path}")
                    print(f"    Model: {model_path}")
                elif evt["type"] == "component.datasheet_needed":
                    print(f"  [MISSING] Datasheet needed!")
                    print(f"    Search URL: {evt['payload']['search_url']}")
                elif evt["type"] == "component.model_generated":
                    print(f"  [NEW] WGSL model generated: {evt['payload']['path']}")
        else:
            print("  (No events returned)")

    # Show document requests (datasheets needed)
    requests = state.get_pending_documents()
    if requests:
        print("\n" + "=" * 60)
        print("DOCUMENT REQUESTS (datasheets needed)")
        print("=" * 60)
        for req in requests:
            print(f"  - {req['description']}")
            print(f"    URL: {req.get('url', 'N/A')}")

    # Show what we have in the component library
    print("\n" + "=" * 60)
    print("COMPONENT LIBRARY STATUS")
    print("=" * 60)

    ics_dir = components_dir / "ics"
    if ics_dir.exists():
        for part_dir in sorted(ics_dir.iterdir()):
            if part_dir.is_dir():
                has_spec = (part_dir / "spec.json").exists()
                has_model = (part_dir / "model.wgsl").exists()
                status = "[OK]" if (has_spec and has_model) else "[--]"
                print(f"  {status} {part_dir.name.upper()}: spec={has_spec}, model={has_model}")


async def demo_wgsl_output():
    """Show example WGSL output for a component."""
    project_root = Path(__file__).parent.parent.parent
    model_path = project_root / "research" / "components" / "ics" / "ca3046" / "model.wgsl"

    print("\n" + "=" * 60)
    print("EXAMPLE WGSL OUTPUT (CA3046)")
    print("=" * 60)

    if model_path.exists():
        content = model_path.read_text()
        # Print first 40 lines
        lines = content.split("\n")[:40]
        for line in lines:
            print(f"  {line}")
        if len(content.split("\n")) > 40:
            print("  ...")
    else:
        print("  (Model not found - run demo_component_lookup first)")


if __name__ == "__main__":
    asyncio.run(demo_component_lookup())
    asyncio.run(demo_wgsl_output())
