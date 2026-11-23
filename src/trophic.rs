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

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Supported 3D side-view living diorama biomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeKind {
    NeotropicalRainforest,
    PacificKelpForest,
    IndoPacificCoralReef,
    TemperateFreshwaterLake,
    DeepSeaHydrothermalVent,
    PnwOldGrowthSalmonCreek,
    MidwaterPelagicRift,
    RockyIntertidalTidepool,
    ColdSeepBrinePool,
}

impl Default for BiomeKind {
    fn default() -> Self {
        Self::RockyIntertidalTidepool
    }
}

impl BiomeKind {
    pub const ALL: [BiomeKind; 9] = [
        BiomeKind::RockyIntertidalTidepool,
        BiomeKind::ColdSeepBrinePool,
        BiomeKind::MidwaterPelagicRift,
        BiomeKind::PnwOldGrowthSalmonCreek,
        BiomeKind::DeepSeaHydrothermalVent,
        BiomeKind::TemperateFreshwaterLake,
        BiomeKind::IndoPacificCoralReef,
        BiomeKind::PacificKelpForest,
        BiomeKind::NeotropicalRainforest,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::NeotropicalRainforest => "Neotropical Lowland Rainforest",
            Self::PacificKelpForest => "Northeast Pacific Giant Kelp Forest",
            Self::IndoPacificCoralReef => "Indo-Pacific Shallow Fore-Reef & Lagoon",
            Self::TemperateFreshwaterLake => "Temperate Freshwater Lake (Underwater & Waterbirds)",
            Self::DeepSeaHydrothermalVent => "Deep-Sea Hydrothermal Vent (2,600m Black Smoker)",
            Self::PnwOldGrowthSalmonCreek => "PNW Old-Growth & Spawning Salmon Creek",
            Self::MidwaterPelagicRift => "Midwater Pelagic Rift Canyon (200-1,500m Twilight Drop-Off)",
            Self::RockyIntertidalTidepool => "NE Pacific Rocky Intertidal Tidepool & Surge Bench",
            Self::ColdSeepBrinePool => "Cold-Seep Brine Pool & Methane Ice Underwater Lake (1,050m)",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Self::NeotropicalRainforest => "Rainforest",
            Self::PacificKelpForest => "Kelp Forest",
            Self::IndoPacificCoralReef => "Coral Reef",
            Self::TemperateFreshwaterLake => "Freshwater Lake",
            Self::DeepSeaHydrothermalVent => "Hydrothermal Vent",
            Self::PnwOldGrowthSalmonCreek => "PNW Old-Growth",
            Self::MidwaterPelagicRift => "Pelagic Rift",
            Self::RockyIntertidalTidepool => "Tidepool",
            Self::ColdSeepBrinePool => "Brine Pool",
        }
    }

    pub fn keystone_guild_label(self) -> &'static str {
        match self {
            Self::RockyIntertidalTidepool => "Pisaster Ochre Sea Stars (Paine 1966)",
            Self::ColdSeepBrinePool => "Bathymodiolus & Lamellibrachia Engineers",
            Self::MidwaterPelagicRift => "Sperm Whales & Giant Squid",
            Self::PnwOldGrowthSalmonCreek => "Spawning Coho Salmon & Spotted Owls",
            Self::DeepSeaHydrothermalVent => "Ghost Sharks & Vulcanoctopus",
            Self::TemperateFreshwaterLake => "Northern Pike & Osprey",
            Self::IndoPacificCoralReef => "Blacktip Sharks & Parrotfish",
            Self::PacificKelpForest => "Enhydra Sea Otters",
            Self::NeotropicalRainforest => "Panthera Jaguars & Panthers",
        }
    }

    pub fn is_marine(self) -> bool {
        matches!(
            self,
            Self::PacificKelpForest
                | Self::IndoPacificCoralReef
                | Self::DeepSeaHydrothermalVent
                | Self::MidwaterPelagicRift
                | Self::RockyIntertidalTidepool
                | Self::ColdSeepBrinePool
        )
    }

    pub fn is_aquatic(self) -> bool {
        matches!(
            self,
            Self::PacificKelpForest
                | Self::IndoPacificCoralReef
                | Self::TemperateFreshwaterLake
                | Self::DeepSeaHydrothermalVent
                | Self::MidwaterPelagicRift
                | Self::RockyIntertidalTidepool
                | Self::ColdSeepBrinePool
        )
    }

    /// Canonical telemetry unit for the biome's primary water/air/salinity readout.
    pub fn telemetry_unit(self) -> &'static str {
        match self {
            Self::ColdSeepBrinePool => " PSU",
            b if b.is_aquatic() => "% DO",
            _ => "% RH",
        }
    }

    /// Concise ecological tooltip displayed when hovering biome tabs or inspecting the F1 reference.
    pub fn hover_tooltip(self) -> &'static str {
        match self {
            Self::RockyIntertidalTidepool => {
                "NE Pacific Rocky Intertidal Tidepool (0 m, Solar PAR & Semidiurnal Tides)\n\
                 L1: Postelsia sea palms & coralline turf | L2: Mytilus mussels & barnacles\n\
                 Keystone: Pisaster ochraceus ochre sea stars (Paine 1966 cascade)"
            }
            Self::ColdSeepBrinePool => {
                "Cold-Seep Brine Pool & Methane Ice (1,050 m Aphotic, 120 PSU Brine, PAR = 0)\n\
                 Energy: Methanotrophic & thiotrophic chemosynthesis (CH4 / H2S)\n\
                 Keystone: Bathymodiolus mussels & 250-yr Lamellibrachia tubeworms"
            }
            Self::MidwaterPelagicRift => {
                "Midwater Pelagic Rift Canyon (200-1,500 m, 475 nm Twilight & DVM)\n\
                 Energy: Epipelagic phytodetritus marine snow & Deep Scattering Layer\n\
                 Keystone: Echolocating sperm whales & Architeuthis giant squid"
            }
            Self::PnwOldGrowthSalmonCreek => {
                "PNW Old-Growth & Spawning Salmon Creek (Temperate Rainforest & Riffles)\n\
                 Energy: Solar PAR + marine-derived 15N salmon nutrient subsidy\n\
                 Keystone: Spawning Coho salmon & Northern Spotted Owls"
            }
            Self::DeepSeaHydrothermalVent => {
                "Deep-Sea Hydrothermal Vent (2,600 m Abyssal Black Smokers, PAR = 0)\n\
                 Energy: 100% H2S/Fe2+ lithoautotrophic chemosynthesis (Riftia & Beggiatoa)\n\
                 Keystone: Abyssal ghost sharks & Vulcanoctopus vent predators"
            }
            Self::TemperateFreshwaterLake => {
                "Temperate Freshwater Lake (Split-Level Littoral & Pelagic Zone)\n\
                 Energy: Solar PAR macrophytes (eelgrass, lilies, cattails) + leaf packs\n\
                 Keystone: Northern pike & plunging osprey apex piscivores"
            }
            Self::IndoPacificCoralReef => {
                "Indo-Pacific Fore-Reef & Lagoon (Shallow Oligotrophic Tropical Reef)\n\
                 Energy: Zooxanthellate coral holobiont photosynthesis (Darwin's Paradox)\n\
                 Keystone: Blacktip reef sharks & grazing steephead parrotfish"
            }
            Self::PacificKelpForest => {
                "Northeast Pacific Giant Kelp Forest (Cold Upwelling Macrocystis Reef)\n\
                 Energy: Solar PAR bull & giant kelp canopy (upwelling NO3/PO4)\n\
                 Keystone: Enhydra lutris sea otters suppressing purple sea urchin barrens"
            }
            Self::NeotropicalRainforest => {
                "Neotropical Lowland Rainforest (Multi-Tier Emerald Canopy & Soil)\n\
                 Energy: High solar PAR canopy sunflecks + leafcutter ant detritus loop\n\
                 Keystone: Canopy jaguars & melanistic black panthers"
            }
        }
    }
}

/// Five canonical ecological strata across terrestrial and marine dioramas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrophicStratum {
    Decomposers = 0,
    Producers = 1,
    PrimaryConsumers = 2,
    SecondaryConsumers = 3,
    TertiaryCarnivores = 4,
}

