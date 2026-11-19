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

use crate::trophic::BiomeKind;
use glam::Vec3;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StochasticEventKind {
    // Rainforest events
    TropicalSunshower,
    CanopyFruitMastDrop,
    AntTrailForagingWave,
    MonkeyCanopyTransit,
    PantherUnderbrushProwl,
    BoaAmbushAlert,
    CanopyWindGust,
    BioluminescentSporePulse,
    // Kelp Forest events
    CoastalUpwellingPulse,
    DriftKelpDetachment,
    CrabScentPlumeWave,
    SeaOtterUrchinDive,
    HarborSealKelpGlide,
    OctopusChromatophoreShift,
    DinoflagellateBiolumSwirl,
    // Coral Reef events
    LagoonTidalFlush,
    ParrotfishSandCloud,
    HermitCrabMucusPlume,
    SeaTurtleSurfaceBreath,
    BlacktipSharkPass,
    CoralGfpBiofluorescence,
    // Temperate Freshwater Lake events
    MayflySurfaceHatch,
    OspreyPlungeDive,
    CommonLoonSubmergeDive,
    MallardDabbleRipple,
    CrayfishLeafPackPlume,
    FireflyLoonNocturne,
    // Deep-Sea Hydrothermal Vent events
    BlackSmokerSuperPlume,
    GhostSharkAbyssalGlide,
    AtollaBiolumBurglarAlarm,
    YetiCrabBacterialDance,
    VulcanoctopusTubewormHunt,
    SulfideFissureFlocRelease,
    // PNW Old-Growth & Spawning Salmon Creek events
    CohoSalmonReddLeap,
    MycorrhizalNitrogenPulse,
    DipperUnderwaterWalk,
    BananaSlugSporeTrail,
    SpottedOwlDuskGlide,
    PacificDrizzleSunbeam,
    // Midwater Pelagic Rift Canyon events
    DielVerticalMigrationSurge,
    MarineSnowMucusFall,
    VampyroteuthisMucusCloud,
    BarreleyeDomeTargetLock,
    SpermWhaleSonarDive,
    ArchiteuthisJetEncounter,
    // NE Pacific Rocky Intertidal Tidepool events
    TidepoolHighSurfSurge,
    PisasterMusselForage,
    SculpinPoolToPoolLeap,
    OystercatcherLowTideProbe,
    AnemoneSubmergeBloom,
    HermitCrabWrackScavenge,
    // Cold-Seep Brine Pool & Methane Ice events
    MethaneHydrateBubblePlume,
    BrineShorelineInternalWave,
    CutthroatEelBrineSkim,
    IceWormClathrateGraze,
    LamellibrachiaRootUptake,
    RedCrabMusselPatrol,
}

