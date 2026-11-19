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
    pub phase_name: &'static str,
}

impl DiurnalCycle {
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
                    relative_humidity_pct: 120.0, // Brine salinity ~120 PSU!
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
        }
    }
}

impl TrophicEnergyLedger {
    pub fn new_rainforest_baseline() -> Self {
        Self {
            biomass_kcal: [420.0, 4850.0, 640.0, 210.0, 88.0],
            cumulative_gpp_kcal: 0.0,
            assimilation_kcal: [0.0; 5],
            respiration_kcal: [0.0; 5],
            detritus_pool_g: 145.0,
            mineral_nutrient_pool_mg: 92.0,
            cumulative_mineralized_mg: 0.0,
            ..Default::default()
        }
    }

    pub fn new_kelp_forest_baseline() -> Self {
        Self {
            biomass_kcal: [510.0, 5620.0, 820.0, 310.0, 125.0],
            cumulative_gpp_kcal: 0.0,
            assimilation_kcal: [0.0; 5],
            respiration_kcal: [0.0; 5],
            detritus_pool_g: 195.0,
            mineral_nutrient_pool_mg: 148.0,
            cumulative_mineralized_mg: 0.0,
            ..Default::default()
        }
    }

    pub fn new_tidepool_baseline() -> Self {
        let base = [490.0, 3920.0, 1080.0, 345.0, 134.0];
        Self {
            biomass_kcal: base,
            detritus_pool_g: 176.0,
            mineral_nutrient_pool_mg: 142.0,
            biomass_history: vec![base; 24],
            ..Default::default()
        }
    }

    pub fn new_brine_pool_baseline() -> Self {
        let base = [640.0, 4620.0, 1040.0, 330.0, 126.0];
        Self {
            biomass_kcal: base,
            detritus_pool_g: 285.0,
            mineral_nutrient_pool_mg: 248.0,
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

    /// Integrates smooth Lotka-Volterra keystone trophic cascade relaxation and records rolling sparkline samples.
    pub fn step_keystone_cascade(&mut self, biome: BiomeKind, dt: f32) {
        let base = Self::baseline_for_biome(biome);
        let mults: [f32; 5] = if !self.keystone_removed {
            [1.0, 1.0, 1.0, 1.0, 1.0]
        } else {
            match biome {
                // Paine (1966): Removing Pisaster causes Mytilus mussel monoculture (L2 x2.05) crowding out algae & anemones!
                BiomeKind::RockyIntertidalTidepool => [1.18, 0.48, 2.05, 0.52, 0.18],
                // Cold-Seep Engineers removed: biogenic tubeworm/mussel habitat & predators decline
                BiomeKind::ColdSeepBrinePool => [1.35, 0.44, 0.46, 0.58, 0.45],
                // Estes & Palmisano: Sea Otter removal -> Urchin Barren (L2 x2.15, Kelp L1 x0.42)!
                BiomeKind::PacificKelpForest => [1.22, 0.42, 2.15, 0.35, 0.58],
                // Salmon blockade / Owl loss -> Marine 15N & mycorrhizal subsidy drop!
                BiomeKind::PnwOldGrowthSalmonCreek => [0.62, 0.74, 0.38, 0.68, 0.25],
                // Apex predator removal -> mesopredator / herbivore release across remaining biomes
                BiomeKind::MidwaterPelagicRift => [1.12, 0.85, 0.64, 1.68, 0.20],
                BiomeKind::DeepSeaHydrothermalVent => [1.24, 0.62, 1.75, 1.32, 0.22],
                BiomeKind::TemperateFreshwaterLake => [1.28, 0.58, 1.88, 0.72, 0.20],
                BiomeKind::IndoPacificCoralReef => [1.25, 0.55, 1.72, 0.68, 0.22],
                BiomeKind::NeotropicalRainforest => [1.18, 0.62, 1.85, 1.38, 0.18],
            }
        };

        let rate = (dt * 0.45).min(1.0);
        for i in 0..5 {
            let target = base[i] * mults[i];
            self.biomass_kcal[i] += (target - self.biomass_kcal[i]) * rate;
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

    pub fn new_pelagic_rift_baseline() -> Self {
        Self {
            // Marine snow + huge mesopelagic DVM lanternfish/krill biomass supporting diving sperm whales & giant squid!
            biomass_kcal: [540.0, 4480.0, 1120.0, 390.0, 156.0],
            cumulative_gpp_kcal: 0.0,
            assimilation_kcal: [0.0; 5],
            respiration_kcal: [0.0; 5],
            detritus_pool_g: 310.0,
            mineral_nutrient_pool_mg: 188.0,
            cumulative_mineralized_mg: 0.0,
            ..Default::default()
        }
    }

    pub fn new_pnw_old_growth_baseline() -> Self {
        Self {
            // Massive standing Douglas-fir/Redcedar biomass + marine-derived 15N salmon subsidy!
            biomass_kcal: [580.0, 6420.0, 740.0, 260.0, 108.0],
            cumulative_gpp_kcal: 0.0,
            assimilation_kcal: [0.0; 5],
            respiration_kcal: [0.0; 5],
            detritus_pool_g: 245.0,
            mineral_nutrient_pool_mg: 164.0,
            cumulative_mineralized_mg: 0.0,
            ..Default::default()
        }
    }

    pub fn new_hydrothermal_vent_baseline() -> Self {
        Self {
            // 100% Chemosynthetic RuBisCO-II & rTCA sulfur/methane-oxidizing primary production!
            biomass_kcal: [620.0, 5180.0, 960.0, 340.0, 132.0],
            cumulative_gpp_kcal: 0.0,
            assimilation_kcal: [0.0; 5],
            respiration_kcal: [0.0; 5],
            detritus_pool_g: 230.0,
            mineral_nutrient_pool_mg: 215.0,
            cumulative_mineralized_mg: 0.0,
            ..Default::default()
        }
    }

    pub fn new_freshwater_lake_baseline() -> Self {
        Self {
            biomass_kcal: [440.0, 4680.0, 790.0, 295.0, 118.0],
            cumulative_gpp_kcal: 0.0,
            assimilation_kcal: [0.0; 5],
            respiration_kcal: [0.0; 5],
            detritus_pool_g: 168.0,
            mineral_nutrient_pool_mg: 86.0,
            cumulative_mineralized_mg: 0.0,
            ..Default::default()
        }
    }

    pub fn new_coral_reef_baseline() -> Self {
        Self {
            // Tightly recycled Darwin-Paradox Symbiodinium coral holobiont & high reef fish standing stock!
            biomass_kcal: [460.0, 4380.0, 910.0, 365.0, 148.0],
            cumulative_gpp_kcal: 0.0,
            assimilation_kcal: [0.0; 5],
            respiration_kcal: [0.0; 5],
            detritus_pool_g: 112.0,
            mineral_nutrient_pool_mg: 64.0,
            cumulative_mineralized_mg: 0.0,
            ..Default::default()
        }
    }

    pub fn transfer_efficiency_pct(&self, stratum: TrophicStratum) -> f32 {
        let idx = stratum.index();
        if idx <= 1 {
            return 100.0;
        }
        let prev = self.biomass_kcal[idx - 1].max(1e-3);
        ((self.biomass_kcal[idx] / prev) * 100.0).clamp(0.0, 100.0)
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
