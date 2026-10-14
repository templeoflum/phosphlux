Phosphlux · T-0521 check
# Cadet IV startup

---

**Patch:** Cadet IV H triangle → Wobulator → Wavefold Colorizer, with the Colorizer as output. Cadet IV starts selected. H ADJ has a slow LFO from 0.7 to 1.0. Existing palette entries remain unchanged.

**Reset:** File → Default Patch restores wiring and automation together. File → New Patch also resets editor automation, preventing old LFOs from attaching to reused device IDs. Reset behavior was checked in code, not by mouse.

**Scope:** The manager amended verification after the first run could not select Phosphlux through computer use. GPU tests replace manual add, wire, control, and motion checks. By-eye playability remains for Hedawn's performance session.

**Motion:** The final startup patch was rendered on Apple M5 Max/Metal at 640×480, at 0.0 and 1.5 seconds. Both the app's LFO update and shader time were applied. All three composed passes ran with sRGB 8-bit textures and linear clamp-to-edge sampling. The two saved PNGs show different colored wave patterns; both were opened and inspected.

- **Startup:** 303,570 of 307,200 display pixels differed.
- **Registry wiring:** Adding `lzx_cadet_iv` through the palette factory and connecting its V triangle to Wobulator changed 305,775 pixels.
- **Control:** Changing that added Cadet's V ADJ from 1.0 to 0.5 changed 307,066 pixels.

The wiring and control renders used the same time, isolating those changes. Assertions require each change to affect more than 1% of the picture. The test uses the same registry, graph connections, and parameter storage as the UI; it does not simulate mouse gestures.

**GPU check:** Ran:

```sh
CARGO_TARGET_DIR=.scratch/phosphlux-check/target \
PHOSPHLUX_STARTUP_FRAMES=/Users/donovan/work/phosphlux/docs/checks \
cargo test --locked --features startup-checks \
startup_patch_moves_and_registry_controls_work -- --ignored --nocapture
```

Exit 0: `1 passed; 0 failed; 0 ignored`. Log: `.scratch/T-0521-gpu.log`. The feature is opt-in so the circuit comparison's test inventory stays stable. An initial rerun changed only the report's filtered-test count; the opt-in feature resolved that difference. The existing float-render path and Cadet IV behavior were not changed.

**Launcher:** Executed `/Users/donovan/work/studio-operations/launchers/Open Phosphlux.command` once in this resumed run, through bash. An exported Cargo wrapper redirected its `/tmp` destination into this repo's `.scratch/launcher-target`; the shared launcher was not edited. The release build finished and logged:

```text
GPU: Apple M5 Max
Render plan: 3 reachable devices out of 3 total (output=2)
Render plan: 3 segments (1 scan modulators)
Window created with node editor layout
```

The log has a framebuffer-format warning and no logged error or panic. The app remains running, so no final launcher exit code is claimed. Log: `.scratch/T-0521-launcher-resumed.log`.

**Window capture:** One `screencapture -x .scratch/T-0521-desktop.png` attempt exited 0. The unedited capture is saved as `T-0521-window.png`. It shows the three wired devices, active LFO, Preview, and `OK` shader status. At capture time Cadet IV was marked as output and Preview showed its grey ramp. The log also records output changes after startup. Their cause was not observed. This capture establishes the window's visible state, not the initial Colorizer output or motion. The separate startup frames establish the startup patch's rendered output.

**Required checks:** Final `./check.sh` exited 0: build PASS (113 warnings), test PASS (106 warnings), clippy PASS (153 warnings), Overall PASS. Tests: `16 passed; 0 failed; 3 ignored`. Logs: `.scratch/phosphlux-check/run.6Fpp5U` and `.scratch/T-0521-check-resumed.log`.

**Circuit comparison:** Final `sh research/devices/lzx_cadet_iv/run_all.sh` exited 0. Output: `Checks judged: 120. Passed: 120. Failed: 0. Conditions not run: 0.` Verdict: `a match by FAITHFUL.md`. SHA-256 checks confirmed all seven `comparison/` files byte-identical to the pre-run manifest. FAITHFUL.md, its margins, and the Cadet IV implementation remain unchanged. Log: `.scratch/T-0521-comparison-resumed.log`; manifest: `.scratch/T-0521-comparison-before.json`.

**Limits:** No manual add, wire, slider, reset, or motion check was performed. No playing blocker was observed in the automated checks. Mouse usability and performance suitability await Hedawn's session. Saving patches remains unavailable, as the task anticipated.

**Hand-in:** Work is on master. The resumed instruction explicitly requests a local task commit; no push or network write is permitted. The first run's deferred-commit note is superseded. `git diff --check` passed before hand-in.

**Next:** Existing T-0522/T-0526 cover the play sheet; T-0524 covers Hedawn performing once and naming what is missing. No duplicate follow-up was proposed.
