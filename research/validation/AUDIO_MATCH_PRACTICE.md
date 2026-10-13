# How audio emulations judge a match, and what carries over to video (T-0289)

Reading only. Gathered for T-0293, which writes down what "faithful" means for the Cadet IV
(draft: [`../devices/lzx_cadet_iv/FAITHFUL.md`](../devices/lzx_cadet_iv/FAITHFUL.md)).

Each statement is marked:
- **read**: from the cited source. Quotes are word for word from the PDF text.
- **mine**: my reading or arithmetic. No source says it.

I read the full text of five papers (Esqueda 2017, Holters 2011, Werner 2014, Wright 2020 and Chowdhury 2020) from PDFs
downloaded into `.scratch/pdfs/`. The two maker sources (Brainworx and noshigeSound) are reported through a trade site and
a blog. I did not open the makers' own pages for them.

## Summary

| # | Approach | Compared against | Measurements | Match threshold |
|---|---|---|---|---|
| 1 | Circuit model checked against SPICE (Esqueda et al. 2017) | LTspice simulation of the schematic | Transfer curve (output volts for each input volt) and its absolute difference; waveforms over time; an aliasing score | Difference "below 1 mV" reported as a good match, not set in advance. Aliasing: ANMR below −10 dB counts as inaudible |
| 2 | Circuit model checked against the real unit (Holters & Zölzer 2011) | A real Dunlop Crybaby wah pedal, measured | Frequency response at 3 pedal positions; output level and THD against input level | None stated. The authors judge by eye and put remaining gaps down to part tolerances |
| 3 | Circuit model checked against both SPICE and recordings (Werner et al. 2014) | SPICE, and a sample set recorded from a real TR-808 | Waveform over time and pitch over time against SPICE; tuning, timing and behaviour features against the recordings; informal listening | None stated. "good agreement" |
| 4 | Black-box model trained on the real unit (Wright et al. 2020) | Recordings of the real amplifiers | Error-to-signal ratio (ESR) on held-out audio; MUSHRA listening test | None set in advance. ESR of 0.2–4.2 % reported; a mean MUSHRA score of 90 or more read as "Excellent" |
| 5 | Model trained on SPICE because the real unit was too costly (Chowdhury 2020) | SPICE simulation of the Klon Centaur | ESR; time and frequency plots; subjective listening | Validation ESR "less than 2%" reported, not set in advance |
| 6 | Makers (secondary sources) | Brainworx: the real console, plus part tolerances from datasheets. noshigeSound: captures of the real unit | Live comparison; null test (model output flipped and added to the hardware output, leaving only the difference) | Brainworx: none published. noshigeSound: residual 43–58 dB below the signal, 52 dB on average, by the maker's own examples |

## 1. Circuit model checked against SPICE: Esqueda, Pöntynen, Parker, Bilbao, "Virtual Analog Models of the Lockhart and Serge Wavefolders", *Applied Sciences* 7(12):1328, 2017

