#!/usr/bin/env python3
"""
Swarm Agent CLI

Run the multi-agent swarm for development tasks.
"""

import asyncio
import argparse
import logging
import sys
from pathlib import Path

# Add swarm module to path
sys.path.insert(0, str(Path(__file__).parent))

from swarm import Orchestrator, run_swarm


async def interactive_session(goal: str, state_dir: str = "swarm_state",
                               research_dir: str = None,
                               ollama_url: str = None, ollama_model: str = None):
    """Run an interactive swarm session."""
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s [%(name)s] %(levelname)s: %(message)s",
    )

    orchestrator = Orchestrator(
        state_dir,
        research_dir=research_dir,
        ollama_url=ollama_url,
        ollama_model=ollama_model,
    )
    paused_reason = None
    pending_pdf = None

    def on_pause(reason):
        nonlocal paused_reason
        paused_reason = reason
        print(f"\n{'='*60}")
        print(f"[SWARM PAUSED] {reason}")
        print(f"{'='*60}")
        print("\nCommands:")
        print("  resume     - Continue processing")
        print("  status     - Show swarm status")
        print("  tokens     - Show token usage stats")
        print("  benchmark  - Show model benchmark rankings")
        print("  stop       - Stop the swarm")
        print("  directive  - Send a directive to agents")
        print("  decide X   - Provide decision X")
        print("  commit     - Approve and prepare commit")
        print("  pdf_done <id> <path> - Confirm PDF downloaded")
        print("  pdfs       - List pending PDF downloads")
        print()

    def on_decision(payload):
        print(f"\n[DECISION NEEDED]")
        print(f"  Reason: {payload.get('reason', 'Unknown')}")
        if 'options' in payload:
            print(f"  Options: {payload['options']}")
        print()

    def on_pdf_needed(pdf_info):
        nonlocal pending_pdf
        pending_pdf = pdf_info
        print(f"\n{'='*60}")
        print("PDF DOWNLOAD REQUIRED")
        print(f"{'='*60}")
        print(f"Description: {pdf_info.get('description', 'Unknown')}")
        print(f"URL: {pdf_info.get('url', 'N/A')}")
        print(f"Save to: {pdf_info.get('suggested_path', 'research/devices/')}")
        print(f"Request ID: {pdf_info.get('request_id')}")
        print(f"{'='*60}")
        print("\nAfter downloading, use: pdf_done <request_id> <file_path>")
        print("Then 'resume' to continue.\n")

    orchestrator.set_callbacks(
        on_pause=on_pause,
        on_decision=on_decision,
        on_pdf_needed=on_pdf_needed,
    )

    # Start swarm in background
    swarm_task = asyncio.create_task(orchestrator.start(goal))

    print(f"\n{'='*60}")
    print(f"SWARM STARTED")
    print(f"Goal: {goal}")
    if ollama_model:
        print(f"Ollama model: {ollama_model}")
    print(f"{'='*60}\n")

    # Interactive command loop
    try:
        while orchestrator._running:
            # Check if swarm is paused
            if orchestrator._paused:
                try:
                    cmd = await asyncio.get_event_loop().run_in_executor(
                        None, input, "> "
                    )
                except EOFError:
                    break

                cmd = cmd.strip()
                cmd_lower = cmd.lower()

                if cmd_lower == "resume":
                    await orchestrator.resume()

                elif cmd_lower == "status":
                    status = orchestrator.get_status()
                    print(f"\nStatus: {'PAUSED' if status['paused'] else 'RUNNING'}")
                    print(f"Uptime: {status['uptime']}")
                    print(f"Agents:")
                    for name, info in status['agents'].items():
                        print(f"  {name}: {info.get('status', 'unknown')}")
                    print(f"Recent events: {len(status['recent_events'])}")
                    print(f"Pending PDFs: {status['pending_pdfs']}")
                    print(f"Token stats: cloud={status['token_stats']['cloud_tokens']}, "
                          f"local={status['token_stats']['local_tokens']}, "
                          f"saved=${status['token_stats']['savings_usd']:.4f}")
                    print(f"Best model: {status['best_model']['model']} / {status['best_model']['variant']}")
                    print()

                elif cmd_lower == "tokens":
                    stats = orchestrator.get_token_stats()
                    print(f"\n{'='*60}")
                    print("TOKEN USAGE")
                    print(f"{'='*60}")
                    print(f"Session duration: {stats['session_duration_seconds']:.1f}s")
                    print(f"\nCloud (Claude):")
                    print(f"  Input:  {stats['cloud_tokens']['input']:,} tokens")
                    print(f"  Output: {stats['cloud_tokens']['output']:,} tokens")
                    print(f"  Total:  {stats['cloud_tokens']['total']:,} tokens")
                    print(f"\nLocal (Ollama):")
                    print(f"  Input:  {stats['local_tokens']['input']:,} tokens")
                    print(f"  Output: {stats['local_tokens']['output']:,} tokens")
                    print(f"  Total:  {stats['local_tokens']['total']:,} tokens")
                    print(f"\nDistribution:")
                    print(f"  Cloud: {stats['totals']['cloud_percentage']:.1f}%")
                    print(f"  Local: {stats['totals']['local_percentage']:.1f}%")
                    print(f"\nCost Analysis:")
                    print(f"  Actual cloud cost: ${stats['cost_analysis']['actual_cloud_cost_usd']:.4f}")
                    print(f"  Local saved:       ${stats['cost_analysis']['savings_usd']:.4f}")
                    print(f"  Savings:           {stats['cost_analysis']['savings_percentage']:.1f}%")
                    print(f"{'='*60}\n")

                elif cmd_lower == "benchmark":
                    print(f"\n{'='*60}")
                    print("MODEL BENCHMARK RANKINGS")
                    print(f"{'='*60}")
                    orchestrator.print_benchmark_rankings()
                    best_model, best_variant = orchestrator.get_best_model()
                    print(f"Current recommendation: {best_model} with {best_variant}")
                    print()

                elif cmd_lower == "stop":
                    break

                elif cmd_lower.startswith("directive "):
                    task = cmd[10:]
                    orchestrator.send_directive(task)
                    print(f"Sent directive: {task}")

                elif cmd_lower.startswith("decide "):
                    decision = cmd[7:]
                    orchestrator.provide_decision(decision)
                    print(f"Decision recorded: {decision}")
                    await orchestrator.resume()

                elif cmd_lower == "commit":
                    commit_info = orchestrator.approve_commit()
                    print(f"\nCommit prepared:")
                    print(f"  Files: {commit_info.get('files', [])}")
                    print(f"  Message:\n{commit_info.get('message', '')}")
                    print()

                elif cmd_lower == "pdfs":
                    pdfs = orchestrator.get_pending_pdfs()
                    if pdfs:
                        print(f"\nPending PDF downloads ({len(pdfs)}):")
                        for pdf in pdfs:
                            print(f"  ID: {pdf.get('request_id')}")
                            print(f"    URL: {pdf.get('url')}")
                            print(f"    Save to: {pdf.get('suggested_path')}")
                            print()
                    else:
                        print("\nNo pending PDF downloads.")

                elif cmd_lower.startswith("pdf_done "):
                    parts = cmd[9:].split(" ", 1)
                    if len(parts) == 2:
                        request_id, file_path = parts
                        orchestrator.confirm_pdf_downloaded(request_id, file_path)
                        print(f"PDF confirmed: {file_path}")
                    else:
                        print("Usage: pdf_done <request_id> <file_path>")

                else:
                    print(f"Unknown command: {cmd}")

            else:
                # Not paused, just wait a bit
                await asyncio.sleep(0.5)

    except KeyboardInterrupt:
        print("\n\nInterrupted by user")

    finally:
        await orchestrator.stop()
        swarm_task.cancel()
        try:
            await swarm_task
        except asyncio.CancelledError:
            pass

    print("\nSwarm stopped.")


