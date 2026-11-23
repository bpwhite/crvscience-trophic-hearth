# TrophicHearth User Guide (`v0.9.3`)

**TrophicHearth** is a standalone safe-Rust (`#![forbid(unsafe_code)]` library core) nine-biome 3D living trophic diorama, fire-pit hearth, and bioacoustic ecosystem simulator distributed under the **GNU General Public License v3.0 or later (`GPL-3.0-or-later`)**.

---

## 1. Installation & Build

### Prerequisites
- Rust stable toolchain (`1.78+` recommended) with `cargo`
- OS audio backend (optional; compile with `--no-default-features` to disable `rodio` live audio playback if running on a headless Linux/CI node without ALSA/PulseAudio)
- Vulkan / DX12 / Metal GPU adapter for interactive GUI or `--headless-screenshot` rendering (all ecological simulations, SQLite WAL persistence, WAV export, and headless unit/integration tests run purely on CPU)

### Build & Run
```bash
# Build release binary
cargo build --release

# Launch GUI (defaults to NE Pacific Rocky Intertidal Tidepool)
cargo run --release

# Launch directly into a chosen biome and Zen Hearth Mode
cargo run --release -- --biome brine --hearth-mode

# Build without hardware audio dependency
cargo build --release --no-default-features
```

---

## 2. GUI Layout & Workflow

1. **Top Control Ribbon (`hearth_top_bar`)**:
   - **Nine-Biome Switcher**: Click any of the 9 biome tabs (`Tidepool`, `Brine Pool`, `Pelagic Rift`, `PNW Old-Growth`, `Hydrothermal Vent`, `Freshwater Lake`, `Coral Reef`, `Kelp Forest`, `Rainforest`) or hover for a field summary of depth/energy basis, keystone guild, and dominant strata.
   - **Keystone & Flux Controls**: Toggle `Keystone: Intact` / `[!] Keystone: Removed` (`K`) and `Flux Arcs` (`F`), pause/resume (`Space`), adjust simulation pace (`0.25x`–`4.00x`), focus trophic strata (`All`, `L0`–`L4`), toggle audio (`M`), enter borderless `Hearth Mode (Tab)`, or show/hide the `Telemetry` inspector drawer.
   - **Hydro-Atmospheric & Camera Ribbon**: Displays live `Day HH:MM (Phase) | Temp (degC) | Hydro Readout | PAR (umol)`:
     - `ColdSeepBrinePool` reports hypersaline pool salinity in **`PSU`** (`120 PSU`, `PAR 0 umol`).
     - Other aquatic biomes report dissolved oxygen saturation in **`% DO`**.
     - Terrestrial biomes (`NeotropicalRainforest`, `PnwOldGrowthSalmonCreek`) report relative humidity in **`% RH`**.
     - Includes `Solar Hour` scrubber, `Dawn` / `Noon` / `Dusk` / `Night` quick-jump buttons, and 4 biome-tailored smooth camera preset buttons.

2. **Interactive 3D Viewport (Central Panel)**:
   - Displays depth-sorted 3D terrain, water cutaways, animated producer stands (`trees`, `kelp_stalks`, `underbrush`), articulated fauna (`L0`, `L2`, `L3`, `L4`), pheromone/scent plumes, hydrothermal black-smoker plumes, and calm 3D food-web energy-flux arcs (`L0 -> L1 -> L2 -> L3 -> L4`).
   - Removed keystone organisms are excluded from both 3D geometry and clickable `hitboxes`, preventing ghost selection when `keystone_removed` is active.
   - Scroll-wheel zoom is strictly guarded by viewport hover (`resp.hovered()`), so scrolling inside the Ecological Inspector drawer or `F1` modal never zooms the 3D camera.

