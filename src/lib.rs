// Copyright (C) 2026 CRV Science Network (https://crvscience.com)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://gnu.org>.

#![forbid(unsafe_code)]

//! # TrophicHearth (`trophic-hearth` v0.9.3)
//!
//! Nine-biome native safe-Rust 3D living trophic diorama, fire-pit hearth, and bioacoustic ecosystem simulator
//! spanning all nine [`trophic::BiomeKind`] ecosystems (`L0`–`L4` strata):
//! - **NE Pacific Rocky Intertidal Tidepool & Surge Bench** (`RockyIntertidalTidepool` — semidiurnal tides, *Pisaster* ochre sea stars, *Mytilus* mussels, *Anthopleura* anemones, sculpins, *Postelsia*, black oystercatchers)
//! - **Cold-Seep Brine Pool & Methane Ice Underwater Lake** (`ColdSeepBrinePool` — `1,050 m` hypersaline mirror lake, *Bathymodiolus* mussels, *Lamellibrachia* tubeworms, methane-ice worms, cutthroat eels)
//! - **Midwater Pelagic Rift Canyon** (`MidwaterPelagicRift` — `200–1,500 m` twilight drop-off, diel vertical migration, sperm whales, giant/vampire squid, *Praya* siphonophores, lanternfish)
//! - **PNW Old-Growth & Spawning Salmon Creek** (`PnwOldGrowthSalmonCreek` — spawning *Oncorhynchus kisutch* salmon, mycorrhizal nurse-log forest, spotted owls, dippers, banana slugs)
//! - **Deep-Sea Hydrothermal Vent** (`DeepSeaHydrothermalVent` — `2,600 m` black smokers, chemosynthetic *Riftia* tubeworms, *Rimicaris* shrimp, yeti crabs, *Vulcanoctopus*, ghost sharks)
//! - **Temperate Freshwater Lake** (`TemperateFreshwaterLake` — split-level waterline with northern pike, osprey, river otters, *Elodea*, crayfish, waterbirds)
//! - **Indo-Pacific Shallow Fore-Reef & Lagoon** (`IndoPacificCoralReef` — blacktip reef sharks, giant morays, lionfish, green sea turtles, parrotfish, clownfish, *Acropora* corals)
//! - **Northeast Pacific Giant Kelp Forest** (`PacificKelpForest` — *Enhydra lutris* sea otters, harbor seals, leopard sharks, giant Pacific octopus, *Macrocystis*/*Nereocystis* kelp)
//! - **Neotropical Lowland Rainforest** (`NeotropicalRainforest` — jaguars/black panthers, *Boa constrictor*, squirrel monkeys, sloths, macaws, toucans, tapirs, leafcutter ants, *Mycena* fungi).

pub mod app;
pub mod audio;
pub mod events;
pub mod icon;
pub mod models_3d;
pub mod organisms;
pub mod persistence;
pub mod pheromone;
pub mod physics;
pub mod renderer;
pub mod trophic;
pub mod world;

pub const APP_NAME: &str = "TrophicHearth";
pub const APP_VERSION: &str = "0.9.3";
pub const SCHEMA_VERSION: &str = "trophic_hearth_v1";
