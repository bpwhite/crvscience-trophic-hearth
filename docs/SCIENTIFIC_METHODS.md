# TrophicHearth Scientific Methods (`v0.9.3`)

This document specifies the mathematical equations, physical constants, trophic ODE formulation, optical/atmospheric gating, and numerical boundary guards implemented in `trophic-hearth`.

**License**: GNU General Public License v3.0 or later (`GPL-3.0-or-later`).

---

## 1. Coupled Trophic Ecosystem ODEs

TrophicHearth models five coupled biomass strata `B = [B0, B1, B2, B3, B4]` (in `kcal`) plus an organic detritus compartment `D_kcal = detritus_pool_g * KCAL_PER_G_DETRITUS` (`KCAL_PER_G_DETRITUS = 4.5 kcal/g` ash-free dry weight) via a **Rosenzweig-MacArthur / Lotka-Volterra system** with Holling Type II saturating functional responses.

### 1.1 Compartment Definitions
- `B0` (`L0`): Decomposers & benthic detritivores
- `B1` (`L1`): Autotrophic primary producers (photosynthetic or chemosynthetic)
- `B2` (`L2`): Primary consumers (grazers, filter-feeders, bacterivores)
- `B3` (`L3`): Secondary consumers / mesopredators
- `B4` (`L4`): Tertiary / apex carnivores
- `D_kcal` (`D`): Particulate & dissolved organic detritus energy pool (`kcal`)

### 1.2 Functional Responses & Ingestion Fluxes
For consumer stratum `j` feeding on resource `i` with half-saturation constant `H_ij = kappa_ij * B_i_base` (`kappa_12 = 0.65`, `kappa_23 = kappa_24 = kappa_34 = 0.70`, `kappa_03 = 0.75`, `kappa_D0 = 0.65`) and diurnal/metabolic activity drive `a_drive in [0.04, 1.4]`:

```text
F_{i->j}(B_i, B_j) = a_drive * c_ij * m_ij * B_j * ( B_i / (H_ij + B_i) )
```

where `c_ij` (`s^-1`) is the biome equilibrium-calibrated maximum specific attack rate and `m_ij` is the keystone guild modulation factor (`m_ij = 1.0` when intact; shifted mechanistically by consumptive predator loss and trait-mediated prey foraging release when `keystone_removed = true`).

### 1.3 Primary Production & Self-Shading
Input primary production rate `G_in` (`kcal/s`) depends on biome PAR/chemosynthetic drive `par_norm`, mean flora vigor (`underbrush`, `trees`, `kelp_stalks`), and dissolved mineral nutrients (`mineral_nutrient_pool_mg`). Realized Gross Primary Production `GPP_eff` incorporates smooth self-shading crowding without any hard artificial floor/ceiling clamp:

```text
shading_crowding = clamp(1.55 - 0.55 * (B1 / B1_base), 0.15, 1.55)
GPP_eff          = max(0.0, G_in * shading_crowding * gpp_facilitation)
```

### 1.4 Assimilation, Respiration, Egestion & Mortality
Each trophic transfer `F_{i->j}` partitions strictly into assimilated energy `A_j = alpha_j * F_{i->j}` and unassimilated egestion `(1 - alpha_j) * F_{i->j} -> D_kcal`:

```text
alpha = [0.65 (L0), 1.00 (L1 GPP), 0.54 (L2), 0.72 (L3), 0.78 (L4)]
```

Stratum metabolic respiration `R_i` combines basal maintenance (`m_i * B_i`) and Specific Dynamic Action / growth respiration (`r_sda_i * A_i`):

```text
R_0 = a_drive * m_0 * B_0 + r_0_sda * A_0
R_1 = a_drive * m_1 * B_1 + 0.40 * GPP_eff
R_2 = a_drive * m_2 * B_2 + 0.34 * A_2
R_3 = a_drive * m_3 * B_3 + 0.40 * A_3
R_4 = a_drive * m_4 * B_4 + 0.44 * A_4
```