3. **Right Ecological Inspector Drawer (`Telemetry`)**:
   - **Stochastic Events & Cues**: Biome-specific ecological disturbances and behavioral triggers (e.g., `Surf Surge`, `CH4 Bubbles`, `DVM Ascent`, `360 degC Surge`, `Upwelling`, `Rain Squall`).
   - **Keystone Cascade & Sparkline**: Rolling 64-sample (`~22 s`) multi-stratum biomass plot displaying:
     - `L1 Producers` (green)
     - `L2 Primary Consumers` (cyan)
     - `L3 Mesopredators` (amber)
     - `L4 Apex Predators` (rose)
     - Dashed **`100%` baseline line**, Y-axis ticks (`200%`, `100%`, `0%`), X-axis time ticks (`-22s`, `-11s`, `Now`), and an interactive vertical hover crosshair reporting live vs. historical `kcal` and `% of baseline`.
   - **Trophic Energy Pyramid**: Live standing biomass (`kcal`) and Lindeman production transfer efficiency (`%`) per stratum (`L4` down to `L0`), plus active pheromone/scent units, detritus pool (`g`), and dissolved nutrient/substrate pool (`mg` or `mM`).
   - **Selected Organism & Field Observation Log**: Live ethological state, velocity (`m/s`), energy reserve (`kcal`), and timestamped event log.
   - **Storage, Export & Snapshots**: Native `rfd` file dialogs for `.trophic.json` snapshot export/import, `12 s` stereo `.wav` soundscape export, and SQLite WAL save/load slots (`trophic_hearth.sqlite3`).

4. **Controls & Biome Reference Modal (`F1`)**:
   - Press `F1` (or click `Controls & About (F1)` in the bottom status bar) to view the full hotkey matrix, 9-biome quick-reference table with one-click biome jumping, and license/network links.

---

## 3. Interactive Controls & Hotkeys

All single-key shortcuts (`1`–`4`, `0`, `Shift+0..4`, `[`, `]`, `K`, `F`, `Space`, `Tab`, `M`) are guarded by `!ctx.wants_keyboard_input()`, so typing in the SQLite slot name field never triggers unintended toggles.

| Input / Hotkey | Scope | Action |
| :--- | :--- | :--- |
| `1`, `2`, `3`, `4` | Unfocused keyboard | Smoothly interpolate camera to presets 1–4 (`Side Diorama`, `Macro`, `Canopy/Apex`, `Substrate/Decomposer`) |
| `0` | Unfocused keyboard | Reset Trophic Lens to `AllStrata` |
| `Shift+0` .. `Shift+4` | Unfocused keyboard | Focus single stratum (`Shift+0` = `L0`, `Shift+1` = `L1`, `Shift+2` = `L2`, `Shift+3` = `L3`, `Shift+4` = `L4`) |
| `[` / `]` | Unfocused keyboard | Cycle backward / forward across all 9 biomes (`BiomeKind::ALL`) |
| `K` | Unfocused keyboard | Toggle Keystone Guild Removal (`keystone_removed`) |
| `F` | Unfocused keyboard | Toggle 3D food-web energy-flux arcs (`show_flux_arcs`) |
| `Space` | Unfocused keyboard | Pause / resume simulation |
| `Tab` | Unfocused keyboard | Toggle borderless Zen Hearth Mode |
| `M` | Unfocused keyboard | Mute / unmute stereo bioacoustic audio |
| `F1` | Global | Toggle Controls, Biome Reference & About modal |
| `Esc` | Global | Cascade close: `F1` Modal -> Zen Hearth Mode -> Selected Organism |
| `Left-Drag` | 3D Viewport | Free-orbit camera yaw & pitch (`FreeOrbit`) |
| `Right-Drag` | 3D Viewport | Pan camera target in XY plane (`FreeOrbit`) |
| `Scroll Wheel` | 3D Viewport (`hovered`) | Zoom camera distance (`4.5 m` – `32.0 m`; ignored over UI panels/modals) |
| `Click` | 3D Viewport | Select nearest active organism within `26 px` |

---

## 4. CLI Reference

### Global Flags & Direct Launch
```bash
trophic-hearth [OPTIONS] [COMMAND]
```