def main():
    parser = argparse.ArgumentParser(description="Run the swarm agent system")
    parser.add_argument(
        "goal",
        nargs="?",
        default="Test the swarm system",
        help="The goal for the swarm to work on",
    )
    parser.add_argument(
        "--state-dir",
        default=".swarm",
        help="Directory for swarm operational state (default: .swarm)",
    )
    parser.add_argument(
        "--research-dir",
        default="../../research",
        help="Directory for research outputs (default: ../../research - project root)",
    )
    parser.add_argument(
        "--ollama-url",
        default="http://localhost:11434",
        help="Ollama server URL (default: http://localhost:11434)",
    )
    parser.add_argument(
        "--ollama-model",
        default="llama3.2",
        help="Ollama model to use (default: llama3.2)",
    )
    parser.add_argument(
        "--non-interactive",
        action="store_true",
        help="Run without interactive prompt",
    )
    parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="Enable verbose logging",
    )

    args = parser.parse_args()

    if args.verbose:
        logging.basicConfig(level=logging.DEBUG)

    if args.non_interactive:
        asyncio.run(run_swarm(
            args.goal, args.state_dir,
            research_dir=args.research_dir,
            ollama_url=args.ollama_url,
            ollama_model=args.ollama_model,
        ))
    else:
        asyncio.run(interactive_session(
            args.goal, args.state_dir,
            research_dir=args.research_dir,
            ollama_url=args.ollama_url,
            ollama_model=args.ollama_model,
        ))


if __name__ == "__main__":
    main()