### 1.5 Coupled Differential Equations & First-Law Conservation
```text
dB_0 / dt = A_0 - R_0 - M_0 - F_{0->3}
dB_1 / dt = GPP_eff - R_1 - M_1 - F_{1->2}
dB_2 / dt = A_2 - R_2 - M_2 - F_{2->3} - F_{2->4}
dB_3 / dt = A_3 - R_3 - M_3 - F_{3->4}
dB_4 / dt = A_4 - R_4 - M_4

dD_kcal / dt = Sum_{i=0..4} M_i + E_2 + E_3 + E_4 - A_0
```

where `M_4 = a_drive * (d_4 + extra_m4 + q_4 * B_4) * B_4` includes quadratic territorial density dependence and `E_j = (1 - alpha_j) * F_in_j` is egestion. Summing all six equations cancels every internal grazing, predation, egestion, decomposition, and mortality flux identically:

```text
d/dt ( Sum_{i=0..4} B_i + D_kcal ) = GPP_eff - Sum_{i=0..4} R_i
```

Integrated over `[0, T]` (subtracting any external non-ODE detritus inputs `D_ext` such as falling canopy fruit or marine snow pulses):

```text
Delta( Sum B_i + D_kcal ) - D_ext = Cumulative_GPP - Sum_{i=0..4} Cumulative_Respiration_i
```

### 1.6 Lindeman Trophic Transfer Efficiency
Net production for each stratum is `P_i = max(0, A_i - R_i)` (with `P_1 = NPP = GPP_eff - R_1`). `TrophicEnergyLedger::transfer_efficiency_pct` computes:

```text
TE_0 = (P_0 / A_0) * 100%      [Microbial Carbon/Energy Use Efficiency]
TE_1 = (P_1 / A_1) * 100%      [Autotrophic NPP / GPP Efficiency]
TE_n = (P_n / P_{n-1}) * 100%  [Lindeman Production Transfer Efficiency, n in {2, 3, 4}]
```

At baseline steady state across the 9 biomes, `TE_2 ≈ 16.7%`, `TE_3 ≈ 16.5%`, and `TE_4 ≈ 16–22%` (Lindeman 1942; Pauly & Christensen 1995).

---

## 2. Physical Constants, Units & Telemetry

| Quantity | Symbol / Field | Units | Value / Range |
| :--- | :--- | :--- | :--- |
| Detritus Energy Density | `KCAL_PER_G_DETRITUS` | `kcal / g` | `4.5 kcal/g` dry ash-free organic matter |
| Gravitational Acceleration | `GRAVITY_M_S2` | `m / s^2` | `9.81 m/s^2` (`src/physics.rs`) |
| Surface Solar Irradiance | `par_umol_m2_s` | `umol photons m^-2 s^-1` | `0.0` (Vent/Brine/Night), up to `~1950` (Rainforest noon), `~24` (475 nm Pelagic Rift) |
| Cold-Seep Brine Salinity | `salinity_psu` | `PSU` | `120 PSU` (`ColdSeepBrinePool` hypersaline mirror lake) |
| Aquatic Dissolved Oxygen | `relative_humidity_pct` | `% DO` | `45–118 % DO` (non-brine aquatic biomes) |
| Terrestrial Air Humidity | `relative_humidity_pct` | `% RH` | `62–99.5 % RH` (Rainforest & PNW Old-Growth) |
| Bioacoustic Sample Rate | `sample_rate` | `Hz` | `44,100 Hz` 16-bit stereo PCM (`src/audio.rs`) |

---

## 3. Diurnal Irradiance, Aphotic Chemosynthesis & Optical Gating

### 3.1 Solar vs. Chemosynthetic Primary Production
- **Photosynthetic Biomes** (`Tidepool`, `PNW Old-Growth`, `Freshwater Lake`, `Coral Reef`, `Kelp Forest`, `Rainforest`): `par_umol_m2_s` follows solar elevation `sin(elev) > 0` attenuated by cloud cover and rain; `par_norm = 0.0` at night (`GPP = 0.0`), while consumer/producer metabolic activity scales smoothly via `activity_drive = 0.08 + 0.92 * (par_norm / 0.85)`.
- **Aphotic Chemosynthetic Biomes** (`DeepSeaHydrothermalVent` at `2,600 m` and `ColdSeepBrinePool` at `1,050 m`): `par_umol_m2_s = 0.0` and `night_factor = 1.0` continuously; primary production runs 24/7 (`activity_drive = 1.0`) via lithoautotrophic `H2S` / `CH4` oxidation (`par_norm ~ 0.86–1.18`).
- **Mesopelagic Rift** (`200–1,500 m`): Narrowband `475 nm` sapphire downwelling twilight (`<= 24 umol m^-2 s^-1`) cues Diel Vertical Migration (DVM) while phytodetritus marine-snow export sustains `L1` continuously.

