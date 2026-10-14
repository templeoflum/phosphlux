# Playing the Cadet IV — one page

Read from the code at master `5a41a29` (T-0526, updated for the Timing switch by T-0652, 14 Oct 2026). Nothing here was watched on screen. Lines marked **unchecked** are guesses. Code paths are short: `cadet` = `apps/phosphlux/src/devices/lzx/cadet_iv.rs`, `editor` = `apps/phosphlux/src/ui/node_editor.rs`, `registry` = `apps/phosphlux/src/ui/device_registry.rs`, `main` = `apps/phosphlux/src/main.rs`, `abe` = `apps/phosphlux/src/devices/paik_abe/mod.rs`.

**What it is.** It's a ramp generator with no inputs: two sawtooth ramps, one across the picture (H) and one down it (V). Each ramp gives three shapes (`cadet:127-131`, `cadet:321-324`). On its own it's a still grey picture. It doesn't use time (`cadet:232-262`), so any movement comes from LFOs or from the devices after it.

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

## The three controls (Inspector, left panel, after you click the node; `cadet:309-319`)

- **H ADJ (R1)**, 0–1, starts at 1. Sets how tall the H ramp is (`cadet:39-44`). Turning it down makes the three H jacks move differently (`cadet:252-261`): `h_ramp` gets darker, `h_inv` gets brighter (it is 1.006 V minus the ramp), and `h_tri` gets brighter to the left of its black line and darker to the right, as the line slides right: about 45% across at 1.0, 58% at 0.7 and 71% at 0.5. Below about 0.24 the line goes off the right edge and the triangle becomes a plain slope. These positions are worked out from the code; whether the screen shows the same is **unchecked**.
- **V ADJ (R3)**, the same for the V jacks: `v_ramp` darker, `v_inv` brighter, and `v_tri` brighter above its black line and darker below it. The line moves down, from about 43% at 1.0 to 68% at 0.5.
- **Timing**, a switch: two buttons, **NTSC** and **PAL**, then the word Timing (`cadet:317`, `editor:1196-1212`). It starts on NTSC. Click one to pick it; the picture uses PAL at 0.5 and up, so NTSC is 0 and PAL is 1 (`cadet:234`). It changes only the Cadet's own sums, and only slightly. The app itself always runs NTSC (`main:67`).
- **Every slider has an LFO**; the Timing switch has none (`editor:1196`). The `~` button to the left of a slider steps it Off → Slow → Medium → Fast (`editor:1229-1263`), and a right-click stops it (`editor:1241-1252`). Moving the slider yourself also stops it (`editor:1320-1323`). The speed follows the BPM under Tempo (`editor:160-187`).

## Adding and wiring it

1. In **Devices**, open **LZX** (it starts closed, `editor:1018`) and click **[!] Cadet IV Ramps**. The node appears below the others (`editor:1040-1048`). The `[!]` means "needs validation" (`registry:53`, `registry:93-98`), even though the comparison passed 120 of 120 checks (`../comparison/report.md`).
2. **To connect**, drag from a jack's dot to another device's input dot (`editor:608-626`, `editor:755-793`). Every Paik/Abe device has a single input, `video_in` (`abe:82-84`, `abe:146-148`, `abe:211-213`, `abe:273-275`, `abe:340-342`, `abe:447-449`). Test Signal, like the Cadet, has no input (`apps/phosphlux/src/devices/generators.rs:118-121`). A new cable into an input replaces the old one (`apps/phosphlux/src/gpu/shader_composer.rs:287-297`). So to change which jack feeds a device, drag the new jack onto the same input. Dragging a cable's input end keeps the same jack and only moves the cable to another input (`editor:653-665`, `editor:755-793`).
3. **To hear it, so to speak**, select a device and press **O** with the mouse over the canvas, or click **Set as Output** (`editor:593-600`, `editor:1177-1183`). That device fills the Preview. If the Cadet itself is the output, you see only `h_ramp` (`cadet:172`, `cadet:265-266`). To look at another jack, run it through a device and make that device the output.
4. To unplug, drag the cable's input end off and let go on empty canvas (`editor:628-668`, `editor:786-793`), or right-click the cable (`editor:801-809`).

## Three patches to try

**A. The start-up patch, pulled apart.** The app opens with Cadet `h_tri` → Wobulator → Wavefold Colorizer, with H ADJ swept 0.7–1.0 by a slow LFO (`main:83-101`); File → Default Patch brings it back (`main:341-345`). (1) Drag from the Cadet's `v_tri` dot onto the Wobulator's `video_in` dot; it replaces the `h_tri` cable and the black line turns sideways. (2) Drag from `h_inv` onto the same `video_in` dot; the line goes and you get one bright-to-dark sweep. (3) Raise the Wobulator's **Video Mod** (`abe:441`), so the ramp's own brightness bends the picture (`abe:403-430`). On-screen look **unchecked**.

**B. Plain ramp into colour.** File → New Patch. Add Cadet IV Ramps and **Wavefold Colorizer**. Wire `v_tri` → Colorizer `video_in` and make the Colorizer the output. Turn the Colorizer's **Fold Depth** up (`abe:334`), then put LFOs on V ADJ (Slow) and Red Gain (Medium, `abe:331`). Folding a ramp should give bands of colour that crawl as V ADJ moves; that's **unchecked** on screen.

**C. Moving raster.** File → New Patch. Wire Cadet `h_ramp` → **Wobulator** → **PED Colorizer**, and make PED the output. On the Wobulator, **Speed** (starts at 0.5, `abe:442`) sets how fast it moves (`abe:412`); set **H Amplitude** to about 0.2 (`abe:438`). On PED, move **Threshold 1–3** (`abe:203-205`) to cut the ramp into flat colour steps. Then drag from the Cadet's `h_inv` dot onto the Wobulator's `video_in` dot to swap jacks, and compare (**unchecked** on screen).

## What doesn't work yet

- **No save or open.** The File menu has New Patch, Default Patch and Exit, nothing else (`main:334-349`). A patch and its LFOs are gone when you quit.
- **No way into the Cadet.** It has no inputs and no outside sync; the app's timing stands in for sync (`cadet:321-324`). There's no MIDI, clock, camera or audio either (from the T-0122 note `phosphlux-playing-today-2026-10.md`, written before the Cadet).
- **Only one jack per device input**, and only through `video_in`. No device has a separate modulation input, so "modulating" with the Cadet means feeding it in as the picture.
- **Mouse and keyboard only.** Ctrl-click connects only the first jack (`h_ramp`) to the first input (`editor:699-722`). Whether that's Control or Cmd on the Mac is **unchecked**. The Help menu doesn't list **O** (`main:351-364`).
- **You can't delete the output device** (`editor:567`, `editor:812`). Make another device the output first.
