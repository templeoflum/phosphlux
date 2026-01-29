#!/usr/bin/env python3
"""
Phosphlux Project State Verification

Verifies that project_state.json accurately reflects the codebase.
Run at session start to establish ground truth.
Run at session end to confirm work completed.

Usage:
    python scripts/verify_state.py           # Verify all versions
    python scripts/verify_state.py --version 0.12  # Verify specific version
    python scripts/verify_state.py --update  # Update state file with results
"""

import json
import subprocess
import sys
import os
import re
from pathlib import Path
from datetime import datetime
from typing import Tuple, Dict, Any, List

# ANSI colors for terminal output
class Colors:
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    BOLD = '\033[1m'
    END = '\033[0m'

def color(text: str, c: str) -> str:
    """Wrap text in color codes if terminal supports it."""
    if sys.stdout.isatty():
        return f"{c}{text}{Colors.END}"
    return text

def search_in_files(pattern: str, search_path: Path, recursive: bool = True) -> List[Path]:
    """
    Search for a pattern in files using pure Python.
    Returns list of files containing the pattern.
    """
    matches = []

    if not search_path.exists():
        return matches

    # If it's a file, search just that file
    if search_path.is_file():
        try:
            content = search_path.read_text(encoding='utf-8', errors='ignore')
            if pattern in content:
                matches.append(search_path)
        except Exception:
            pass
        return matches

    # It's a directory - search files within
    glob_pattern = '**/*' if recursive else '*'
    for filepath in search_path.glob(glob_pattern):
        if filepath.is_file() and filepath.suffix in ('.rs', '.wgsl', '.toml', '.json', '.md'):
            try:
                content = filepath.read_text(encoding='utf-8', errors='ignore')
                if pattern in content:
                    matches.append(filepath)
            except Exception:
                pass

    return matches

def run_check(check_cmd: str, project_root: Path) -> Tuple[bool, str]:
    """
    Execute a gate check command, return (passed, output).

    Supports:
    - grep -r "pattern" path  -> search for pattern in files
    - dir path / test -f path -> check file exists
    - cargo build             -> build project
    """
    try:
        # Handle grep commands - use Python-native search
        if check_cmd.startswith("grep"):
            # Parse: grep -r "pattern" path
            match = re.match(r'grep\s+(?:-[a-z]+\s+)*"([^"]+)"\s+(.+)', check_cmd)
            if match:
                pattern, search_path_str = match.groups()
                search_path_str = search_path_str.strip()

                # Normalize path
                search_path = project_root / search_path_str.replace('\\', '/')

                matches = search_in_files(pattern, search_path)

                if matches:
                    rel_paths = [str(m.relative_to(project_root)) for m in matches[:3]]
                    return True, f"Found in: {', '.join(rel_paths)}"
                return False, "Pattern not found"

        # Handle file existence checks
        if check_cmd.startswith("dir ") or check_cmd.startswith("test -f "):
            filepath = check_cmd.replace("dir ", "").replace("test -f ", "").strip()
            filepath = filepath.replace('\\', '/')
            full_path = project_root / filepath

            # Handle glob patterns
            if '*' in filepath:
                matches = list(full_path.parent.glob(full_path.name))
                if matches:
                    return True, f"Found: {', '.join(m.name for m in matches)}"
                return False, "File not found"

            if full_path.exists():
                return True, f"File exists: {filepath}"
            return False, f"File not found: {filepath}"

        # Handle cargo commands
        if check_cmd.startswith("cargo "):
            result = subprocess.run(
                check_cmd,
                shell=True,
                cwd=project_root,
                capture_output=True,
                text=True,
                timeout=300  # 5 min timeout for builds
            )
            if result.returncode == 0:
                return True, "Build successful"
            # Extract key error info
            stderr = result.stderr
            if "error" in stderr.lower():
                errors = [line for line in stderr.split('\n') if 'error' in line.lower()][:3]
                return False, f"Build failed: {'; '.join(errors)}"
            return False, f"Build failed with code {result.returncode}"

        # Unknown command type - try running it directly
        result = subprocess.run(
            check_cmd,
            shell=True,
            cwd=project_root,
            capture_output=True,
            text=True,
            timeout=60
        )
        return result.returncode == 0, result.stdout.strip() or result.stderr.strip()

    except subprocess.TimeoutExpired:
        return False, "Check timed out"
    except Exception as e:
        return False, f"Check error: {str(e)}"

