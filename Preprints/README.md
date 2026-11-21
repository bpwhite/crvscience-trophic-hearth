# Preprints for TrophicHearth (`crvscience-trophic-hearth`)

**Author:** Bryan P. White (ORCID: [0000-0003-4191-3511](https://orcid.org/0000-0003-4191-3511))  
**Hub:** https://crvscience.com/apps

This directory contains the self-contained LaTeX source (`.tex`) and single-column PDF (`.pdf`) editions for both:

1. **Companion Application Preprint (`APP-PREPRINT-20`)**
   - **Title:** TrophicHearth: Nine-Biome 3D Living Trophic Diorama, Fire-Pit Hearth, and Bioacoustic Ecosystem Simulator in Rust
   - **Files:** `trophic-hearth-3d-ecosystem-diorama.tex`, `trophic-hearth-3d-ecosystem-diorama.pdf`
   - **Canonical Zenodo DOI:** https://doi.org/10.5281/zenodo.24359107
   - **Figshare Mirror:** https://figshare.com/articles/preprint/TrophicHearth_Nine-Biome_3D_Living_Trophic_Diorama_Fire-Pit_Hearth_and_Bioacoustic_Ecosystem_Simulator_in_Rust/35056465
   - **Web Preprint:** https://crvscience.com/apps/preprints/trophic-hearth-3d-ecosystem-diorama

2. **CRV Science Public Apps Network Overview (`APP-PREPRINT-01`)**
   - **Title:** The CRV Science App Network: Twenty Open-Source Rust Programs for Science, Data, and Simulation
   - **Files:** `crv-science-public-apps-network.tex`, `crv-science-public-apps-network.pdf`
   - **Canonical Zenodo DOI:** https://doi.org/10.5281/zenodo.24222398
   - **Figshare Mirror:** https://figshare.com/articles/preprint/The_CRV_Science_App_Network_Seventeen_Open-Source_Rust_Programs_for_Science_Data_and_Simulation/35013736
   - **Web Preprint:** https://crvscience.com/apps/preprints/crv-science-public-apps-network

## Compiling Standalone LaTeX

Both `.tex` files inline their bibliography so each compiles standalone with standard `pdflatex`:

```bash
pdflatex trophic-hearth-3d-ecosystem-diorama.tex
pdflatex crv-science-public-apps-network.tex
```
