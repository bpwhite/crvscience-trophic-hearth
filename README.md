# TrophicHearth (`trophic-hearth` v0.9.3)

A 9-biome native safe-Rust (`#![forbid(unsafe_code)]` library core) **3D Trophic Diorama, Fire-Pit Hearth, & Bioacoustic Ecosystem Simulator** featuring **Interactive Keystone Trophic Cascades** (with live rolling multi-stratum biomass sparklines) and **Calm 3D Food-Web Energy-Flux Arcs** (`kcal` flow streams).

- **Website**: [CRV Science Network](https://crvscience.com) · [CRV Science Public Apps](https://crvscience.com/apps)
- **License**: GNU General Public License v3.0 or later (`GPL-3.0-or-later`, see [LICENSE](LICENSE))

## 9 Live-Switchable Biomes (`L0`–`L4` Strata)

1. **NE Pacific Rocky Intertidal Tidepool & Surge Bench** (`--biome tidepool`, default in `v0.9.3`): Semidiurnal tides flooding and draining stepped basalt pools, *Pisaster ochraceus* ochre sea stars vs. *Mytilus californianus* mussel beds (Paine 1966 keystone cascade), *Anthopleura* green anemones, leaping tidepool sculpins, *Postelsia* sea palms, hermit crabs, chitons, and black oystercatchers.
2. **Cold-Seep Brine Pool & Methane Ice — 1,050 m Underwater Lake** (`--biome brine`): Denser-than-seawater `120 PSU` hypersaline mirror lake ringed by *Bathymodiolus* methanotrophic mussels, bubbling golden methane clathrate mounds with pink *Sirsoe* ice worms, 250-year *Lamellibrachia* tubeworms, *Chaceon* red crabs, and *Synaphobranchus* cutthroat eels.
3. **Midwater Pelagic Rift Canyon — 200–1,500 m Twilight Drop-Off** (`--biome rift`)
4. **PNW Old-Growth & Spawning Salmon Creek** (`--biome pnw`)
5. **Deep-Sea Hydrothermal Vent — 2,600 m Black Smoker** (`--biome vent`)
6. **Temperate Freshwater Lake — Split-Level Underwater & Waterbirds** (`--biome lake`)
7. **Indo-Pacific Spur-and-Groove Coral Reef & Lagoon** (`--biome coral`)
8. **Northeast Pacific Giant Kelp Forest** (`--biome kelp`)
9. **Neotropical Lowland Rainforest** (`--biome rainforest`)

## Features & Controls

- **Native File Dialogs (`rfd`)**: Export and import `.trophic.json` ecological snapshots, synthesize stereo `.wav` bioacoustic soundscapes, or open SQLite WAL persistence databases from the telemetry drawer.
- **Borderless Hearth Mode (`Tab`)**: Hide chrome to run the living biome diorama as an ambient desktop hearth (`Space` pauses/resumes).
- **Headless CLI & Screenshot Pipeline**: Run diurnal ecosystem sweeps (`simulate`), synthesize soundscapes (`export-wav`), or render offscreen `1600x1000` PNG captures (`--headless-screenshot`).

## Quick Start

```bash
# Launch the GUI on a chosen biome
cargo run --release -- --biome tidepool
cargo run --release -- --biome brine

# Run a headless diurnal ecological sweep to JSON
cargo run --release -- simulate --biome vent --hours 12 --output vent_sim.json

# Synthesize an 18-second stereo bioacoustic soundscape WAV
cargo run --release -- export-wav --seconds 18 --output soundscape.wav

# Render an offscreen 1600x1000 PNG screenshot (view 1=Tidepool, 2=Coral Reef, 3=Vent)
cargo run --release -- --headless-screenshot validation/headless_screenshot.png --screenshot-view 1

# Run full test suite
cargo test --all-targets
```