def verify_version(version: str, version_data: Dict[str, Any], project_root: Path,
                   skip_build: bool = False) -> Tuple[int, int, int, Dict]:
    """
    Verify all gates for a version.
    Returns (passed, failed, mismatched, updated_gates).
    """
    passed = 0
    failed = 0
    mismatched = 0
    updated_gates = {}

    status = version_data.get('status', 'unknown')
    title = version_data.get('title', 'Untitled')

    print(f"\n{color('='*60, Colors.BLUE)}")
    print(f"{color(f'Version {version}', Colors.BOLD)}: {title}")
    print(f"Claimed status: {color(status, Colors.YELLOW)}")
    print(f"{color('='*60, Colors.BLUE)}")

    gates = version_data.get('gates', {})

    for gate_name, gate_data in gates.items():
        gate_type = gate_data.get('type', 'machine')  # default to machine gate
        check_cmd = gate_data.get('check', '')
        claimed_status = gate_data.get('status', 'pending')
        description = gate_data.get('description', '')

        # Handle inspection gates (visual verification)
        if gate_type == 'inspection':
            verified_by = gate_data.get('verified_by')
            screenshot = gate_data.get('screenshot')

            # Inspection gate passes only if it has been verified
            if claimed_status == 'pass' and verified_by and screenshot:
                status_str = color('VERIFIED', Colors.GREEN)
                passed += 1
            elif claimed_status == 'pass' and (not verified_by or not screenshot):
                # Claims pass but no evidence
                status_str = color('NO EVIDENCE', Colors.RED)
                mismatched += 1
                failed += 1
                print(f"  [{status_str:^18}] {gate_name}")
                print(f"             {color('^ Claims PASS but missing verification evidence', Colors.RED)}")
                print(f"             Need: workbench screenshot + verified_by")
                updated_gates[gate_name] = gate_data.copy()
                continue
            else:
                status_str = color('NEEDS VISUAL', Colors.YELLOW)

            print(f"  [{status_str:^18}] {gate_name}")
            updated_gates[gate_name] = gate_data.copy()
            continue

        # Skip build checks if requested
        if skip_build and 'cargo build' in check_cmd:
            print(f"  [{color('SKIP', Colors.YELLOW):^18}] {gate_name}")
            updated_gates[gate_name] = gate_data.copy()
            continue

        actual_pass, output = run_check(check_cmd, project_root)
        actual_status = 'pass' if actual_pass else 'pending'

        # Prepare updated gate data
        updated_gate = gate_data.copy()
        updated_gate['status'] = actual_status
        updated_gate['last_checked'] = datetime.now().isoformat()
        updated_gates[gate_name] = updated_gate

        # Determine display status
        if claimed_status == 'pass' and not actual_pass:
            # Claimed pass but actually failing - this is bad
            status_str = color('MISMATCH', Colors.RED)
            mismatched += 1
            failed += 1
        elif actual_pass:
            status_str = color('PASS', Colors.GREEN)
            passed += 1
        else:
            status_str = color('pending', Colors.YELLOW)

        print(f"  [{status_str:^18}] {gate_name}")

        if claimed_status == 'pass' and not actual_pass:
            print(f"             {color('^ Claimed PASS but actually FAILING', Colors.RED)}")
            print(f"             Check: {check_cmd}")
            print(f"             Result: {output}")

    return passed, failed, mismatched, updated_gates

def print_next_actions(state: Dict[str, Any]):
    """Print the next actions from state."""
    actions = state.get('next_actions', [])
    if actions:
        print(f"\n{color('Next Actions:', Colors.BOLD)}")
        for i, action in enumerate(actions, 1):
            print(f"  {i}. {action.get('action', 'Unknown')}")
            if action.get('description'):
                print(f"     {action['description']}")

def main():
    import argparse
    parser = argparse.ArgumentParser(description='Verify Phosphlux project state')
    parser.add_argument('--version', '-v', help='Verify specific version only')
    parser.add_argument('--update', '-u', action='store_true', help='Update state file with results')
    parser.add_argument('--skip-build', action='store_true', help='Skip cargo build checks')
    args = parser.parse_args()

    # Find project root (where project_state.json lives)
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    state_file = project_root / 'project_state.json'

    if not state_file.exists():
        print(color("ERROR: project_state.json not found", Colors.RED))
        print(f"Expected at: {state_file}")
        sys.exit(1)

    with open(state_file) as f:
        state = json.load(f)

    print(color("\n========== PHOSPHLUX STATE VERIFICATION ==========", Colors.BOLD))
    print(f"Current version: {color(state.get('current_version', 'unknown'), Colors.BLUE)}")
    print(f"Current phase: {color(state.get('current_phase', 'unknown'), Colors.BLUE)}")
    print(f"Last verified: {state.get('last_verified', 'never')}")
    if state.get('last_session_summary'):
        summary = state.get('last_session_summary')
        print(f"Last session: {summary[:80]}{'...' if len(summary) > 80 else ''}")

    total_passed = 0
    total_failed = 0
    total_mismatched = 0

    updated_versions = {}

    versions = state.get('versions', {})
    for version, version_data in versions.items():
        # Skip if specific version requested and this isn't it
        if args.version and version != args.version:
            updated_versions[version] = version_data
            continue

        passed, failed, mismatched, updated_gates = verify_version(
            version, version_data, project_root, skip_build=args.skip_build
        )
        total_passed += passed
        total_failed += failed
        total_mismatched += mismatched

        # Update version data
        updated_version = version_data.copy()
        updated_version['gates'] = updated_gates
        updated_versions[version] = updated_version

    # Summary
    print(f"\n{color('='*60, Colors.BLUE)}")
    print(color("SUMMARY", Colors.BOLD))
    print(f"  Passed: {color(str(total_passed), Colors.GREEN)}")
    print(f"  Pending: {total_failed - total_mismatched}")
    if total_mismatched > 0:
        print(f"  {color(f'MISMATCHED: {total_mismatched}', Colors.RED)} (claimed pass, actually fail)")
    print(f"{color('='*60, Colors.BLUE)}")

    # Print next actions
    print_next_actions(state)

    # Update state file if requested
    if args.update:
        state['versions'] = updated_versions
        state['last_verified'] = datetime.now().isoformat()

        with open(state_file, 'w') as f:
            json.dump(state, f, indent=2)
        print(f"\n{color('State file updated.', Colors.GREEN)}")

    # Open issues
    issues = state.get('open_issues', [])
    if issues:
        print(f"\n{color('Open Issues:', Colors.YELLOW)}")
        for issue in issues:
            print(f"  - {issue}")

    # Exit status
    if total_mismatched > 0:
        print(f"\n{color('ERROR: State claims do not match codebase!', Colors.RED)}")
        print("Run with --update to fix state file, or investigate the mismatches.")
        sys.exit(1)
    else:
        print(f"\n{color('State verified successfully.', Colors.GREEN)}")
        sys.exit(0)

if __name__ == "__main__":
    main()
