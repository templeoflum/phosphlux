# Playing the Cadet IV — one page

Read from the code at master `5781856` (T-0526, 14 Oct 2026). Nothing here was watched on screen. Lines marked **unchecked** are guesses. Code paths are short: `cadet` = `apps/phosphlux/src/devices/lzx/cadet_iv.rs`, `editor` = `apps/phosphlux/src/ui/node_editor.rs`, `registry` = `apps/phosphlux/src/ui/device_registry.rs`, `main` = `apps/phosphlux/src/main.rs`.

**What it is.** It's a ramp generator with no inputs: two sawtooth ramps, one across the picture (H) and one down it (V). Each ramp gives three shapes (`cadet:127-131`, `cadet:320-323`). On its own it's a still grey picture. It doesn't use time (`cadet:232-262`), so any movement comes from LFOs or from the devices after it.

## The six jacks (outputs, top of the node, left to right; `cadet:173`)

| Jack | Panel | What you see if you look at it |
|---|---|---|
| `h_ramp` | J8 H /\| | Dark on the left, bright on the right |
| `h_inv` | J6 H \|\\ | The reverse: bright on the left, black at the right edge |
| `h_tri` | J10 H V | Bright at both edges with a sharp black line a little left of centre |
| `v_ramp` | J3 V /\| | Dark at the top, bright at the bottom |
| `v_inv` | J2 V \|\\ | The reverse |
| `v_tri` | J4 V V | Bright at the top and bottom with a black line a little above the middle |

The shapes come from `cadet:252-261`. Levels run from about 0.03 V to 1.08 V, where 1 V shows as full white (`cadet:231`). I worked them out from the constants at `cadet:17-33` and `cadet:88-97`. The black line is where the ramp crosses about 0.50 V (`cadet:33`).

Pictures of all six jacks, the circuit beside the app: [trim 1.0](../comparison/side_by_side_ntsc_pos1.0.png), [0.5](../comparison/side_by_side_ntsc_pos0.5.png), [0.0](../comparison/side_by_side_ntsc_pos0.0.png), [PAL 0.5](../comparison/side_by_side_pal_pos0.5.png).

## The three controls (Inspector, left panel, after you click the node; `cadet:309-318`)

- **H ADJ (R1)**, 0–1, starts at 1. Sets how tall the H ramp is (`cadet:39-44`). Turning it down dims the H jacks and slides the H triangle's black line to the right: about 45% across at 1.0, 58% at 0.7 and 71% at 0.5. Below about 0.24 the line goes off the right edge and the triangle becomes a plain slope. These positions are worked out from the code; whether the screen shows the same is **unchecked**.
- **V ADJ (R3)**, the same for the V jacks. The black line moves down, from about 43% at 1.0 to 68% at 0.5.
- **Timing (0 NTSC / 1 PAL)**. Anything from 0.5 up is PAL (`cadet:234`). It changes only the Cadet's own sums, and only slightly. The app itself always runs NTSC (`main:67`). T-0527 is changing this control into a switch.
- **Every control has an LFO.** The `~` button to the left of a slider steps it Off → Slow → Medium → Fast (`editor:1212-1245`), and a right-click stops it (`editor:1223-1234`). Moving the slider yourself also stops it (`editor:1302-1305`). The speed follows the BPM under Tempo (`editor:160-187`).

## Adding and wiring it

1. In **Devices**, open **LZX** (it starts closed, `editor:1018`) and click **[!] Cadet IV Ramps**. The node appears below the others (`editor:1040-1048`). The `[!]` means "needs validation" (`registry:53`, `registry:93-98`), even though the comparison passed 120 of 120 checks (`../comparison/report.md`).
2. **To connect**, drag from a jack's dot to another device's input dot (`editor:608-626`, `editor:755-793`). Every other device has a single input, `video_in` (`apps/phosphlux/src/devices/paik_abe/mod.rs:82-84` and the matching lines for each device). A new cable into an input replaces the old one (`apps/phosphlux/src/gpu/shader_composer.rs:280-282`).
3. **To hear it, so to speak**, select a device and press **O** with the mouse over the canvas, or click **Set as Output** (`editor:593-600`, `editor:1177-1183`). That device fills the Preview. If the Cadet itself is the output, you see only `h_ramp` (`cadet:172`, `cadet:265-266`). To look at another jack, run it through a device and make that device the output.
4. To unplug, drag the cable off the input (`editor:628-668`) or right-click the cable (`editor:801-809`).

## Three patches to try

**A. The start-up patch, pulled apart.** The app opens with Cadet `h_tri` → Wobulator → Wavefold Colorizer, with H ADJ swept 0.7–1.0 by a slow LFO (`main:83-101`); File → Default Patch brings it back (`main:341-345`). Drag the Wobulator's input cable from `h_tri` to `v_tri`, then to `h_inv`, and watch the black line turn sideways or disappear. Then raise the Wobulator's **Video Mod**, so the ramp's own brightness bends the picture (`apps/phosphlux/src/devices/paik_abe/mod.rs:403-430`).

**B. Plain ramp into colour.** File → New Patch. Add Cadet IV Ramps and **Wavefold Colorizer**. Wire `v_tri` → Colorizer `video_in` and make the Colorizer the output. Turn the Colorizer's **Fold Depth** up, then put LFOs on V ADJ (Slow) and Red Gain (Medium). Folding a ramp should give bands of colour that crawl as V ADJ moves; that's **unchecked** on screen.

**C. Moving raster.** File → New Patch. Wire Cadet `h_ramp` → **Wobulator** → **PED Colorizer**, and make PED the output. On the Wobulator, set **Speed** above 0 for motion (`mod.rs:412`) and **H Amplitude** to about 0.2. On PED, move **Threshold 1–3** to cut the ramp into flat colour steps. Then swap `h_ramp` for `h_inv` and compare (**unchecked** on screen).

## What doesn't work yet

- **No save or open.** The File menu has New Patch, Default Patch and Exit, nothing else (`main:334-349`). A patch and its LFOs are gone when you quit.
- **No way into the Cadet.** It has no inputs and no outside sync; the app's timing stands in for sync (`cadet:320-323`). There's no MIDI, clock, camera or audio either (from the T-0122 note `phosphlux-playing-today-2026-10.md`, written before the Cadet).
- **Only one jack per device input**, and only through `video_in`. No device has a separate modulation input, so "modulating" with the Cadet means feeding it in as the picture.
- **Mouse and keyboard only.** Ctrl-click connects only the first jack (`h_ramp`) to the first input (`editor:699-722`). Whether that's Control or Cmd on the Mac is **unchecked**. The Help menu doesn't list **O** (`main:351-364`).
- **You can't delete the output device** (`editor:567`, `editor:812`). Make another device the output first.