| Flag | Default | Description |
| :--- | :--- | :--- |
| `--biome <BIOME>` | `tidepool` | Initial biome (`tidepool`, `brine`, `rift`, `pnw`, `vent`, `lake`, `coral`, `kelp`, `rainforest`) |
| `--stratum <0..=4>` | None | Focus initial trophic stratum (`0=Decomposers` .. `4=Tertiary Carnivores`) |
| `--hearth-mode` | `false` | Start directly in borderless Zen Hearth Mode |
| `--db <PATH>` | `trophic_hearth.sqlite3` | SQLite WAL persistence database path |
| `--seed <U64>` | `20261116` | Deterministic `ChaCha8Rng` world seed |
| `--headless-screenshot <PNG>` | None | Render an offscreen `1600x1000` RGBA PNG without opening a window |
| `--screenshot-view <U8>` | `1` | Headless capture preset (`1=Tidepool`, `2=Coral Reef`, `3=Hydrothermal Vent`) |

### Subcommands
1. **`gui`**: Explicit GUI launch with the same `--biome`, `--stratum`, `--hearth-mode`, `--db`, and `--seed` options.
2. **`simulate`**: Run a deterministic headless diurnal ecological sweep (`minutes_per_real_sec = 10.0`) and write `TrophicSnapshotV1` JSON telemetry:
   ```bash
   trophic-hearth simulate --biome kelp --hours 24 --dt 0.05 --output kelp_24h.json --seed 20261116
   ```
3. **`export-wav`**: Synthesize a stereo `44,100 Hz` 16-bit PCM procedural bioacoustic soundscape WAV file:
   ```bash
   trophic-hearth export-wav --seconds 18 --output soundscape.wav
   ```

---

## 5. Scientific Features Guide

- **Keystone Trophic Cascades (`K`)**:
  - In **Rocky Intertidal Tidepool**, removing *Pisaster ochraceus* (`L4`) releases *Mytilus* mussel beds (`L2`) from predation, causing `L2` biomass expansion and grazing/space suppression of *Postelsia* & coralline algae (`L1`).
  - In **Pacific Kelp Forest**, removing *Enhydra lutris* sea otters (`L3`) releases purple sea urchins (`L2`) to overgraze *Macrocystis* kelp (`L1`) cleanly below `3,800 kcal` into an urchin barren.
  - In **Cold-Seep Brine Pool**, removing biogenic *Bathymodiolus* & *Lamellibrachia* habitat engineers degrades chemosynthetic facilitation while shifting carbon into detrital recycling (`L0`).
- **3D Energy-Flux Arcs (`F`)**:
  - Quadratic Bezier arcs with calm non-strobing `kcal` beads trace energy from decomposers (`L0`) into real producer stands (`kelp_stalks`, `trees`, `underbrush` at `L1`), through herbivores/grazers (`L2`), mesopredators (`L3`), and apex carnivores (`L4`).
- **Snapshots & Database Persistence**:
  - Export/import portable `TrophicSnapshotV1` JSON files (`schema: "trophic_hearth_v1"`) or persist multiple named world slots in SQLite WAL (`save_slots` table).

---

## 6. Troubleshooting

- **Offscreen `--headless-screenshot` reports no GPU adapter**: Ensure a Vulkan/DX12/Metal driver or software rasterizer (e.g., Mesa `lavapipe` on headless Linux) is installed; CPU-only `simulate`, `export-wav`, and `cargo test` runs do not require a GPU adapter.
- **No audio output in GUI**: Press `M` or click `Unmute` in the top ribbon; verify the binary was built with the default `live-audio` feature enabled and an OS audio output device is active.
- **SQLite WAL lock / read-only folder**: Pass `--db <writable_path>/trophic_hearth.sqlite3` when launching from a read-only installation directory.
- **WGSL Shader Validation**: `shaders/rainforest_diorama.wgsl` is validated during `cargo test` via `wgpu::naga` (`parse_str` + `Validator`).