### 3.2 PAR-Gated Wave Caustics & Shader Validation
Both CPU triangle shading (`surface_caustic_or_dapple` in `src/renderer.rs`) and volumetric sunbeam shafts strictly require `atmo.par_umol_m2_s > 0.0`:

```text
if par_umol_m2_s <= 0.0  =>  caustic_factor = 1.0  (no sunlight caustics or sunbeams)
else                     =>  solar_atten = clamp(par_umol_m2_s / 600.0, 0.0, 1.0)
```

This guarantees zero spurious solar caustics in aphotic hydrothermal vents, cold-seep brine pools, or nocturnal scenes. `shaders/rainforest_diorama.wgsl` clamps Fresnel `1.0 - max(dot(n, v), 0.0)` to `[0.0, 1.0]` before `pow(..., 3.2)` to avoid domain `NaN`s and passes `wgpu::naga` WGSL parsing and full validation.

---

## 4. Numerical Integration & Boundary Guards

1. **Midpoint RK2 Sub-Stepping**: `TrophicEnergyLedger::step_trophic_ode` subdivides every `dt` into `substeps = clamp(ceil(dt / 0.025), 1, 32)` steps (`h <= 0.025 s`). Both biomass state updates and `cumulative_gpp_kcal` / `assimilation_kcal` / `respiration_kcal` accumulators use the exact stage-2 midpoint fluxes so conservation holds across arbitrary step sizes.
2. **Proportional Positive-Preserving Flux Capping**: Within each substep, if total outflow from any pool exceeds `0.85 * avail / h`, all outgoing fluxes from that pool are scaled uniformly before partitioning into assimilation and egestion. This prevents negative compartments under extreme grazing shocks while preserving exact energy accounting.
3. **Keystone Removal Hitbox & Selection Guard**: Organisms with `species.is_keystone_species() == true` are omitted simultaneously from 3D mesh generation, food-web flux arcs, and viewport `hitboxes` whenever `world.ledger.keystone_removed == true`.
4. **3D Flux-Arc Producer Anchoring**: `L0 -> L1 -> L2` energy-flux arcs anchor `L1` endpoints to up to 4 spatially distributed stands from `kelp_stalks`, `trees`, and `underbrush` (`producer_flux_anchors`).

---

## 5. Peer-Reviewed References

1. Lindeman, R. L. (1942). *The Trophic-Dynamic Aspect of Ecology*. Ecology, 23(4), 399–417. https://doi.org/10.2307/1930126
2. Paine, R. T. (1966). *Food Web Complexity and Species Diversity*. The American Naturalist, 100(910), 65–75. https://doi.org/10.1086/282400
3. Estes, J. A., & Palmisano, J. F. (1974). *Sea Otters: Their Role in Structuring Nearshore Communities*. Science, 185(4156), 1058–1060. https://doi.org/10.1126/science.185.4156.1058
4. Rosenzweig, M. L., & MacArthur, R. H. (1963). *Graphical Representation and Stability Conditions of Predator-Prey Interactions*. The American Naturalist, 97(895), 209–223. https://doi.org/10.1086/282272
5. Holling, C. S. (1959). *Some Characteristics of Simple Types of Predation and Parasitism*. The Canadian Entomologist, 91(7), 385–398. https://doi.org/10.4039/Ent91385-7
6. Pauly, D., & Christensen, V. (1995). *Primary production required to sustain global fisheries*. Nature, 374, 255–257. https://doi.org/10.1038/374255a0
7. White, B. P. (2026). *TrophicHearth: Nine-Biome 3D Living Trophic Diorama, Fire-Pit Hearth, and Bioacoustic Ecosystem Simulator in Rust*. CRV Science Apps Preprints (`APP-PREPRINT-20`). https://doi.org/10.5281/zenodo.24359107