impl TrophicStratum {
    pub const ALL: [TrophicStratum; 5] = [
        TrophicStratum::Decomposers,
        TrophicStratum::Producers,
        TrophicStratum::PrimaryConsumers,
        TrophicStratum::SecondaryConsumers,
        TrophicStratum::TertiaryCarnivores,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Decomposers),
            1 => Some(Self::Producers),
            2 => Some(Self::PrimaryConsumers),
            3 => Some(Self::SecondaryConsumers),
            4 => Some(Self::TertiaryCarnivores),
            _ => None,
        }
    }

    pub fn label_for_biome(self, biome: BiomeKind) -> &'static str {
        match (biome, self) {
            (BiomeKind::NeotropicalRainforest, Self::Decomposers) => "L0: Decomposers (Fungi, Bacteria & Ants)",
            (BiomeKind::NeotropicalRainforest, Self::Producers) => "L1: Producers (Trees, Shrubs & Vines)",
            (BiomeKind::NeotropicalRainforest, Self::PrimaryConsumers) => "L2: Primary Consumers (Herbivores)",
            (BiomeKind::NeotropicalRainforest, Self::SecondaryConsumers) => "L3: Secondary Consumers (Omnivores)",
            (BiomeKind::NeotropicalRainforest, Self::TertiaryCarnivores) => "L4: Tertiary Consumers (Apex Carnivores)",
            (BiomeKind::PacificKelpForest, Self::Decomposers) => "L0: Benthic Detritivores (Sea Stars, Cucumbers & Crabs)",
            (BiomeKind::PacificKelpForest, Self::Producers) => "L1: Macroalgae Producers (Macrocystis & Bull Kelp)",
            (BiomeKind::PacificKelpForest, Self::PrimaryConsumers) => "L2: Grazers & Planktivores (Urchins, Abalone & Chromis)",
            (BiomeKind::PacificKelpForest, Self::SecondaryConsumers) => "L3: Keystone Mesopredators (Sea Otter, Octopus & Wrasse)",
            (BiomeKind::PacificKelpForest, Self::TertiaryCarnivores) => "L4: Reef & Pelagic Predators (Harbor Seal, Leopard Shark & Wolf Eel)",
            (BiomeKind::IndoPacificCoralReef, Self::Decomposers) => "L0: Cleaners & Detritivores (Cleaner Shrimp, Linckia Stars & Hermit Crabs)",
            (BiomeKind::IndoPacificCoralReef, Self::Producers) => "L1: Symbiotic Holobionts (Acropora, Brain Coral, Sea Fans & Anemones)",
            (BiomeKind::IndoPacificCoralReef, Self::PrimaryConsumers) => "L2: Grazers & Planktivores (Sea Turtle, Parrotfish, Giant Clam & Chromis)",
            (BiomeKind::IndoPacificCoralReef, Self::SecondaryConsumers) => "L3: Reef Mutualists & Mesopredators (Clownfish, Moorish Idol & Stingray)",
            (BiomeKind::IndoPacificCoralReef, Self::TertiaryCarnivores) => "L4: Apex & Crevice Predators (Blacktip Reef Shark, Giant Moray & Lionfish)",
            (BiomeKind::TemperateFreshwaterLake, Self::Decomposers) => "L0: Benthic Detritivores (Signal Crayfish & Leaf-Pack Microbes)",
            (BiomeKind::TemperateFreshwaterLake, Self::Producers) => "L1: Aquatic & Littoral Flora (Eelgrass, Water Lilies, Cattails & Willows)",
            (BiomeKind::TemperateFreshwaterLake, Self::PrimaryConsumers) => "L2: Herbivores & Planktivores (Golden Shiners, Sunfish & Dragonflies)",
            (BiomeKind::TemperateFreshwaterLake, Self::SecondaryConsumers) => "L3: Waterfowl & Piscivores (Mallards, Kingfisher, Rainbow Trout, Bass & Turtle)",
            (BiomeKind::TemperateFreshwaterLake, Self::TertiaryCarnivores) => "L4: Apex Piscivores (Osprey, Common Loon, Great Blue Heron & Northern Pike)",
            (BiomeKind::DeepSeaHydrothermalVent, Self::Decomposers) => "L0: Sulfide Plume Detritivores (Munidopsis Squat Lobsters & Pompeii Worms)",
            (BiomeKind::DeepSeaHydrothermalVent, Self::Producers) => "L1: Chemoautotrophs (Riftia Tubeworm Symbionts & Beggiatoa Mats)",
            (BiomeKind::DeepSeaHydrothermalVent, Self::PrimaryConsumers) => "L2: Bacterivores (Kiwa Yeti Crabs, Rimicaris Vent Shrimp & Scaly-Foot Snails)",
            (BiomeKind::DeepSeaHydrothermalVent, Self::SecondaryConsumers) => "L3: Vent & Bioluminescent Predators (Zoarcid Eelpouts, Atolla & Comb Jellies)",
            (BiomeKind::DeepSeaHydrothermalVent, Self::TertiaryCarnivores) => "L4: Abyssal Apex Predators (Ghost Sharks, Sixgill Sharks & Vulcanoctopus)",
            (BiomeKind::PnwOldGrowthSalmonCreek, Self::Decomposers) => "L0: Mycorrhizal Network, Chanterelles, Banana Slugs & Caddisflies",
            (BiomeKind::PnwOldGrowthSalmonCreek, Self::Producers) => "L1: Old-Growth Douglas-Fir, Nurse Logs, Sword Ferns & Huckleberries",
            (BiomeKind::PnwOldGrowthSalmonCreek, Self::PrimaryConsumers) => "L2: Spawning Coho Salmon & Douglas's Squirrels",
            (BiomeKind::PnwOldGrowthSalmonCreek, Self::SecondaryConsumers) => "L3: Underwater-Walking Dippers, Giant Salamanders & Varied Thrushes",
            (BiomeKind::PnwOldGrowthSalmonCreek, Self::TertiaryCarnivores) => "L4: Northern Spotted Owls & Salmon-Fishing Black Bears",
            (BiomeKind::MidwaterPelagicRift, Self::Decomposers) => "L0: Marine-Snow Sinkers (Bathochordaeus Larvacean Houses & Vampire Squid)",
            (BiomeKind::MidwaterPelagicRift, Self::Producers) => "L1: Epipelagic Phytodetritus Snow, Bubblegum Octocorals & Glass Sponges",
            (BiomeKind::MidwaterPelagicRift, Self::PrimaryConsumers) => "L2: DVM Deep Scattering Layer (Myctophid Lanternfish, Hatchetfish & Krill)",
            (BiomeKind::MidwaterPelagicRift, Self::SecondaryConsumers) => "L3: Twilight Ambushers (Macropinna Barreleye, Praya Siphonophore & Jewel Squid)",
            (BiomeKind::MidwaterPelagicRift, Self::TertiaryCarnivores) => "L4: Deep-Diving Apex Predators (Echolocating Sperm Whale & Architeuthis Giant Squid)",
            (BiomeKind::RockyIntertidalTidepool, Self::Decomposers) => "L0: Basalt Scavengers (Pagurus Hermit Crabs & Black Katy Chitons)",
            (BiomeKind::RockyIntertidalTidepool, Self::Producers) => "L1: Wave-Swept Algae (Postelsia Sea Palms, Coralline Crusts & Ulva)",
            (BiomeKind::RockyIntertidalTidepool, Self::PrimaryConsumers) => "L2: Sessile Filter Beds (Mytilus California Mussels & Gooseneck Barnacles)",
            (BiomeKind::RockyIntertidalTidepool, Self::SecondaryConsumers) => "L3: Tidepool Mesopredators (Anthopleura Green Anemones & Leaping Sculpins)",
            (BiomeKind::RockyIntertidalTidepool, Self::TertiaryCarnivores) => "L4: Intertidal Keystone Predators (Pisaster Ochre Sea Stars & Black Oystercatchers)",
            (BiomeKind::ColdSeepBrinePool, Self::Decomposers) => "L0: Clathrate Detritivores (Sirsoe Methane Ice Worms & Squat Lobsters)",
            (BiomeKind::ColdSeepBrinePool, Self::Producers) => "L1: Methanotrophic & Thiotrophic Producers (Lamellibrachia & Beggiatoa Mats)",
            (BiomeKind::ColdSeepBrinePool, Self::PrimaryConsumers) => "L2: Seep Symbiotic Grazers (Bathymodiolus Brine-Beach Mussels & Alvinocaris Shrimp)",
            (BiomeKind::ColdSeepBrinePool, Self::SecondaryConsumers) => "L3: Seep Benthic Predators (Chaceon Red Crabs & Cold-Seep Hagfish)",
            (BiomeKind::ColdSeepBrinePool, Self::TertiaryCarnivores) => "L4: Brine-Edge Apex Scavengers (Synaphobranchus Cutthroat Eels & Seep Chimaeras)",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::Decomposers => "L0 Decomposers",
            Self::Producers => "L1 Producers",
            Self::PrimaryConsumers => "L2 Primary",
            Self::SecondaryConsumers => "L3 Secondary",
            Self::TertiaryCarnivores => "L4 Tertiary",
        }
    }

    pub fn badge_rgb(self) -> [u8; 3] {
        match self {
            Self::Decomposers => [168, 118, 76],
            Self::Producers => [64, 178, 92],
            Self::PrimaryConsumers => [102, 194, 165],
            Self::SecondaryConsumers => [238, 174, 66],
            Self::TertiaryCarnivores => [224, 92, 78],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrophicFocus {
    AllStrata,
    Single(TrophicStratum),
}

impl Default for TrophicFocus {
    fn default() -> Self {
        Self::AllStrata
    }
}

impl TrophicFocus {
    pub fn highlights(self, stratum: TrophicStratum) -> bool {
        match self {
            Self::AllStrata => true,
            Self::Single(target) => target == stratum,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiurnalCycle {
    pub hour_of_day: f32,
    pub day_count: u32,
    pub minutes_per_real_sec: f32,
    pub cloud_cover: f32,
    pub rain_intensity: f32,
    pub foliage_wetness: f32,
    pub sunfleck_boost: f32,
}

impl Default for DiurnalCycle {
    fn default() -> Self {
        Self {
            hour_of_day: 10.15,
            day_count: 1,
            minutes_per_real_sec: 1.5,
            cloud_cover: 0.12,
            rain_intensity: 0.0,
            foliage_wetness: 0.25,
            sunfleck_boost: 0.25,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DiurnalAtmosphere {
    pub sun_elevation_rad: f32,
    pub sun_direction: [f32; 3],
    pub sun_intensity: f32,
    pub sun_rgb: [f32; 3],
    pub sky_top_rgb: [f32; 3],
    pub sky_horizon_rgb: [f32; 3],
    pub fog_rgb: [f32; 3],
    pub fog_density: f32,
    pub par_umol_m2_s: f32,
    pub night_factor: f32,
    pub air_temp_c: f32,
    pub relative_humidity_pct: f32,
    pub hydro_unit: &'static str,
    pub salinity_psu: Option<f32>,
    pub phase_name: &'static str,
}

impl DiurnalCycle {
    /// Formats the canonical top-bar ecological telemetry readout for `biome`
    /// (reporting brine salinity in `PSU`, aquatic oxygen in `% DO`, and air humidity in `% RH`).
    pub fn telemetry_summary(&self, biome: BiomeKind) -> String {
        let atmo = self.atmosphere_for_biome(biome);
        let h = self.hour_of_day.floor() as u32 % 24;
        let m = ((self.hour_of_day.fract() * 60.0) as u32).min(59);
        let hydro_val = atmo.salinity_psu.unwrap_or(atmo.relative_humidity_pct);
        format!(
            "Day {} {:02}:{:02} ({}) | {:.1} degC | {:.0}{} | PAR {:.0} umol",
            self.day_count,
            h,
            m,
            atmo.phase_name,
            atmo.air_temp_c,
            hydro_val,
            atmo.hydro_unit,
            atmo.par_umol_m2_s,
        )
    }

    pub fn step(&mut self, dt_sec: f32) {
        let delta_hours = (dt_sec * self.minutes_per_real_sec) / 60.0;
        self.hour_of_day += delta_hours;
        while self.hour_of_day >= 24.0 {
            self.hour_of_day -= 24.0;
            self.day_count = self.day_count.saturating_add(1);
        }
        while self.hour_of_day < 0.0 {
            self.hour_of_day += 24.0;
        }

        let sun_elev = self.sun_elevation_rad().sin().max(0.0);
        if self.rain_intensity > 0.02 {
            self.foliage_wetness = (self.foliage_wetness + self.rain_intensity * dt_sec * 0.25).min(1.0);
        } else {
            let evap_rate = 0.012 + 0.035 * sun_elev;
            self.foliage_wetness = (self.foliage_wetness - evap_rate * dt_sec).max(0.08);
        }
    }

    pub fn sun_elevation_rad(&self) -> f32 {
        ((self.hour_of_day - 6.0) / 12.0 * PI).sin() * (PI * 0.48)
    }

    pub fn is_night(&self) -> bool {
        self.hour_of_day < 5.75 || self.hour_of_day > 18.25
    }

    pub fn is_crepuscular(&self) -> bool {
        (5.2..=7.2).contains(&self.hour_of_day) || (17.0..=19.2).contains(&self.hour_of_day)
    }

    pub fn atmosphere(&self) -> DiurnalAtmosphere {
        self.atmosphere_for_biome(BiomeKind::NeotropicalRainforest)
    }

    pub fn atmosphere_for_biome(&self, biome: BiomeKind) -> DiurnalAtmosphere {
        let angle = (self.hour_of_day - 6.0) / 24.0 * 2.0 * PI;
        let elev = self.sun_elevation_rad();
        let sin_e = elev.sin();
        let day_t = ((sin_e + 0.12) / 1.12).clamp(0.0, 1.0);
        let night_factor = (1.0 - ((sin_e + 0.18) / 0.36).clamp(0.0, 1.0)).clamp(0.0, 1.0);
        let twilight = (1.0 - (sin_e.abs() / 0.28).clamp(0.0, 1.0)).powf(1.4);

        let cloud_atten = 1.0 - 0.62 * self.cloud_cover - 0.22 * self.rain_intensity;
        let sun_dir = if sin_e >= -0.05 {
            normalize3([angle.cos() * 0.78, sin_e.max(0.14), 0.46])
        } else {
            normalize3([-angle.cos() * 0.65, (-sin_e).max(0.28), 0.52])
        };

        match biome {
            BiomeKind::RockyIntertidalTidepool => {
                // Crisp Oregon / Olympic outer-coast marine air + semidiurnal tidal cycle!
                let sun_intensity = if sin_e >= 0.0 {
                    (0.36 + 1.02 * sin_e.powf(0.75)) * cloud_atten.max(0.28)
                } else {
                    0.22
                };
                let sun_rgb = lerp3([0.55, 0.75, 0.96], lerp3([0.99, 0.95, 0.84], [0.99, 0.62, 0.36], twilight), day_t);
                let sky_top_rgb = lerp3([0.04, 0.09, 0.18], [0.16, 0.48, 0.68], day_t);
                let sky_horizon_rgb = lerp3([0.06, 0.13, 0.22], lerp3([0.42, 0.72, 0.76], [0.92, 0.54, 0.34], twilight * 0.65), day_t);
                let fog_rgb = lerp3([0.05, 0.11, 0.18], [0.34, 0.58, 0.62], day_t);
                let fog_density = 0.020 + 0.014 * self.rain_intensity;
                let par_umol_m2_s = if sin_e > 0.0 { 1680.0 * sin_e.powf(0.84) * cloud_atten.max(0.25) } else { 0.0 };
                let tide_cos = ((self.hour_of_day - 3.0) / 12.0 * 2.0 * PI).cos();
                let phase_name = if tide_cos > 0.45 {
                    "High-Tide Surf Surge & Anemone Bloom"
                } else if tide_cos < -0.45 {
                    "Minus Low-Tide Pools & Oystercatcher"
                } else if sin_e < 0.0 {
                    "Moonlit Barnacle & Sculpin Pools"
                } else {
                    "Flooding Basalt Mussel Bench"
                };
                DiurnalAtmosphere {
                    sun_elevation_rad: elev,
                    sun_direction: sun_dir,
                    sun_intensity,
                    sun_rgb,
                    sky_top_rgb,
                    sky_horizon_rgb,
                    fog_rgb,
                    fog_density,
                    par_umol_m2_s,
                    night_factor,
                    air_temp_c: 12.8 + 3.2 * day_t - 1.1 * self.rain_intensity,
                    relative_humidity_pct: 88.0 + 8.0 * self.rain_intensity,
                    hydro_unit: biome.telemetry_unit(),
                    salinity_psu: None,
                    phase_name,
                }
            }
            BiomeKind::ColdSeepBrinePool => {
                // 1,050m Aphotic Continental-Slope Cold Seep & Hypersaline Brine Lake (PAR = 0.0, 100% methanotrophy/thiotrophy)
                let tide_wave = ((self.hour_of_day / 12.0) * 2.0 * PI).cos();
                let phase_name = match self.hour_of_day {
                    h if h < 4.5 => "CH4 Hydrate Outgassing Surge",
                    h if h < 9.0 => "Hypersaline Mirror Halocline",
                    h if h < 13.5 => "Cutthroat Eel Brine Skimming",
                    h if h < 18.0 => "Ice-Worm Clathrate Grazing",
                    _ => "Methanotrophic Mussel Ring Glow",
                };
                DiurnalAtmosphere {
                    sun_elevation_rad: -0.95,
                    sun_direction: [0.0, 1.0, 0.0].into(),
                    sun_intensity: 0.24 + 0.08 * self.rain_intensity,
                    sun_rgb: [0.38, 0.58, 0.88],
                    sky_top_rgb: [0.015, 0.035, 0.085],
                    sky_horizon_rgb: [0.04, 0.06, 0.14],
                    fog_rgb: [0.025, 0.045, 0.11],
                    fog_density: 0.028 + 0.010 * self.rain_intensity,
                    par_umol_m2_s: 0.0,
                    night_factor: 1.0,
                    air_temp_c: 4.1 + 0.2 * tide_wave,
                    relative_humidity_pct: 120.0, // Hypersaline brine ~120 PSU
                    hydro_unit: biome.telemetry_unit(),
                    salinity_psu: Some(120.0),
                    phase_name,
                }
            }
            BiomeKind::MidwaterPelagicRift => {
                // 200-1,500m Mesopelagic-to-Bathypelagic Rift Canyon: narrowband 475nm blue twilight cues Diel Vertical Migration!
                let sun_intensity = if sin_e >= 0.0 {
                    (0.24 + 0.56 * sin_e.powf(0.85)) * cloud_atten.max(0.35)
                } else {
                    0.18
                };
                let sun_rgb = lerp3([0.16, 0.44, 0.88], [0.28, 0.68, 0.98], day_t);
                let sky_top_rgb = lerp3([0.01, 0.04, 0.12], [0.04, 0.22, 0.48], day_t);
                let sky_horizon_rgb = lerp3([0.01, 0.02, 0.07], [0.02, 0.09, 0.22], day_t);
                let fog_rgb = lerp3([0.01, 0.02, 0.06], [0.02, 0.08, 0.20], day_t);
                let fog_density = 0.024 + 0.010 * self.rain_intensity;
                let par_umol_m2_s = if sin_e > 0.0 {
                    24.0 * sin_e.powf(0.92) * cloud_atten.max(0.30)
                } else {
                    0.0
                };
                let water_temp_c = 5.4 + 0.8 * day_t - 0.3 * self.rain_intensity;
                let phase_name = match self.hour_of_day {
                    h if h < 5.5 => "Midnight DVM Surface Ascent",
                    h if h < 8.2 => "Dawn DSL Downward Descent",
                    h if h < 12.0 => "475nm Sapphire Twilight Shafts",
                    h if h < 15.8 => "Deep Canyon Counter-Illumination",
                    h if h < 18.2 => "Sperm Whale Sonar Dive Window",
                    h if h < 20.2 => "Dusk Lanternfish Vertical Migration",
                    _ => "Bioluminescent Siphonophore Night",
                };
                DiurnalAtmosphere {
                    sun_elevation_rad: elev,
                    sun_direction: normalize3([sun_dir[0] * 0.28, 0.96, -0.14]),
                    sun_intensity,
                    sun_rgb,
                    sky_top_rgb,
                    sky_horizon_rgb,
                    fog_rgb,
                    fog_density,
                    par_umol_m2_s,
                    night_factor,
                    air_temp_c: water_temp_c,
                    relative_humidity_pct: 100.0,
                    hydro_unit: biome.telemetry_unit(),
                    salinity_psu: None,
                    phase_name,
                }
            }
            BiomeKind::PnwOldGrowthSalmonCreek => {
                // Cool misty Oregon Coast Range / Cascade old-growth conifer cathedral & clear salmon creek
                let sun_intensity = if sin_e >= 0.0 {
                    (0.32 + 0.98 * sin_e.powf(0.78)) * cloud_atten.max(0.26)
                } else {
                    0.24 * (1.0 - 0.35 * self.cloud_cover)
                };
                let sun_rgb = lerp3([0.52, 0.72, 0.96], lerp3([0.98, 0.92, 0.76], [0.98, 0.66, 0.38], twilight), day_t);
                let sky_top_rgb = lerp3([0.03, 0.07, 0.14], [0.18, 0.42, 0.46], day_t);
                let sky_horizon_rgb = lerp3(
                    [0.05, 0.10, 0.16],
                    lerp3([0.28, 0.48, 0.40], [0.76, 0.48, 0.32], twilight * 0.58),
                    day_t,
                );
                let fog_rgb = lerp3([0.04, 0.08, 0.13], [0.22, 0.38, 0.34], day_t);
                let dawn_fog = if (5.2..=9.4).contains(&self.hour_of_day) { 0.012 } else { 0.0 };
                let fog_density = 0.025 + dawn_fog + 0.016 * self.rain_intensity;
                let par_umol_m2_s = if sin_e > 0.0 {
                    1480.0 * sin_e.powf(0.86) * cloud_atten.max(0.22)
                } else {
                    0.0
                };
                let air_temp_c = 11.4 + 4.8 * ((self.hour_of_day - 10.0) / 12.0 * PI).sin() - 1.4 * self.rain_intensity;
                let rh_pct = (82.0 + 12.0 * night_factor + 14.0 * self.rain_intensity - 8.0 * sin_e.max(0.0)).clamp(68.0, 99.5);
                let phase_name = match self.hour_of_day {
                    h if h < 5.4 => "Spotted Owl & Mycorrhizal Night",
                    h if h < 8.0 => "Coast Range Mist & Coho Leap",
                    h if h < 11.8 => "Cathedral Fir Sunbeams",
                    h if h < 15.4 => "Dipper Riffle & Nurse-Log Glow",
                    h if h < 17.8 => "Mossy Amber Understory",
                    h if h < 19.4 => "Varied Thrush & Owl Dusk",
                    _ => "Bioluminescent Hyphae & Stars",
                };
                DiurnalAtmosphere {
                    sun_elevation_rad: elev,
                    sun_direction: sun_dir,
                    sun_intensity,
                    sun_rgb,
                    sky_top_rgb,
                    sky_horizon_rgb,
                    fog_rgb,
                    fog_density,
                    par_umol_m2_s,
                    night_factor,
                    air_temp_c,
                    relative_humidity_pct: rh_pct,
                    hydro_unit: biome.telemetry_unit(),
                    salinity_psu: None,
                    phase_name,
                }
            }
            BiomeKind::DeepSeaHydrothermalVent => {
                // 2,600m Bathypelagic/Abyssal Rift: ZERO solar PAR (0.0 umol/m2/s)!
                // 24h cycle reflects semi-diurnal abyssal barotropic tidal pressure & chimney thermal glow.
                let tidal_surge = 0.5 + 0.5 * ((self.hour_of_day / 12.0) * 2.0 * PI).cos();
                let sun_dir = normalize3([0.0, 1.0, 0.25]);
                let sun_intensity = 0.22 + 0.14 * tidal_surge;
                let sun_rgb = [0.95, 0.48, 0.22]; // Warm chimney blackbody & mineral scatter!
                let sky_top_rgb = [0.01, 0.02, 0.05];
                let sky_horizon_rgb = lerp3([0.02, 0.04, 0.09], [0.06, 0.05, 0.11], tidal_surge);
                let fog_rgb = [0.02, 0.04, 0.08];
                let fog_density = 0.032 + 0.016 * self.rain_intensity;
                let ambient_temp_c = 2.1 + 1.8 * tidal_surge + 2.4 * self.rain_intensity;
                let dissolved_o2_pct = (58.0 + 12.0 * (1.0 - tidal_surge)).clamp(45.0, 78.0);
                let phase_name = match self.hour_of_day {
                    h if h < 4.0 => "Peak 360°C Black-Smoker Surge",
                    h if h < 8.0 => "Abyssal Flood-Tide Crossflow",
                    h if h < 12.0 => "Bioluminescent Ctenophore Drift",
                    h if h < 16.0 => "Low-Pressure H2S Vent Pulse",
                    h if h < 20.0 => "Ghost Shark Rift Patrol",
                    _ => "Atolla & Siphonophore Abyss",
                };
                DiurnalAtmosphere {
                    sun_elevation_rad: -0.65,
                    sun_direction: sun_dir,
                    sun_intensity,
                    sun_rgb,
                    sky_top_rgb,
                    sky_horizon_rgb,
                    fog_rgb,
                    fog_density,
                    par_umol_m2_s: 0.0, // Strictly zero sunlight at 2,600m depth!
                    night_factor: 1.0,  // Continuous abyssal bioluminescence!
                    air_temp_c: ambient_temp_c,
                    relative_humidity_pct: dissolved_o2_pct,
                    hydro_unit: biome.telemetry_unit(),
                    salinity_psu: None,
                    phase_name,
                }
            }
            BiomeKind::TemperateFreshwaterLake => {
                // Split-level sky + emerald-amber sun-dappled freshwater lake optics
                let sun_intensity = if sin_e >= 0.0 {
                    (0.36 + 1.05 * sin_e.powf(0.75)) * cloud_atten.max(0.28)
                } else {
                    0.26 * (1.0 - 0.35 * self.cloud_cover)
                };
                let sun_rgb = lerp3([0.52, 0.70, 0.96], [1.00, 0.96, 0.86], day_t);
                let sky_top_rgb = lerp3([0.04, 0.08, 0.18], [0.24, 0.58, 0.84], day_t);
                let sky_horizon_rgb = lerp3(
                    [0.05, 0.11, 0.17],
                    lerp3([0.16, 0.44, 0.42], [0.85, 0.54, 0.34], twilight * 0.65),
                    day_t,
                );
                let fog_rgb = lerp3([0.04, 0.09, 0.14], [0.14, 0.38, 0.36], day_t);
                let fog_density = 0.024 + 0.014 * self.rain_intensity;
                let par_umol_m2_s = if sin_e > 0.0 {
                    1720.0 * sin_e.powf(0.84) * cloud_atten.max(0.22)
                } else {
                    0.0
                };
                let water_temp_c =
                    17.8 + 2.4 * ((self.hour_of_day - 10.5) / 12.0 * PI).sin() - 1.1 * self.rain_intensity;
                let dissolved_o2_pct =
                    (96.0 + 12.0 * day_t - 5.0 * night_factor + 4.0 * self.rain_intensity).clamp(84.0, 114.0);
                let phase_name = match self.hour_of_day {
                    h if h < 5.3 => "Loon & Bullfrog Nocturne",
                    h if h < 7.3 => "Morning Mist & Mayfly Rise",
                    h if h < 11.5 => "Sunlit Littoral Ripples",
                    h if h < 15.3 => "Osprey Thermal Zenith",
                    h if h < 17.8 => "Golden Cattail Reflection",
                    h if h < 19.3 => "Evening Trout Rise & Dusk",
                    _ => "Starlit Lily-Pad & Firefly Lake",
                };
                DiurnalAtmosphere {
                    sun_elevation_rad: elev,
                    sun_direction: sun_dir,
                    sun_intensity,
                    sun_rgb,
                    sky_top_rgb,
                    sky_horizon_rgb,
                    fog_rgb,
                    fog_density,
                    par_umol_m2_s,
                    night_factor,
                    air_temp_c: water_temp_c,
                    relative_humidity_pct: dissolved_o2_pct,
                    hydro_unit: biome.telemetry_unit(),
                    salinity_psu: None,
                    phase_name,
                }
            }
            BiomeKind::IndoPacificCoralReef => {
                // Oligotrophic crystal-clear turquoise tropical lagoon -> sapphire fore-reef drop-off
                let sun_intensity = if sin_e >= 0.0 {
                    (0.38 + 1.02 * sin_e.powf(0.72)) * cloud_atten.max(0.30)
                } else {
                    0.28 * (1.0 - 0.35 * self.cloud_cover)
                };
                let sun_rgb = lerp3([0.38, 0.66, 1.00], [0.88, 0.99, 1.00], day_t);
                let sky_top_rgb = lerp3([0.02, 0.06, 0.20], [0.06, 0.72, 0.86], day_t);
                let sky_horizon_rgb = lerp3([0.02, 0.05, 0.16], [0.04, 0.42, 0.66], day_t);
                let fog_rgb = lerp3([0.02, 0.06, 0.18], [0.05, 0.48, 0.70], day_t);
                let fog_density = 0.022 + 0.010 * self.rain_intensity; // High tropical water clarity!

                let par_umol_m2_s = if sin_e > 0.0 {
                    1880.0 * sin_e.powf(0.82) * cloud_atten.max(0.25)
                } else {
                    0.0
                };
                let water_temp_c =
                    27.4 + 1.1 * ((self.hour_of_day - 10.0) / 12.0 * PI).sin() - 0.6 * self.rain_intensity;
                let dissolved_o2_pct =
                    (99.0 + 14.0 * day_t - 6.0 * night_factor + 4.0 * self.rain_intensity).clamp(88.0, 118.0);

                let phase_name = match self.hour_of_day {
                    h if h < 5.4 => "Coral GFP Biofluorescence & Polyp Feeding",
                    h if h < 7.5 => "Lagoon Dawn Refraction",
                    h if h < 11.5 => "Turquoise Reef Crest Caustics",
                    h if h < 15.2 => "Zooxanthellae Peak Photosynthesis",
                    h if h < 17.8 => "Late Afternoon Fore-Reef Drift",
                    h if h < 19.3 => "Cobalt Twilight Cleaning Hour",
                    _ => "Moonlit Fluorescent Coral Garden",
                };

                DiurnalAtmosphere {
                    sun_elevation_rad: elev,
                    sun_direction: sun_dir,
                    sun_intensity,
                    sun_rgb,
                    sky_top_rgb,
                    sky_horizon_rgb,
                    fog_rgb,
                    fog_density,
                    par_umol_m2_s,
                    night_factor,
                    air_temp_c: water_temp_c,
                    relative_humidity_pct: dissolved_o2_pct,
                    hydro_unit: biome.telemetry_unit(),
                    salinity_psu: None,
                    phase_name,
                }
            }

            BiomeKind::PacificKelpForest => {
                let sun_intensity = if sin_e >= 0.0 {
                    (0.32 + 0.92 * sin_e.powf(0.75)) * cloud_atten.max(0.25)
                } else {
                    0.24 * (1.0 - 0.4 * self.cloud_cover)
                };
                let sun_rgb = lerp3([0.32, 0.72, 0.96], [0.76, 0.98, 0.96], day_t);
                let sky_top_rgb = lerp3([0.02, 0.08, 0.16], [0.08, 0.52, 0.62], day_t);
                let sky_horizon_rgb = lerp3([0.02, 0.06, 0.12], [0.04, 0.28, 0.36], day_t);
                let fog_rgb = lerp3([0.02, 0.07, 0.13], [0.05, 0.32, 0.40], day_t);
                let fog_density = 0.032 + 0.015 * self.rain_intensity;

                let par_umol_m2_s = if sin_e > 0.0 {
                    1480.0 * sin_e.powf(0.85) * cloud_atten.max(0.22)
                } else {
                    0.0
                };
                let water_temp_c =
                    12.2 + 1.2 * ((self.hour_of_day - 10.0) / 12.0 * PI).sin() - 2.1 * self.rain_intensity;
                let dissolved_o2_pct =
                    (96.0 + 12.0 * day_t - 4.0 * night_factor + 5.0 * self.rain_intensity).clamp(84.0, 114.0);

                let phase_name = match self.hour_of_day {
                    h if h < 5.4 => "Nocturnal Dinoflagellate Glow",
                    h if h < 7.5 => "First Surface Refraction",
                    h if h < 11.5 => "Morning Kelp Caustic Beams",
                    h if h < 15.2 => "Solar Zenith Canopy Photosynthesis",
                    h if h < 17.8 => "Afternoon Emerald Swell",
                    h if h < 19.3 => "Pelagic Indigo Dusk",
                    _ => "Moonlit Bioluminescent Reef",
                };

                DiurnalAtmosphere {
                    sun_elevation_rad: elev,
                    sun_direction: sun_dir,
                    sun_intensity,
                    sun_rgb,
                    sky_top_rgb,
                    sky_horizon_rgb,
                    fog_rgb,
                    fog_density,
                    par_umol_m2_s,
                    night_factor,
                    air_temp_c: water_temp_c,
                    relative_humidity_pct: dissolved_o2_pct,
                    hydro_unit: biome.telemetry_unit(),
                    salinity_psu: None,
                    phase_name,
                }
            }

            BiomeKind::NeotropicalRainforest => {
                let sun_intensity = if sin_e >= 0.0 {
                    (0.25 + 0.95 * sin_e.powf(0.75)) * cloud_atten.max(0.22) + 0.20 * self.sunfleck_boost
                } else {
                    0.22 * (1.0 - 0.5 * self.cloud_cover)
                };
                let warm_dawn_rgb = [1.00, 0.64, 0.36];
                let noon_sun_rgb = [1.00, 0.97, 0.88];
                let moon_rgb = [0.46, 0.62, 0.88];
                let day_sun = lerp3(warm_dawn_rgb, noon_sun_rgb, sin_e.max(0.0).powf(0.6));
                let sun_rgb = lerp3(moon_rgb, lerp3(day_sun, warm_dawn_rgb, twilight * 0.65), day_t);

                let base_top = lerp3([0.03, 0.06, 0.13], [0.16, 0.44, 0.52], day_t);
                let sky_top_rgb = lerp3(
                    base_top,
                    [0.14, 0.20, 0.23],
                    (self.cloud_cover * 0.7 + self.rain_intensity * 0.3).clamp(0.0, 1.0),
                );
                let base_hor = lerp3([0.06, 0.11, 0.18], [0.44, 0.68, 0.58], day_t);
                let sky_horizon_rgb = lerp3(
                    base_hor,
                    [0.78, 0.45, 0.30],
                    twilight * 0.62 * (1.0 - self.rain_intensity * 0.7),
                );
                let fog_rgb = lerp3(
                    [0.05, 0.10, 0.15],
                    lerp3([0.26, 0.44, 0.38], [0.68, 0.52, 0.36], twilight * 0.55),
                    day_t,
                );
                let dawn_mist = if (5.3..=8.8).contains(&self.hour_of_day) {
                    1.0 - ((self.hour_of_day - 7.05).abs() / 1.75).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let fog_density = 0.026 + 0.014 * dawn_mist + 0.020 * self.rain_intensity;

                let par_umol_m2_s = if sin_e > 0.0 {
                    1950.0 * sin_e.powf(0.85) * cloud_atten.max(0.18) * (1.0 + 0.18 * self.sunfleck_boost)
                } else {
                    0.0
                };
                let air_temp_c =
                    22.8 + 6.4 * ((self.hour_of_day - 9.0) / 12.0 * PI).sin() - 1.8 * self.rain_intensity;
                let relative_humidity_pct =
                    (78.0 + 16.0 * night_factor + 18.0 * self.rain_intensity - 10.0 * sin_e.max(0.0))
                        .clamp(62.0, 99.5);

                let phase_name = match self.hour_of_day {
                    h if h < 5.2 => "Deep Nocturnal Chorus",
                    h if h < 7.0 => "Dawn Mist & First Light",
                    h if h < 11.0 => "Morning Canopy Sunflecks",
                    h if h < 15.0 => "Solar Zenith & Thermal Convection",
                    h if h < 17.3 => "Late Afternoon Understory Filter",
                    h if h < 19.1 => "Crepuscular Amber Dusk",
                    _ => "Moonlit Bioluminescent Night",
                };

                DiurnalAtmosphere {
                    sun_elevation_rad: elev,
                    sun_direction: sun_dir,
                    sun_intensity,
                    sun_rgb,
                    sky_top_rgb,
                    sky_horizon_rgb,
                    fog_rgb,
                    fog_density,
                    par_umol_m2_s,
                    night_factor,
                    air_temp_c,
                    relative_humidity_pct,
                    hydro_unit: biome.telemetry_unit(),
                    salinity_psu: None,
                    phase_name,
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrophicEnergyLedger {
    pub biomass_kcal: [f32; 5],
    pub cumulative_gpp_kcal: f32,
    pub assimilation_kcal: [f32; 5],
    pub respiration_kcal: [f32; 5],
    pub detritus_pool_g: f32,
    pub mineral_nutrient_pool_mg: f32,
    pub cumulative_mineralized_mg: f32,
    #[serde(default)]
    pub keystone_removed: bool,
    #[serde(default = "default_flux_arcs_on")]
    pub show_flux_arcs: bool,
    #[serde(default)]
    pub biomass_history: Vec<[f32; 5]>,
    #[serde(default)]
    pub history_sample_accum: f32,
    #[serde(default)]
    pub external_detritus_kcal: f32,
    #[serde(default)]
    pub last_assim_rate_kcal_s: [f32; 5],
    #[serde(default)]
    pub last_resp_rate_kcal_s: [f32; 5],
}

fn default_flux_arcs_on() -> bool {
    true
}

impl Default for TrophicEnergyLedger {
    fn default() -> Self {
        Self {
            biomass_kcal: [420.0, 4850.0, 640.0, 210.0, 88.0],
            cumulative_gpp_kcal: 0.0,
            assimilation_kcal: [0.0; 5],
            respiration_kcal: [0.0; 5],
            detritus_pool_g: 145.0,
            mineral_nutrient_pool_mg: 92.0,
            cumulative_mineralized_mg: 0.0,
            keystone_removed: false,
            show_flux_arcs: true,
            biomass_history: Vec::new(),
            history_sample_accum: 0.0,
            external_detritus_kcal: 0.0,
            last_assim_rate_kcal_s: [0.0; 5],
            last_resp_rate_kcal_s: [0.0; 5],
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TrophicWebParams {
    base: [f32; 5],
    gpp_star: f32,
    r1_growth_frac: f32,
    m1: f32,
    d1: f32,
    alpha: [f32; 5],
    c_12: f32,
    h_12: f32,
    m2: f32,
    r2_sda: f32,
    d2: f32,
    c_23: f32,
    h_23: f32,
    c_24: f32,
    h_24: f32,
    c_03: f32,
    h_03: f32,
    m3: f32,
    r3_sda: f32,
    d3: f32,
    c_34: f32,
    h_34: f32,
    m4: f32,
    r4_sda: f32,
    d4: f32,
    q4: f32,
    c_d0: f32,
    h_d0: f32,
    m0: f32,
    r0_sda: f32,
    d0: f32,
}

#[derive(Debug, Clone, Copy)]
struct TrophicFluxSample {
    gpp_eff: f32,
    assim: [f32; 5],
    resp: [f32; 5],
    db: [f32; 5],
    dd_kcal: f32,
}

impl TrophicWebParams {
    fn for_biome(biome: BiomeKind) -> Self {
        let base = TrophicEnergyLedger::baseline_for_biome(biome);
        let detritus_base_kcal =
            TrophicEnergyLedger::baseline_detritus_g(biome) * TrophicEnergyLedger::KCAL_PER_G_DETRITUS;
        let [b0, b1, b2, b3, b4] = base;

        // Primary production & producer respiration/senescence
        let gpp_rate_factor = match biome {
            BiomeKind::IndoPacificCoralReef
            | BiomeKind::RockyIntertidalTidepool
            | BiomeKind::PacificKelpForest => 0.17,
            BiomeKind::DeepSeaHydrothermalVent
            | BiomeKind::ColdSeepBrinePool
            | BiomeKind::TemperateFreshwaterLake
            | BiomeKind::MidwaterPelagicRift => 0.16,
            BiomeKind::NeotropicalRainforest | BiomeKind::PnwOldGrowthSalmonCreek => 0.145,
        };
        let gpp_star = gpp_rate_factor * b1;
        let r1_growth_frac = 0.40;
        let r1_star = 0.46 * gpp_star;
        let m1 = (r1_star - r1_growth_frac * gpp_star).max(0.0) / b1;
        let npp_star = gpp_star - r1_star;
        let s1_star = 0.14 * npp_star;
        let d1 = s1_star / b1;

        // L1 -> L2 grazing (Holling Type II)
        let f12_star = npp_star - s1_star;
        let h_12 = 0.65 * b1;
        let c_12 = (1.65 * f12_star) / b2;

        // L2 assimilation, respiration, mortality & predation split
        let alpha2 = 0.54;
        let a2_star = alpha2 * f12_star;
        let r2_sda = 0.34;
        let r2_star = 0.64 * a2_star;
        let m2 = (r2_star - r2_sda * a2_star).max(0.0) / b2;
        let p2_star = a2_star - r2_star;
        let d2 = (0.12 * p2_star) / b2;

        let (apex_direct_frac, meso_frac) = match biome {
            BiomeKind::RockyIntertidalTidepool => (0.68, 0.20),
            BiomeKind::PacificKelpForest => (0.16, 0.72),
            BiomeKind::MidwaterPelagicRift => (0.14, 0.74),
            BiomeKind::ColdSeepBrinePool => (0.26, 0.62),
            BiomeKind::TemperateFreshwaterLake => (0.34, 0.54),
            BiomeKind::NeotropicalRainforest => (0.32, 0.56),
            BiomeKind::PnwOldGrowthSalmonCreek
            | BiomeKind::DeepSeaHydrothermalVent
            | BiomeKind::IndoPacificCoralReef => (0.28, 0.60),
        };
        let f23_star = meso_frac * p2_star;
        let f24_star = apex_direct_frac * p2_star;
        let h_23 = 0.70 * b2;
        let h_24 = 0.70 * b2;
        let c_23 = (1.70 * f23_star) / b3;
        let c_24 = (1.70 * f24_star) / b4;

        // L3 secondary consumers (plus modest benthic/detritivore L0->L3 link)
        let f03_star = 0.18 * f23_star;
        let h_03 = 0.75 * b0;
        let c_03 = (1.75 * f03_star) / b3;
        let alpha3 = 0.72;
        let a3_star = alpha3 * (f23_star + f03_star);
        let r3_sda = 0.40;
        let r3_star = 0.70 * a3_star;
        let m3 = (r3_star - r3_sda * a3_star).max(0.0) / b3;
        let p3_star = a3_star - r3_star;
        let d3 = (0.16 * p3_star) / b3;
        let f34_star = 0.84 * p3_star;
        let h_34 = 0.70 * b3;
        let c_34 = (1.70 * f34_star) / b4;

        // L4 apex predators (Holling II intake + logistic territorial mortality -> detritus)
        let alpha4 = 0.78;
        let a4_star = alpha4 * (f24_star + f34_star);
        let r4_sda = 0.44;
        let r4_star = 0.75 * a4_star;
        let m4 = (r4_star - r4_sda * a4_star).max(0.0) / b4;
        let p4_star = a4_star - r4_star;
        let d4 = (0.45 * p4_star) / b4;
        let q4 = (0.55 * p4_star) / (b4 * b4);

        // L0 decomposers & detritus recycling closed to exact steady state
        let r0_star = (gpp_star - r1_star - r2_star - r3_star - r4_star).max(1.0);
        let mort0_star = 0.18 * r0_star;
        let d0 = mort0_star / b0;
        let a0_star = r0_star + mort0_star + f03_star;
        let alpha0 = 0.65;
        let fd0_star = a0_star / alpha0;
        let h_d0 = 0.65 * detritus_base_kcal;
        let c_d0 = (1.65 * fd0_star) / b0;
        let r0_sda = 0.60 * (r0_star / a0_star);
        let m0 = (0.40 * r0_star) / b0;

        Self {
            base,
            gpp_star,
            r1_growth_frac,
            m1,
            d1,
            alpha: [alpha0, 1.0, alpha2, alpha3, alpha4],
            c_12,
            h_12,
            m2,
            r2_sda,
            d2,
            c_23,
            h_23,
            c_24,
            h_24,
            c_03,
            h_03,
            m3,
            r3_sda,
            d3,
            c_34,
            h_34,
            m4,
            r4_sda,
            d4,
            q4,
            c_d0,
            h_d0,
            m0,
            r0_sda,
            d0,
        }
    }

    fn eval_fluxes(
        &self,
        biome: BiomeKind,
        b: [f32; 5],
        d_kcal: f32,
        gpp_rate_in: f32,
        activity_drive: f32,
        keystone_removed: bool,
        h_step: f32,
    ) -> TrophicFluxSample {
        let act = activity_drive.clamp(0.04, 1.4);
        let b0 = b[0].max(0.0);
        let b1 = b[1].max(0.0);
        let b2 = b[2].max(0.0);
        let b3 = b[3].max(0.0);
        let b4 = b[4].max(0.0);
        let d = d_kcal.max(0.0);

        // Keystone guild mechanistic modulations (consumptive & trait-mediated release / facilitation loss)
        let (
            gpp_facil,
            c12_mult,
            c23_mult,
            c24_mult,
            c34_mult,
            extra_m2,
            extra_m3,
            extra_m4,
        ) = if !keystone_removed {
            (1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32, 0.0_f32, 0.0_f32, 0.0_f32)
        } else {
            match biome {
                // Paine (1966): Pisaster (L4) removal removes direct mussel predation & triggers Mytilus bed dominance
                BiomeKind::RockyIntertidalTidepool => (0.94, 1.65, 0.68, 0.14, 0.14, 0.0, 0.055, 0.36),
                // Estes & Palmisano (1974): Sea otter (L3) removal releases sea urchins (L2) to overgraze Macrocystis kelp
                BiomeKind::PacificKelpForest => (0.94, 1.72, 0.18, 0.65, 0.65, 0.0, 0.34, 0.06),
                // Biogenic Bathymodiolus & Lamellibrachia cold-seep foundation engineering removed
                BiomeKind::ColdSeepBrinePool => (0.64, 0.72, 0.92, 0.92, 0.92, 0.10, 0.03, 0.03),
                // Spawning Coho salmon & Spotted Owls removed: marine 15N subsidy and apex control drop
                BiomeKind::PnwOldGrowthSalmonCreek => (0.72, 0.58, 0.95, 0.25, 0.25, 0.26, 0.05, 0.30),
                // Diving sperm whales (L4) removed -> mesopredator squid release suppresses DVM lanternfish/krill
                BiomeKind::MidwaterPelagicRift => (0.96, 1.02, 1.38, 0.18, 0.18, 0.0, 0.0, 0.36),
                // Apex predator removal across Reef, Lake, Vent, Rainforest releases herbivore/grazer foraging
                BiomeKind::IndoPacificCoralReef
                | BiomeKind::TemperateFreshwaterLake
                | BiomeKind::DeepSeaHydrothermalVent
                | BiomeKind::NeotropicalRainforest => (0.96, 1.52, 0.85, 0.18, 0.18, 0.0, 0.02, 0.36),
            }
        };

        // Light/nutrient-driven GPP with smooth self-shading carrying-capacity regulation (no hard clamp)
        let shading_crowding = (1.55 - 0.55 * (b1 / self.base[1].max(1.0))).clamp(0.15, 1.55);
        let gpp_eff = (gpp_rate_in * shading_crowding * gpp_facil).max(0.0);

        // Holling Type II trophic interactions
        let sat12 = b1 / (self.h_12 + b1 + 1e-6);
        let sat23 = b2 / (self.h_23 + b2 + 1e-6);
        let sat24 = b2 / (self.h_24 + b2 + 1e-6);
        let sat34 = b3 / (self.h_34 + b3 + 1e-6);
        let sat03 = b0 / (self.h_03 + b0 + 1e-6);
        let satd0 = d / (self.h_d0 + d + 1e-6);

        let mut f_12 = act * self.c_12 * c12_mult * b2 * sat12;
        let mut f_23 = act * self.c_23 * c23_mult * b3 * sat23;
        let mut f_24 = act * self.c_24 * c24_mult * b4 * sat24;
        let mut f_34 = act * self.c_34 * c34_mult * b4 * sat34;
        let mut f_03 = act * self.c_03 * b3 * sat03;
        let mut f_d0 = act * self.c_d0 * b0 * satd0;

        // Basal maintenance respiration + non-predatory senescence/mortality
        let mut r1_base = act * self.m1 * b1;
        let mut mort1 = act * self.d1 * b1;

        let mut r2_base = act * self.m2 * b2;
        let mut mort2 = act * (self.d2 + extra_m2) * b2;

        let mut r3_base = act * self.m3 * b3;
        let mut mort3 = act * (self.d3 + extra_m3) * b3;

        let mut r4_base = act * self.m4 * b4;
        let mut mort4 = act * (self.d4 + extra_m4 + self.q4 * b4) * b4;

        let mut r0_base = act * self.m0 * b0;
        let mut mort0 = act * self.d0 * b0;

        // Proportional flux capping so no compartment ever goes negative while keeping conservation exact
        let dt_safe = h_step.max(1e-4);
        let cap_pool = |avail: f32, out_total: f32| -> f32 {
            let max_rate = (avail * 0.85) / dt_safe;
            if out_total > max_rate && out_total > 1e-6 {
                max_rate / out_total
            } else {
                1.0
            }
        };

        let s0 = cap_pool(b0, r0_base + mort0 + f_03);
        r0_base *= s0;
        mort0 *= s0;
        f_03 *= s0;

        let s1 = cap_pool(b1 + gpp_eff * dt_safe, r1_base + mort1 + f_12);
        r1_base *= s1;
        mort1 *= s1;
        f_12 *= s1;

        let s2 = cap_pool(b2, r2_base + mort2 + f_23 + f_24);
        r2_base *= s2;
        mort2 *= s2;
        f_23 *= s2;
        f_24 *= s2;

        let s3 = cap_pool(b3, r3_base + mort3 + f_34);
        r3_base *= s3;
        mort3 *= s3;
        f_34 *= s3;

        let s4 = cap_pool(b4, r4_base + mort4);
        r4_base *= s4;
        mort4 *= s4;

        let a0_raw = self.alpha[0] * f_d0;
        let sd = cap_pool(d, a0_raw);
        f_d0 *= sd;

        // Assimilation, SDA respiration, and egestion into detritus
        let a0 = self.alpha[0] * f_d0;
        let a1 = gpp_eff;
        let a2 = self.alpha[2] * f_12;
        let a3 = self.alpha[3] * (f_23 + f_03);
        let a4 = self.alpha[4] * (f_24 + f_34);

        let r0 = r0_base + self.r0_sda * a0;
        let r1 = r1_base + self.r1_growth_frac * gpp_eff;
        let r2 = r2_base + self.r2_sda * a2;
        let r3 = r3_base + self.r3_sda * a3;
        let r4 = r4_base + self.r4_sda * a4;

        let egest_2 = (1.0 - self.alpha[2]) * f_12;
        let egest_3 = (1.0 - self.alpha[3]) * (f_23 + f_03);
        let egest_4 = (1.0 - self.alpha[4]) * (f_24 + f_34);

        let db = [
            a0 - r0 - mort0 - f_03,
            a1 - r1 - mort1 - f_12,
            a2 - r2 - mort2 - f_23 - f_24,
            a3 - r3 - mort3 - f_34,
            a4 - r4 - mort4,
        ];
        let dd_kcal = (mort0 + mort1 + mort2 + mort3 + mort4) + (egest_2 + egest_3 + egest_4) - a0;

        TrophicFluxSample {
            gpp_eff,
            assim: [a0, a1, a2, a3, a4],
            resp: [r0, r1, r2, r3, r4],
            db,
            dd_kcal,
        }
    }
}

impl TrophicEnergyLedger {
    /// Energy density of ash-free organic detritus (kcal per gram dry mass).
    pub const KCAL_PER_G_DETRITUS: f32 = 4.5;

    pub fn baseline_detritus_g(biome: BiomeKind) -> f32 {
        match biome {
            BiomeKind::RockyIntertidalTidepool => 176.0,
            BiomeKind::ColdSeepBrinePool => 285.0,
            BiomeKind::MidwaterPelagicRift => 310.0,
            BiomeKind::PnwOldGrowthSalmonCreek => 245.0,
            BiomeKind::DeepSeaHydrothermalVent => 230.0,
            BiomeKind::TemperateFreshwaterLake => 168.0,
            BiomeKind::IndoPacificCoralReef => 112.0,
            BiomeKind::PacificKelpForest => 195.0,
            BiomeKind::NeotropicalRainforest => 145.0,
        }
    }

    pub fn baseline_mineral_mg(biome: BiomeKind) -> f32 {
        match biome {
            BiomeKind::RockyIntertidalTidepool => 142.0,
            BiomeKind::ColdSeepBrinePool => 248.0,
            BiomeKind::MidwaterPelagicRift => 188.0,
            BiomeKind::PnwOldGrowthSalmonCreek => 164.0,
            BiomeKind::DeepSeaHydrothermalVent => 215.0,
            BiomeKind::TemperateFreshwaterLake => 86.0,
            BiomeKind::IndoPacificCoralReef => 64.0,
            BiomeKind::PacificKelpForest => 148.0,
            BiomeKind::NeotropicalRainforest => 92.0,
        }
    }

    pub fn detritus_kcal(&self) -> f32 {
        self.detritus_pool_g * Self::KCAL_PER_G_DETRITUS
    }

    pub fn total_system_energy_kcal(&self) -> f32 {
        self.biomass_kcal.iter().sum::<f32>() + self.detritus_kcal()
    }

    /// Deposits non-ODE detritus (e.g. falling fruit, kelp wrack, marine snow) while keeping conservation tracked.
    pub fn add_external_detritus_g(&mut self, grams: f32) {
        let g = grams.max(0.0);
        self.detritus_pool_g += g;
        self.external_detritus_kcal += g * Self::KCAL_PER_G_DETRITUS;
    }

    /// Deposits a direct autotrophic chemosynthetic/photosynthetic pulse into L1 producers and GPP.
    pub fn add_chemosynthetic_gpp_pulse(&mut self, kcal: f32) {
        let k = kcal.max(0.0);
        self.cumulative_gpp_kcal += k;
        self.assimilation_kcal[TrophicStratum::Producers.index()] += k;
        self.biomass_kcal[TrophicStratum::Producers.index()] += k;
    }

    /// Computes primary production input `kcal` over `dt` from irradiance/chemosynthesis, flora vigor, and nutrients.
    pub fn compute_biome_gpp_step(
        biome: BiomeKind,
        par_norm: f32,
        mean_vigor: f32,
        trees_len: usize,
        kelp_len: usize,
        underbrush_len: usize,
        mineral_mg: f32,
        dt: f32,
    ) -> f32 {
        if dt <= 0.0 || par_norm <= 0.0 {
            return 0.0;
        }
        let params = TrophicWebParams::for_biome(biome);
        let par_ref = match biome {
            BiomeKind::ColdSeepBrinePool | BiomeKind::DeepSeaHydrothermalVent => 0.86,
            BiomeKind::MidwaterPelagicRift => 0.78,
            _ => 0.85,
        };
        let par_ratio = (par_norm / par_ref).clamp(0.0, 1.45);
        let stand_weight = if trees_len + kelp_len + underbrush_len == 0 {
            1.0
        } else {
            (0.65 + 0.35 * mean_vigor.clamp(0.3, 1.5)).clamp(0.45, 1.35)
        };
        let nut_ratio = (mineral_mg / Self::baseline_mineral_mg(biome).max(1.0)).clamp(0.45, 1.40);
        params.gpp_star * par_ratio * stand_weight * nut_ratio * dt
    }

    pub fn new_rainforest_baseline() -> Self {
        let base = Self::baseline_for_biome(BiomeKind::NeotropicalRainforest);
        Self {
            biomass_kcal: base,
            detritus_pool_g: Self::baseline_detritus_g(BiomeKind::NeotropicalRainforest),
            mineral_nutrient_pool_mg: Self::baseline_mineral_mg(BiomeKind::NeotropicalRainforest),
            biomass_history: vec![base; 24],
            ..Default::default()
        }
    }

    pub fn new_kelp_forest_baseline() -> Self {
        let base = Self::baseline_for_biome(BiomeKind::PacificKelpForest);
        Self {
            biomass_kcal: base,
            detritus_pool_g: Self::baseline_detritus_g(BiomeKind::PacificKelpForest),
            mineral_nutrient_pool_mg: Self::baseline_mineral_mg(BiomeKind::PacificKelpForest),
            biomass_history: vec![base; 24],
            ..Default::default()
        }
    }

    pub fn new_tidepool_baseline() -> Self {
        let base = Self::baseline_for_biome(BiomeKind::RockyIntertidalTidepool);
        Self {
            biomass_kcal: base,
            detritus_pool_g: Self::baseline_detritus_g(BiomeKind::RockyIntertidalTidepool),
            mineral_nutrient_pool_mg: Self::baseline_mineral_mg(BiomeKind::RockyIntertidalTidepool),
            biomass_history: vec![base; 24],
            ..Default::default()
        }
    }

    pub fn new_brine_pool_baseline() -> Self {
        let base = Self::baseline_for_biome(BiomeKind::ColdSeepBrinePool);
        Self {
            biomass_kcal: base,
            detritus_pool_g: Self::baseline_detritus_g(BiomeKind::ColdSeepBrinePool),
            mineral_nutrient_pool_mg: Self::baseline_mineral_mg(BiomeKind::ColdSeepBrinePool),
            biomass_history: vec![base; 24],
            ..Default::default()
        }
    }

    pub fn baseline_for_biome(biome: BiomeKind) -> [f32; 5] {
        match biome {
            BiomeKind::RockyIntertidalTidepool => [490.0, 3920.0, 1080.0, 345.0, 134.0],
            BiomeKind::ColdSeepBrinePool => [640.0, 4620.0, 1040.0, 330.0, 126.0],
            BiomeKind::MidwaterPelagicRift => [540.0, 4480.0, 1120.0, 390.0, 156.0],
            BiomeKind::PnwOldGrowthSalmonCreek => [580.0, 6420.0, 740.0, 260.0, 108.0],
            BiomeKind::DeepSeaHydrothermalVent => [620.0, 5180.0, 960.0, 340.0, 132.0],
            BiomeKind::TemperateFreshwaterLake => [440.0, 4680.0, 790.0, 295.0, 118.0],
            BiomeKind::IndoPacificCoralReef => [460.0, 4380.0, 910.0, 365.0, 148.0],
            BiomeKind::PacificKelpForest => [510.0, 5620.0, 820.0, 310.0, 125.0],
            BiomeKind::NeotropicalRainforest => [420.0, 4850.0, 640.0, 210.0, 88.0],
        }
    }

    /// Integrates coupled Rosenzweig-MacArthur / Lotka-Volterra trophic ODEs via midpoint RK2
    /// with explicit assimilation, respiration, detritus accounting, and rolling sparkline sampling.
    pub fn step_trophic_ode(
        &mut self,
        biome: BiomeKind,
        gpp_input_kcal: f32,
        activity_drive: f32,
        dt: f32,
    ) {
        if dt <= 0.0 {
            return;
        }
        let params = TrophicWebParams::for_biome(biome);
        let gpp_rate_in = (gpp_input_kcal.max(0.0)) / dt;
        let substeps = ((dt / 0.025).ceil() as usize).clamp(1, 32);
        let h = dt / substeps as f32;

        let mut d_kcal = self.detritus_pool_g.max(0.0) * Self::KCAL_PER_G_DETRITUS;
        let mut last_flux = TrophicFluxSample {
            gpp_eff: 0.0,
            assim: [0.0; 5],
            resp: [0.0; 5],
            db: [0.0; 5],
            dd_kcal: 0.0,
        };

        for _ in 0..substeps {
            let k1 = params.eval_fluxes(
                biome,
                self.biomass_kcal,
                d_kcal,
                gpp_rate_in,
                activity_drive,
                self.keystone_removed,
                0.5 * h,
            );
            let mut b_mid = [0.0_f32; 5];
            for i in 0..5 {
                b_mid[i] = (self.biomass_kcal[i] + 0.5 * h * k1.db[i]).max(0.0);
            }
            let d_mid = (d_kcal + 0.5 * h * k1.dd_kcal).max(0.0);

            let k2 = params.eval_fluxes(
                biome,
                b_mid,
                d_mid,
                gpp_rate_in,
                activity_drive,
                self.keystone_removed,
                h,
            );

            for i in 0..5 {
                self.biomass_kcal[i] = (self.biomass_kcal[i] + h * k2.db[i]).max(0.0);
                self.assimilation_kcal[i] += h * k2.assim[i];
                self.respiration_kcal[i] += h * k2.resp[i];
            }
            d_kcal = (d_kcal + h * k2.dd_kcal).max(0.0);
            self.cumulative_gpp_kcal += h * k2.gpp_eff;
            last_flux = k2;
        }

        self.detritus_pool_g = d_kcal / Self::KCAL_PER_G_DETRITUS;

        let ema_a = (dt / 2.5).clamp(0.02, 1.0);
        for i in 0..5 {
            if self.last_assim_rate_kcal_s[1] <= 1e-6 {
                self.last_assim_rate_kcal_s[i] = last_flux.assim[i];
                self.last_resp_rate_kcal_s[i] = last_flux.resp[i];
            } else {
                self.last_assim_rate_kcal_s[i] +=
                    (last_flux.assim[i] - self.last_assim_rate_kcal_s[i]) * ema_a;
                self.last_resp_rate_kcal_s[i] +=
                    (last_flux.resp[i] - self.last_resp_rate_kcal_s[i]) * ema_a;
            }
        }

        if self.keystone_removed && biome == BiomeKind::PnwOldGrowthSalmonCreek {
            self.mineral_nutrient_pool_mg = (self.mineral_nutrient_pool_mg - 6.5 * dt).max(48.0);
        }

        self.history_sample_accum += dt;
        if self.biomass_history.is_empty() || self.history_sample_accum >= 0.35 {
            self.history_sample_accum = 0.0;
            self.biomass_history.push(self.biomass_kcal);
            if self.biomass_history.len() > 64 {
                self.biomass_history.remove(0);
            }
        }
    }

    /// Advances the coupled trophic ODEs using nominal daytime/chemosynthetic primary production.
    pub fn step_keystone_cascade(&mut self, biome: BiomeKind, dt: f32) {
        let params = TrophicWebParams::for_biome(biome);
        self.step_trophic_ode(biome, params.gpp_star * dt, 1.0, dt);
    }

    pub fn new_pelagic_rift_baseline() -> Self {
        let base = Self::baseline_for_biome(BiomeKind::MidwaterPelagicRift);
        Self {
            // Marine snow + huge mesopelagic DVM lanternfish/krill biomass supporting diving sperm whales & giant squid!
            biomass_kcal: base,
            detritus_pool_g: Self::baseline_detritus_g(BiomeKind::MidwaterPelagicRift),
            mineral_nutrient_pool_mg: Self::baseline_mineral_mg(BiomeKind::MidwaterPelagicRift),
            biomass_history: vec![base; 24],
            ..Default::default()
        }
    }

    pub fn new_pnw_old_growth_baseline() -> Self {
        let base = Self::baseline_for_biome(BiomeKind::PnwOldGrowthSalmonCreek);
        Self {
            // Massive standing Douglas-fir/Redcedar biomass + marine-derived 15N salmon subsidy!
            biomass_kcal: base,
            detritus_pool_g: Self::baseline_detritus_g(BiomeKind::PnwOldGrowthSalmonCreek),
            mineral_nutrient_pool_mg: Self::baseline_mineral_mg(BiomeKind::PnwOldGrowthSalmonCreek),
            biomass_history: vec![base; 24],
            ..Default::default()
        }
    }

    pub fn new_hydrothermal_vent_baseline() -> Self {
        let base = Self::baseline_for_biome(BiomeKind::DeepSeaHydrothermalVent);
        Self {
            // 100% Chemosynthetic RuBisCO-II & rTCA sulfur/methane-oxidizing primary production!
            biomass_kcal: base,
            detritus_pool_g: Self::baseline_detritus_g(BiomeKind::DeepSeaHydrothermalVent),
            mineral_nutrient_pool_mg: Self::baseline_mineral_mg(BiomeKind::DeepSeaHydrothermalVent),
            biomass_history: vec![base; 24],
            ..Default::default()
        }
    }

    pub fn new_freshwater_lake_baseline() -> Self {
        let base = Self::baseline_for_biome(BiomeKind::TemperateFreshwaterLake);
        Self {
            biomass_kcal: base,
            detritus_pool_g: Self::baseline_detritus_g(BiomeKind::TemperateFreshwaterLake),
            mineral_nutrient_pool_mg: Self::baseline_mineral_mg(BiomeKind::TemperateFreshwaterLake),
            biomass_history: vec![base; 24],
            ..Default::default()
        }
    }

    pub fn new_coral_reef_baseline() -> Self {
        let base = Self::baseline_for_biome(BiomeKind::IndoPacificCoralReef);
        Self {
            // Tightly recycled Darwin-Paradox Symbiodinium coral holobiont & high reef fish standing stock!
            biomass_kcal: base,
            detritus_pool_g: Self::baseline_detritus_g(BiomeKind::IndoPacificCoralReef),
            mineral_nutrient_pool_mg: Self::baseline_mineral_mg(BiomeKind::IndoPacificCoralReef),
            biomass_history: vec![base; 24],
            ..Default::default()
        }
    }

    /// Net trophic production `P_i = max(0, A_i - R_i)` (cumulative kcal).
    pub fn net_production_kcal(&self, stratum: TrophicStratum) -> f32 {
        let idx = stratum.index();
        (self.assimilation_kcal[idx] - self.respiration_kcal[idx]).max(0.0)
    }

    /// Lindeman production transfer efficiency (`P_n / P_{n-1} * 100%` for `L2..L4`;
    /// `NPP / GPP` for `L1` producers; microbial carbon-use efficiency `P_0 / A_0` for `L0`).
    pub fn transfer_efficiency_pct(&self, stratum: TrophicStratum) -> f32 {
        let idx = stratum.index();
        let use_live = self.last_assim_rate_kcal_s[1] > 1e-4;
        if use_live {
            let net = |k: usize| -> f32 {
                (self.last_assim_rate_kcal_s[k] - self.last_resp_rate_kcal_s[k]).max(0.0)
            };
            return match idx {
                0 => {
                    let a0 = self.last_assim_rate_kcal_s[0];
                    if a0 > 1e-4 {
                        ((net(0) / a0) * 100.0).clamp(0.0, 100.0)
                    } else {
                        35.0
                    }
                }
                1 => {
                    let a1 = self.last_assim_rate_kcal_s[1];
                    if a1 > 1e-4 {
                        ((net(1) / a1) * 100.0).clamp(0.0, 100.0)
                    } else {
                        54.0
                    }
                }
                _ => {
                    let prev_p = net(idx - 1);
                    if prev_p > 1e-4 {
                        ((net(idx) / prev_p) * 100.0).clamp(0.0, 100.0)
                    } else {
                        0.0
                    }
                }
            };
        }

        if self.assimilation_kcal[1] > 1e-4 {
            return match idx {
                0 => ((self.net_production_kcal(stratum) / self.assimilation_kcal[0].max(1e-4))
                    * 100.0)
                    .clamp(0.0, 100.0),
                1 => ((self.net_production_kcal(stratum) / self.assimilation_kcal[1].max(1e-4))
                    * 100.0)
                    .clamp(0.0, 100.0),
                _ => {
                    let prev = TrophicStratum::ALL[idx - 1];
                    let prev_p = self.net_production_kcal(prev).max(1e-4);
                    ((self.net_production_kcal(stratum) / prev_p) * 100.0).clamp(0.0, 100.0)
                }
            };
        }

        match idx {
            0 => 35.0,
            1 => 54.0,
            2 => 16.7,
            3 => 16.5,
            _ => 18.0,
        }
    }
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
    [v[0] / len, v[1] / len, v[2] / len]
}
