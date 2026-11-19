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
//! Three-biome native safe-Rust 3D living trophic diorama and ambient "fireplace" simulator:
//! - **Indo-Pacific Coral Reef** (*Carcharhinus melanopterus* blacktip reef shark, *Gymnothorax javanicus*
//!   giant moray eel, *Pterois volitans* lionfish, *Taeniura lymma* bluespotted stingray,
//!   *Amphiprion ocellaris* clownfish in host sea anemone, *Zanclus cornutus* Moorish idol,
//!   *Chelonia mydas* green sea turtle, *Chlorurus* parrotfish bioerosion sand clouds, *Chromis viridis*
//!   staghorn-hiding damselfish cloud, *Tridacna gigas* giant clam, *Lysmata* cleaner shrimp,
//!   *Linckia* blue sea star, *Calcinus* hermit crabs, and GFP-biofluorescent *Acropora*/brain corals)
//! - **Northeast Pacific Giant Kelp Forest** (*Enhydra lutris*, *Phoca vitulina*, *Triakis semifasciata*,
//!   *Enteroctopus dofleini*, *Macrocystis*/*Nereocystis*)
//! - **Neotropical Lowland Rainforest** (*Boa constrictor*, *Panthera onca*, *Saimiri*, *Bradypus*,
//!   *Ara*, *Ramphastos*, *Tapirus*, *Dasyprocta*, *Morpho*, *Atta*, *Mycena*).

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
