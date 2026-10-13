# Move probe (T-0137, 12 Oct 2026)

Probe: make /Users/donovan/work/phosphlux a full working copy of
/Users/donovan/work/0000000/analog-claude-video-synthesis-foundation-8LfZV ("old") and check it builds.
The old folder was only read. This file is deliberately uncommitted.

## Commands run (in /Users/donovan/work/phosphlux unless noted)

    git clone --no-hardlinks --no-checkout <old> .
    git fetch --no-tags <old> '+refs/heads/*:refs/heads/*' '+refs/remotes/origin/*:refs/remotes/origin/*' --update-head-ok
    git fetch --tags <old>                      # old repo has no tags; nothing came
    git checkout task/T-0074-ped-boundary
    git remote set-url origin git@github.com:templeoflum/phosphlux.git
    git branch --set-upstream-to=origin/master master
    # the clone had made refs/remotes/origin/task/* (old's local branches seen as a remote); removed to match old:
    git update-ref -d refs/remotes/origin/task/<each of the 7 task branches>
    git branch --unset-upstream task/T-0074-ped-boundary
    git symbolic-ref refs/remotes/origin/HEAD refs/remotes/origin/master
    # carry list = files changed vs HEAD + untracked + ignored, minus target/, .DS_Store, __pycache__/, .pytest_cache/
    rsync -a --files-from=.scratch/carry.txt <old>/ ./
    PATH=$HOME/.cargo/bin:$PATH CARGO_NET_OFFLINE=true ./check.sh

No network was used: every fetch was from the local path, and cargo ran offline.

## Checks

- Refs: `diff` of `git for-each-ref --format='%(refname) %(objectname) %(upstream)'` old vs new: no differences
  (9 local branches, remote-tracking refs origin/master, origin/archive/node-editor-v1 and origin/HEAD, 0 tags). Checked-out branch in both: task/T-0074-ped-boundary (a44fde5).
- origin URL: git@github.com:templeoflum/phosphlux.git in both.
- Carried work files: 44 (Cargo.toml, Cargo.lock, BASELINE.md, BLOCK_COMPATIBILITY_VALIDATION.md, research/schematics/rutt-etra/ (2),
  vendor/block-0.1.6/ (5), the five ignored source PDFs, .claude/settings.local.json, screenshot.png, 27 test_signals PNGs).
  `shasum -a 256` lists old vs new: identical.
- All 442 tracked files: `shasum -a 256` old vs new: identical.
- `git status --porcelain -uall`: identical in both (new also has .scratch/, ignored by its own .gitignore, and this file).
- `git fsck --no-dangling`: exit 0, no output.
- ./check.sh output (exit 0):

      Phosphlux check (cargo build/test/clippy --locked)
      build: PASS (exit 0, 112 warnings)
      test: PASS (exit 0, 106 warnings)
      test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
      clippy: PASS (exit 0, 152 warnings)
      Logs: /tmp/phosphlux-check/run.nmJkMy
      Overall: PASS

  cargo 1.99.0, rustc 1.99.0. Logs show the phosphlux crate built from /Users/donovan/work/phosphlux/apps/phosphlux,
  with the `block` patch pointing at the new folder's vendor/block-0.1.6.

## Rough edges / what differs

- Not a clean build: check.sh always uses CARGO_TARGET_DIR=/tmp/phosphlux-check/target, shared with runs from the old
  folder, so dependencies were reused and only the phosphlux crate was recompiled (whole run ~7 s). A from-scratch build
  in the new folder has not been tried.
- File modes: the old repo has core.filemode=false (and core.symlinks=false), Windows-era settings, and almost every file
  in its working tree is mode 755 on disk. The new clone uses git defaults (filemode=true), so tracked files were checked
  out as 644, and the two copied files Cargo.toml and Cargo.lock show `mode change 100644 => 100755` in `git diff` here.
  Content is identical; I didn't change any modes or git config.
- Not carried (as the task said): build output (target/ at the root and under tools/device-workbench/workbench-app/),
  .DS_Store, __pycache__, .pytest_cache. The first plain `cargo build` here without check.sh will build from scratch.
- .claude/settings.local.json (local Claude settings) came across; I didn't review it.
- The old folder's .git/hooks has only samples, no stash, no extra worktrees, so nothing was lost there.
- Still pointing at the old folder (out of scope here): the launcher (studio-operations/launchers/Open Phosphlux.command)
  and the board's project folder. Branches are not merged.
- check.sh warns about future incompatibility (float_literal_f32_fallback, "will become a hard error in a future release"). I didn't compare the warnings with a run from the old folder.