impl StochasticEventKind {
    pub fn title(self) -> &'static str {
        match self {
            Self::TropicalSunshower => "Warm Tropical Sunshower",
            Self::CanopyFruitMastDrop => "Ripe Canopy Fig & Berry Cascade",
            Self::AntTrailForagingWave => "Leafcutter Ant Recruitment Column",
            Self::MonkeyCanopyTransit => "Squirrel Monkey Vine-Swing Transit",
            Self::PantherUnderbrushProwl => "Black Panther Underbrush Stalk",
            Self::BoaAmbushAlert => "Boa Constrictor Chemosensory Coiling",
            Self::CanopyWindGust => "Trade-Wind Canopy Gust & Leaf Fall",
            Self::BioluminescentSporePulse => "Nocturnal Mycena Bioluminescent Bloom",
            Self::CoastalUpwellingPulse => "Nutrient-Rich Coastal Upwelling Surge",
            Self::DriftKelpDetachment => "Wave-Detached Drift Kelp Blade Fall",
            Self::CrabScentPlumeWave => "Benthic Kelp Crab & Sea Star Scent Plume",
            Self::SeaOtterUrchinDive => "Sea Otter Urchin Dive & Chest-Stone Tap",
            Self::HarborSealKelpGlide => "Harbor Seal Kelp Cathedral Glide",
            Self::OctopusChromatophoreShift => "Giant Pacific Octopus Reef Emergence",
            Self::DinoflagellateBiolumSwirl => "Nocturnal Dinoflagellate Bioluminescence",
            Self::LagoonTidalFlush => "Clear Oceanic Lagoon Tidal Flush",
            Self::ParrotfishSandCloud => "Parrotfish Coral Grazing & Carbonate Sand Cloud",
            Self::HermitCrabMucusPlume => "Hermit Crab & Linckia Coral-Mucus Plume",
            Self::SeaTurtleSurfaceBreath => "Green Sea Turtle Surface Breath & Reef Glide",
            Self::BlacktipSharkPass => "Blacktip Reef Shark Pass & Chromis Coral Shelter",
            Self::CoralGfpBiofluorescence => "Nocturnal Acropora GFP Biofluorescence & Spawning",
            Self::MayflySurfaceHatch => "Emerging Mayfly Hatch & Rainbow Trout Surface Rise",
            Self::OspreyPlungeDive => "Osprey Aerial Stoop & Talons-First Lake Plunge",
            Self::CommonLoonSubmergeDive => "Common Loon Underwater Pursuit Dive",
            Self::MallardDabbleRipple => "Mallard Pair Tip-Up Pondweed Dabbling",
            Self::CrayfishLeafPackPlume => "Sunken Alder Leaf-Pack & Signal Crayfish Recruitment",
            Self::FireflyLoonNocturne => "Dusk Loon Tremolo & Shoreline Firefly Twinkle",
            Self::BlackSmokerSuperPlume => "360°C Black-Smoker Superheated Sulfide Surge",
            Self::GhostSharkAbyssalGlide => "Abyssal Ghost Shark Pectoral-Wing Flyby",
            Self::AtollaBiolumBurglarAlarm => "Atolla Medusae & Comb-Jelly Bioluminescent Cascade",
            Self::YetiCrabBacterialDance => "Kiwa Yeti Crab Chemosynthetic Claw-Farming Wave",
            Self::VulcanoctopusTubewormHunt => "Vulcanoctopus Crawl & Riftia Plume Retraction",
            Self::SulfideFissureFlocRelease => "Axial Basalt Crack & Sulfide Bacterial Floc Plume",
            Self::CohoSalmonReddLeap => "Spawning Coho Salmon Weir Leap & Redd Digging",
            Self::MycorrhizalNitrogenPulse => "Marine 15N Mycorrhizal Root-Network Transfer Pulse",
            Self::DipperUnderwaterWalk => "American Dipper Riffle Plunge & Underwater Boulder Walk",
            Self::BananaSlugSporeTrail => "Pacific Banana Slug Nurse-Log Chanterelle Spore Trail",
            Self::SpottedOwlDuskGlide => "Northern Spotted Owl Silent Cathedral Bough Glide",
            Self::PacificDrizzleSunbeam => "Warm Coast Range Drizzle & Nurse-Log Moss Flush",
            Self::DielVerticalMigrationSurge => "Deep Scattering Layer Lanternfish & Hatchetfish DVM Surge",
            Self::MarineSnowMucusFall => "Bathochordaeus Larvacean Mucus-House & Marine Snow Pulse",
            Self::VampyroteuthisMucusCloud => "Vampyroteuthis Pineapple Posture & Bioluminescent Mucus Cloud",
            Self::BarreleyeDomeTargetLock => "Macropinna Barreleye Tubular-Eye Rotation at Praya Curtain",
            Self::SpermWhaleSonarDive => "Physeter Sperm Whale Foraging Dive & Spermaceti Click Coda",
            Self::ArchiteuthisJetEncounter => "Architeuthis Giant Squid Chasm Mantle-Jet Maneuver",
            Self::TidepoolHighSurfSurge => "High-Tide Pacific Wave Surge & Postelsia Flex",
            Self::PisasterMusselForage => "Pisaster Ochre Sea Star Keystone Mussel Predation",
            Self::SculpinPoolToPoolLeap => "Tidepool Sculpin Ballistic Rock-Pool Leap",
            Self::OystercatcherLowTideProbe => "Black Oystercatcher Low-Tide Mussel Bench Probe",
            Self::AnemoneSubmergeBloom => "Anthopleura Giant Green Anemone Tentacle Bloom",
            Self::HermitCrabWrackScavenge => "Pagurus Hermit Crab & Chiton Drift-Kelp Wrack Feast",
            Self::MethaneHydrateBubblePlume => "Methane Clathrate ('Methane Ice') Outgassing Plume",
            Self::BrineShorelineInternalWave => "120-PSU Hypersaline Brine Lake Mirror Wave",
            Self::CutthroatEelBrineSkim => "Synaphobranchus Cutthroat Eel Anoxic Brine-Edge Skim",
            Self::IceWormClathrateGraze => "Pink Sirsoe Ice-Worm Methane Hydrate Bacterial Graze",
            Self::LamellibrachiaRootUptake => "250-Year Lamellibrachia Sulfide Root-Uptake Pulse",
            Self::RedCrabMusselPatrol => "Chaceon Deep-Sea Red Crab Methanotrophic Mussel Patrol",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcologicalLogEntry {
    pub day: u32,
    pub hour_of_day: f32,
    pub title: String,
    pub detail: String,
}

impl EcologicalLogEntry {
    pub fn formatted_clock(&self) -> String {
        let h = self.hour_of_day.floor() as u32 % 24;
        let m = ((self.hour_of_day.fract() * 60.0).floor() as u32).min(59);
        format!("Day {} {:02}:{:02}", self.day, h, m)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SoundCue {
    pub kind: SoundCueKind,
    pub world_pos: Vec3,
    pub gain: f32,
    pub pitch_mul: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundCueKind {
    MonkeyChatter,
    MacawSquawk,
    ToucanCroak,
    BoaHiss,
    PantherLowRumble,
    UnderbrushRustle,
    FruitDropThud,
    TreefrogCoqui,
    CicadaStridulation,
    RainShower,
    DistantThunder,
    OtterChestStoneClick,
    UnderwaterBubbleStream,
    OceanSwellSurge,
    KelpFrondSwish,
    ReefUrchinCrunch,
    DistantWhaleHydrophone,
    // Coral Reef bioacoustics
    SnappingShrimpCrackle,
    ParrotfishCoralCrunch,
    // Temperate Freshwater Lake bioacoustics
    LoonTremoloWail,
    MallardQuack,
    KingfisherRattle,
    BullfrogJugORum,
    WaterPlungeSplash,
    // Deep-Sea Hydrothermal Vent bioacoustics
    HydrothermalVentRumble,
    TectonicFissureCrack,
    AbyssalBiolumPing,
    // PNW Old-Growth & Salmon Creek bioacoustics
    SpottedOwlFourNoteHoot,
    VariedThrushFlute,
    GravelCreekRiffle,
    SalmonWeirSplash,
    // Midwater Pelagic Rift bioacoustics
    SpermWhaleClickCoda,
    DslBiolumShimmer,
    PelagicCanyonCurrent,
    ArchiteuthisMantleJet,
    // Tidepool & Cold-Seep Brine Pool bioacoustics
    SurfSurgeBreaker,
    BlackOystercatcherPipe,
    TidepoolBubbleTrickle,
    MethaneClathrateBubbles,
    BrinePoolDensityHum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StochasticEventScheduler {
    pub cooldown_sec: f32,
    pub active_rain_remaining_sec: f32,
    pub active_wind_remaining_sec: f32,
    pub wind_vector_xz: [f32; 2],
    pub log: Vec<EcologicalLogEntry>,
}

impl Default for StochasticEventScheduler {
    fn default() -> Self {
        Self {
            cooldown_sec: 14.0,
            active_rain_remaining_sec: 0.0,
            active_wind_remaining_sec: 0.0,
            wind_vector_xz: [0.35, 0.08],
            log: Vec::new(),
        }
    }
}

impl StochasticEventScheduler {
    pub fn record(
        &mut self,
        day: u32,
        hour: f32,
        title: impl Into<String>,
        detail: impl Into<String>,
    ) {
        self.log.push(EcologicalLogEntry {
            day,
            hour_of_day: hour,
            title: title.into(),
            detail: detail.into(),
        });
        if self.log.len() > 48 {
            let excess = self.log.len() - 48;
            self.log.drain(0..excess);
        }
    }

    pub fn poll_next_event<R: Rng>(
        &mut self,
        biome: BiomeKind,
        dt_sec: f32,
        hour_of_day: f32,
        is_night: bool,
        rng: &mut R,
    ) -> Option<StochasticEventKind> {
        if self.active_rain_remaining_sec > 0.0 {
            self.active_rain_remaining_sec = (self.active_rain_remaining_sec - dt_sec).max(0.0);
        }
        if self.active_wind_remaining_sec > 0.0 {
            self.active_wind_remaining_sec = (self.active_wind_remaining_sec - dt_sec).max(0.0);
        } else {
            self.wind_vector_xz[0] += (0.32 - self.wind_vector_xz[0]) * (dt_sec * 0.8).min(1.0);
            self.wind_vector_xz[1] += (0.06 - self.wind_vector_xz[1]) * (dt_sec * 0.8).min(1.0);
        }

        self.cooldown_sec -= dt_sec;
        if self.cooldown_sec > 0.0 {
            return None;
        }

        let p = (dt_sec / 12.0).clamp(0.0, 0.25);
        if rng.gen::<f32>() > p {
            return None;
        }

        self.cooldown_sec = rng.gen_range(16.0..28.0);
        let roll: f32 = rng.gen();

        match biome {
            BiomeKind::RockyIntertidalTidepool => {
                let chosen = if roll < 0.20 {
                    StochasticEventKind::TidepoolHighSurfSurge
                } else if roll < 0.40 {
                    StochasticEventKind::PisasterMusselForage
                } else if roll < 0.58 {
                    StochasticEventKind::SculpinPoolToPoolLeap
                } else if roll < 0.75 {
                    StochasticEventKind::OystercatcherLowTideProbe
                } else if roll < 0.88 {
                    StochasticEventKind::AnemoneSubmergeBloom
                } else {
                    StochasticEventKind::HermitCrabWrackScavenge
                };
                Some(chosen)
            }
            BiomeKind::ColdSeepBrinePool => {
                let chosen = if roll < 0.22 {
                    StochasticEventKind::MethaneHydrateBubblePlume
                } else if roll < 0.42 {
                    StochasticEventKind::BrineShorelineInternalWave
                } else if roll < 0.60 {
                    StochasticEventKind::CutthroatEelBrineSkim
                } else if roll < 0.76 {
                    StochasticEventKind::IceWormClathrateGraze
                } else if roll < 0.88 {
                    StochasticEventKind::LamellibrachiaRootUptake
                } else {
                    StochasticEventKind::RedCrabMusselPatrol
                };
                Some(chosen)
            }
            BiomeKind::MidwaterPelagicRift => {
                let chosen = if is_night {
                    if roll < 0.36 {
                        StochasticEventKind::DielVerticalMigrationSurge
                    } else if roll < 0.68 {
                        StochasticEventKind::VampyroteuthisMucusCloud
                    } else {
                        StochasticEventKind::ArchiteuthisJetEncounter
                    }
                } else if roll < 0.22 {
                    StochasticEventKind::SpermWhaleSonarDive
                } else if roll < 0.44 {
                    StochasticEventKind::BarreleyeDomeTargetLock
                } else if roll < 0.64 {
                    StochasticEventKind::MarineSnowMucusFall
                } else if roll < 0.82 {
                    StochasticEventKind::ArchiteuthisJetEncounter
                } else {
                    StochasticEventKind::VampyroteuthisMucusCloud
                };
                Some(chosen)
            }
            BiomeKind::PnwOldGrowthSalmonCreek => {
                let chosen = if is_night {
                    if roll < 0.42 {
                        StochasticEventKind::SpottedOwlDuskGlide
                    } else if roll < 0.78 {
                        StochasticEventKind::MycorrhizalNitrogenPulse
                    } else {
                        StochasticEventKind::BananaSlugSporeTrail
                    }
                } else if roll < 0.22 {
                    StochasticEventKind::CohoSalmonReddLeap
                } else if roll < 0.44 {
                    StochasticEventKind::DipperUnderwaterWalk
                } else if roll < 0.64 {
                    StochasticEventKind::MycorrhizalNitrogenPulse
                } else if roll < 0.82 {
                    StochasticEventKind::BananaSlugSporeTrail
                } else {
                    StochasticEventKind::PacificDrizzleSunbeam
                };
                Some(chosen)
            }
            BiomeKind::DeepSeaHydrothermalVent => {
                let chosen = if roll < 0.22 {
                    StochasticEventKind::BlackSmokerSuperPlume
                } else if roll < 0.42 {
                    StochasticEventKind::GhostSharkAbyssalGlide
                } else if roll < 0.60 {
                    StochasticEventKind::AtollaBiolumBurglarAlarm
                } else if roll < 0.76 {
                    StochasticEventKind::VulcanoctopusTubewormHunt
                } else if roll < 0.90 {
                    StochasticEventKind::SulfideFissureFlocRelease
                } else {
                    StochasticEventKind::YetiCrabBacterialDance
                };
                Some(chosen)
            }
            BiomeKind::TemperateFreshwaterLake => {
                let chosen = if is_night {
                    if roll < 0.45 {
                        StochasticEventKind::FireflyLoonNocturne
                    } else if roll < 0.75 {
                        StochasticEventKind::CommonLoonSubmergeDive
                    } else {
                        StochasticEventKind::CrayfishLeafPackPlume
                    }
                } else if roll < 0.22 {
                    StochasticEventKind::OspreyPlungeDive
                } else if roll < 0.44 {
                    StochasticEventKind::CommonLoonSubmergeDive
                } else if roll < 0.64 {
                    StochasticEventKind::MayflySurfaceHatch
                } else if roll < 0.82 {
                    StochasticEventKind::MallardDabbleRipple
                } else {
                    StochasticEventKind::CrayfishLeafPackPlume
                };
                Some(chosen)
            }
            BiomeKind::IndoPacificCoralReef => {
                let chosen = if is_night {
                    if roll < 0.38 {
                        StochasticEventKind::CoralGfpBiofluorescence
                    } else if roll < 0.68 {
                        StochasticEventKind::BlacktipSharkPass
                    } else {
                        StochasticEventKind::LagoonTidalFlush
                    }
                } else if roll < 0.22 {
                    StochasticEventKind::SeaTurtleSurfaceBreath
                } else if roll < 0.44 {
                    StochasticEventKind::BlacktipSharkPass
                } else if roll < 0.64 {
                    StochasticEventKind::ParrotfishSandCloud
                } else if roll < 0.82 {
                    StochasticEventKind::HermitCrabMucusPlume
                } else {
                    StochasticEventKind::LagoonTidalFlush
                };
                Some(chosen)
            }
            BiomeKind::PacificKelpForest => {
                let chosen = if is_night {
                    if roll < 0.34 {
                        StochasticEventKind::DinoflagellateBiolumSwirl
                    } else if roll < 0.62 {
                        StochasticEventKind::OctopusChromatophoreShift
                    } else if roll < 0.84 {
                        StochasticEventKind::HarborSealKelpGlide
                    } else {
                        StochasticEventKind::CoastalUpwellingPulse
                    }
                } else if roll < 0.22 {
                    StochasticEventKind::SeaOtterUrchinDive
                } else if roll < 0.42 {
                    StochasticEventKind::HarborSealKelpGlide
                } else if roll < 0.60 {
                    StochasticEventKind::DriftKelpDetachment
                } else if roll < 0.76 {
                    StochasticEventKind::CrabScentPlumeWave
                } else if roll < 0.90 {
                    StochasticEventKind::OctopusChromatophoreShift
                } else {
                    StochasticEventKind::CoastalUpwellingPulse
                };
                Some(chosen)
            }
            BiomeKind::NeotropicalRainforest => {
                let chosen = if is_night {
                    if roll < 0.28 {
                        StochasticEventKind::BioluminescentSporePulse
                    } else if roll < 0.56 {
                        StochasticEventKind::PantherUnderbrushProwl
                    } else if roll < 0.78 {
                        StochasticEventKind::BoaAmbushAlert
                    } else {
                        StochasticEventKind::TropicalSunshower
                    }
                } else if (13.0..=16.8).contains(&hour_of_day) && roll < 0.30 {
                    StochasticEventKind::TropicalSunshower
                } else if roll < 0.20 {
                    StochasticEventKind::CanopyFruitMastDrop
                } else if roll < 0.40 {
                    StochasticEventKind::AntTrailForagingWave
                } else if roll < 0.60 {
                    StochasticEventKind::MonkeyCanopyTransit
                } else if roll < 0.78 {
                    StochasticEventKind::PantherUnderbrushProwl
                } else if roll < 0.90 {
                    StochasticEventKind::CanopyWindGust
                } else {
                    StochasticEventKind::BoaAmbushAlert
                };
                Some(chosen)
            }
        }
    }
}