Source: [open-access PDF (Aalto)](https://acris.aalto.fi/ws/portalfiles/portal/16820117/esqueda_et_al_applsci_07_01328_v2_1.pdf) · [MDPI page](https://www.mdpi.com/2076-3417/7/12/1328)

This is the closest audio case to Phosphlux: a synthesizer circuit rebuilt from its schematic.

- **Reference (read):** "validated by comparing it against a reference simulation obtained using the SPICE … software LTspice". No real Lockhart or Serge unit is measured.
- **Measurements (read):** the transfer curve for inputs from −1.5 to 1.5 V, at four load resistor values; "the absolute value (in mV) of the difference between both simulations" (Fig. 3); time-domain waveforms against SPICE (Figs. 4, 8).
- **Threshold (read):** "the difference between the curves is indeed very small, below 1 mV for all values of Vin measured. These small differences are perceptually insignificant". The Serge result is the same: "the maximum difference was once again found to be below 1 mV". This is reported after the fact, not set beforehand as a pass mark.
- **Where SPICE itself differs (read):** the authors attribute the residue partly to SPICE: "SPICE introduces a small fictitious conductance in parallel with each p–n junction in order to aid the convergence of its iterative solvers."
- **Second criterion, aliasing (read):** "Signals with an ANMR value below −10 dB are considered to be completely free from perceivable aliasing." This one is perceptual, and it has a fixed threshold.

## 2. Circuit model checked against the real unit: Holters & Zölzer, "Physical Modelling of a Wah-wah Effect Pedal as a Case Study for Application of the Nodal DK Method to Circuits with Variable Parts", DAFx-11, 2011

Source: [PDF (HSU Hamburg)](https://www.hsu-hh.de/ant/wp-content/uploads/sites/699/2017/10/Holters-Z%C3%B6lzer-2011-Physical-Modelling-of-a-Wah-wah-Effect-Pedal-as-a-Case-Study-for-Application-of-the-Nodal-DK-Method-to-Circuit.pdf) · [DAFx archive](https://www.dafx.de/paper-archive/details/hR2kI9NQ8b1kjRgzxNk0pw)

- **Reference (read):** the real Crybaby pedal. "frequency responses of the model and the real circuit are compared for three different potentiometer settings". The pedal's position in the model came from measuring the real potentiometer: "The corresponding value of α was determined by measuring the potentiometer."
- **Measurements (read):** Fig. 4, "measured (crosses) and simulated (line) small-signal frequency responses". Fig. 5, output RMS level and THD against input level for a sine at the response peak.
- **Threshold (read):** none in numbers. "Clearly, the model fits the real circuit quite well. Any deviations are easily explained by the tolerances of the physical circuit elements". On distortion: "the THD of the simulation is constantly a few dB higher than that of the real circuit, the shape of the curves is very similar".
- **What the model was built from (read):** "nominal component values where used and no parameter matching to the real circuit was performed". In other words, it was built from the schematic as printed and then checked against one real unit.

## 3. Circuit model checked against SPICE and recordings: Werner, Abel, Smith, "A Physically-Informed, Circuit-Bendable, Digital Model of the Roland TR-808 Bass Drum Circuit", DAFx-14, 2014

Source: [PDF (DAFx-14)](https://dafx14.fau.de/papers/dafx14_kurt_james_werner_a_physically_informed,_ci.pdf) · [DAFx archive](https://www.dafx.de/paper-archive/details/hwkd1Lfe8vmtOxCEjsNY-g)

- **Reference (read):** two references. SPICE, for waveforms: "a time-domain plot of the first 13 ms of a single bass drum note, showing good agreement between the physical model and a SPICE simulation". Real-unit recordings, for features: "shows the same features as a SPICE model and recordings of a TR-808 [26]". Reference [26] is a 1994 sample set recorded from a TR-808.
- **Measurements (read):** waveform over time; "estimated instantaneous frequency" (pitch over time) over 300 ms. Feature checks: "basic tuning and timing, a lack of the 'machine gun effect' …, accurate transient behavior on different accent levels". Its centre frequency (≈49.5 Hz) is set against Roland's own tuning chart (56 Hz).
- **Threshold (read):** none in numbers. The authors say "good agreement" and that the "correspondance is consistent with the results of informal listening tests".
- **Tolerance (read):** "the voice circuits featured ±20% capacitors and ±5% resistors. These variations have significant effects on the gain, center frequency, Q, decay time".

## 4. Black-box model checked against recordings of the real unit: Wright, Damskägg, Juvela, Välimäki, "Real-Time Guitar Amplifier Emulation with Deep Learning", *Applied Sciences* 10(3):766, 2020

Source: [open-access PDF (Aalto)](https://acris.aalto.fi/ws/portalfiles/portal/41964332/Real_time_guitar_amplifier_emulation.pdf) · [Aalto record](https://research.aalto.fi/en/publications/real-time-guitar-amplifier-emulation-with-deep-learning/)

- **Reference (read):** audio recorded from the real amplifiers (Blackstar HT-5 Metal and Mesa Boogie 5:50 Plus), from an existing dataset. For their own pedal recordings, the authors also recorded the audio interface's own loopback "in order to estimate the effect of the audio interface in the measurement" (§2.5).
- **Measurements (read):** the error-to-signal ratio, "ESR". It is the energy of the difference divided by the energy of the reference (Eq. 10). During training it is filtered first to emphasise the frequencies that matter to the ear. The test-set ESR is reported without that filter (Table 2). For the amplifier models, training also adds a separate DC-offset term: "an additional loss function representing the difference in DC offset between the target and neural network output was also included" (§4.2, Eq. 19). The loss is then the sum: "E = E_ESR + E_DC" (Eq. 20). Then a MUSHRA listening test, the ITU-R BS.1534 method with "hidden reference and anchor".
- **Thresholds (read):** none set in advance. Test ESR ranged from 0.20 % to 4.2 % (Table 2). For MUSHRA: "the mean score is at or above 90, indicating that they were rated as Excellent on average". For the best Mesa models, the subjects "were often unable to hear any difference between the models and the reference". Mean scores ran from 75 to 98 (Table 3).
- **Screening the judges (read):** 14 people took the test, and "The data of five of the test participants was excluded … as they rated the hidden reference below 90 in more than 15% of the trials."
- **ESR against hearing (read):** for the HT5M, the WaveNet1 model's ESR was 2.8 % and its mean score 75; the RNN-32 model had a *worse* ESR of 4.2 % and a *better* score of 80 (Tables 2 and 3). A lower number does not always mean a better sound.

## 5. SPICE as the stand-in for an unaffordable unit: Chowdhury, "A Comparison of Virtual Analog Modelling Techniques for Desktop and Embedded Implementations", arXiv:2009.02833, 2020

Source: [arXiv PDF](https://arxiv.org/pdf/2009.02833)

- **Reference (read):** "Since the original Klon Centaur pedal is quite expensive (> 1500 USD), we used a SPICE simulation of the Centaur circuit in order to obtain a 'ground truth' reference dataset." The circuit-based model is shown against an "analog reference" (Fig. 5). The paper does not say whether that reference was a real unit.
- **Measurements (read):** ESR, the same formula as in approach 4. Time and frequency plots against the reference (Fig. 7). "subjective listening".
- **Threshold (read):** "the trained network achieved a validation ESR of less than 2%" (Table 1: 0.50–1.72 %). The paper still reports a defect the number does not show: "the high frequency response of the RNN output is slightly damped compared to the reference".

## 6. Makers (secondary sources; weaker evidence)

**Brainworx, Tolerance Modeling Technology (bx_console).** Source: [Production Expert, 27 May 2017](https://www.production-expert.com/home-page/2017/5/27/plugin-alliance-preview-tolerance-modeling-technology-on-new-bxconsole-e-and-g-plug-ins)
- **Reference (read, as reported):** engineers "catalogued the manufacturer's stated tolerances for over 150 components" of one Neve VXS 72 console. "These tolerances vary from 1% on some parts up to 20%". The plug-in offers 72 slightly different channels. Dirk Ulrich was "comparing it live and against the real thing".
- **Threshold:** none published.
- **Why it matters (mine):** a maker treating unit-to-unit spread as part of the target, rather than a single "golden" value.

**noshigeSound, ENHANCER VINTAGE-73 (null test).** Source: [Bedroom Producers Blog, 25 June 2026](https://bedroomproducersblog.com/2026/06/25/enhancer-vintage-73/)
- **Reference (read, as reported):** the maker captured the real hardware across many settings.
- **Measurement (read, as reported):** a null test against the measured hardware. The product page provides the null-test material.
- **Threshold (read, as reported):** the residual sits 43–58 dB below the real signal, 52 dB on average. These figures are the maker's own examples, not checked by the blog or by me.

The Universal Audio interview I tried ([Tape Op #69, David Berners](https://tapeop.com/interviews/btg/69/dr-david-berners)) is paywalled after the first question. I could not read UA's own method and do not cite it for anything.

## What the sources have in common (mine, drawn from the sections above)

1. **Two references, two different jobs.** SPICE checks that the model follows the schematic (approaches 1, 3 and 5). The real unit checks the final claim (approaches 2, 3, 4 and 6). Werner uses both. Chowdhury uses SPICE only because the real pedal cost too much. FAITHFUL.md already makes the same split: ngspice now, the real unit later, and the real unit wins.
2. **No paper sets a pass mark before testing.** The numbers (1 mV, ESR below 2 %, a mean score of 90) are reported as results and described as "good". The only fixed limits are perceptual ones taken from outside work: ANMR below −10 dB, and BS.1534's screening of listeners.
3. **When the reference is a real unit, part tolerance is used to *explain* gaps, not to set a numeric pass band.** Holters (§4) says the deviations from the one real pedal "are easily explained by the tolerances of the physical circuit elements". That is a qualitative explanation. Werner (§11) says the ±20 % capacitors and ±5 % resistors cause variation between units. Neither paper turns those tolerances into an allowed band on the output. Brainworx models the spread as variation between channels, but publishes no pass band either.
4. **One number is never the whole judgement.** Wright's ESR and MUSHRA rankings disagree. Chowdhury's ESR passes but the paper still names a high-frequency loss. Every paper adds plots, features or listening.

## What carries over to a video ramp generator (mine; Phosphlux facts linked to the files they come from)

The Cadet IV ramp is a generator locked to sync. It has no audio-style input, and its output is a DC-coupled 0–1 V
level that becomes brightness directly. FAITHFUL.md quotes the Visual Cortex manual: "0-1V, DC Coupled".

**Carries over directly**
- **SPICE as the first reference, with the volt difference over the whole sweep (approach 1).** This is exactly what
  `faithful/compare.py` does against the T-0278 ngspice run. For scale: Esqueda's simplified analytical models, with diode
  parameters matched to the SPICE models, came within 1 mV of SPICE. FAITHFUL.md's raw tolerance is 0.02, which is
  20 mV. Writing the model as a shader does not stop the same voltage comparison. The 1 mV figure still cannot simply
  be borrowed: it belongs to a different circuit, different modelling assumptions, different parameters and a
  different input range (−1.5 to 1.5 V).
- **The real unit wins (approaches 2, 3 and 6).** The audio sources use part tolerance only to explain gaps against a
  real unit, as set out in common-ground item 3. **Proposal, not established by any source:** for the real-unit stage, the
  Cadet IV's allowed gap could be an envelope worked out from its parts and trimmer ranges. Getting it means
  propagating the BOM tolerances through the circuit (for example SPICE runs across the tolerance range), or measuring
  several units. Neither has been done. Until then, guess 1 in FAITHFUL.md stays a guess.
- **Comparing features as well as waveforms (approach 3).** Werner checks tuning, timing and transients. The ramp's
  counterparts are start level, peak level, slope, straightness and where the reset falls. FAITHFUL.md's table already
  has these.
- **Recording the capture chain on its own (approach 4, §2.5).** When a real Cadet IV is captured, capture the
  digitiser's own response too, so its offset and gain are not blamed on the module.
- **The null test (approach 6).** The "difference ×4" column in `faithful/try1_side_by_side.png` is a null test in picture
  form. To convert scales: 52 dB below 1 V is 10^(−52/20) ≈ 0.0025, or 2.5 mV. FAITHFUL.md's linearity limit, 0.004,
  is about 48 dB below 1 V.

**Carries over only with changes**
- **ESR (approaches 4 and 5).** It can be computed on a ramp, but on its own it folds a constant offset into one error
  figure along with everything else. Audio does attend to DC: Wright et al. add a separate DC-offset loss term during
  training (approach 4, Eqs. 19–20). Even so, that term is a training loss, and what they report and judge is ESR and
  listening. In video, DC level *is* brightness, so offset should be measured and judged on its own, as FAITHFUL.md's
  offset row does.
  An illustration with made-up signals, not the ESR of the actual Cadet reference: take an ideal 0–1 ramp and the same
  ramp shifted up by 0.08. The reference is the shifted ramp, with mean-square energy 1/3 + 0.08 + 0.08² ≈ 0.420.
  The ESR is then 0.08² ÷ 0.420 ≈ 1.5 %, and Wright's E_DC term is the same again (≈1.5 %, sum ≈3.0 %). The
  real Cadet ramp also has gain error, so its figure would differ. An offset that size sits inside the ESR range the
  audio papers report as good, yet it is the most visible mismatch in try 1. The per-measure table in FAITHFUL.md is the
  better tool. ESR, with or without the DC term, could sit beside it as a summary at most.
- **Weighting the error by perception.** Audio weights by hearing (pre-emphasis, A-weighting, ANMR). Video would need its
  own weighting, for example in display steps after sRGB. That is guess 2 in FAITHFUL.md. No video-synth source for a
  weighting was found in this task.
- **The listening test, as a viewing test.** BS.1534's design can be reused for pictures: a hidden reference, an anchor,
  and dropping judges who do not spot the reference. A still ramp gives viewers little to judge, though, and the test
  needs a real-unit capture as its reference. I did not look up a video viewing-test standard.

**Does not carry over**
- **Frequency response and THD from sine sweeps (approach 2).** These describe a circuit that processes an input.
  The ramp generator has no signal input to sweep, and THD of a sawtooth means nothing here. The video counterpart is
  ramp straightness: the residual after a straight-line fit. Broadcast video has long measured that as "luminance
  nonlinearity", using a staircase signal differentiated into pulses that should all be the same height
  ([TekWiki, Tektronix 015-0075-00](https://w140.com/tekwiki/wiki/015-0075-00)). Frequency response would matter
  later, for Cadet modules that process video (edge sharpness), but not for the ramp.
- **Audio aliasing scores (ANMR).** They are built on hearing's masking. The ramp's counterpart is whether the reset edge
  lands on the right pixel. FAITHFUL.md already has that as "same pixel ±1".

## What I could not check
- Whether Chowdhury's Fig. 5 "analog reference" was a real pedal or a circuit solver. The paper does not say.
- The ITU-R BS.1534 text itself. The download returned an HTML page, not the PDF. MUSHRA details here come from Wright et al.'s own description.
- Universal Audio's method (paywalled), and Brainworx's and noshigeSound's own pages. Both maker entries rest on secondary reports.
- Any published work that judges an *analog video* emulation against a circuit or a unit. I did not search for one specifically, and none came up.
