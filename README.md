# SoundGen

**A procedural sound-effect studio for arcade and action games.** No sample library: every sound —
from a shotgun blast to a confetti pop, from a propeller loop to a creature growl — is synthesised
from scratch out of layers. Audition it instantly, shape it in real time, export it as WAV.

![The SoundGen studio](docs/assets/screenshot.png)

```
Rust 1.85+ · egui/eframe 0.37 · cpal 0.18 · Windows (WASAPI)
```

## Why

An arcade or action game needs hundreds of sounds, and most of them are siblings: thirty weapon
variants, forty creature vocalisations, twenty footstep surfaces. SoundGen turns that into a
**parameter + seed** problem:

- **Layer** — every effect is 2–6 stacked layers: transient, body, tail, garnish.
- **Style axis** — `0.0` physical/realistic ↔ `1.0` cartoon/exaggerated, applied per layer.
- **Family** — not one sound but 8–12 seeded variations that share a character.

One recipe yields twelve different but recognisably related takes. Star the ones you like and export
only those.

## How it works

A sound is a **recipe**: a stack of layers, each with its own generator (noise, tone, modal bank,
granular cloud, rotor), envelope, filter sweep, gain, pan and delay. The whole recipe renders
offline in ~18 ms, which is fast enough to re-render on every knob move and still feel instant —
so there is no real-time DSP graph to glitch, and every audition is exactly what you will export.

The style axis morphs the same layers between two readings: at `0.0` you get tighter damping, more
inharmonicity and a drier tail; at `1.0` the pitch bends, the modes ring longer and a bit of
bit-crush creeps in.

## Library

66 factory recipes across 9 categories:

| Category | Count | Examples |
|---|---|---|
| Weapons | 10 | pistol, rifle, shotgun, SMG, suppressed, dry fire, reload, casing, bolt, shell |
| Creatures | 9 | low/high growl, scream, pain hit, lunge, bite, death rattle, idle breath, horde bed |
| Gore | 8 | flesh hit, slash, bone break, head pop, body fall, blood splat, dismember, crunch |
| Foley | 8 | footstep ×4 surfaces, body land, cloth rustle, door creak, glass debris |
| Loops | 6 | small/large propeller, drone motor, helicopter, wind, campfire |
| Crates | 6 | wood, metal, plastic: open and break variants |
| Celebration | 6 | confetti pop, party horn, crowd cheer, coin, fanfare |
| UI | 8 | click, hover, confirm, cancel, error, notify, swipe, toggle |
| Ambience | 5 | wind, cave drip, distant moan, room tone, rain |

Loops are tuned to a whole number of blade passes and joined by the renderer's circular crossfade,
so they repeat without a click: measured seam ratio 0.00–0.57 (threshold 1.5).

## Quick start

```powershell
# the app
cargo run --release -p soundgen

# open with a populated variation family (this is how the screenshot was taken)
cargo run --release -p soundgen -- --demo

# list the library
cargo run -p soundgen-cli -- list

# export the whole library with 4 variations each
cargo run --release -p soundgen-cli -- render --out out --variations 4 --style 0.35 --bits 16

# weapons only, 24-bit
cargo run --release -p soundgen-cli -- render --out out --only weapon --bits 24

# quality control (duration / peak / LUFS / NaN / loop seam)
cargo run -p soundgen-cli -- stats
```

## Keyboard shortcuts

| Key | Action |
|---|---|
| `Space` | play / stop |
| `R` | randomize into a new variation |
| `L` | lock the selected layer (randomize leaves it alone) |
| `S` / `M` | solo / mute the selected layer |
| `E` | export WAV |
| `↑` / `↓` | previous / next preset |
| `Ctrl+Z` | undo (50 steps) |

## Workspace layout

| Path | Contents |
|---|---|
| `crates/sg-dsp` | RNG, noise colours, envelopes, TPT state-variable filter, biquad, oscillator, modal bank, effects |
| `crates/sg-engine` | `Recipe` / `Layer` model, offline renderer, `AudioBuffer` analysis |
| `crates/sg-presets` | the factory library (one module per category) |
| `crates/sg-io` | WAV writer and cpal player |
| `apps/soundgen` | the egui studio |
| `apps/soundgen-cli` | batch render and QA |

## Docs

The research and design notes are written in Turkish; the code and identifiers are English.

- [`docs/00-BRIEF.md`](docs/00-BRIEF.md) — product brief
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — architecture decisions and the render pipeline
- [`docs/API-CHEATSHEET.md`](docs/API-CHEATSHEET.md) — how to write a preset
- [`docs/QA.md`](docs/QA.md) — measured duration / peak / LUFS / seam report for the exported pack
- [`docs/adr/`](docs/adr) — architecture decision records
- `docs/research/01-dsp-sentez.md` — synthesis techniques, per-category recipes, style map
- `docs/research/02-arac-ux.md` — teardown of professional sound tools, UI and workflow
- `docs/research/03-stack.md` — technology stack comparison
- `docs/research/04-entegrasyon.md` — Unity/FMOD/Wwise contract, loudness, naming

## Status

- [x] DSP core (modal, granular, rotor, sweep, tone, noise)
- [x] Offline render + low-latency playback
- [x] Style axis (realistic ↔ cartoon)
- [x] Seeded variation families
- [x] WAV export (16/24-bit, mono/stereo)
- [x] Category library (weapons, creatures, gore, foley, loops, crates, celebration, UI, ambience)
- [x] egui studio (waveform, spectrogram, layer inspector, variation family)
- [x] Seamless loops (circular crossfade, exact length)
- [ ] Physical modelling v0.2 (contact force, modal bodies, formant voices)
- [ ] Sample layers — drop your own recordings into a recipe
- [ ] JSON sidecar + Unity Addressables importer
- [ ] VST3 / runtime library

## License

MIT for the code. The sounds you generate are yours.
