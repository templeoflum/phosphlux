#!/usr/bin/env python3
"""
Basic tests for the swarm system.
"""

import asyncio
import sys
import tempfile
from pathlib import Path
import pytest

# Add parent to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from swarm import SharedState, EventBus, Event


def test_shared_state():
    """Test SharedState basic operations."""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Use tmpdir for both state and research
        state = SharedState(tmpdir, research_dir=tmpdir)

        # Test goal
        state.set_goal("Test goal")
        assert state.get_goal() == "Test goal"

        # Test research notes (new document-based approach)
        note_path = state.add_research_note(
            device="test-device",
            title="Test Note",
            content="This is test content",
            sources=["http://example.com"],
        )
        assert note_path != ""

        notes = state.get_research_notes()
        assert len(notes) == 1
        assert notes[0]["device"] == "test-device"

        # Test parameters
        state.set_parameters({"gain": 0.5, "offset": 0.1})
        params = state.get_parameters()
        assert params["gain"] == 0.5

        # Test agent status
        state.update_agent_status("test_agent", "running", {"task": "testing"})
        statuses = state.get_agent_statuses()
        assert "test_agent" in statuses
        assert statuses["test_agent"]["status"] == "running"

        # Test document collection tracking
        docs = state.get_collected_documents()
        assert "notes" in docs

        print("SharedState tests passed!")


def test_event_bus():
    """Test EventBus basic operations."""
    with tempfile.TemporaryDirectory() as tmpdir:
        bus = EventBus(tmpdir)

        # Subscribe
        bus.subscribe("test_agent", ["research.*", "dev.screenshot"])

        # Emit events
        event1 = bus.emit("research", "research.finding", {
            "topic": "Test",
            "finding": "Test finding",
        })
        assert event1.id is not None

        event2 = bus.emit("development", "dev.code_changed", {
            "file": "test.wgsl",
        })

        event3 = bus.emit("development", "dev.screenshot", {
            "path": "/tmp/test.png",
        })

        # Poll - should get finding and screenshot, not code_changed
        events = bus.poll("test_agent")
        assert len(events) == 2

        event_types = [e.event_type for e in events]
        assert "research.finding" in event_types
        assert "dev.screenshot" in event_types
        assert "dev.code_changed" not in event_types

        # Poll again - should be empty (already read)
        events = bus.poll("test_agent")
        assert len(events) == 0

        print("EventBus tests passed!")


@pytest.mark.asyncio
async def test_agents_basic():
    """Test basic agent instantiation."""
    with tempfile.TemporaryDirectory() as tmpdir:
        state = SharedState(tmpdir)
        bus = EventBus(tmpdir)

        from swarm import ResearchAgent, DevelopmentAgent, DocumentationAgent, ComponentAgent

        # Create agents
        research = ResearchAgent(state, bus)
        dev = DevelopmentAgent(state, bus)
        docs = DocumentationAgent(state, bus, docs_dir=f"{tmpdir}/docs")
        component = ComponentAgent(state, bus, components_dir=f"{tmpdir}/components")

        # Check they initialized
        assert research.agent_type == "research"
        assert dev.agent_type == "development"
        assert docs.agent_type == "documentation"
        assert component.agent_type == "component"

        print("Agent instantiation tests passed!")


@pytest.mark.asyncio
async def test_component_agent():
    """Test ComponentAgent component lookup and model generation."""
    with tempfile.TemporaryDirectory() as tmpdir:
        state = SharedState(tmpdir)
        bus = EventBus(tmpdir)

        from swarm import ComponentAgent

        components_dir = Path(tmpdir) / "components"
        agent = ComponentAgent(state, bus, components_dir=str(components_dir))

        # Simulate a component needed event
        event = Event(
            id="test-event-1",
            source="development",
            event_type="dev.component_needed",
            payload={"part_number": "CA3046", "device": "Paik/Abe"},
            timestamp="2024-01-01T00:00:00",
        )
        await agent.process_event(event)

        # Check that the lookup was queued
        assert len(agent._pending_lookups) == 1
        assert agent._pending_lookups[0]["part_number"] == "CA3046"

        # Run do_work to process the lookup AND model generation (both happen in one call)
        result = await agent.do_work()

        # Should have created spec.json
        spec_path = components_dir / "ics" / "ca3046" / "spec.json"
        assert spec_path.exists(), "spec.json should exist after lookup"

        # Should have created model.wgsl (both lookup and model gen happen in same do_work)
        model_path = components_dir / "ics" / "ca3046" / "model.wgsl"
        assert model_path.exists(), "model.wgsl should exist after do_work"

        # Verify model content has key functions
        model_content = model_path.read_text()
        assert "CA3046_BETA" in model_content
        assert "ca3046_bjt" in model_content
        assert "ca3046_diff_pair" in model_content

        # Verify spec.json content
        import json
        spec = json.loads(spec_path.read_text())
        assert spec["name"] == "CA3046"
        assert spec["type"] == "transistor_array"
        assert "hfe_typical" in spec["specs"]

        print("ComponentAgent tests passed!")


def test_model_benchmark():
    """Test ModelBenchmark basic operations."""
    with tempfile.TemporaryDirectory() as tmpdir:
        from swarm import ModelBenchmark

        benchmark = ModelBenchmark(tmpdir)

        # Check default variants were created
        assert len(benchmark.variants) >= 4
        assert "concise_v1" in benchmark.variants
        assert "structured_v1" in benchmark.variants

        # Add ground truth
        benchmark.add_ground_truth(
            topic="test topic",
            facts=["fact 1", "fact 2"],
            source="test source",
            confidence=0.9,
        )
        assert "test topic" in benchmark.ground_truth

        # Get best model (should return defaults if no data)
        model, variant = benchmark.get_best_model()
        assert model is not None
        assert variant is not None

        print("ModelBenchmark tests passed!")


def test_token_tracker():
    """Test TokenTracker basic operations."""
    with tempfile.TemporaryDirectory() as tmpdir:
        from swarm import TokenTracker

        tracker = TokenTracker(tmpdir)

        # Record cloud usage
        tracker.record_cloud(
            operation="test_op",
            input_tokens=100,
            output_tokens=50,
            latency_ms=500,
            agent="test",
        )

        # Record local usage
        tracker.record_local(
            model_name="llama3.2",
            operation="research_query",
            input_tokens=200,
            output_tokens=100,
            latency_ms=1000,
            agent="research",
        )

        # Get stats
        stats = tracker.get_session_stats()

        assert stats["cloud_tokens"]["input"] == 100
        assert stats["cloud_tokens"]["output"] == 50
        assert stats["local_tokens"]["input"] == 200
        assert stats["local_tokens"]["output"] == 100

        # Check cost analysis
        assert stats["cost_analysis"]["actual_cloud_cost_usd"] > 0
        assert stats["cost_analysis"]["savings_usd"] > 0

        print("TokenTracker tests passed!")


def main():
    print("Running swarm tests...\n")

    test_shared_state()
    test_event_bus()
    test_model_benchmark()
    test_token_tracker()
    asyncio.run(test_agents_basic())

    print("\n All tests passed!")


if __name__ == "__main__":
    main()
