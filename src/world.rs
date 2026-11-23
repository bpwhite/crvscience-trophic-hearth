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

use crate::events::{SoundCue, SoundCueKind, StochasticEventKind, StochasticEventScheduler};
use crate::organisms::{ArticulatedPose, BehaviorState, Organism, Species};
use crate::pheromone::ForestFloorField;
use crate::physics::{
    brine_pool_height_at, coral_reef_height_at, ground_height_at, lake_bed_height_at, ocean_swell_vx, pelagic_rift_height_at, pnw_creek_forest_height_at, reef_height_at, tidepool_bench_height_at, vent_field_height_at, BranchSegment, BRINE_POOL_SURFACE_Y, LAKE_SURFACE_Y, PNW_CREEK_SURFACE_Y, DetritusKind, FallingDetritus,
    KelpStalk, LianaVine, UnderbrushKind, UnderbrushPlant, GRAVITY_M_S2, KELP_SURFACE_Y,
};
use crate::trophic::{BiomeKind, DiurnalCycle, TrophicEnergyLedger, TrophicFocus};
use glam::{Vec2, Vec3};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntForagePatch {
    pub pos: Vec3,
    pub remaining_fragments: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanopyTree {
    pub id: usize,
    pub base_pos: Vec3,
    pub height_m: f32,
    pub trunk_radius_m: f32,
    pub crown_radius_m: f32,
    pub depth_tier: u8,
}

/// Multi-biome 3D ecosystem world supporting both Neotropical Rainforest and Pacific Giant Kelp Forest.
/// Retains the `RainforestWorld` struct name so existing save snapshots and callers remain 100% compatible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RainforestWorld {
    #[serde(default)]
    pub biome: BiomeKind,
    pub biome_name: String,
    pub sim_time_sec: f64,
    pub rng_seed: u64,
    pub focus: TrophicFocus,
    pub diurnal: DiurnalCycle,
    pub ledger: TrophicEnergyLedger,
    pub floor_field: ForestFloorField,
    pub trees: Vec<CanopyTree>,
    #[serde(default)]
    pub kelp_stalks: Vec<KelpStalk>,
    pub branches: Vec<BranchSegment>,
    pub lianas: Vec<LianaVine>,
    pub underbrush: Vec<UnderbrushPlant>,
    pub falling_detritus: Vec<FallingDetritus>,
    pub ant_nest_pos: Vec3,
    pub ant_patches: Vec<AntForagePatch>,
    pub organisms: Vec<Organism>,
    pub events: StochasticEventScheduler,
    #[serde(skip)]
    pub pending_sounds: Vec<SoundCue>,
    next_entity_id: u32,
    ambient_chorus_timer: f32,
}

impl Default for RainforestWorld {
    fn default() -> Self {
        Self::new_rocky_intertidal_tidepool(2026_1116)
    }
}

impl RainforestWorld {
    pub fn new_for_biome(biome: BiomeKind, seed: u64) -> Self {
        match biome {
            BiomeKind::NeotropicalRainforest => Self::new_neotropical_rainforest(seed),
            BiomeKind::PacificKelpForest => Self::new_pacific_kelp_forest(seed),
            BiomeKind::IndoPacificCoralReef => Self::new_indo_pacific_coral_reef(seed),
            BiomeKind::TemperateFreshwaterLake => Self::new_temperate_freshwater_lake(seed),
            BiomeKind::DeepSeaHydrothermalVent => Self::new_deep_sea_hydrothermal_vent(seed),
            BiomeKind::PnwOldGrowthSalmonCreek => Self::new_pnw_old_growth_salmon_creek(seed),
            BiomeKind::MidwaterPelagicRift => Self::new_midwater_pelagic_rift(seed),
            BiomeKind::RockyIntertidalTidepool => Self::new_rocky_intertidal_tidepool(seed),
            BiomeKind::ColdSeepBrinePool => Self::new_cold_seep_brine_pool(seed),
        }
    }

    /// Initializes Biome 8: NE Pacific Rocky Intertidal Tidepool & Wave-Cut Surge Bench!
    pub fn new_rocky_intertidal_tidepool(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut underbrush = Vec::new();
        for i in 0..38 {
            let ux = rng.gen_range(-12.5..10.5);
            let uz = rng.gen_range(-3.8..1.6);
            let uy = tidepool_bench_height_at(ux, uz);
            let kind = if i % 3 == 0 {
                UnderbrushKind::PostelsiaSeaPalm
            } else {
                UnderbrushKind::CorallineAlgaeTurf
            };
            underbrush.push(UnderbrushPlant {
                id: i,
                root_pos: Vec3::new(ux, uy, uz),
                scale: rng.gen_range(0.85..1.35),
                kind,
                bend_xz: [0.0, 0.0],
                bend_vel_xz: [0.0, 0.0],
                ripe_berries: 0,
                growth_progress: 0.85,
                vigor: 1.2,
            });
        }

        let wrack_origin = Vec3::new(-7.5, tidepool_bench_height_at(-7.5, 0.5), 0.5);
        let wrack_patch = Vec3::new(-1.2, tidepool_bench_height_at(-1.2, 0.5), 0.5);
        let mut floor_field = ForestFloorField::default();
        for s in 0..14 {
            let p = wrack_origin.lerp(wrack_patch, s as f32 / 13.0);
            floor_field.deposit_ant_pheromone(Vec2::new(p.x, p.z), 1.9);
        }

        let mut organisms = Vec::new();
        let mut next_id = 1_u32;
        let mut alloc_id = || {
            let id = next_id;
            next_id += 1;
            id
        };

        // L4: 5 Ochre Keystone Sea Stars (Pisaster ochraceus) + 2 Black Oystercatchers (Haematopus bachmani)
        for ps in 0..5 {
            let px = -8.5 + ps as f32 * 3.4;
            let pz = -0.3 + (ps % 2) as f32 * 0.7;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::OchreSeaStar,
                pos: Vec3::new(px, tidepool_bench_height_at(px, pz) + 0.06, pz),
                vel: Vec3::ZERO,
                scale: 1.05,
                energy_kcal: 8.5,
                max_energy_kcal: 11.0,
                age_hours: 240.0,
                state: BehaviorState::BenthicGrazerCreep { target_xz: [px + 0.6, pz] },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }
        for oy in 0..2 {
            let ox = -11.2 + oy as f32 * 3.2;
            let oz = -0.5 + oy as f32 * 0.8;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::BlackOystercatcher,
                pos: Vec3::new(ox, tidepool_bench_height_at(ox, oz) + 0.04, oz),
                vel: Vec3::ZERO,
                scale: 1.04,
                energy_kcal: 14.0,
                max_energy_kcal: 18.0,
                age_hours: 310.0,
                state: BehaviorState::FloorHerbivoreBrowse { target_x: ox + 1.2, target_z: oz, gnawing_timer: 1.5 },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 10.0,
            });
        }

        // L3: 8 Giant Green Surf Anemones (Anthopleura xanthogrammica) & 4 Leaping Tidepool Sculpins (Oligocottus maculosus)
        for an in 0..8 {
            let pools = [-6.8_f32, -0.4, 5.6];
            let ax = pools[an % 3] + ((an as f32 * 0.7).sin() * 1.1);
            let az = -0.2 + (an % 2) as f32 * 0.65;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::GiantGreenAnemone,
                pos: Vec3::new(ax, tidepool_bench_height_at(ax, az) + 0.05, az),
                vel: Vec3::ZERO,
                scale: 0.95 + (an % 3) as f32 * 0.16,
                energy_kcal: 4.5,
                max_energy_kcal: 6.0,
                age_hours: 190.0,
                state: BehaviorState::SessileMantleFilter { open_fraction: 1.0 },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }
        for sc in 0..4 {
            let pa = Vec3::new(-6.8 + sc as f32 * 0.4, tidepool_bench_height_at(-6.8, 0.2) + 0.12, 0.2);
            let pb = Vec3::new(-0.4 + sc as f32 * 0.4, tidepool_bench_height_at(-0.4, 0.2) + 0.12, 0.2);
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::TidepoolSculpin,
                pos: if sc % 2 == 0 { pa } else { pb },
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 3.0,
                max_energy_kcal: 4.2,
                age_hours: 85.0,
                state: BehaviorState::SculpinTidepoolHop { pool_a: pa, pool_b: pb, leap_timer: 0.0, at_b: sc % 2 == 1 },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        // L2: 10 California Mussel Bed Clusters (Mytilus californianus) & 6 Gooseneck Barnacle Clusters (Pollicipes polymerus)
        for mb in 0..10 {
            let mx = -10.5 + mb as f32 * 1.85;
            let mz = -0.8 + (mb % 3) as f32 * 0.7;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::CaliforniaMusselBed,
                pos: Vec3::new(mx, tidepool_bench_height_at(mx, mz) + 0.04, mz),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 5.0,
                max_energy_kcal: 6.8,
                age_hours: 260.0,
                state: BehaviorState::SessileMantleFilter { open_fraction: 0.9 },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }
        for gb in 0..6 {
            let gx = -9.2 + gb as f32 * 2.6;
            let gz = 0.65;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::GooseneckBarnacle,
                pos: Vec3::new(gx, tidepool_bench_height_at(gx, gz) + 0.04, gz),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 3.2,
                max_energy_kcal: 4.4,
                age_hours: 180.0,
                state: BehaviorState::SessileMantleFilter { open_fraction: 0.85 },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        // L0: 6 Tidepool Hermit Crabs (Pagurus samuelis) & 5 Black Katy Chitons (Katharina tunicata)
        for hc in 0..6 {
            let hx = -7.2 + hc as f32 * 2.2;
            let hz = 0.55;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::TidepoolHermitCrab,
                pos: Vec3::new(hx, tidepool_bench_height_at(hx, hz) + 0.04, hz),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 1.8,
                max_energy_kcal: 2.5,
                age_hours: 72.0,
                state: BehaviorState::BenthicGrazerCreep { target_xz: [hx + 0.7, hz] },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }
        for ch in 0..5 {
            let cx = -5.5 + ch as f32 * 2.5;
            let cz = -0.45;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::BlackKatyChiton,
                pos: Vec3::new(cx, tidepool_bench_height_at(cx, cz) + 0.03, cz),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 1.6,
                max_energy_kcal: 2.2,
                age_hours: 95.0,
                state: BehaviorState::BenthicGrazerCreep { target_xz: [cx + 0.3, cz] },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        let mut events = StochasticEventScheduler::default();
        events.record(
            1,
            8.4,
            "NE Pacific Rocky Intertidal Tidepool Active",
            "Semidiurnal tides flood and drain basalt rock pools; Pisaster ochre sea stars regulate Mytilus mussel beds alongside giant green anemones, leaping sculpins & black oystercatchers.",
        );

        Self {
            biome: BiomeKind::RockyIntertidalTidepool,
            biome_name: "NE Pacific Rocky Intertidal Tidepool & Surge Bench".to_string(),
            sim_time_sec: 0.0,
            rng_seed: seed,
            focus: TrophicFocus::AllStrata,
            diurnal: DiurnalCycle::default(),
            ledger: TrophicEnergyLedger::new_tidepool_baseline(),
            floor_field,
            trees: Vec::new(),
            kelp_stalks: Vec::new(),
            branches: Vec::new(),
            lianas: Vec::new(),
            underbrush,
            falling_detritus: Vec::new(),
            ant_nest_pos: wrack_origin,
            ant_patches: vec![AntForagePatch { pos: wrack_patch, remaining_fragments: 18 }],
            organisms,
            events,
            pending_sounds: Vec::new(),
            next_entity_id: next_id,
            ambient_chorus_timer: 3.5,
        }
    }

    /// Initializes Biome 9: 1,050m Cold-Seep Brine Pool & Methane Ice ("Underwater Lake")!
    pub fn new_cold_seep_brine_pool(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut underbrush = Vec::new();
        for i in 0..32 {
            let (ux, uz, kind) = if i < 12 {
                (rng.gen_range(-11.2..-5.6), rng.gen_range(-2.4..1.8), UnderbrushKind::MethaneClathrateOutcrop)
            } else {
                (rng.gen_range(-10.5..9.8), rng.gen_range(-4.5..1.8), UnderbrushKind::LamellibrachiaSeepBush)
            };
            let uy = brine_pool_height_at(ux, uz);
            underbrush.push(UnderbrushPlant {
                id: i,
                root_pos: Vec3::new(ux, uy, uz),
                scale: rng.gen_range(0.90..1.45),
                kind,
                bend_xz: [0.0, 0.0],
                bend_vel_xz: [0.0, 0.0],
                ripe_berries: 0,
                growth_progress: 0.9,
                vigor: 1.25,
            });
        }

        let seep_origin = Vec3::new(-8.6, brine_pool_height_at(-8.6, 0.2), 0.2);
        let seep_patch = Vec3::new(-4.5, brine_pool_height_at(-4.5, 0.4), 0.4);
        let mut floor_field = ForestFloorField::default();
        for s in 0..14 {
            let p = seep_origin.lerp(seep_patch, s as f32 / 13.0);
            floor_field.deposit_ant_pheromone(Vec2::new(p.x, p.z), 2.1);
        }

        let mut organisms = Vec::new();
        let mut next_id = 1_u32;
        let mut alloc_id = || {
            let id = next_id;
            next_id += 1;
            id
        };

        // L4: 3 Synaphobranchus Cutthroat Eels skimming the Brine Lake + 1 Seep Ghost Chimaera
        for ce in 0..3 {
            let cpos = Vec3::new(-1.0 + ce as f32 * 3.2, BRINE_POOL_SURFACE_Y + 0.35 + (ce % 2) as f32 * 0.25, -0.2 + ce as f32 * 0.3);
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::CutthroatBrineEel,
                pos: cpos,
                vel: Vec3::new(0.65, 0.0, 0.0),
                scale: 1.08,
                energy_kcal: 19.0,
                max_energy_kcal: 25.0,
                age_hours: 340.0,
                state: BehaviorState::CutthroatEelBrineCruiser { center: cpos, skim_timer: 0.0, phase: ce as f32 * 1.9 },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::SeepGhostChimaera,
            pos: Vec3::new(0.0, 5.4, -0.8),
            vel: Vec3::new(0.75, 0.0, 0.0),
            scale: 1.12,
            energy_kcal: 34.0,
            max_energy_kcal: 45.0,
            age_hours: 480.0,
            state: BehaviorState::GhostSharkPectoralFlight {
                waypoint: Vec3::new(10.5, 5.4, -0.8),
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 99.0,
        });

        // L3: 5 Deep-Sea Red Crabs (Chaceon quinquedens) & 2 Cold-Seep Hagfish
        for rc in 0..5 {
            let ang = (rc as f32 / 5.0) * PI * 1.4 - 0.5;
            let rx = 2.4 + ang.cos() * 6.3;
            let rz = -0.4 + ang.sin() * 3.2;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::DeepSeaRedCrab,
                pos: Vec3::new(rx, brine_pool_height_at(rx, rz) + 0.06, rz),
                vel: Vec3::ZERO,
                scale: 1.05,
                energy_kcal: 9.5,
                max_energy_kcal: 13.0,
                age_hours: 210.0,
                state: BehaviorState::BenthicGrazerCreep { target_xz: [rx - 0.6, rz] },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }
        for hf in 0..2 {
            let hx = -6.8 + hf as f32 * 2.2;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::ColdSeepHagfish,
                pos: Vec3::new(hx, 2.6 + hf as f32 * 0.4, 0.5),
                vel: Vec3::new(0.4, 0.0, 0.0),
                scale: 1.0,
                energy_kcal: 6.2,
                max_energy_kcal: 8.5,
                age_hours: 160.0,
                state: BehaviorState::CutthroatEelBrineCruiser { center: Vec3::new(hx, 2.6, 0.5), skim_timer: 0.0, phase: hf as f32 },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        // L2: 16 Methanotrophic Brine-Shoreline Mussels (Bathymodiolus childressi) & 10 Alvinocaris Seep Shrimp
        for bm in 0..16 {
            let a = (bm as f32 / 16.0) * 2.0 * PI;
            let mx = 2.4 + a.cos() * 6.0;
            let mz = -0.4 + a.sin() * 4.1;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::BrineMusselRing,
                pos: Vec3::new(mx, BRINE_POOL_SURFACE_Y + 0.06, mz),
                vel: Vec3::ZERO,
                scale: 1.05,
                energy_kcal: 4.8,
                max_energy_kcal: 6.5,
                age_hours: 320.0,
                state: BehaviorState::SessileMantleFilter { open_fraction: 0.95 },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }
        for sh in 0..10 {
            let sx = -9.2 + (sh % 5) as f32 * 0.95;
            let sy = 2.8 + (sh / 5) as f32 * 0.85;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::AlvinocarisSeepShrimp,
                pos: Vec3::new(sx, sy, 0.25),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 1.9,
                max_energy_kcal: 2.6,
                age_hours: 65.0,
                state: BehaviorState::BiolumPelagicPulse { drift_center: Vec3::new(sx, sy, 0.25), flash_timer: 0.0 },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        // L0: 8 Pink Methane Hydrate Ice Worms (Sirsoe methanicola) & 4 Seep Squat Lobsters
        for iw in 0..8 {
            let ix = -9.6 + (iw % 4) as f32 * 0.72;
            let iz = -0.4 + (iw / 4) as f32 * 0.85;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::MethaneIceWorm,
                pos: Vec3::new(ix, brine_pool_height_at(ix, iz) + 0.08, iz),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 1.2,
                max_energy_kcal: 1.8,
                age_hours: 58.0,
                state: BehaviorState::BenthicGrazerCreep { target_xz: [ix + 0.25, iz] },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }
        for sl in 0..4 {
            let lx = -5.4 + sl as f32 * 0.9;
            let lz = 0.75;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::SeepSquatLobster,
                pos: Vec3::new(lx, brine_pool_height_at(lx, lz) + 0.05, lz),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 2.1,
                max_energy_kcal: 3.0,
                age_hours: 88.0,
                state: BehaviorState::BenthicGrazerCreep { target_xz: [lx + 0.5, lz] },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        let mut events = StochasticEventScheduler::default();
        events.record(
            1,
            8.4,
            "1,050m Cold-Seep Brine Pool & Methane Ice Active",
            "120-PSU hypersaline underwater lake shimmers inside a Bathymodiolus mussel beach while pink ice worms graze bubbling golden methane hydrate mounds.",
        );

        Self {
            biome: BiomeKind::ColdSeepBrinePool,
            biome_name: "Cold-Seep Brine Pool & Methane Ice (1,050m Underwater Lake)".to_string(),
            sim_time_sec: 0.0,
            rng_seed: seed,
            focus: TrophicFocus::AllStrata,
            diurnal: DiurnalCycle::default(),
            ledger: TrophicEnergyLedger::new_brine_pool_baseline(),
            floor_field,
            trees: Vec::new(),
            kelp_stalks: Vec::new(),
            branches: Vec::new(),
            lianas: Vec::new(),
            underbrush,
            falling_detritus: Vec::new(),
            ant_nest_pos: seep_origin,
            ant_patches: vec![AntForagePatch { pos: seep_patch, remaining_fragments: 20 }],
            organisms,
            events,
            pending_sounds: Vec::new(),
            next_entity_id: next_id,
            ambient_chorus_timer: 3.5,
        }
    }

    /// Initializes the 200-1,500m Midwater Pelagic Rift Canyon (Faulted Escarpments, DVM Lanternfish/Hatchetfish, Barreleye, Praya Siphonophore, Vampyroteuthis, Sperm Whale & Architeuthis)!
    pub fn new_midwater_pelagic_rift(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        // Overhanging faulted basalt rift ledges jutting from left & right canyon walls!
        let branches = vec![
            BranchSegment {
                id: 0,
                tree_id: 0,
                start: Vec3::new(-13.8, 8.6, -0.6),
                end: Vec3::new(-6.2, 8.1, -0.3),
                radius_start: 0.78,
                radius_end: 0.36,
                is_perchable: false,
            },
            BranchSegment {
                id: 1,
                tree_id: 0,
                start: Vec3::new(-12.6, 4.4, 0.2),
                end: Vec3::new(-5.6, 4.0, 0.4),
                radius_start: 0.84,
                radius_end: 0.40,
                is_perchable: false,
            },
            BranchSegment {
                id: 2,
                tree_id: 0,
                start: Vec3::new(13.8, 6.8, -0.5),
                end: Vec3::new(7.4, 6.3, -0.2),
                radius_start: 0.76,
                radius_end: 0.38,
                is_perchable: false,
            },
        ];

        // 36 Cold-Water Bubblegum Octocorals (Paragorgia arborea) & Cloud Glass Sponges (Aphrocallistes vastus) on the rift walls!
        let mut underbrush = Vec::new();
        for i in 0..36 {
            let left_wall = i % 2 == 0;
            let wx = if left_wall {
                rng.gen_range(-14.2..-5.5)
            } else {
                rng.gen_range(7.5..14.4)
            };
            let wz = rng.gen_range(-5.2..1.6);
            let wy = pelagic_rift_height_at(wx, wz);
            let kind = if i % 3 == 0 {
                UnderbrushKind::CloudGlassSponge
            } else {
                UnderbrushKind::BubblegumOctocoral
            };
            underbrush.push(UnderbrushPlant {
                id: i,
                root_pos: Vec3::new(wx, wy, wz),
                scale: rng.gen_range(0.85..1.42),
                kind,
                bend_xz: [0.0, 0.0],
                bend_vel_xz: [0.0, 0.0],
                ripe_berries: 0,
                growth_progress: 0.82,
                vigor: 1.15,
            });
        }

        let snow_origin = Vec3::new(-3.2, 2.8, 0.5);
        let snow_eddy = Vec3::new(3.6, 2.4, 0.4);
        let mut floor_field = ForestFloorField::default();
        for s in 0..16 {
            let p = snow_origin.lerp(snow_eddy, s as f32 / 15.0);
            floor_field.deposit_ant_pheromone(Vec2::new(p.x, p.z), 2.0);
        }

        let mut organisms = Vec::new();
        let mut next_id = 1_u32;
        let mut alloc_id = || {
            let id = next_id;
            next_id += 1;
            id
        };

        // L4: Diving Bull Sperm Whale (Physeter macrocephalus) & Bathypelagic Giant Squid (Architeuthis dux)
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::DivingSpermWhale,
            pos: Vec3::new(-1.5, 10.6, -1.2),
            vel: Vec3::new(0.85, 0.0, 0.0),
            scale: 1.15,
            energy_kcal: 145.0,
            max_energy_kcal: 180.0,
            age_hours: 720.0,
            state: BehaviorState::SpermWhaleEcholocationDive {
                cruise_y: 10.6,
                dive_timer: 0.0,
                sonar_pulse: 0.0,
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 9.0,
        });

        let squid_home = Vec3::new(1.4, 3.1, -0.6);
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::ArchiteuthisGiantSquid,
            pos: squid_home,
            vel: Vec3::new(-0.55, 0.0, 0.0),
            scale: 1.12,
            energy_kcal: 68.0,
            max_energy_kcal: 90.0,
            age_hours: 410.0,
            state: BehaviorState::ArchiteuthisChasmProwl {
                home_pos: squid_home,
                jet_timer: 0.0,
                phase: 0.0,
            },
            pose: ArticulatedPose { body_yaw: PI, ..Default::default() },
            vocal_cooldown_sec: 18.0,
        });

        // L3: Pacific Barreleye (Macropinna microstoma), Giant Praya Siphonophore & Cock-Eyed Jewel Squid
        let barreleye_home = Vec3::new(-2.1, 6.8, 0.55);
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::PacificBarreleye,
            pos: barreleye_home,
            vel: Vec3::ZERO,
            scale: 1.05,
            energy_kcal: 8.5,
            max_energy_kcal: 11.0,
            age_hours: 140.0,
            state: BehaviorState::BarreleyeTubularScan {
                hover_anchor: barreleye_home,
                lock_timer: 0.0,
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 99.0,
        });

        let praya_anchor = Vec3::new(2.2, 8.2, -0.2);
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::PrayaGiantSiphonophore,
            pos: praya_anchor,
            vel: Vec3::ZERO,
            scale: 1.10,
            energy_kcal: 14.0,
            max_energy_kcal: 18.0,
            age_hours: 220.0,
            state: BehaviorState::BiolumPelagicPulse {
                drift_center: praya_anchor,
                flash_timer: 0.0,
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 99.0,
        });

        for jq in 0..2 {
            let jpos = Vec3::new(-0.8 + jq as f32 * 4.2, 5.4 + jq as f32 * 0.9, 0.35);
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::CockEyedJewelSquid,
                pos: jpos,
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 7.2,
                max_energy_kcal: 9.5,
                age_hours: 115.0,
                state: BehaviorState::BiolumPelagicPulse {
                    drift_center: jpos,
                    flash_timer: 0.0,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        // L2: Diel Vertical Migration (DVM) Deep Scattering Layer: 16 Myctophid Lanternfish, 8 Silver Hatchetfish & 12 Krill!
        for mf in 0..16 {
            let hx = -4.6 + (mf % 8) as f32 * 1.45;
            let hz = -1.2 + (mf / 8) as f32 * 1.35;
            let day_y = 4.2 + (mf % 4) as f32 * 0.45;
            let night_y = 9.6 + (mf % 4) as f32 * 0.50;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::MyctophidLanternfish,
                pos: Vec3::new(hx, day_y, hz),
                vel: Vec3::new(0.45, 0.0, 0.0),
                scale: 0.95 + (mf % 3) as f32 * 0.12,
                energy_kcal: 2.8,
                max_energy_kcal: 4.0,
                age_hours: 68.0,
                state: BehaviorState::DvmVerticalMigrator {
                    day_y,
                    night_y,
                    home_xz: [hx, hz],
                    phase: mf as f32 * 0.7,
                    dvm_boost: 0.0,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        for hf in 0..8 {
            let hx = -3.4 + hf as f32 * 1.25;
            let hz = 0.25 + (hf % 2) as f32 * 0.55;
            let day_y = 5.2 + (hf % 3) as f32 * 0.40;
            let night_y = 8.9 + (hf % 3) as f32 * 0.42;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::SilverHatchetfish,
                pos: Vec3::new(hx, day_y, hz),
                vel: Vec3::new(-0.35, 0.0, 0.0),
                scale: 1.0,
                energy_kcal: 2.5,
                max_energy_kcal: 3.6,
                age_hours: 72.0,
                state: BehaviorState::DvmVerticalMigrator {
                    day_y,
                    night_y,
                    home_xz: [hx, hz],
                    phase: hf as f32 * 1.1,
                    dvm_boost: 0.0,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        for kr in 0..12 {
            let kx = -2.8 + (kr % 6) as f32 * 1.15;
            let kz = 0.65 + (kr / 6) as f32 * 0.45;
            let day_y = 3.4 + (kr % 3) as f32 * 0.35;
            let night_y = 10.4 + (kr % 3) as f32 * 0.38;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::PelagicEuphausiidKrill,
                pos: Vec3::new(kx, day_y, kz),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 1.1,
                max_energy_kcal: 1.6,
                age_hours: 42.0,
                state: BehaviorState::DvmVerticalMigrator {
                    day_y,
                    night_y,
                    home_xz: [kx, kz],
                    phase: kr as f32 * 0.9,
                    dvm_boost: 0.0,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        // L0: 4 Giant Larvacean Mucus Houses (Bathochordaeus charon) & 2 Vampire Squid (Vampyroteuthis infernalis)
        for lv in 0..4 {
            let lpos = Vec3::new(-3.4 + lv as f32 * 2.6, 2.8 + (lv % 2) as f32 * 1.6, 0.25);
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::GiantLarvaceanHouse,
                pos: lpos,
                vel: Vec3::ZERO,
                scale: 1.08,
                energy_kcal: 2.2,
                max_energy_kcal: 3.0,
                age_hours: 54.0,
                state: BehaviorState::BiolumPelagicPulse {
                    drift_center: lpos,
                    flash_timer: 0.0,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        for vs in 0..2 {
            let vpos = Vec3::new(-2.4 + vs as f32 * 5.8, 2.1 + vs as f32 * 0.8, 0.55);
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::VampireSquid,
                pos: vpos,
                vel: Vec3::ZERO,
                scale: 1.05,
                energy_kcal: 6.4,
                max_energy_kcal: 8.5,
                age_hours: 185.0,
                state: BehaviorState::BiolumPelagicPulse {
                    drift_center: vpos,
                    flash_timer: 0.0,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        let mut events = StochasticEventScheduler::default();
        events.record(
            1,
            8.8,
            "Midwater Pelagic Rift Canyon (200-1,500m) Active",
            "475nm sapphire twilight fades down faulted basalt rift walls into bathypelagic dark; lanternfish & hatchetfish counter-illuminate, barreleye scans Praya siphonophores, and a sperm whale echolocates giant squid.",
        );

        Self {
            biome: BiomeKind::MidwaterPelagicRift,
            biome_name: "Midwater Pelagic Rift Canyon (200-1,500m Twilight Drop-Off)".to_string(),
            sim_time_sec: 0.0,
            rng_seed: seed,
            focus: TrophicFocus::AllStrata,
            diurnal: DiurnalCycle::default(),
            ledger: TrophicEnergyLedger::new_pelagic_rift_baseline(),
            floor_field,
            trees: Vec::new(),
            kelp_stalks: Vec::new(),
            branches,
            lianas: Vec::new(),
            underbrush,
            falling_detritus: Vec::new(),
            ant_nest_pos: snow_origin,
            ant_patches: vec![AntForagePatch {
                pos: snow_eddy,
                remaining_fragments: 22,
            }],
            organisms,
            events,
            pending_sounds: Vec::new(),
            next_entity_id: next_id,
            ambient_chorus_timer: 3.5,
        }
    }

    /// Initializes the PNW Old-Growth Douglas-Fir / Nurse-Log / Mycorrhizal Root Cutaway & Spawning Coho Salmon Creek!
    pub fn new_pnw_old_growth_salmon_creek(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        // Colossal Old-Growth Douglas-Fir & Western Redcedar Trunks on the left/center forest terrace!
        let tree_specs: &[(f32, f32, f32, f32, f32, u8)] = &[
            (-12.2, -1.2, 16.5, 1.25, 5.2, 1),
            (-4.2, -1.8, 17.2, 1.38, 5.6, 1),
            (0.6, -2.4, 16.0, 1.12, 4.8, 2),
            (-8.8, -6.5, 17.8, 1.42, 6.0, 2),
            (13.2, -2.2, 15.8, 1.05, 4.8, 2),
            (-13.8, -10.5, 18.2, 1.45, 6.2, 3),
            (-2.5, -11.0, 18.5, 1.50, 6.4, 3),
            (8.5, -10.2, 17.4, 1.28, 5.6, 3),
        ];
        let trees: Vec<CanopyTree> = tree_specs
            .iter()
            .enumerate()
            .map(|(id, &(x, z, h, r, cr, tier))| CanopyTree {
                id,
                base_pos: Vec3::new(x, pnw_creek_forest_height_at(x, z), z),
                height_m: h,
                trunk_radius_m: r,
                crown_radius_m: cr,
                depth_tier: tier,
            })
            .collect();

        // Fallen Mossy Redcedar Nurse Log (`id = 0`, `x = -9.6..-1.8`) + High Spotted Owl Douglas-Fir Boughs!
        let branches = vec![
            BranchSegment {
                id: 0,
                tree_id: 0,
                start: Vec3::new(-9.8, 3.55, 0.25),
                end: Vec3::new(-1.6, 3.45, 0.15), // Giant Fallen Mossy Nurse Log!
                radius_start: 0.68,
                radius_end: 0.56,
                is_perchable: true,
            },
            BranchSegment {
                id: 1,
                tree_id: 0,
                start: Vec3::new(-12.0, 11.4, -0.8),
                end: Vec3::new(-5.2, 11.1, -0.6), // Spotted Owl West Canopy Bough
                radius_start: 0.36,
                radius_end: 0.22,
                is_perchable: true,
            },
            BranchSegment {
                id: 2,
                tree_id: 1,
                start: Vec3::new(-4.0, 10.8, -1.2),
                end: Vec3::new(2.8, 10.4, -0.8), // Spotted Owl East Creek-Overlook Bough
                radius_start: 0.38,
                radius_end: 0.22,
                is_perchable: true,
            },
        ];

        // 48 Western Sword Ferns, Nurse-Log Licorice Ferns & Red Huckleberry Shrubs!
        let mut underbrush = Vec::new();
        for i in 0..48 {
            let on_nurse_log = i < 12;
            let (x, y, z) = if on_nurse_log {
                let f = (i as f32 + 0.5) / 12.0;
                let p = branches[0].point_at(f);
                (p.x, p.y + 0.55, p.z)
            } else {
                let fx = if i % 5 == 0 {
                    rng.gen_range(11.6..14.2)
                } else {
                    rng.gen_range(-13.8..1.2)
                };
                let fz = rng.gen_range(-5.5..1.6);
                (fx, pnw_creek_forest_height_at(fx, fz), fz)
            };
            let kind = if on_nurse_log {
                if i % 2 == 0 { UnderbrushKind::LicoriceFernNurseMoss } else { UnderbrushKind::RedHuckleberryBush }
            } else if i % 3 == 0 {
                UnderbrushKind::RedHuckleberryBush
            } else {
                UnderbrushKind::SwordFernClump
            };
            underbrush.push(UnderbrushPlant {
                id: i,
                root_pos: Vec3::new(x, y, z),
                scale: rng.gen_range(0.88..1.38),
                kind,
                bend_xz: [0.0, 0.0],
                bend_vel_xz: [0.0, 0.0],
                ripe_berries: if kind == UnderbrushKind::RedHuckleberryBush { 6 } else { 0 },
                growth_progress: 0.75,
                vigor: 1.20,
            });
        }

        let slug_home = Vec3::new(-9.2, pnw_creek_forest_height_at(-9.2, 0.95), 0.95);
        let chanterelle_patch = Vec3::new(-1.4, pnw_creek_forest_height_at(-1.4, 0.85), 0.85);
        let mut floor_field = ForestFloorField::default();
        for s in 0..18 {
            let f = s as f32 / 17.0;
            let p = slug_home.lerp(chanterelle_patch, f);
            floor_field.deposit_ant_pheromone(Vec2::new(p.x, p.z), 2.1);
        }

        let mut organisms = Vec::new();
        let mut next_id = 1_u32;
        let mut alloc_id = || {
            let id = next_id;
            next_id += 1;
            id
        };

        // L4: Northern Spotted Owl (Strix occidentalis caurina) & Salmon-Fishing Black Bear (Ursus americanus)
        let owl_a = branches[1].point_at(0.75) + Vec3::new(0.0, 0.32, 0.0);
        let owl_b = branches[2].point_at(0.72) + Vec3::new(0.0, 0.32, 0.0);
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::NorthernSpottedOwl,
            pos: owl_a,
            vel: Vec3::ZERO,
            scale: 1.06,
            energy_kcal: 18.0,
            max_energy_kcal: 24.0,
            age_hours: 320.0,
            state: BehaviorState::SpottedOwlCanopyPerchAndGlide {
                perch_a: owl_a,
                perch_b: owl_b,
                glide_timer: 0.0,
                at_b: false,
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 11.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::AmericanBlackBear,
            pos: Vec3::new(10.8, pnw_creek_forest_height_at(10.8, -0.4), -0.4),
            vel: Vec3::ZERO,
            scale: 1.10,
            energy_kcal: 95.0,
            max_energy_kcal: 125.0,
            age_hours: 540.0,
            state: BehaviorState::FloorHerbivoreBrowse {
                target_x: 9.8,
                target_z: -0.3,
                gnawing_timer: 2.0,
            },
            pose: ArticulatedPose { body_yaw: PI, ..Default::default() },
            vocal_cooldown_sec: 22.0,
        });

        // L3: Underwater-Walking American Dipper, Pacific Giant Salamander & Varied Thrush
        let dipper_rock = Vec3::new(4.8, PNW_CREEK_SURFACE_Y + 0.18, 0.35);
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::AmericanDipper,
            pos: dipper_rock,
            vel: Vec3::ZERO,
            scale: 1.0,
            energy_kcal: 4.5,
            max_energy_kcal: 6.0,
            age_hours: 92.0,
            state: BehaviorState::DipperRiffleSubmergeWalk {
                rock_perch: dipper_rock,
                walk_timer: 0.0,
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 12.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::PacificGiantSalamander,
            pos: Vec3::new(3.6, pnw_creek_forest_height_at(3.6, 0.65) + 0.05, 0.65),
            vel: Vec3::ZERO,
            scale: 1.02,
            energy_kcal: 5.2,
            max_energy_kcal: 7.0,
            age_hours: 150.0,
            state: BehaviorState::BenthicGrazerCreep {
                target_xz: [5.4, 0.65],
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 99.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::VariedThrush,
            pos: branches[0].point_at(0.35) + Vec3::new(0.0, 0.68, 0.0),
            vel: Vec3::ZERO,
            scale: 1.0,
            energy_kcal: 4.0,
            max_energy_kcal: 5.5,
            age_hours: 84.0,
            state: BehaviorState::AvianPerchedForage {
                branch_id: 0,
                branch_t: 0.35,
                remaining_sec: 6.0,
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 9.0,
        });

        // L2: 4 Spawning Coho Salmon (Oncorhynchus kisutch) in the gravel creek + 2 Douglas's Squirrels on Nurse Log!
        for c_i in 0..4 {
            let rx = 3.8 + c_i as f32 * 1.65;
            let rz = -0.25 + (c_i % 2) as f32 * 0.55;
            let redd = Vec3::new(rx, 1.35 + (c_i % 2) as f32 * 0.35, rz);
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::SpawningCohoSalmon,
                pos: redd,
                vel: Vec3::new(-0.45, 0.0, 0.0),
                scale: if c_i % 2 == 0 { 1.08 } else { 0.96 },
                energy_kcal: 16.0,
                max_energy_kcal: 22.0,
                age_hours: 280.0,
                state: BehaviorState::CohoUpstreamReddAndLeap {
                    redd_pos: redd,
                    leap_timer: 0.0,
                    phase: c_i as f32 * 1.5,
                },
                pose: ArticulatedPose { body_yaw: PI, ..Default::default() },
                vocal_cooldown_sec: 25.0,
            });
        }

        for sq_i in 0..2 {
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::DouglasSquirrel,
                pos: branches[0].point_at(0.2 + sq_i as f32 * 0.5) + Vec3::new(0.0, 0.65, 0.0),
                vel: Vec3::ZERO,
                scale: 0.95,
                energy_kcal: 5.5,
                max_energy_kcal: 7.5,
                age_hours: 96.0,
                state: BehaviorState::MonkeyBranchTraverse {
                    branch_id: 0,
                    branch_t: 0.2 + sq_i as f32 * 0.5,
                    dir: if sq_i == 0 { 1.0 } else { -1.0 },
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 14.0,
            });
        }

        // L0: 6 Pacific Banana Slugs, 8 Golden Chanterelle Fruiting Clusters & 5 Creek Caddisflies!
        for sl_i in 0..6 {
            let f = (sl_i as f32 + 0.5) / 6.0;
            let p = slug_home.lerp(chanterelle_patch, f);
            let ret = sl_i % 2 == 0;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::PacificBananaSlug,
                pos: Vec3::new(p.x, pnw_creek_forest_height_at(p.x, p.z) + 0.04, p.z),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 1.8,
                max_energy_kcal: 2.5,
                age_hours: 80.0,
                state: if ret {
                    BehaviorState::AntReturningWithLeaf { nest_pos: slug_home }
                } else {
                    BehaviorState::AntOutwardTrail { patch_pos: chanterelle_patch, wandering: 0.08 }
                },
                pose: ArticulatedPose { carrying_item: ret, ..Default::default() },
                vocal_cooldown_sec: 99.0,
            });
        }

        for ch_i in 0..8 {
            let cx = -11.5 + ch_i as f32 * 1.65;
            let cz = 0.55 + (ch_i % 2) as f32 * 0.45;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::GoldenChanterelle,
                pos: Vec3::new(cx, pnw_creek_forest_height_at(cx, cz), cz),
                vel: Vec3::ZERO,
                scale: 0.95 + (ch_i % 3) as f32 * 0.18,
                energy_kcal: 1.5,
                max_energy_kcal: 2.0,
                age_hours: 60.0,
                state: BehaviorState::FungalFruiting {
                    maturity: 0.92,
                    spore_release_timer: 8.0,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        for cd_i in 0..5 {
            let cx = 3.4 + cd_i as f32 * 1.4;
            let cz = 0.75;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::CasedCaddisflyLarva,
                pos: Vec3::new(cx, pnw_creek_forest_height_at(cx, cz) + 0.03, cz),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 0.6,
                max_energy_kcal: 0.9,
                age_hours: 40.0,
                state: BehaviorState::BenthicGrazerCreep {
                    target_xz: [cx + 0.4, cz],
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        let mut events = StochasticEventScheduler::default();
        events.record(
            1,
            8.8,
            "PNW Old-Growth & Spawning Salmon Creek Active",
            "Coho salmon spawn in the gravel creek while an American dipper walks underwater; mycorrhizal hyphae pulse 15N into Douglas-fir roots beneath nurse logs, banana slugs & spotted owls.",
        );

        Self {
            biome: BiomeKind::PnwOldGrowthSalmonCreek,
            biome_name: "PNW Old-Growth & Spawning Salmon Creek (Side Diorama)".to_string(),
            sim_time_sec: 0.0,
            rng_seed: seed,
            focus: TrophicFocus::AllStrata,
            diurnal: DiurnalCycle::default(),
            ledger: TrophicEnergyLedger::new_pnw_old_growth_baseline(),
            floor_field,
            trees,
            kelp_stalks: Vec::new(),
            branches,
            lianas: Vec::new(),
            underbrush,
            falling_detritus: Vec::new(),
            ant_nest_pos: slug_home,
            ant_patches: vec![AntForagePatch {
                pos: chanterelle_patch,
                remaining_fragments: 20,
            }],
            organisms,
            events,
            pending_sounds: Vec::new(),
            next_entity_id: next_id,
            ambient_chorus_timer: 3.5,
        }
    }

    /// Initializes the 2,600m Deep-Sea Hydrothermal Vent ("Black Smoker") chemosynthetic ecosystem.
    pub fn new_deep_sea_hydrothermal_vent(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut underbrush = Vec::new();

        // 85 Riftia Tubeworm Thickets, Beggiatoa Sulfur Mats, Pagoda Sulfide Flanges & Vent Mussel Beds!
        for i in 0..85 {
            let x = -13.2 + (i as f32 * 0.314) + rng.gen_range(-0.18..0.18);
            let z = match i % 4 {
                0 => rng.gen_range(0.35..1.55),
                1 => rng.gen_range(-0.9..0.35),
                2 => rng.gen_range(-2.6..-0.9),
                _ => rng.gen_range(-6.2..-2.6),
            };
            let kind = match i % 5 {
                0 | 1 => UnderbrushKind::RiftiaTubewormCluster,
                2 => UnderbrushKind::BeggiatoaSulfurMat,
                3 => UnderbrushKind::VentMusselBed,
                _ => UnderbrushKind::SulfidePagodaFlange,
            };
            let climb = if kind == UnderbrushKind::SulfidePagodaFlange {
                rng.gen_range(1.2..4.4)
            } else {
                0.0
            };
            underbrush.push(UnderbrushPlant {
                id: i,
                root_pos: Vec3::new(x, vent_field_height_at(x, z) + climb, z),
                scale: rng.gen_range(0.92..1.48),
                kind,
                bend_xz: [0.0, 0.0],
                bend_vel_xz: [0.0, 0.0],
                ripe_berries: 0,
                growth_progress: 0.85,
                vigor: 1.22,
            });
        }

        // 5 Towering Polymetallic Sulfide Black-Smoker & Beehive Chimneys!
        let branches = vec![
            BranchSegment {
                id: 0,
                tree_id: 0,
                start: Vec3::new(-0.6, 0.4, -1.2),
                end: Vec3::new(-0.4, 8.6, -1.1), // Central "Inferno" Black-Smoker Spire!
                radius_start: 1.35,
                radius_end: 0.52,
                is_perchable: false,
            },
            BranchSegment {
                id: 1,
                tree_id: 0,
                start: Vec3::new(-6.8, 0.3, -1.5),
                end: Vec3::new(-6.5, 6.8, -1.4), // West Twin Sulfide Smoker
                radius_start: 1.10,
                radius_end: 0.44,
                is_perchable: false,
            },
            BranchSegment {
                id: 2,
                tree_id: 0,
                start: Vec3::new(5.4, 0.35, -1.4),
                end: Vec3::new(5.2, 7.4, -1.25), // East Beehive Flange Smoker
                radius_start: 1.22,
                radius_end: 0.48,
                is_perchable: false,
            },
            BranchSegment {
                id: 3,
                tree_id: 0,
                start: Vec3::new(-9.8, 0.25, -0.9),
                end: Vec3::new(-9.6, 4.4, -0.85), // Anhydrite White-Smoker Pinnacle
                radius_start: 0.72,
                radius_end: 0.30,
                is_perchable: false,
            },
            BranchSegment {
                id: 4,
                tree_id: 0,
                start: Vec3::new(9.6, 0.3, -1.8),
                end: Vec3::new(9.4, 5.6, -1.7),
                radius_start: 0.92,
                radius_end: 0.38,
                is_perchable: false,
            },
        ];

        let munidopsis_home = Vec3::new(-8.2, vent_field_height_at(-8.2, 0.95), 0.95);
        let sulfide_fissure = Vec3::new(2.0, vent_field_height_at(2.0, 0.85), 0.85);
        let mut floor_field = ForestFloorField::default();
        for s in 0..20 {
            let f = s as f32 / 19.0;
            let p = munidopsis_home.lerp(sulfide_fissure, f);
            floor_field.deposit_ant_pheromone(Vec2::new(p.x, p.z), 2.3);
        }

        let mut organisms = Vec::new();
        let mut next_id = 1_u32;
        let mut alloc_id = || {
            let id = next_id;
            next_id += 1;
            id
        };

        // L4: Abyssal Ghost Shark (Hydrolagus), Bluntnose Sixgill Shark & Vulcanoctopus
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::AbyssalGhostShark,
            pos: Vec3::new(-5.5, 6.8, 0.15),
            vel: Vec3::new(1.15, 0.0, 0.0),
            scale: 1.08,
            energy_kcal: 22.0,
            max_energy_kcal: 28.0,
            age_hours: 310.0,
            state: BehaviorState::GhostSharkPectoralFlight {
                waypoint: Vec3::new(6.8, 6.4, 0.25),
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 15.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::BluntnoseSixgillShark,
            pos: Vec3::new(7.5, 11.2, -1.8),
            vel: Vec3::new(-1.25, 0.0, 0.0),
            scale: 1.22,
            energy_kcal: 85.0,
            max_energy_kcal: 110.0,
            age_hours: 620.0,
            state: BehaviorState::SharkReefCruise {
                waypoint: Vec3::new(-8.5, 10.8, -1.6),
            },
            pose: ArticulatedPose { body_yaw: PI, ..Default::default() },
            vocal_cooldown_sec: 24.0,
        });

        let octo_home = Vec3::new(-1.8, vent_field_height_at(-1.8, 0.25) + 0.45, 0.25);
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::VulcanoctopusVent,
            pos: octo_home,
            vel: Vec3::ZERO,
            scale: 0.95,
            energy_kcal: 14.0,
            max_energy_kcal: 18.0,
            age_hours: 180.0,
            state: BehaviorState::OctopusDenCamouflage {
                den_pos: octo_home,
                remaining_sec: 5.5,
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 18.0,
        });

        // L3: 2 Vent Zoarcid Eelpouts, 2 Bioluminescent Atolla Crown Jellies & 4 Iridescent Comb Jellies
        for e_i in 0..2 {
            let ex = -3.2 + e_i as f32 * 6.8;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::ZoarcidVentEelpout,
                pos: Vec3::new(ex, 2.2, 0.1),
                vel: Vec3::new(0.45, 0.0, 0.0),
                scale: 1.0,
                energy_kcal: 6.5,
                max_energy_kcal: 8.5,
                age_hours: 140.0,
                state: BehaviorState::ReefFishForage {
                    waypoint: Vec3::new(-ex, 2.6, 0.2),
                    pause_sec: 0.0,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 30.0,
            });
        }

        for a_i in 0..2 {
            let center = Vec3::new(-4.8 + a_i as f32 * 9.2, 9.4 + a_i as f32 * 1.1, -0.5);
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::AtollaCrownJelly,
                pos: center,
                vel: Vec3::ZERO,
                scale: 1.05,
                energy_kcal: 4.8,
                max_energy_kcal: 6.5,
                age_hours: 95.0,
                state: BehaviorState::BiolumPelagicPulse {
                    drift_center: center,
                    flash_timer: if a_i == 0 { 2.5 } else { 0.0 },
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 16.0,
            });
        }

        for c_i in 0..4 {
            let center = Vec3::new(-8.0 + c_i as f32 * 5.4, 5.6 + (c_i % 2) as f32 * 2.8, -0.2 + (c_i % 2) as f32 * 0.6);
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::BiolumCombJelly,
                pos: center,
                vel: Vec3::ZERO,
                scale: 0.98,
                energy_kcal: 2.6,
                max_energy_kcal: 3.5,
                age_hours: 65.0,
                state: BehaviorState::BiolumPelagicPulse {
                    drift_center: center,
                    flash_timer: 0.0,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        // L2: 6 Hairy-Clawed Kiwa Yeti Crabs, 16 Rimicaris Chimney Shrimp & 4 Iron Scaly-Foot Snails!
        for y_i in 0..6 {
            let yx = -6.2 + y_i as f32 * 2.45;
            let yz = 0.25 + (y_i % 2) as f32 * 0.35;
            let hp = Vec3::new(yx, vent_field_height_at(yx, yz) + 0.15, yz);
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::KiwaYetiCrab,
                pos: hp,
                vel: Vec3::ZERO,
                scale: 1.02,
                energy_kcal: 4.2,
                max_energy_kcal: 5.5,
                age_hours: 130.0,
                state: BehaviorState::YetiCrabBacterialWave {
                    home_pos: hp,
                    wave_phase: y_i as f32 * 1.1,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        for s_i in 0..16 {
            let chimney = if s_i < 10 {
                Vec3::new(-0.5, 4.8 + (s_i % 5) as f32 * 0.55, -0.7)
            } else {
                Vec3::new(5.3, 4.2 + (s_i % 4) as f32 * 0.55, -0.9)
            };
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::RimicarisVentShrimp,
                pos: chimney,
                vel: Vec3::ZERO,
                scale: 0.95,
                energy_kcal: 1.1,
                max_energy_kcal: 1.5,
                age_hours: 52.0,
                state: BehaviorState::ChimneyShrimpSwarm {
                    chimney_pos: chimney,
                    swirl_phase: s_i as f32 * 0.85,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        for sn_i in 0..4 {
            let sx = -2.2 + sn_i as f32 * 1.6;
            let sz = 0.65;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::ScalyFootSnail,
                pos: Vec3::new(sx, vent_field_height_at(sx, sz) + 0.06, sz),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 2.0,
                max_energy_kcal: 2.8,
                age_hours: 160.0,
                state: BehaviorState::BenthicGrazerCreep {
                    target_xz: [sx + 0.5, sz],
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        // L0: 8 Munidopsis Squat Lobsters tracking dissolved H2S bacterial floc plume!
        for m_i in 0..8 {
            let f = (m_i as f32 + 0.5) / 8.0;
            let p = munidopsis_home.lerp(sulfide_fissure, f);
            let ret = m_i % 2 == 0;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::MunidopsisSquatLobster,
                pos: Vec3::new(p.x, vent_field_height_at(p.x, p.z) + 0.04, p.z),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 2.1,
                max_energy_kcal: 2.8,
                age_hours: 115.0,
                state: if ret {
                    BehaviorState::AntReturningWithLeaf { nest_pos: munidopsis_home }
                } else {
                    BehaviorState::AntOutwardTrail { patch_pos: sulfide_fissure, wandering: 0.1 }
                },
                pose: ArticulatedPose { carrying_item: ret, ..Default::default() },
                vocal_cooldown_sec: 99.0,
            });
        }

        let mut events = StochasticEventScheduler::default();
        events.record(
            1,
            9.0,
            "2,600m Deep-Sea Hydrothermal Vent Active",
            "100% chemosynthetic black-smoker spire belches 360°C sulfide plumes; Riftia tubeworms, Kiwa yeti crabs, Rimicaris vent shrimp, bioluminescent jellies & ghost sharks active.",
        );

        Self {
            biome: BiomeKind::DeepSeaHydrothermalVent,
            biome_name: "Deep-Sea Hydrothermal Vent — 2,600m Black Smoker (Side Diorama)".to_string(),
            sim_time_sec: 0.0,
            rng_seed: seed,
            focus: TrophicFocus::AllStrata,
            diurnal: DiurnalCycle::default(),
            ledger: TrophicEnergyLedger::new_hydrothermal_vent_baseline(),
            floor_field,
            trees: Vec::new(),
            kelp_stalks: Vec::new(),
            branches,
            lianas: Vec::new(),
            underbrush,
            falling_detritus: Vec::new(),
            ant_nest_pos: munidopsis_home,
            ant_patches: vec![AntForagePatch {
                pos: sulfide_fissure,
                remaining_fragments: 24,
            }],
            organisms,
            events,
            pending_sounds: Vec::new(),
            next_entity_id: next_id,
            ambient_chorus_timer: 3.5,
        }
    }

    /// Initializes the Split-Level Temperate Freshwater Lake (Underwater `y < 8.4m` + Surface & Aerial Waterbirds `y >= 8.4m`).
    pub fn new_temperate_freshwater_lake(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut underbrush = Vec::new();

        // 75 Submerged Vallisneria/Coontail beds, Floating Nymphaea Water Lilies & Emergent Cattails!
        for i in 0..75 {
            let x = -14.0 + (i as f32 * 0.378) + rng.gen_range(-0.16..0.16);
            let z = match i % 4 {
                0 => rng.gen_range(0.35..1.45),
                1 => rng.gen_range(-0.9..0.35),
                2 => rng.gen_range(-2.8..-0.9),
                _ => rng.gen_range(-6.2..-2.8),
            };
            let kind = if x.abs() > 8.8 && i % 2 == 0 {
                UnderbrushKind::ShorelineCattail
            } else if i % 6 == 0 && x.abs() > 3.2 {
                UnderbrushKind::FloatingWaterLily
            } else if i % 5 == 0 {
                UnderbrushKind::SubmergedDriftwoodMoss
            } else if i % 2 == 0 {
                UnderbrushKind::VallisneriaEelgrass
            } else {
                UnderbrushKind::CoontailMacrophyte
            };
            let root_y = if kind == UnderbrushKind::FloatingWaterLily {
                LAKE_SURFACE_Y
            } else {
                lake_bed_height_at(x, z)
            };
            underbrush.push(UnderbrushPlant {
                id: i,
                root_pos: Vec3::new(x, root_y, z),
                scale: rng.gen_range(0.90..1.42),
                kind,
                bend_xz: [0.0, 0.0],
                bend_vel_xz: [0.0, 0.0],
                ripe_berries: 0,
                growth_progress: 0.7,
                vigor: 1.15,
            });
        }

        // Sunken cedar nurse-log snags underwater + Overhanging shoreline Willow boughs above the waterline (`y = 8.4..11.6m`)!
        let branches = vec![
            BranchSegment {
                id: 0,
                tree_id: 0,
                start: Vec3::new(-13.2, 10.6, -0.7),
                end: Vec3::new(-6.8, 10.9, -0.4), // Kingfisher overhanging willow perch!
                radius_start: 0.32,
                radius_end: 0.16,
                is_perchable: true,
            },
            BranchSegment {
                id: 1,
                tree_id: 0,
                start: Vec3::new(13.4, 10.8, -0.9),
                end: Vec3::new(7.4, 11.2, -0.6),
                radius_start: 0.34,
                radius_end: 0.18,
                is_perchable: true,
            },
            BranchSegment {
                id: 2,
                tree_id: 0,
                start: Vec3::new(-6.2, 0.45, -0.6),
                end: Vec3::new(-0.8, 2.35, -0.4), // Sunken mossy driftwood log!
                radius_start: 0.48,
                radius_end: 0.34,
                is_perchable: true,
            },
            BranchSegment {
                id: 3,
                tree_id: 0,
                start: Vec3::new(3.8, 6.9, 0.2),
                end: Vec3::new(7.8, 8.65, 0.3), // Half-submerged Painted Turtle basking log breaking the waterline!
                radius_start: 0.38,
                radius_end: 0.28,
                is_perchable: true,
            },
        ];

        let crayfish_burrow = Vec3::new(-6.8, lake_bed_height_at(-6.8, 0.95), 0.95);
        let leaf_pack = Vec3::new(2.4, lake_bed_height_at(2.4, 0.90), 0.90);
        let mut floor_field = ForestFloorField::default();
        for s in 0..20 {
            let f = s as f32 / 19.0;
            let p = crayfish_burrow.lerp(leaf_pack, f);
            floor_field.deposit_ant_pheromone(Vec2::new(p.x, p.z), 2.0);
        }

        let mut organisms = Vec::new();
        let mut next_id = 1_u32;
        let mut alloc_id = || {
            let id = next_id;
            next_id += 1;
            id
        };

        // L4 ABOVE & ON WATER: Osprey, Common Loon & Great Blue Heron + UNDERWATER Northern Pike!
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::Osprey,
            pos: Vec3::new(-3.5, 12.4, -0.8),
            vel: Vec3::new(2.2, 0.0, 0.0),
            scale: 1.05,
            energy_kcal: 28.0,
            max_energy_kcal: 36.0,
            age_hours: 210.0,
            state: BehaviorState::OspreyAerialSoar {
                orbit_phase: 0.0,
                plunge_timer: 0.0,
                plunge_x: 0.5,
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 14.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::CommonLoon,
            pos: Vec3::new(-2.2, LAKE_SURFACE_Y + 0.04, 0.25),
            vel: Vec3::new(0.65, 0.0, 0.0),
            scale: 1.05,
            energy_kcal: 34.0,
            max_energy_kcal: 44.0,
            age_hours: 260.0,
            state: BehaviorState::WaterbirdSurfaceSwim {
                target_x: 4.2,
                target_z: 0.35,
                dabble_timer: 0.0,
                dive_timer: 0.0,
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 11.0,
        });

        let heron_spot = Vec3::new(9.8, LAKE_SURFACE_Y - 0.35, 0.35);
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::GreatBlueHeron,
            pos: heron_spot,
            vel: Vec3::ZERO,
            scale: 1.08,
            energy_kcal: 24.0,
            max_energy_kcal: 32.0,
            age_hours: 195.0,
            state: BehaviorState::HeronShalllowStalk {
                anchor_pos: heron_spot,
                strike_timer: 0.0,
            },
            pose: ArticulatedPose { body_yaw: PI, ..Default::default() },
            vocal_cooldown_sec: 18.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::NorthernPike,
            pos: Vec3::new(-4.5, 3.2, -0.25),
            vel: Vec3::ZERO,
            scale: 1.12,
            energy_kcal: 38.0,
            max_energy_kcal: 48.0,
            age_hours: 340.0,
            state: BehaviorState::PikeWeedbedAmbush {
                hover_pos: Vec3::new(-4.5, 3.2, -0.25),
                glide_target: Vec3::new(3.2, 3.6, -0.15),
                gliding: false,
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 25.0,
        });

        // L3 SURFACE & AERIAL: Mallard Pair (Drake + Hen) & Belted Kingfisher
        for m_i in 0..2 {
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::MallardDuck,
                pos: Vec3::new(1.8 + m_i as f32 * 0.95, LAKE_SURFACE_Y + 0.05, 0.65 + m_i as f32 * 0.2),
                vel: Vec3::new(-0.45, 0.0, 0.0),
                scale: if m_i == 0 { 1.02 } else { 0.94 },
                energy_kcal: 14.0,
                max_energy_kcal: 18.0,
                age_hours: 120.0,
                state: BehaviorState::WaterbirdSurfaceSwim {
                    target_x: -4.8 + m_i as f32,
                    target_z: 0.65,
                    dabble_timer: 0.0,
                    dive_timer: 0.0,
                },
                pose: ArticulatedPose { body_yaw: PI, ..Default::default() },
                vocal_cooldown_sec: 9.0 + m_i as f32 * 4.0,
            });
        }

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::BeltedKingfisher,
            pos: branches[0].point_at(0.82) + Vec3::new(0.0, 0.22, 0.0),
            vel: Vec3::ZERO,
            scale: 0.98,
            energy_kcal: 6.2,
            max_energy_kcal: 8.0,
            age_hours: 88.0,
            state: BehaviorState::AvianPerchedForage {
                branch_id: 0,
                branch_t: 0.82,
                remaining_sec: 8.5,
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 10.0,
        });

        // L3 UNDERWATER & WATERLINE: 2 Rainbow Trout, Largemouth Bass, Western Painted Turtle & Bullfrog
        for t_i in 0..2 {
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::RainbowTrout,
                pos: Vec3::new(-1.5 + t_i as f32 * 3.8, 5.4 + t_i as f32 * 0.9, 0.15),
                vel: Vec3::new(0.95, 0.0, 0.0),
                scale: 1.0,
                energy_kcal: 15.0,
                max_energy_kcal: 20.0,
                age_hours: 140.0,
                state: BehaviorState::ReefFishForage {
                    waypoint: Vec3::new(4.8 - t_i as f32 * 6.0, 6.2, 0.2),
                    pause_sec: 0.0,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 20.0,
            });
        }

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::LargemouthBass,
            pos: Vec3::new(-2.4, 2.5, 0.1),
            vel: Vec3::new(0.65, 0.0, 0.0),
            scale: 1.06,
            energy_kcal: 18.0,
            max_energy_kcal: 24.0,
            age_hours: 175.0,
            state: BehaviorState::ReefFishForage {
                waypoint: Vec3::new(3.4, 2.8, 0.2),
                pause_sec: 0.0,
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 22.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::WesternPaintedTurtle,
            pos: Vec3::new(6.2, 8.25, 0.28),
            vel: Vec3::new(-0.35, 0.0, 0.0),
            scale: 0.95,
            energy_kcal: 9.5,
            max_energy_kcal: 13.0,
            age_hours: 210.0,
            state: BehaviorState::SeaTurtleReefFlight {
                waypoint: Vec3::new(0.8, 4.6, 0.35),
                surfacing_for_breath: false,
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 30.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::AmericanBullfrog,
            pos: Vec3::new(-7.6, LAKE_SURFACE_Y - 0.05, 0.72),
            vel: Vec3::ZERO,
            scale: 1.0,
            energy_kcal: 5.5,
            max_energy_kcal: 7.5,
            age_hours: 95.0,
            state: BehaviorState::BenthicGrazerCreep {
                target_xz: [-7.2, 0.72],
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 8.0,
        });

        // L2: 5 Pumpkinseed Sunfish, 12 Golden Shiners & 3 Aerial Blue Dasher Dragonflies
        for p_i in 0..5 {
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::PumpkinseedSunfish,
                pos: Vec3::new(-5.5 + p_i as f32 * 2.6, 2.2 + (p_i % 3) as f32 * 1.1, 0.35),
                vel: Vec3::new(0.55, 0.0, 0.0),
                scale: 0.95,
                energy_kcal: 4.5,
                max_energy_kcal: 6.0,
                age_hours: 75.0,
                state: BehaviorState::ReefFishForage {
                    waypoint: Vec3::new(-3.0 + p_i as f32 * 2.1, 3.2, 0.25),
                    pause_sec: 0.0,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 30.0,
            });
        }

        for s_i in 0..12 {
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::GoldenShiner,
                pos: Vec3::new(-0.5 + (s_i % 4) as f32 * 0.65, 4.5 + (s_i / 4) as f32 * 0.55, -0.15),
                vel: Vec3::new(0.85, 0.0, 0.0),
                scale: 0.90,
                energy_kcal: 1.8,
                max_energy_kcal: 2.4,
                age_hours: 48.0,
                state: BehaviorState::PlanktivoreBoidSchool {
                    school_center: Vec3::new(0.2, 4.8, -0.1),
                    phase_seed: s_i as f32 * 0.75,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        for d_i in 0..3 {
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::BlueDasherDragonfly,
                pos: Vec3::new(-4.5 + d_i as f32 * 4.5, LAKE_SURFACE_Y + 0.85 + d_i as f32 * 0.35, 0.45),
                vel: Vec3::new(1.1, 0.0, 0.0),
                scale: 1.0,
                energy_kcal: 0.6,
                max_energy_kcal: 0.9,
                age_hours: 22.0,
                state: BehaviorState::ButterflyPollinationFlutter {
                    target_plant_id: (d_i * 6) % underbrush.len(),
                    hover_timer: 1.2,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        // L0: 8 Pacific Signal Crayfish tracking sunken alder-leaf detritus plume!
        for c_i in 0..8 {
            let f = (c_i as f32 + 0.5) / 8.0;
            let p = crayfish_burrow.lerp(leaf_pack, f);
            let ret = c_i % 2 == 0;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::SignalCrayfish,
                pos: Vec3::new(p.x, lake_bed_height_at(p.x, p.z) + 0.04, p.z),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 2.2,
                max_energy_kcal: 3.0,
                age_hours: 110.0,
                state: if ret {
                    BehaviorState::AntReturningWithLeaf { nest_pos: crayfish_burrow }
                } else {
                    BehaviorState::AntOutwardTrail { patch_pos: leaf_pack, wandering: 0.1 }
                },
                pose: ArticulatedPose { carrying_item: ret, ..Default::default() },
                vocal_cooldown_sec: 99.0,
            });
        }

        let mut events = StochasticEventScheduler::default();
        events.record(
            1,
            9.4,
            "Temperate Freshwater Lake Initialized",
            "Split-level waterline active at 8.4m: osprey, common loon, mallards, heron & kingfisher above; pike, rainbow trout, bass, sunfish & signal crayfish among Vallisneria below.",
        );

        Self {
            biome: BiomeKind::TemperateFreshwaterLake,
            biome_name: "Temperate Freshwater Lake — Underwater & Waterbirds (Side Diorama)".to_string(),
            sim_time_sec: 0.0,
            rng_seed: seed,
            focus: TrophicFocus::AllStrata,
            diurnal: DiurnalCycle::default(),
            ledger: TrophicEnergyLedger::new_freshwater_lake_baseline(),
            floor_field,
            trees: Vec::new(),
            kelp_stalks: Vec::new(),
            branches,
            lianas: Vec::new(),
            underbrush,
            falling_detritus: Vec::new(),
            ant_nest_pos: crayfish_burrow,
            ant_patches: vec![AntForagePatch {
                pos: leaf_pack,
                remaining_fragments: 22,
            }],
            organisms,
            events,
            pending_sounds: Vec::new(),
            next_entity_id: next_id,
            ambient_chorus_timer: 3.5,
        }
    }

    /// Initializes the Indo-Pacific Shallow Fore-Reef & Lagoon side-view 3D diorama across all 5 strata.
    pub fn new_indo_pacific_coral_reef(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut underbrush = Vec::new();

        // 120 Dense Scleractinian Corals, Foliate Scrolls, Pillar Corals, Barrel Sponges, Sea Fans & Anemones across the Spur-and-Groove Reef Wall!
        for i in 0..120 {
            let x = -14.4 + (i as f32 * 0.242) + rng.gen_range(-0.18..0.18);
            let z = match i % 6 {
                0 => rng.gen_range(0.35..1.35),   // Reef toe & foreground bommies
                1 => rng.gen_range(-0.85..0.25),  // Lower fore-reef wall
                2 => rng.gen_range(-2.2..-0.85),  // Mid-wall cantilevered terraces
                3 => rng.gen_range(-3.8..-2.2),   // Upper spur buttresses
                4 => rng.gen_range(-6.2..-3.8),   // Reef crest gardens
                _ => rng.gen_range(-10.5..-6.2),  // Outer escarpment silhouette
            };
            let kind = match i % 10 {
                0 | 1 => UnderbrushKind::StaghornAcropora,
                2 | 3 => UnderbrushKind::TableAcropora,
                4 => UnderbrushKind::FolioseMontipora,
                5 => UnderbrushKind::PoritesBoulder,
                6 => UnderbrushKind::BrainCoralDome,
                7 => UnderbrushKind::PillarCoral,
                8 => if z < -0.5 { UnderbrushKind::BarrelSponge } else { UnderbrushKind::HostSeaAnemone },
                _ => if (-6.6..-2.8).contains(&x) && z > -2.2 { UnderbrushKind::SunCupTubastraea } else { UnderbrushKind::GorgonianSeaFan },
            };
            let base_y = coral_reef_height_at(x, z)
                + if kind == UnderbrushKind::SunCupTubastraea { 2.1 } else { 0.0 };
            underbrush.push(UnderbrushPlant {
                id: i,
                root_pos: Vec3::new(x, base_y, z),
                scale: rng.gen_range(0.90..1.52),
                kind,
                bend_xz: [0.0, 0.0],
                bend_vel_xz: [0.0, 0.0],
                ripe_berries: 0,
                growth_progress: 0.75,
                vigor: 1.18,
            });
        }

        // Sculpted Carbonate Reef Buttresses & Vaulted Swim-Through Reef Arch (`x = -6.5..-2.8`)
        let branches = vec![
            BranchSegment {
                id: 0,
                tree_id: 0,
                start: Vec3::new(-6.6, 0.2, -1.1),
                end: Vec3::new(-5.2, 3.9, -0.8),
                radius_start: 1.25,
                radius_end: 0.92,
                is_perchable: true,
            },
            BranchSegment {
                id: 1,
                tree_id: 0,
                start: Vec3::new(-5.2, 3.9, -0.8),
                end: Vec3::new(-3.4, 3.8, -0.8),
                radius_start: 0.88,
                radius_end: 0.88,
                is_perchable: true,
            },
            BranchSegment {
                id: 2,
                tree_id: 0,
                start: Vec3::new(-3.4, 3.8, -0.8),
                end: Vec3::new(-2.3, 0.3, -1.0),
                radius_start: 0.90,
                radius_end: 1.20,
                is_perchable: true,
            },
            BranchSegment {
                id: 3,
                tree_id: 0,
                start: Vec3::new(-0.8, 0.3, -1.1),
                end: Vec3::new(1.4, 3.8, -0.7),
                radius_start: 1.45,
                radius_end: 1.05,
                is_perchable: true,
            },
            BranchSegment {
                id: 4,
                tree_id: 0,
                start: Vec3::new(3.6, 0.4, -1.4),
                end: Vec3::new(7.8, 4.8, -1.1),
                radius_start: 1.55,
                radius_end: 1.10,
                is_perchable: true,
            },
            BranchSegment {
                id: 5,
                tree_id: 0,
                start: Vec3::new(8.4, 0.5, -1.8),
                end: Vec3::new(12.6, 5.4, -1.5),
                radius_start: 1.60,
                radius_end: 1.15,
                is_perchable: true,
            },
        ];

        let hermit_home = Vec3::new(-10.4, coral_reef_height_at(-10.4, 1.1), 1.1);
        let mucus_patch = Vec3::new(1.8, coral_reef_height_at(1.8, 0.9), 0.9);
        let mut floor_field = ForestFloorField::default();
        for s in 0..20 {
            let f = s as f32 / 19.0;
            let p = hermit_home.lerp(mucus_patch, f);
            floor_field.deposit_ant_pheromone(Vec2::new(p.x, p.z), 2.1);
        }

        let mut organisms = Vec::new();
        let mut next_id = 1_u32;
        let mut alloc_id = || {
            let id = next_id;
            next_id += 1;
            id
        };

        // L4: Blacktip Reef Shark, Giant Moray Eel & Red Lionfish
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::BlacktipReefShark,
            pos: Vec3::new(-8.5, 9.8, -1.4),
            vel: Vec3::new(1.75, 0.0, 0.0),
            scale: 1.05,
            energy_kcal: 54.0,
            max_energy_kcal: 68.0,
            age_hours: 310.0,
            state: BehaviorState::SharkReefCruise {
                waypoint: Vec3::new(9.5, 9.2, -1.1),
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 12.0,
        });

        let moray_den = Vec3::new(8.2, coral_reef_height_at(8.2, -0.5) + 0.65, -0.5);
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::GiantMorayEel,
            pos: moray_den,
            vel: Vec3::ZERO,
            scale: 1.05,
            energy_kcal: 44.0,
            max_energy_kcal: 56.0,
            age_hours: 380.0,
            state: BehaviorState::WolfEelCreviceAmbush {
                den_pos: moray_den,
                lunge_progress: 0.0,
                lunge_target: moray_den + Vec3::new(-1.9, 0.25, 0.2),
            },
            pose: ArticulatedPose {
                body_yaw: PI,
                ..Default::default()
            },
            vocal_cooldown_sec: 16.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::RedLionfish,
            pos: Vec3::new(-6.4, 4.2, -0.4),
            vel: Vec3::new(0.22, 0.05, 0.0),
            scale: 1.0,
            energy_kcal: 16.0,
            max_energy_kcal: 22.0,
            age_hours: 145.0,
            state: BehaviorState::ReefFishForage {
                waypoint: Vec3::new(-3.8, 4.6, -0.3),
                pause_sec: 2.0,
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 20.0,
        });

        // L3: Ocellaris Clownfish Pair in Host Anemone, Moorish Idol, Emperor Angelfish & Bluespotted Stingray
        let anemone_host = underbrush
            .iter()
            .find(|u| u.kind == UnderbrushKind::HostSeaAnemone && u.root_pos.z > -2.0)
            .map(|u| u.root_pos + Vec3::new(0.0, 0.55, 0.0))
            .unwrap_or(Vec3::new(-3.2, 1.1, 0.3));
        for c_i in 0..2 {
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::OcellarisClownfish,
                pos: anemone_host + Vec3::new(c_i as f32 * 0.28, 0.15, 0.0),
                vel: Vec3::ZERO,
                scale: if c_i == 0 { 1.08 } else { 0.86 }, // Protandrous female larger than male!
                energy_kcal: 6.5,
                max_energy_kcal: 8.5,
                age_hours: 90.0,
                state: BehaviorState::ClownfishAnemoneSymbiosis {
                    anemone_pos: anemone_host,
                    dart_phase: c_i as f32 * 2.4,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 9.0,
            });
        }

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::MoorishIdol,
            pos: Vec3::new(-1.2, 5.8, -0.5),
            vel: Vec3::new(0.85, 0.0, 0.0),
            scale: 1.0,
            energy_kcal: 11.0,
            max_energy_kcal: 15.0,
            age_hours: 84.0,
            state: BehaviorState::ReefFishForage {
                waypoint: Vec3::new(4.5, 6.2, -0.6),
                pause_sec: 0.0,
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 18.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::EmperorAngelfish,
            pos: Vec3::new(3.8, 4.4, -0.2),
            vel: Vec3::new(-0.75, 0.0, 0.0),
            scale: 1.05,
            energy_kcal: 14.5,
            max_energy_kcal: 19.0,
            age_hours: 130.0,
            state: BehaviorState::ReefFishForage {
                waypoint: Vec3::new(-3.2, 4.0, -0.4),
                pause_sec: 0.0,
            },
            pose: ArticulatedPose {
                body_yaw: PI,
                ..Default::default()
            },
            vocal_cooldown_sec: 18.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::BluespottedStingray,
            pos: Vec3::new(-4.5, coral_reef_height_at(-4.5, 0.5) + 0.32, 0.5),
            vel: Vec3::new(0.92, 0.0, 0.0),
            scale: 1.05,
            energy_kcal: 24.0,
            max_energy_kcal: 32.0,
            age_hours: 175.0,
            state: BehaviorState::StingraySandUndulate {
                waypoint: Vec3::new(6.8, 0.65, 0.4),
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 22.0,
        });

        // L2: Green Sea Turtle, Steephead Parrotfish, Blue-Green Chromis Coral Cloud & Giant Tridacna Clams
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::GreenSeaTurtle,
            pos: Vec3::new(2.4, 8.2, -0.3),
            vel: Vec3::new(-1.15, 0.15, 0.0),
            scale: 1.08,
            energy_kcal: 78.0,
            max_energy_kcal: 96.0,
            age_hours: 520.0,
            state: BehaviorState::SeaTurtleReefFlight {
                waypoint: Vec3::new(-7.5, 6.8, -0.2),
                surfacing_for_breath: false,
            },
            pose: ArticulatedPose {
                body_yaw: PI,
                ..Default::default()
            },
            vocal_cooldown_sec: 10.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::SteepheadParrotfish,
            pos: Vec3::new(0.8, 3.1, 0.1),
            vel: Vec3::new(0.88, 0.0, 0.0),
            scale: 1.05,
            energy_kcal: 21.0,
            max_energy_kcal: 28.0,
            age_hours: 150.0,
            state: BehaviorState::ReefFishForage {
                waypoint: Vec3::new(5.4, 2.7, 0.0),
                pause_sec: 1.5,
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 6.0,
        });

        let staghorn_anchor = Vec3::new(0.4, 2.6, -0.6);
        for ch_i in 0..16 {
            let a = ch_i as f32 * 0.62;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::BlueGreenChromis,
                pos: staghorn_anchor + Vec3::new(a.cos() * 1.6, 1.8 + (a * 1.4).sin() * 0.7, (ch_i % 3) as f32 * 0.35),
                vel: Vec3::new(0.6, 0.05, 0.0),
                scale: 0.90 + (ch_i % 3) as f32 * 0.08,
                energy_kcal: 2.8,
                max_energy_kcal: 4.0,
                age_hours: 42.0,
                state: BehaviorState::PlanktivoreBoidSchool {
                    school_center: staghorn_anchor + Vec3::new(0.0, 1.9, 0.0),
                    phase_seed: ch_i as f32 * 0.71,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        for cl_i in 0..3 {
            let cx = -6.2 + cl_i as f32 * 6.1;
            let cz = 0.75;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::GiantTridacnaClam,
                pos: Vec3::new(cx, coral_reef_height_at(cx, cz) + 0.08, cz),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 18.0,
                max_energy_kcal: 24.0,
                age_hours: 600.0,
                state: BehaviorState::SessileMantleFilter { open_fraction: 1.0 },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        // L0: Pacific Cleaner Shrimp, Cobalt Linckia Sea Stars & Banded Hermit Crabs on Mucus Plume
        for sh_i in 0..3 {
            let sx = -7.8 + sh_i as f32 * 7.8;
            let sz = 0.35;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::PacificCleanerShrimp,
                pos: Vec3::new(sx, coral_reef_height_at(sx, sz) + 0.32, sz),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 1.4,
                max_energy_kcal: 2.0,
                age_hours: 50.0,
                state: BehaviorState::BenthicGrazerCreep {
                    target_xz: [sx + 0.35, sz],
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        for lk_i in 0..4 {
            let lx = -9.2 + lk_i as f32 * 5.8;
            let lz = 1.15 - (lk_i % 2) as f32 * 0.4;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::BlueLinckiaStar,
                pos: Vec3::new(lx, coral_reef_height_at(lx, lz) + 0.04, lz),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 2.2,
                max_energy_kcal: 3.0,
                age_hours: 110.0,
                state: BehaviorState::BenthicGrazerCreep {
                    target_xz: [lx + 0.7, lz],
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        for hc_i in 0..9 {
            let frac = (hc_i as f32 + 0.5) / 9.0;
            let base = hermit_home.lerp(mucus_patch, frac);
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::ReefHermitCrab,
                pos: Vec3::new(base.x, coral_reef_height_at(base.x, base.z) + 0.04, base.z),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 1.6,
                max_energy_kcal: 2.4,
                age_hours: 65.0,
                state: if hc_i % 2 == 0 {
                    BehaviorState::AntOutwardTrail {
                        patch_pos: mucus_patch,
                        wandering: 0.0,
                    }
                } else {
                    BehaviorState::AntReturningWithLeaf {
                        nest_pos: hermit_home,
                    }
                },
                pose: ArticulatedPose {
                    carrying_item: hc_i % 2 != 0,
                    limb_phase: hc_i as f32 * 0.9,
                    ..Default::default()
                },
                vocal_cooldown_sec: 99.0,
            });
        }

        let mut events = StochasticEventScheduler::default();
        events.record(
            1,
            10.15,
            "Indo-Pacific Coral Reef Initialized",
            "Turquoise caustics dance over staghorn Acropora; clownfish tend their host anemone while a green sea turtle and blacktip reef shark glide overhead.",
        );

        Self {
            biome: BiomeKind::IndoPacificCoralReef,
            biome_name: "Indo-Pacific Shallow Fore-Reef & Lagoon (Side Diorama)".to_string(),
            sim_time_sec: 0.0,
            rng_seed: seed,
            focus: TrophicFocus::AllStrata,
            diurnal: DiurnalCycle::default(),
            ledger: TrophicEnergyLedger::new_coral_reef_baseline(),
            floor_field,
            trees: Vec::new(),
            kelp_stalks: Vec::new(),
            branches,
            lianas: Vec::new(),
            underbrush,
            falling_detritus: Vec::new(),
            ant_nest_pos: hermit_home,
            ant_patches: vec![AntForagePatch {
                pos: mucus_patch,
                remaining_fragments: 20,
            }],
            organisms,
            events,
            pending_sounds: Vec::new(),
            next_entity_id: next_id,
            ambient_chorus_timer: 3.5,
        }
    }

    /// Initializes the Northeast Pacific Giant Kelp Forest side-view 3D diorama across 5 trophic strata.
    pub fn new_pacific_kelp_forest(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut kelp_stalks = Vec::new();

        // Multi-depth Giant Kelp (Macrocystis pyrifera) & Bull Kelp (Nereocystis luetkeana) columns
        let stalk_specs: &[(f32, f32, f32, bool, u8)] = &[
            (-13.2, 0.8, 14.8, false, 1),
            (-10.4, -0.9, 15.1, false, 1),
            (-7.2, 0.6, 14.6, true, 1),
            (-4.2, -1.6, 15.2, false, 1),
            (-0.8, -2.4, 15.0, false, 2),
            (2.6, 0.4, 14.9, true, 1),
            (5.8, -1.2, 15.2, false, 1),
            (9.2, 0.7, 14.7, false, 1),
            (12.4, -1.0, 15.0, true, 1),
            // Mid-depth & misty pelagic background kelp forest columns
            (-11.8, -6.4, 15.2, false, 2),
            (-6.2, -7.2, 15.0, false, 2),
            (0.8, -6.8, 15.3, true, 2),
            (7.4, -7.5, 15.1, false, 2),
            (13.0, -6.9, 14.9, false, 2),
            (-9.5, -12.8, 15.4, false, 3),
            (-2.0, -13.4, 15.4, false, 3),
            (5.5, -12.6, 15.4, true, 3),
            (11.8, -13.2, 15.2, false, 3),
        ];

        for (id, &(x, z, h, bull, tier)) in stalk_specs.iter().enumerate() {
            kelp_stalks.push(KelpStalk {
                id,
                holdfast_pos: Vec3::new(x, reef_height_at(x, z), z),
                height_m: h,
                is_bull_kelp: bull,
                depth_tier: tier,
                bend_offset_xz: [0.0, 0.0],
                bend_vel_xz: [0.0, 0.0],
                holdfast_health: 1.0,
            });
        }

        // Rocky basalt reef arches/boulders stored in `branches` for reef collision & crevices
        let branches = vec![
            BranchSegment {
                id: 0,
                tree_id: 0,
                start: Vec3::new(-11.8, 0.2, -0.8),
                end: Vec3::new(-7.4, 1.9, -0.5),
                radius_start: 0.95,
                radius_end: 0.72,
                is_perchable: true,
            },
            BranchSegment {
                id: 1,
                tree_id: 0,
                start: Vec3::new(5.8, 0.1, -0.6),
                end: Vec3::new(11.4, 2.3, -1.1),
                radius_start: 1.15,
                radius_end: 0.82,
                is_perchable: true,
            },
        ];

        // Coralline algae & crimson Rhodymenia understory blades on rocky reef
        let mut underbrush = Vec::new();
        for i in 0..24 {
            let x = -14.0 + i as f32 * 1.18 + rng.gen_range(-0.2..0.2);
            let z = rng.gen_range(-5.5..2.2);
            let kind = if i % 2 == 0 {
                UnderbrushKind::CorallineAlgaeTurf
            } else {
                UnderbrushKind::CrimsonRhodymeniaBlade
            };
            underbrush.push(UnderbrushPlant {
                id: i,
                root_pos: Vec3::new(x, reef_height_at(x, z), z),
                scale: rng.gen_range(0.8..1.28),
                kind,
                bend_xz: [0.0, 0.0],
                bend_vel_xz: [0.0, 0.0],
                ripe_berries: 0,
                growth_progress: 0.5,
                vigor: 1.0,
            });
        }

        // Benthic kelp-drift chemical plume patch (crustaceans & sea stars converge, then plume dissipates!)
        let crab_home = Vec3::new(-10.2, reef_height_at(-10.2, 1.0), 1.0);
        let drift_patch = Vec3::new(1.4, reef_height_at(1.4, 0.8), 0.8);
        let mut floor_field = ForestFloorField::default();
        for s in 0..20 {
            let f = s as f32 / 19.0;
            let p = crab_home.lerp(drift_patch, f);
            floor_field.deposit_ant_pheromone(Vec2::new(p.x, p.z), 2.0);
        }

        let mut organisms = Vec::new();
        let mut next_id = 1_u32;
        let mut alloc_id = || {
            let id = next_id;
            next_id += 1;
            id
        };

        // --- L4: TERTIARY MARINE CARNIVORES ---
        // 1. Pacific Harbor Seal weaving gracefully through mid-water kelp stipes
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::PacificHarborSeal,
            pos: Vec3::new(-7.5, 8.2, -0.4),
            vel: Vec3::new(1.55, 0.15, 0.0),
            scale: 1.05,
            energy_kcal: 68.0,
            max_energy_kcal: 85.0,
            age_hours: 320.0,
            state: BehaviorState::SealKelpGlide {
                waypoint: Vec3::new(7.8, 7.4, -0.6),
            },
            pose: ArticulatedPose {
                body_yaw: 0.0,
                ..Default::default()
            },
            vocal_cooldown_sec: 7.0,
        });

        // 2. Pacific Leopard Shark cruising sinuously above the rocky reef channels
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::LeopardShark,
            pos: Vec3::new(6.8, 2.8, -1.2),
            vel: Vec3::new(-1.25, 0.0, 0.0),
            scale: 1.05,
            energy_kcal: 42.0,
            max_energy_kcal: 54.0,
            age_hours: 260.0,
            state: BehaviorState::SharkReefCruise {
                waypoint: Vec3::new(-8.5, 3.1, -0.9),
            },
            pose: ArticulatedPose {
                body_yaw: PI,
                ..Default::default()
            },
            vocal_cooldown_sec: 15.0,
        });

        // 3. Pacific Wolf Eel peering in ambush from right basalt reef crevice
        let den_wolf = Vec3::new(8.8, reef_height_at(8.8, -0.4) + 0.55, -0.4);
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::PacificWolfEel,
            pos: den_wolf,
            vel: Vec3::ZERO,
            scale: 1.0,
            energy_kcal: 34.0,
            max_energy_kcal: 44.0,
            age_hours: 410.0,
            state: BehaviorState::WolfEelCreviceAmbush {
                den_pos: den_wolf,
                lunge_progress: 0.0,
                lunge_target: den_wolf + Vec3::new(-1.8, 0.2, 0.3),
            },
            pose: ArticulatedPose {
                body_yaw: PI,
                ..Default::default()
            },
            vocal_cooldown_sec: 18.0,
        });

        // --- L3: SECONDARY CONSUMERS (Keystone Sea Otter, Giant Pacific Octopus, Sheephead, Garibaldi) ---
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::SeaOtter,
            pos: Vec3::new(-2.8, KELP_SURFACE_Y - 0.55, -0.3),
            vel: Vec3::ZERO,
            scale: 1.05,
            energy_kcal: 46.0,
            max_energy_kcal: 58.0,
            age_hours: 190.0,
            state: BehaviorState::OtterChestStoneTap {
                remaining_sec: 4.5,
                tap_cooldown: 0.8,
            },
            pose: ArticulatedPose {
                body_yaw: 0.0,
                carrying_item: true,
                ..Default::default()
            },
            vocal_cooldown_sec: 2.0,
        });

        let octo_den = Vec3::new(-8.6, reef_height_at(-8.6, -0.2) + 0.42, -0.2);
        organisms.push(Organism {
            id: alloc_id(),
            species: Species::GiantPacificOctopus,
            pos: octo_den,
            vel: Vec3::ZERO,
            scale: 1.05,
            energy_kcal: 38.0,
            max_energy_kcal: 50.0,
            age_hours: 140.0,
            state: BehaviorState::OctopusDenCamouflage {
                den_pos: octo_den,
                remaining_sec: 5.5,
            },
            pose: ArticulatedPose {
                chromatophore: 0.15,
                ..Default::default()
            },
            vocal_cooldown_sec: 12.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::CaliforniaSheephead,
            pos: Vec3::new(-1.5, 4.8, -0.8),
            vel: Vec3::new(0.85, 0.0, 0.0),
            scale: 1.0,
            energy_kcal: 19.0,
            max_energy_kcal: 25.0,
            age_hours: 110.0,
            state: BehaviorState::ReefFishForage {
                waypoint: Vec3::new(4.8, 4.2, -0.7),
                pause_sec: 0.0,
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 14.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::GaribaldiDamselfish,
            pos: Vec3::new(3.6, 3.6, 0.2),
            vel: Vec3::new(-0.65, 0.1, 0.0),
            scale: 1.0,
            energy_kcal: 9.5,
            max_energy_kcal: 12.0,
            age_hours: 95.0,
            state: BehaviorState::ReefFishForage {
                waypoint: Vec3::new(-2.4, 3.9, 0.1),
                pause_sec: 0.0,
            },
            pose: ArticulatedPose {
                body_yaw: PI,
                ..Default::default()
            },
            vocal_cooldown_sec: 20.0,
        });

        // --- L2: PRIMARY CONSUMERS (Blacksmith Boids School, Purple Sea Urchins, Red Abalone) ---
        for s_idx in 0..14 {
            let angle = s_idx as f32 * 0.65;
            let bpos = Vec3::new(
                1.2 + angle.cos() * 1.8,
                9.2 + (angle * 1.3).sin() * 1.1,
                -1.4 + (s_idx % 3) as f32 * 0.55,
            );
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::BlacksmithDamselfish,
                pos: bpos,
                vel: Vec3::new(0.9, 0.05, 0.0),
                scale: 0.92 + (s_idx % 3) as f32 * 0.08,
                energy_kcal: 3.5,
                max_energy_kcal: 5.0,
                age_hours: 48.0,
                state: BehaviorState::PlanktivoreBoidSchool {
                    school_center: Vec3::new(1.5, 9.2, -1.2),
                    phase_seed: s_idx as f32 * 0.77,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        for u_idx in 0..9 {
            let ux = -9.5 + u_idx as f32 * 2.35;
            let uz = 0.4 + ((u_idx % 3) as f32 - 1.0) * 0.65;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::PurpleSeaUrchin,
                pos: Vec3::new(ux, reef_height_at(ux, uz) + 0.08, uz),
                vel: Vec3::ZERO,
                scale: 0.9 + (u_idx % 3) as f32 * 0.14,
                energy_kcal: 2.8,
                max_energy_kcal: 4.0,
                age_hours: 80.0,
                state: BehaviorState::BenthicGrazerCreep {
                    target_xz: [ux + 1.2, uz],
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        for ab_idx in 0..3 {
            let ax = -5.2 + ab_idx as f32 * 5.4;
            let az = 0.95;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::RedAbalone,
                pos: Vec3::new(ax, reef_height_at(ax, az) + 0.05, az),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 4.2,
                max_energy_kcal: 6.0,
                age_hours: 150.0,
                state: BehaviorState::BenthicGrazerCreep {
                    target_xz: [ax + 0.5, az],
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        // --- L0: BENTHIC DETRITIVORES (Kelp Crabs on dissolved scent plume, Bat Stars, Sea Cucumbers) ---
        for c_idx in 0..8 {
            let frac = (c_idx as f32 + 0.5) / 8.0;
            let base = crab_home.lerp(drift_patch, frac);
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::NorthernKelpCrab,
                pos: Vec3::new(base.x, reef_height_at(base.x, base.z) + 0.04, base.z),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 1.8,
                max_energy_kcal: 2.5,
                age_hours: 60.0,
                state: if c_idx % 2 == 0 {
                    BehaviorState::AntOutwardTrail {
                        patch_pos: drift_patch,
                        wandering: 0.0,
                    }
                } else {
                    BehaviorState::AntReturningWithLeaf {
                        nest_pos: crab_home,
                    }
                },
                pose: ArticulatedPose {
                    carrying_item: c_idx % 2 != 0,
                    limb_phase: c_idx as f32,
                    ..Default::default()
                },
                vocal_cooldown_sec: 99.0,
            });
        }

        for s_idx in 0..5 {
            let sx = -10.8 + s_idx as f32 * 5.2;
            let sz = 1.15 - (s_idx % 2) as f32 * 0.5;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::BatSeaStar,
                pos: Vec3::new(sx, reef_height_at(sx, sz) + 0.03, sz),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 2.1,
                max_energy_kcal: 3.0,
                age_hours: 120.0,
                state: BehaviorState::BenthicGrazerCreep {
                    target_xz: [sx + 0.8, sz],
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        for h_idx in 0..4 {
            let hx = -7.4 + h_idx as f32 * 5.1;
            let hz = 0.85;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::CaliforniaSeaCucumber,
                pos: Vec3::new(hx, reef_height_at(hx, hz) + 0.05, hz),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 3.0,
                max_energy_kcal: 4.5,
                age_hours: 140.0,
                state: BehaviorState::BenthicGrazerCreep {
                    target_xz: [hx + 0.6, hz],
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        let mut events = StochasticEventScheduler::default();
        events.record(
            1,
            9.25,
            "Pacific Kelp Forest Initialized",
            "Buoyant Macrocystis stipes sway in the Pacific groundswell; a sea otter taps an urchin at the surface while a harbor seal glides through the fronds.",
        );

        Self {
            biome: BiomeKind::PacificKelpForest,
            biome_name: "Northeast Pacific Giant Kelp Forest (Side Diorama)".to_string(),
            sim_time_sec: 0.0,
            rng_seed: seed,
            focus: TrophicFocus::AllStrata,
            diurnal: DiurnalCycle::default(),
            ledger: TrophicEnergyLedger::new_kelp_forest_baseline(),
            floor_field,
            trees: Vec::new(),
            kelp_stalks,
            branches,
            lianas: Vec::new(),
            underbrush,
            falling_detritus: Vec::new(),
            ant_nest_pos: crab_home,
            ant_patches: vec![AntForagePatch {
                pos: drift_patch,
                remaining_fragments: 18,
            }],
            organisms,
            events,
            pending_sounds: Vec::new(),
            next_entity_id: next_id,
            ambient_chorus_timer: 4.0,
        }
    }

    pub fn new_neotropical_rainforest(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut trees = Vec::new();
        let mut branches = Vec::new();

        let tree_specs: &[(f32, f32, f32, f32, f32, u8)] = &[
            (-10.8, -0.8, 15.2, 0.92, 5.4, 1),
            (9.6, -1.2, 15.8, 1.05, 5.8, 1),
            (-0.6, -4.6, 16.4, 0.98, 6.2, 2),
            (-14.2, -9.5, 17.0, 1.15, 6.5, 3),
            (-5.2, -10.8, 16.6, 1.02, 6.0, 3),
            (4.8, -9.8, 17.2, 1.12, 6.4, 3),
            (14.0, -11.2, 16.5, 1.05, 6.0, 3),
            (-9.5, -15.2, 18.0, 1.25, 7.0, 3),
            (1.5, -15.6, 18.4, 1.30, 7.2, 3),
            (11.2, -15.0, 17.8, 1.20, 6.8, 3),
        ];

        for (id, &(x, z, h, r, crown, tier)) in tree_specs.iter().enumerate() {
            trees.push(CanopyTree {
                id,
                base_pos: Vec3::new(x, ground_height_at(x, z), z),
                height_m: h,
                trunk_radius_m: r,
                crown_radius_m: crown,
                depth_tier: tier,
            });
        }

        let branch_specs: &[(usize, Vec3, Vec3, f32, f32, bool)] = &[
            (2, Vec3::new(-6.2, 11.4, -1.6), Vec3::new(4.2, 11.8, -1.2), 0.42, 0.28, true),
            (1, Vec3::new(3.4, 11.2, -0.9), Vec3::new(11.8, 10.4, -1.1), 0.48, 0.34, true),
            (0, Vec3::new(-11.2, 7.8, -0.7), Vec3::new(-3.4, 6.8, -0.5), 0.38, 0.22, true),
            (2, Vec3::new(-2.8, 6.6, -1.8), Vec3::new(3.2, 7.1, -1.4), 0.30, 0.20, true),
            (1, Vec3::new(1.8, 3.6, -0.6), Vec3::new(11.2, 8.4, -0.9), 0.52, 0.32, true),
            (0, Vec3::new(-11.0, 11.6, -0.9), Vec3::new(-2.8, 12.2, -1.3), 0.40, 0.24, true),
            (2, Vec3::new(-7.5, 8.8, -5.2), Vec3::new(6.5, 9.2, -4.8), 0.36, 0.26, true),
            (0, Vec3::new(-10.6, 2.2, -0.6), Vec3::new(-5.8, 5.4, -0.8), 0.44, 0.28, true),
        ];
        for (id, &(tree_id, start, end, rs, re, perch)) in branch_specs.iter().enumerate() {
            branches.push(BranchSegment {
                id,
                tree_id,
                start,
                end,
                radius_start: rs,
                radius_end: re,
                is_perchable: perch,
            });
        }

        let liana_specs: &[(Vec3, f32, f32)] = &[
            (Vec3::new(-5.4, 11.7, -0.9), 4.6, -0.35),
            (Vec3::new(-0.8, 11.6, -0.8), 4.8, 0.18),
            (Vec3::new(3.8, 11.3, -0.8), 4.3, -0.12),
            (Vec3::new(-8.2, 11.4, -3.4), 5.4, 0.05),
            (Vec3::new(6.6, 10.7, -3.6), 5.1, -0.06),
        ];
        let lianas: Vec<LianaVine> = liana_specs
            .iter()
            .enumerate()
            .map(|(id, &(anchor, len, theta0))| LianaVine {
                id,
                anchor,
                length_m: len,
                theta_rad: theta0,
                phi_z_rad: 0.04 * (id as f32 - 2.0),
                omega_rad_s: if id == 0 { 0.95 } else { 0.08 },
                omega_z_rad_s: 0.0,
                damping: 0.28,
                occupied_by_entity: None,
            })
            .collect();

        let mut underbrush = Vec::new();
        for i in 0..28 {
            let x = -14.5 + (i as f32) * 1.06 + rng.gen_range(-0.25..0.25);
            let z = match i % 4 {
                0 => rng.gen_range(1.1..2.6),
                1 => rng.gen_range(-0.4..0.9),
                2 => rng.gen_range(-3.2..-1.0),
                _ => rng.gen_range(-7.5..-3.8),
            };
            let kind = match i % 5 {
                0 | 2 => UnderbrushKind::SwordFernClump,
                1 => UnderbrushKind::BlueberryMelastomeShrub,
                3 => UnderbrushKind::HeliconiaFlowerCluster,
                _ => UnderbrushKind::MonsteraBroadleaf,
            };
            underbrush.push(UnderbrushPlant {
                id: i,
                root_pos: Vec3::new(x, ground_height_at(x, z), z),
                scale: rng.gen_range(0.82..1.32),
                kind,
                bend_xz: [0.0, 0.0],
                bend_vel_xz: [0.0, 0.0],
                ripe_berries: if kind == UnderbrushKind::BlueberryMelastomeShrub {
                    rng.gen_range(4..=9)
                } else {
                    0
                },
                growth_progress: rng.gen_range(0.1..0.85),
                vigor: rng.gen_range(0.85..1.15),
            });
        }

        let ant_nest_pos = Vec3::new(-11.4, ground_height_at(-11.4, 0.9), 0.9);
        let patch_a = Vec3::new(-1.8, ground_height_at(-1.8, 0.6), 0.6);
        let ant_patches = vec![AntForagePatch {
            pos: patch_a,
            remaining_fragments: 26,
        }];

        let mut floor_field = ForestFloorField::default();
        for step in 0..24 {
            let frac = step as f32 / 23.0;
            let p = ant_nest_pos.lerp(patch_a, frac);
            let wobble = (frac * PI * 3.0).sin() * 0.28;
            floor_field.deposit_ant_pheromone(Vec2::new(p.x, p.z + wobble), 2.2);
        }

        let mut organisms = Vec::new();
        let mut next_id = 1_u32;
        let mut alloc_id = || {
            let id = next_id;
            next_id += 1;
            id
        };

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::BoaConstrictor,
            pos: branches[0].point_at(0.55) + Vec3::new(0.0, 0.24, 0.0),
            vel: Vec3::ZERO,
            scale: 1.05,
            energy_kcal: 38.0,
            max_energy_kcal: 48.0,
            age_hours: 120.0,
            state: BehaviorState::SnakeCoiledAmbush {
                branch_id: 0,
                branch_t: 0.55,
                patience_sec: 0.0,
                tongue_cooldown: 2.2,
            },
            pose: ArticulatedPose {
                body_yaw: -0.3,
                spine_phase: 0.6,
                ..Default::default()
            },
            vocal_cooldown_sec: 8.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::BlackPanther,
            pos: Vec3::new(-8.5, ground_height_at(-8.5, 0.35), 0.35),
            vel: Vec3::new(0.68, 0.0, 0.0),
            scale: 1.0,
            energy_kcal: 42.0,
            max_energy_kcal: 58.0,
            age_hours: 240.0,
            state: BehaviorState::PantherUnderbrushStalk {
                waypoint_x: 7.5,
                waypoint_z: 0.25,
                stealth_crouch: 0.82,
            },
            pose: ArticulatedPose {
                body_yaw: 0.0,
                crouch: 0.82,
                ..Default::default()
            },
            vocal_cooldown_sec: 6.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::CanopyJaguar,
            pos: branches[1].point_at(0.62) + Vec3::new(0.0, 0.42, 0.0),
            vel: Vec3::ZERO,
            scale: 1.0,
            energy_kcal: 46.0,
            max_energy_kcal: 60.0,
            age_hours: 300.0,
            state: BehaviorState::JaguarCanopyLounge {
                branch_id: 1,
                branch_t: 0.62,
                tail_sway_phase: 0.0,
                remaining_sec: 24.0,
            },
            pose: ArticulatedPose {
                body_yaw: PI,
                crouch: 0.55,
                ..Default::default()
            },
            vocal_cooldown_sec: 12.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::SquirrelMonkey,
            pos: lianas[0].tip_position(),
            vel: lianas[0].tip_velocity(),
            scale: 1.0,
            energy_kcal: 14.0,
            max_energy_kcal: 20.0,
            age_hours: 80.0,
            state: BehaviorState::MonkeyVineSwing {
                vine_id: 0,
                swing_time: 0.0,
                target_branch_id: 3,
            },
            pose: ArticulatedPose {
                tail_angle: -0.4,
                ..Default::default()
            },
            vocal_cooldown_sec: 2.5,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::SquirrelMonkey,
            pos: branches[4].point_at(0.58) + Vec3::new(0.0, 0.36, 0.0),
            vel: Vec3::ZERO,
            scale: 0.96,
            energy_kcal: 15.0,
            max_energy_kcal: 20.0,
            age_hours: 72.0,
            state: BehaviorState::MonkeyPerchedLookAround {
                branch_id: 4,
                branch_t: 0.58,
                remaining_sec: 4.5,
                scan_seed: 1.7,
            },
            pose: ArticulatedPose {
                body_yaw: PI * 0.85,
                head_yaw: 0.55,
                ..Default::default()
            },
            vocal_cooldown_sec: 5.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::ScarletMacaw,
            pos: branches[2].point_at(0.68) + Vec3::new(0.0, 0.34, 0.0),
            vel: Vec3::ZERO,
            scale: 1.0,
            energy_kcal: 12.5,
            max_energy_kcal: 16.0,
            age_hours: 96.0,
            state: BehaviorState::AvianPerchedForage {
                branch_id: 2,
                branch_t: 0.68,
                remaining_sec: 7.5,
            },
            pose: ArticulatedPose {
                body_yaw: 0.35,
                ..Default::default()
            },
            vocal_cooldown_sec: 4.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::KeelBilledToucan,
            pos: branches[3].point_at(0.45) + Vec3::new(0.0, 0.32, 0.0),
            vel: Vec3::ZERO,
            scale: 1.0,
            energy_kcal: 10.5,
            max_energy_kcal: 14.0,
            age_hours: 65.0,
            state: BehaviorState::AvianPerchedForage {
                branch_id: 3,
                branch_t: 0.45,
                remaining_sec: 5.2,
            },
            pose: ArticulatedPose {
                body_yaw: -0.25,
                ..Default::default()
            },
            vocal_cooldown_sec: 3.2,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::ThreeToedSloth,
            pos: branches[2].point_at(0.24) - Vec3::new(0.0, 0.52, 0.0),
            vel: Vec3::ZERO,
            scale: 1.0,
            energy_kcal: 18.0,
            max_energy_kcal: 24.0,
            age_hours: 180.0,
            state: BehaviorState::SlothSuspensoryClimb {
                branch_id: 2,
                branch_t: 0.24,
                dir: 1.0,
            },
            pose: ArticulatedPose::default(),
            vocal_cooldown_sec: 20.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::LowlandTapir,
            pos: Vec3::new(8.4, ground_height_at(8.4, -1.6), -1.6),
            vel: Vec3::ZERO,
            scale: 1.05,
            energy_kcal: 64.0,
            max_energy_kcal: 80.0,
            age_hours: 210.0,
            state: BehaviorState::FloorHerbivoreBrowse {
                target_x: 4.5,
                target_z: -1.8,
                gnawing_timer: 2.0,
            },
            pose: ArticulatedPose {
                body_yaw: PI,
                ..Default::default()
            },
            vocal_cooldown_sec: 15.0,
        });

        organisms.push(Organism {
            id: alloc_id(),
            species: Species::CentralAmericanAgouti,
            pos: Vec3::new(1.6, ground_height_at(1.6, -0.3), -0.3),
            vel: Vec3::ZERO,
            scale: 1.0,
            energy_kcal: 11.0,
            max_energy_kcal: 15.0,
            age_hours: 90.0,
            state: BehaviorState::FloorHerbivoreBrowse {
                target_x: 3.4,
                target_z: 0.2,
                gnawing_timer: 3.2,
            },
            pose: ArticulatedPose {
                body_yaw: 0.2,
                carrying_item: true,
                ..Default::default()
            },
            vocal_cooldown_sec: 11.0,
        });

        for b_idx in 0..4 {
            let bx = -4.5 + b_idx as f32 * 3.1;
            let bz = 0.6 - (b_idx % 2) as f32 * 1.8;
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::MorphoButterfly,
                pos: Vec3::new(bx, 1.8 + 0.4 * b_idx as f32, bz),
                vel: Vec3::new(0.4, 0.1, 0.0),
                scale: 1.0,
                energy_kcal: 1.5,
                max_energy_kcal: 2.0,
                age_hours: 24.0,
                state: BehaviorState::ButterflyPollinationFlutter {
                    target_plant_id: (b_idx * 5 + 3) % underbrush.len(),
                    hover_timer: 1.5,
                },
                pose: ArticulatedPose {
                    wing_phase_amp: [b_idx as f32 * 1.3, 1.0],
                    ..Default::default()
                },
                vocal_cooldown_sec: 99.0,
            });
        }

        for a_idx in 0..22 {
            let frac = (a_idx as f32 + 0.5) / 22.0;
            let base = ant_nest_pos.lerp(patch_a, frac);
            let returning = a_idx % 2 == 0;
            let ax = base.x + rng.gen_range(-0.22..0.22);
            let az = base.z + (frac * PI * 3.0).sin() * 0.24 + rng.gen_range(-0.08..0.08);
            organisms.push(Organism {
                id: alloc_id(),
                species: Species::LeafcutterAnt,
                pos: Vec3::new(ax, ground_height_at(ax, az), az),
                vel: Vec3::ZERO,
                scale: 1.0,
                energy_kcal: 0.4,
                max_energy_kcal: 0.6,
                age_hours: 12.0,
                state: if returning {
                    BehaviorState::AntReturningWithLeaf {
                        nest_pos: ant_nest_pos,
                    }
                } else {
                    BehaviorState::AntOutwardTrail {
                        patch_pos: patch_a,
                        wandering: 0.0,
                    }
                },
                pose: ArticulatedPose {
                    body_yaw: if returning { PI } else { 0.0 },
                    carrying_item: returning,
                    limb_phase: a_idx as f32 * 0.9,
                    ..Default::default()
                },
                vocal_cooldown_sec: 99.0,
            });
        }

        for (m_idx, &(mx, mz, sp)) in [
            (-9.2, 1.1, Species::AmanitaFungus),
            (-3.4, 0.9, Species::MycenaBiolumFungus),
            (2.8, 1.2, Species::AmanitaFungus),
            (6.9, 0.7, Species::MycenaBiolumFungus),
            (10.5, 0.9, Species::MycenaBiolumFungus),
            (-6.1, -1.4, Species::AmanitaFungus),
        ]
        .iter()
        .enumerate()
        {
            organisms.push(Organism {
                id: alloc_id(),
                species: sp,
                pos: Vec3::new(mx, ground_height_at(mx, mz) - 0.02, mz),
                vel: Vec3::ZERO,
                scale: 0.85 + 0.12 * (m_idx % 3) as f32,
                energy_kcal: 3.2,
                max_energy_kcal: 5.0,
                age_hours: 36.0,
                state: BehaviorState::FungalFruiting {
                    maturity: 0.72 + 0.05 * m_idx as f32,
                    spore_release_timer: 8.0 + m_idx as f32 * 2.0,
                },
                pose: ArticulatedPose::default(),
                vocal_cooldown_sec: 99.0,
            });
        }

        let mut events = StochasticEventScheduler::default();
        let mut diurnal = DiurnalCycle::default();
        diurnal.hour_of_day = 8.35;
        events.record(
            1,
            8.35,
            "Rainforest Diorama Initialized",
            "Morning sunbeams filter into the understory; Atta ants harvest fallen leaves while a black panther stalks the ferns.",
        );

        Self {
            biome: BiomeKind::NeotropicalRainforest,
            biome_name: "Neotropical Lowland Rainforest (Side Diorama)".to_string(),
            sim_time_sec: 0.0,
            rng_seed: seed,
            focus: TrophicFocus::AllStrata,
            diurnal,
            ledger: TrophicEnergyLedger::new_rainforest_baseline(),
            floor_field,
            trees,
            kelp_stalks: Vec::new(),
            branches,
            lianas,
            underbrush,
            falling_detritus: Vec::new(),
            ant_nest_pos,
            ant_patches,
            organisms,
            events,
            pending_sounds: Vec::new(),
            next_entity_id: next_id,
            ambient_chorus_timer: 3.5,
        }
    }

    pub fn trigger_event(&mut self, kind: StochasticEventKind) {
        let mut rng = ChaCha8Rng::seed_from_u64(
            self.rng_seed
                .wrapping_add((self.sim_time_sec * 1000.0) as u64)
                .wrapping_add(kind as u64 * 7919),
        );
        let day = self.diurnal.day_count;
        let hr = self.diurnal.hour_of_day;

        match kind {
            // === NE Pacific Rocky Intertidal Tidepool Stochastic Events ===
            StochasticEventKind::TidepoolHighSurfSurge => {
                self.events.active_rain_remaining_sec = 10.0;
                self.diurnal.rain_intensity = 0.85;
                self.pending_sounds.push(SoundCue { kind: SoundCueKind::SurfSurgeBreaker, world_pos: Vec3::new(2.0, 3.2, 0.0), gain: 0.85, pitch_mul: 1.0 });
                self.events.record(day, hr, kind.title(), "Foaming Pacific swell crashes across the basalt bench, flexing rubbery Postelsia sea palms & oxygenating every tidepool!");
            }
            StochasticEventKind::PisasterMusselForage => {
                self.ledger.mineral_nutrient_pool_mg = (self.ledger.mineral_nutrient_pool_mg + 18.0).min(260.0);
                self.pending_sounds.push(SoundCue { kind: SoundCueKind::TidepoolBubbleTrickle, world_pos: Vec3::new(-3.5, 2.2, 0.2), gain: 0.72, pitch_mul: 1.0 });
                self.events.record(day, hr, kind.title(), "Pisaster ochraceus extrudes its cardiac stomach over mid-zone Mytilus mussels, maintaining rock-bench biodiversity (Paine 1966).");
            }
            StochasticEventKind::SculpinPoolToPoolLeap => {
                for org in &mut self.organisms {
                    if let BehaviorState::SculpinTidepoolHop { pool_a, pool_b, at_b, .. } = org.state {
                        org.state = BehaviorState::SculpinTidepoolHop { pool_a, pool_b, leap_timer: 1.8, at_b: !at_b };
                    }
                }
                self.pending_sounds.push(SoundCue { kind: SoundCueKind::SalmonWeirSplash, world_pos: Vec3::new(-3.2, 2.0, 0.2), gain: 0.72, pitch_mul: 1.35 });
                self.events.record(day, hr, kind.title(), "Oligocottus tidepool sculpins vault accurately across exposed coralline rock from upper to middle pools!");
            }
            StochasticEventKind::OystercatcherLowTideProbe => {
                self.pending_sounds.push(SoundCue { kind: SoundCueKind::BlackOystercatcherPipe, world_pos: Vec3::new(-9.5, 3.8, -0.3), gain: 0.84, pitch_mul: 1.0 });
                self.events.record(day, hr, kind.title(), "Haematopus bachmani pipes sharply along the exposed basalt bench and chisels limpets & mussels with its vermilion bill.");
            }
            StochasticEventKind::AnemoneSubmergeBloom => {
                for org in &mut self.organisms {
                    if org.species == Species::GiantGreenAnemone {
                        org.state = BehaviorState::SessileMantleFilter { open_fraction: 1.0 };
                    }
                }
                self.events.record(day, hr, kind.title(), "Flooding emerald tidewater triggers Anthopleura xanthogrammica crowns to unfurl radiant Symbiodinium-green tentacles.");
            }
            StochasticEventKind::HermitCrabWrackScavenge => {
                self.ant_patches = vec![AntForagePatch { pos: Vec3::new(-1.8, 1.8, 0.5), remaining_fragments: 20 }];
                self.ledger.add_external_detritus_g(24.0);
                self.events.record(day, hr, kind.title(), "Pagurus hermit crabs & 8-plated Black Katy chitons converge on newly stranded bull-kelp wrack.");
            }

            // === Cold-Seep Brine Pool & Methane Ice Stochastic Events ===
            StochasticEventKind::MethaneHydrateBubblePlume => {
                self.events.active_rain_remaining_sec = 12.0;
                self.diurnal.rain_intensity = 0.90;
                self.ledger.mineral_nutrient_pool_mg = (self.ledger.mineral_nutrient_pool_mg + 34.0).min(340.0);
                self.pending_sounds.push(SoundCue { kind: SoundCueKind::MethaneClathrateBubbles, world_pos: Vec3::new(-8.2, 2.4, 0.2), gain: 0.82, pitch_mul: 1.0 });
                self.events.record(day, hr, kind.title(), "Streams of buoyant CH4 bubbles fizz upward from golden methane-hydrate mounds, fueling methanotrophic mussel symbionts!");
            }
            StochasticEventKind::BrineShorelineInternalWave => {
                self.pending_sounds.push(SoundCue { kind: SoundCueKind::BrinePoolDensityHum, world_pos: Vec3::new(2.4, BRINE_POOL_SURFACE_Y, 0.0), gain: 0.80, pitch_mul: 1.0 });
                self.events.record(day, hr, kind.title(), "Slow internal halocline waves roll across the mirror-like 120-PSU hypersaline underwater lake shoreline!");
            }
            StochasticEventKind::CutthroatEelBrineSkim => {
                for org in &mut self.organisms {
                    if let BehaviorState::CutthroatEelBrineCruiser { center, phase, .. } = org.state {
                        org.state = BehaviorState::CutthroatEelBrineCruiser { center, skim_timer: 4.5, phase };
                    }
                }
                self.events.record(day, hr, kind.title(), "Synaphobranchus cutthroat eel dips along the anoxic brine chemocline to snatch stunned crustaceans, then darts back into oxygenated water!");
            }
            StochasticEventKind::IceWormClathrateGraze => {
                self.ant_patches = vec![AntForagePatch { pos: Vec3::new(-8.0, 2.2, 0.3), remaining_fragments: 22 }];
                self.events.record(day, hr, kind.title(), "Pink Sirsoe methanicola polychaete ice worms undulate inside carved hollows on golden methane clathrate ice.");
            }
            StochasticEventKind::LamellibrachiaRootUptake => {
                self.ledger.add_chemosynthetic_gpp_pulse(18.0);
                self.events.record(day, hr, kind.title(), "250-year-old Lamellibrachia luymesi tubeworms pump subsurface hydrogen sulfide up into chemoautotrophic gill plumes.");
            }
            StochasticEventKind::RedCrabMusselPatrol => {
                self.events.record(day, hr, kind.title(), "Chaceon deep-sea red crabs stalk the Bathymodiolus mussel beach encircling the hypersaline brine pool.");
            }

            // === Midwater Pelagic Rift Canyon Stochastic Events ===
            StochasticEventKind::DielVerticalMigrationSurge => {
                for org in &mut self.organisms {
                    if let BehaviorState::DvmVerticalMigrator { day_y, night_y, home_xz, phase, .. } = org.state {
                        org.state = BehaviorState::DvmVerticalMigrator {
                            day_y,
                            night_y,
                            home_xz,
                            phase,
                            dvm_boost: 7.5,
                        };
                    }
                }
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::DslBiolumShimmer,
                    world_pos: Vec3::new(0.0, 8.2, 0.0),
                    gain: 0.76,
                    pitch_mul: 1.0,
                });
                self.events.record(day, hr, kind.title(), "Counter-illuminating Myctophid lanternfish, silver hatchetfish & krill surge upward through the twilight rift!");
            }
            StochasticEventKind::MarineSnowMucusFall => {
                self.ledger.add_external_detritus_g(38.0);
                self.ledger.mineral_nutrient_pool_mg = (self.ledger.mineral_nutrient_pool_mg + 28.0).min(300.0);
                self.ant_patches = vec![AntForagePatch { pos: Vec3::new(2.5, 2.4, 0.4), remaining_fragments: 24 }];
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::PelagicCanyonCurrent,
                    world_pos: Vec3::new(-1.0, 4.0, 0.2),
                    gain: 0.72,
                    pitch_mul: 1.0,
                });
                self.events.record(day, hr, kind.title(), "Bathochordaeus larvaceans shed carbon-rich mucus sink-houses into the bathypelagic canyon snow flux.");
            }
            StochasticEventKind::VampyroteuthisMucusCloud => {
                for org in &mut self.organisms {
                    if org.species == Species::VampireSquid {
                        if let BehaviorState::BiolumPelagicPulse { drift_center, .. } = org.state {
                            org.state = BehaviorState::BiolumPelagicPulse {
                                drift_center,
                                flash_timer: 5.5,
                            };
                        }
                    }
                }
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::DslBiolumShimmer,
                    world_pos: Vec3::new(-2.2, 2.5, 0.5),
                    gain: 0.74,
                    pitch_mul: 0.85,
                });
                self.events.record(day, hr, kind.title(), "Vampyroteuthis infernalis inverts its webbed arms into pineapple posture and releases a calm sapphire bioluminescent mucus cloud!");
            }
            StochasticEventKind::BarreleyeDomeTargetLock => {
                for org in &mut self.organisms {
                    if org.species == Species::PacificBarreleye {
                        if let BehaviorState::BarreleyeTubularScan { hover_anchor, .. } = org.state {
                            org.state = BehaviorState::BarreleyeTubularScan {
                                hover_anchor,
                                lock_timer: 5.2,
                            };
                        }
                    } else if org.species == Species::PrayaGiantSiphonophore {
                        if let BehaviorState::BiolumPelagicPulse { drift_center, .. } = org.state {
                            org.state = BehaviorState::BiolumPelagicPulse {
                                drift_center,
                                flash_timer: 4.5,
                            };
                        }
                    }
                }
                self.events.record(day, hr, kind.title(), "Macropinna microstoma rotates its emerald tubular eyes forward inside its clear dome to pick prey off the glowing Praya siphonophore!");
            }
            StochasticEventKind::SpermWhaleSonarDive => {
                for org in &mut self.organisms {
                    if org.species == Species::DivingSpermWhale {
                        if let BehaviorState::SpermWhaleEcholocationDive { cruise_y, .. } = org.state {
                            org.state = BehaviorState::SpermWhaleEcholocationDive {
                                cruise_y,
                                dive_timer: 6.5,
                                sonar_pulse: 1.0,
                            };
                        }
                    }
                }
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::SpermWhaleClickCoda,
                    world_pos: Vec3::new(0.0, 9.2, -1.0),
                    gain: 0.85,
                    pitch_mul: 1.0,
                });
                self.events.record(day, hr, kind.title(), "Diving sperm whale angles into the bathypelagic chasm, sweeping a spermaceti 1+1+3 click coda down the rift walls!");
            }
            StochasticEventKind::ArchiteuthisJetEncounter => {
                for org in &mut self.organisms {
                    if org.species == Species::ArchiteuthisGiantSquid {
                        if let BehaviorState::ArchiteuthisChasmProwl { home_pos, phase, .. } = org.state {
                            org.state = BehaviorState::ArchiteuthisChasmProwl {
                                home_pos,
                                jet_timer: 4.8,
                                phase,
                            };
                        }
                    }
                }
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::ArchiteuthisMantleJet,
                    world_pos: Vec3::new(1.2, 3.2, -0.5),
                    gain: 0.82,
                    pitch_mul: 1.0,
                });
                self.events.record(day, hr, kind.title(), "Architeuthis dux flares its clubbed feeding tentacles and mantle-jets powerfully across the deep rift chasm!");
            }
            // === PNW Old-Growth & Spawning Salmon Creek Stochastic Events ===
            StochasticEventKind::CohoSalmonReddLeap => {
                for org in &mut self.organisms {
                    if org.species == Species::SpawningCohoSalmon {
                        if let BehaviorState::CohoUpstreamReddAndLeap { redd_pos, phase, .. } = org.state {
                            org.state = BehaviorState::CohoUpstreamReddAndLeap {
                                redd_pos,
                                leap_timer: 2.8,
                                phase,
                            };
                            break;
                        }
                    }
                }
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::SalmonWeirSplash,
                    world_pos: Vec3::new(5.5, PNW_CREEK_SURFACE_Y, 0.0),
                    gain: 0.82,
                    pitch_mul: 1.0,
                });
                self.events.record(day, hr, kind.title(), "Crimson-sided kype-jawed Coho salmon leaps dynamically over the mossy boulder weir!");
            }
            StochasticEventKind::MycorrhizalNitrogenPulse => {
                self.diurnal.sunfleck_boost = 1.0;
                self.ledger.mineral_nutrient_pool_mg = (self.ledger.mineral_nutrient_pool_mg + 42.0).min(280.0);
                self.events.record(day, hr, kind.title(), "Salmon-derived marine 15N pulses leftwards through glowing ectomycorrhizal hyphae into old-growth Douglas-fir roots!");
            }
            StochasticEventKind::DipperUnderwaterWalk => {
                for org in &mut self.organisms {
                    if org.species == Species::AmericanDipper {
                        if let BehaviorState::DipperRiffleSubmergeWalk { rock_perch, .. } = org.state {
                            org.state = BehaviorState::DipperRiffleSubmergeWalk {
                                rock_perch,
                                walk_timer: 5.5,
                            };
                        }
                    }
                }
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::GravelCreekRiffle,
                    world_pos: Vec3::new(5.0, PNW_CREEK_SURFACE_Y, 0.3),
                    gain: 0.72,
                    pitch_mul: 1.05,
                });
                self.events.record(day, hr, kind.title(), "Cinclus mexicanus plunges beneath the riffle surface and walks along the submerged gravel hunting caddisflies!");
            }
            StochasticEventKind::BananaSlugSporeTrail => {
                let sx = rng.gen_range(-5.5..0.5);
                let sz = rng.gen_range(0.55..1.15);
                let sp = Vec3::new(sx, pnw_creek_forest_height_at(sx, sz), sz);
                self.ant_patches = vec![AntForagePatch { pos: sp, remaining_fragments: 20 }];
                for s in 0..16 {
                    let p = self.ant_nest_pos.lerp(sp, s as f32 / 15.0);
                    self.floor_field.deposit_ant_pheromone(Vec2::new(p.x, p.z), 2.2);
                }
                self.events.record(day, hr, kind.title(), "Pacific banana slugs glide along a glistening mucus trail to graze fruiting chanterelles & disperse spores.");
            }
            StochasticEventKind::SpottedOwlDuskGlide => {
                for org in &mut self.organisms {
                    if org.species == Species::NorthernSpottedOwl {
                        if let BehaviorState::SpottedOwlCanopyPerchAndGlide { perch_a, perch_b, at_b, .. } = org.state {
                            org.state = BehaviorState::SpottedOwlCanopyPerchAndGlide {
                                perch_a,
                                perch_b,
                                glide_timer: 4.2,
                                at_b: !at_b,
                            };
                        }
                    }
                }
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::SpottedOwlFourNoteHoot,
                    world_pos: Vec3::new(-4.5, 11.2, -0.8),
                    gain: 0.82,
                    pitch_mul: 1.0,
                });
                self.events.record(day, hr, kind.title(), "Northern spotted owl calls its four-note hoot and glides on silent serrated primaries across the misty fir canopy.");
            }
            StochasticEventKind::PacificDrizzleSunbeam => {
                self.events.active_rain_remaining_sec = 14.0;
                self.diurnal.rain_intensity = 0.65;
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::VariedThrushFlute,
                    world_pos: Vec3::new(-6.0, 4.5, 0.2),
                    gain: 0.72,
                    pitch_mul: 1.0,
                });
                self.events.record(day, hr, kind.title(), "Gentle Coast Range mist-drizzle dampens nurse-log mosses as a varied thrush sings in the understory.");
            }
            // === Deep-Sea Hydrothermal Vent Stochastic Events ===
            StochasticEventKind::BlackSmokerSuperPlume => {
                self.events.active_rain_remaining_sec = 14.0;
                self.diurnal.rain_intensity = 0.88;
                self.ledger.mineral_nutrient_pool_mg = (self.ledger.mineral_nutrient_pool_mg + 38.0).min(300.0);
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::HydrothermalVentRumble,
                    world_pos: Vec3::new(-0.5, 8.6, -1.1),
                    gain: 0.88,
                    pitch_mul: 0.92,
                });
                self.events.record(day, hr, kind.title(), "Superheated 360°C H2S-rich end-member fluid roars from the sulfide spire, fueling Riftia & Rimicaris!");
            }
            StochasticEventKind::GhostSharkAbyssalGlide => {
                for org in &mut self.organisms {
                    if org.species == Species::AbyssalGhostShark {
                        let tx = if org.pos.x < 0.0 { 9.2 } else { -9.2 };
                        org.state = BehaviorState::GhostSharkPectoralFlight {
                            waypoint: Vec3::new(tx, 5.4, 0.35),
                        };
                    }
                }
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::AbyssalBiolumPing,
                    world_pos: Vec3::new(0.0, 6.0, 0.2),
                    gain: 0.65,
                    pitch_mul: 1.0,
                });
                self.events.record(day, hr, kind.title(), "Hydrolagus ghost shark rows its wing-like pectoral fins silently past the glowing smoker orifices.");
            }
            StochasticEventKind::AtollaBiolumBurglarAlarm => {
                for org in &mut self.organisms {
                    if matches!(org.species, Species::AtollaCrownJelly | Species::BiolumCombJelly) {
                        if let BehaviorState::BiolumPelagicPulse { drift_center, .. } = org.state {
                            org.state = BehaviorState::BiolumPelagicPulse {
                                drift_center,
                                flash_timer: 6.5,
                            };
                        }
                    }
                }
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::AbyssalBiolumPing,
                    world_pos: Vec3::new(-3.0, 9.5, -0.5),
                    gain: 0.75,
                    pitch_mul: 1.18,
                });
                self.events.record(day, hr, kind.title(), "Atolla wyvillei triggers a rotating sapphire bioluminescent burglar-alarm ring across the abyss!");
            }
            StochasticEventKind::YetiCrabBacterialDance => {
                for org in &mut self.organisms {
                    if org.species == Species::KiwaYetiCrab {
                        org.pose.limb_phase += 6.0;
                    }
                }
                self.events.record(day, hr, kind.title(), "Kiwa yeti crabs wave their bacteria-furred chelipeds vigorously in warm shimmering diffuse vent flow.");
            }
            StochasticEventKind::VulcanoctopusTubewormHunt => {
                for org in &mut self.organisms {
                    if org.species == Species::VulcanoctopusVent {
                        let tx = if org.pos.x < 0.0 { 4.2 } else { -5.4 };
                        org.state = BehaviorState::OctopusReefJet {
                            target_pos: Vec3::new(tx, vent_field_height_at(tx, 0.2) + 0.55, 0.2),
                        };
                    }
                }
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::UnderwaterBubbleStream,
                    world_pos: Vec3::new(0.0, 1.8, 0.2),
                    gain: 0.68,
                    pitch_mul: 0.85,
                });
                self.events.record(day, hr, kind.title(), "Pearly Vulcanoctopus glides across the Riftia bush, causing nearby tubeworm plumes to retract!");
            }
            StochasticEventKind::SulfideFissureFlocRelease => {
                let fx = rng.gen_range(-2.5..5.2);
                let fz = rng.gen_range(0.55..1.15);
                let fp = Vec3::new(fx, vent_field_height_at(fx, fz), fz);
                self.ant_patches = vec![AntForagePatch { pos: fp, remaining_fragments: 24 }];
                for s in 0..16 {
                    let p = self.ant_nest_pos.lerp(fp, s as f32 / 15.0);
                    self.floor_field.deposit_ant_pheromone(Vec2::new(p.x, p.z), 2.4);
                }
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::TectonicFissureCrack,
                    world_pos: fp,
                    gain: 0.76,
                    pitch_mul: 1.0,
                });
                self.events.record(day, hr, kind.title(), "A micro-seismic basalt crack releases snow-white sulfur-bacterial floc, recruiting Munidopsis squat lobsters.");
            }
            // === Temperate Freshwater Lake Stochastic Events ===
            StochasticEventKind::MayflySurfaceHatch => {
                for org in &mut self.organisms {
                    if org.species == Species::RainbowTrout {
                        org.state = BehaviorState::ReefFishForage {
                            waypoint: Vec3::new(org.pos.x.clamp(-5.0, 5.0), LAKE_SURFACE_Y - 0.18, 0.25),
                            pause_sec: 0.5,
                        };
                    }
                }
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::WaterPlungeSplash,
                    world_pos: Vec3::new(0.0, LAKE_SURFACE_Y, 0.2),
                    gain: 0.55,
                    pitch_mul: 1.25,
                });
                self.events.record(day, hr, kind.title(), "Ephemeroptera nymphs emerge at the surface film; rainbow trout rise to sip duns with concentric rings.");
            }
            StochasticEventKind::OspreyPlungeDive => {
                let px = rng.gen_range(-4.5..4.5);
                for org in &mut self.organisms {
                    if org.species == Species::Osprey {
                        org.state = BehaviorState::OspreyAerialSoar {
                            orbit_phase: 0.0,
                            plunge_timer: 4.5,
                            plunge_x: px,
                        };
                    }
                }
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::WaterPlungeSplash,
                    world_pos: Vec3::new(px, LAKE_SURFACE_Y, 0.0),
                    gain: 0.85,
                    pitch_mul: 0.95,
                });
                self.events.record(day, hr, kind.title(), "Pandion haliaetus folds its M-kinked wings and plunges talons-first into the lake surface!");
            }
            StochasticEventKind::CommonLoonSubmergeDive => {
                for org in &mut self.organisms {
                    if org.species == Species::CommonLoon {
                        org.state = BehaviorState::WaterbirdSurfaceSwim {
                            target_x: rng.gen_range(-6.0..6.0),
                            target_z: 0.2,
                            dabble_timer: 0.0,
                            dive_timer: 6.5,
                        };
                    }
                }
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::LoonTremoloWail,
                    world_pos: Vec3::new(0.0, LAKE_SURFACE_Y + 0.3, 0.2),
                    gain: 0.78,
                    pitch_mul: 1.0,
                });
                self.events.record(day, hr, kind.title(), "Common loon arches cleanly beneath the surface to pursue golden shiners through the Vallisneria.");
            }
            StochasticEventKind::MallardDabbleRipple => {
                for org in &mut self.organisms {
                    if org.species == Species::MallardDuck {
                        if let BehaviorState::WaterbirdSurfaceSwim { target_x, target_z, .. } = org.state {
                            org.state = BehaviorState::WaterbirdSurfaceSwim {
                                target_x,
                                target_z,
                                dabble_timer: 4.0,
                                dive_timer: 0.0,
                            };
                        }
                    }
                }
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::MallardQuack,
                    world_pos: Vec3::new(1.5, LAKE_SURFACE_Y + 0.2, 0.6),
                    gain: 0.72,
                    pitch_mul: 1.0,
                });
                self.events.record(day, hr, kind.title(), "Mallard drake and hen tip tail-up at the lily-pad edge to dabble pondweed seeds.");
            }
            StochasticEventKind::CrayfishLeafPackPlume => {
                let lx = rng.gen_range(-2.5..5.0);
                let lz = rng.gen_range(0.55..1.15);
                let lp = Vec3::new(lx, lake_bed_height_at(lx, lz), lz);
                self.ant_patches = vec![AntForagePatch { pos: lp, remaining_fragments: 22 }];
                for s in 0..16 {
                    let p = self.ant_nest_pos.lerp(lp, s as f32 / 15.0);
                    self.floor_field.deposit_ant_pheromone(Vec2::new(p.x, p.z), 2.2);
                }
                self.events.record(day, hr, kind.title(), "Waterlogged alder leaves settle on the cobble bed, recruiting signal crayfish along a dissolved plume.");
            }
            StochasticEventKind::FireflyLoonNocturne => {
                self.diurnal.sunfleck_boost = 0.9;
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::LoonTremoloWail,
                    world_pos: Vec3::new(-2.0, LAKE_SURFACE_Y + 0.3, 0.2),
                    gain: 0.80,
                    pitch_mul: 0.96,
                });
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::BullfrogJugORum,
                    world_pos: Vec3::new(-7.5, LAKE_SURFACE_Y, 0.7),
                    gain: 0.74,
                    pitch_mul: 1.0,
                });
                self.events.record(day, hr, kind.title(), "A haunting loon wail and bullfrog bass echo across the mirror lake as shoreline fireflies twinkle.");
            }
            // === Coral Reef Stochastic Events ===
            StochasticEventKind::LagoonTidalFlush => {
                self.events.active_rain_remaining_sec = 14.0;
                self.diurnal.rain_intensity = 0.65;
                self.ledger.mineral_nutrient_pool_mg = (self.ledger.mineral_nutrient_pool_mg + 24.0).min(220.0);
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::OceanSwellSurge,
                    world_pos: Vec3::new(0.0, 9.0, -2.0),
                    gain: 0.68,
                    pitch_mul: 1.08,
                });
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "Pelagic tide flushes clear oxygenated water across the reef crest, swaying gorgonian fans and anemones.",
                );
            }
            StochasticEventKind::ParrotfishSandCloud => {
                let mut pf_pos = Vec3::new(1.5, 2.8, 0.1);
                for org in &self.organisms {
                    if org.species == Species::SteepheadParrotfish {
                        pf_pos = org.pos;
                        break;
                    }
                }
                for _ in 0..7 {
                    let id = self.next_entity_id;
                    self.next_entity_id += 1;
                    self.falling_detritus.push(FallingDetritus {
                        id,
                        pos: pf_pos + Vec3::new(rng.gen_range(-0.4..0.4), -0.1, rng.gen_range(-0.3..0.3)),
                        vel: Vec3::new(rng.gen_range(-0.3..0.3), -0.2, 0.0),
                        radius: 0.08,
                        energy_kcal: 1.2,
                        kind: DetritusKind::CarbonateCoralSand,
                        settled_on_ground: false,
                        remaining_mass: 1.0,
                    });
                }
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::ParrotfishCoralCrunch,
                    world_pos: pf_pos,
                    gain: 0.66,
                    pitch_mul: 1.0,
                });
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "A steephead parrotfish scrapes algal turf with its fused beak, releasing fresh white aragonite reef sand.",
                );
            }
            StochasticEventKind::HermitCrabMucusPlume => {
                let px = rng.gen_range(-4.0..7.0);
                let pz = rng.gen_range(-0.4..1.3);
                let patch_pos = Vec3::new(px, coral_reef_height_at(px, pz), pz);
                self.ant_patches.push(AntForagePatch {
                    pos: patch_pos,
                    remaining_fragments: 22,
                });
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "Sloughed coral mucus settles in the sand channel, recruiting electric-blue hermit crabs and Linckia sea stars.",
                );
            }
            StochasticEventKind::SeaTurtleSurfaceBreath => {
                for org in &mut self.organisms {
                    if org.species == Species::GreenSeaTurtle {
                        org.state = BehaviorState::SeaTurtleReefFlight {
                            waypoint: Vec3::new(org.pos.x.clamp(-6.0, 6.0), KELP_SURFACE_Y - 0.35, org.pos.z),
                            surfacing_for_breath: true,
                        };
                        self.pending_sounds.push(SoundCue {
                            kind: SoundCueKind::UnderwaterBubbleStream,
                            world_pos: org.pos,
                            gain: 0.65,
                            pitch_mul: 0.92,
                        });
                        break;
                    }
                }
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "The green sea turtle strokes upward on hydrofoil foreflippers to take a breath at the sunlit lagoon surface.",
                );
            }
            StochasticEventKind::BlacktipSharkPass => {
                for org in &mut self.organisms {
                    if org.species == Species::BlacktipReefShark {
                        let nx = if org.pos.x < 0.0 { 10.5 } else { -10.5 };
                        org.state = BehaviorState::SharkReefCruise {
                            waypoint: Vec3::new(nx, 4.6, -0.6),
                        };
                        self.pending_sounds.push(SoundCue {
                            kind: SoundCueKind::KelpFrondSwish,
                            world_pos: org.pos,
                            gain: 0.62,
                            pitch_mul: 1.05,
                        });
                        break;
                    }
                }
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "A blacktip reef shark sweeps low over the Acropora tables; the blue-green chromis cloud darts tight into staghorn shelter!",
                );
            }
            StochasticEventKind::CoralGfpBiofluorescence => {
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::SnappingShrimpCrackle,
                    world_pos: Vec3::new(0.0, 2.0, 0.0),
                    gain: 0.64,
                    pitch_mul: 1.0,
                });
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "Scleractinian corals emit neon GFP/dsRed fluorescence amid warm snapping-shrimp reef crackle.",
                );
            }

            // === Kelp Forest Stochastic Events ===
            StochasticEventKind::CoastalUpwellingPulse => {
                self.events.active_rain_remaining_sec = 16.0;
                self.diurnal.rain_intensity = 0.75; // Surge & cold nitrate upwelling
                self.ledger.mineral_nutrient_pool_mg =
                    (self.ledger.mineral_nutrient_pool_mg + 38.0).min(260.0);
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::OceanSwellSurge,
                    world_pos: Vec3::new(0.0, 8.0, -2.0),
                    gain: 0.72,
                    pitch_mul: 1.0,
                });
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "Cold, nitrate-rich deep water surges onto the rocky shelf, bowing Macrocystis stipes and fueling primary production.",
                );
            }
            StochasticEventKind::DriftKelpDetachment => {
                for _ in 0..6 {
                    let kx = rng.gen_range(-10.5..10.5);
                    let kz = rng.gen_range(-2.5..1.2);
                    let id = self.next_entity_id;
                    self.next_entity_id += 1;
                    self.falling_detritus.push(FallingDetritus {
                        id,
                        pos: Vec3::new(kx, rng.gen_range(9.0..14.0), kz),
                        vel: Vec3::ZERO,
                        radius: 0.18,
                        energy_kcal: 3.5,
                        kind: DetritusKind::DriftKelpBlade,
                        settled_on_ground: false,
                        remaining_mass: 1.0,
                    });
                }
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "Golden-brown drift kelp blades detach from the upper canopy, sinking gently to feed urchins and abalone.",
                );
            }
            StochasticEventKind::CrabScentPlumeWave => {
                let px = rng.gen_range(-4.0..7.0);
                let pz = rng.gen_range(-0.5..1.3);
                let patch_pos = Vec3::new(px, reef_height_at(px, pz), pz);
                self.ant_patches.push(AntForagePatch {
                    pos: patch_pos,
                    remaining_fragments: 20,
                });
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "Sunken kelp detritus releases a dissolved amino-acid plume, drawing shield kelp crabs and bat stars across the reef.",
                );
            }
            StochasticEventKind::SeaOtterUrchinDive => {
                for org in &mut self.organisms {
                    if org.species == Species::SeaOtter {
                        let tx = rng.gen_range(-6.0..6.0);
                        let tz = rng.gen_range(-0.8..0.8);
                        org.state = BehaviorState::OtterForageDive {
                            target_reef_pos: Vec3::new(tx, reef_height_at(tx, tz) + 0.35, tz),
                            ascending: false,
                        };
                        self.pending_sounds.push(SoundCue {
                            kind: SoundCueKind::UnderwaterBubbleStream,
                            world_pos: org.pos,
                            gain: 0.70,
                            pitch_mul: 1.0,
                        });
                        break;
                    }
                }
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "The sea otter rolls from its kelp raft, dives to the rocky reef for a purple sea urchin, and surfaces with its chest stone.",
                );
            }
            StochasticEventKind::HarborSealKelpGlide => {
                for org in &mut self.organisms {
                    if org.species == Species::PacificHarborSeal {
                        let nx = if org.pos.x < 0.0 { 9.5 } else { -9.5 };
                        org.state = BehaviorState::SealKelpGlide {
                            waypoint: Vec3::new(nx, rng.gen_range(5.5..10.5), rng.gen_range(-1.2..0.6)),
                        };
                        self.pending_sounds.push(SoundCue {
                            kind: SoundCueKind::KelpFrondSwish,
                            world_pos: org.pos,
                            gain: 0.65,
                            pitch_mul: 0.95,
                        });
                        break;
                    }
                }
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "A spotted harbor seal banks smoothly between swaying Macrocystis stipes, scattering the blacksmith school.",
                );
            }
            StochasticEventKind::OctopusChromatophoreShift => {
                for org in &mut self.organisms {
                    if org.species == Species::GiantPacificOctopus {
                        let nx = if org.pos.x < 0.0 { 3.2 } else { -8.4 };
                        let nz = rng.gen_range(-0.6..0.7);
                        org.pose.chromatophore = 1.0;
                        org.state = BehaviorState::OctopusReefJet {
                            target_pos: Vec3::new(nx, reef_height_at(nx, nz) + 0.55, nz),
                        };
                        self.pending_sounds.push(SoundCue {
                            kind: SoundCueKind::UnderwaterBubbleStream,
                            world_pos: org.pos,
                            gain: 0.58,
                            pitch_mul: 0.82,
                        });
                        break;
                    }
                }
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "Enteroctopus dofleini flushes its chromatophores coral-crimson and siphon-jets gracefully across the basalt boulders.",
                );
            }
            StochasticEventKind::DinoflagellateBiolumSwirl => {
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::DistantWhaleHydrophone,
                    world_pos: Vec3::new(4.0, 9.0, -10.0),
                    gain: 0.65,
                    pitch_mul: 1.0,
                });
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "Shear-stimulated dinoflagellates ignite cyan bioluminescent trails along swaying kelp blades and swimming predators.",
                );
            }

            // === Rainforest Stochastic Events ===
            StochasticEventKind::TropicalSunshower => {
                self.events.active_rain_remaining_sec = 18.0;
                self.diurnal.rain_intensity = 0.78;
                self.diurnal.cloud_cover = 0.72;
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::RainShower,
                    world_pos: Vec3::new(0.0, 8.0, -2.0),
                    gain: 0.75,
                    pitch_mul: 1.0,
                });
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::DistantThunder,
                    world_pos: Vec3::new(-8.0, 14.0, -12.0),
                    gain: 0.55,
                    pitch_mul: 0.92,
                });
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "Rain streaks drench canopy leaves, accelerating microbial mineralization and washing out volatile ant pheromones.",
                );
            }
            StochasticEventKind::CanopyFruitMastDrop => {
                for i in 0..6 {
                    let bx = rng.gen_range(-7.5..8.5);
                    let bz = rng.gen_range(-2.2..1.0);
                    let id = self.next_entity_id;
                    self.next_entity_id += 1;
                    self.falling_detritus.push(FallingDetritus {
                        id,
                        pos: Vec3::new(bx, rng.gen_range(9.5..13.2), bz),
                        vel: Vec3::new(rng.gen_range(-0.5..0.5), 0.0, 0.0),
                        radius: 0.14,
                        energy_kcal: 4.5,
                        kind: if i % 2 == 0 {
                            DetritusKind::FigFruit
                        } else {
                            DetritusKind::RipeCanopyBerry
                        },
                        settled_on_ground: false,
                        remaining_mass: 1.0,
                    });
                }
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "Ripe figs and berries detach from upper boughs, bouncing through branches onto the forest floor.",
                );
            }
            StochasticEventKind::AntTrailForagingWave => {
                let px = rng.gen_range(-3.5..7.5);
                let pz = rng.gen_range(-0.6..1.4);
                let patch_pos = Vec3::new(px, ground_height_at(px, pz), pz);
                self.ant_patches.push(AntForagePatch {
                    pos: patch_pos,
                    remaining_fragments: 28,
                });
                for org in &mut self.organisms {
                    if org.species == Species::LeafcutterAnt
                        && matches!(org.state, BehaviorState::AntOutwardTrail { .. })
                    {
                        org.state = BehaviorState::AntOutwardTrail {
                            patch_pos,
                            wandering: 0.0,
                        };
                    }
                }
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "Atta scouts establish a fresh volatile pheromone trail toward newly fallen canopy leaves.",
                );
            }
            StochasticEventKind::MonkeyCanopyTransit => {
                if !self.lianas.is_empty() {
                    let vine_count = self.lianas.len();
                    for org in &mut self.organisms {
                        if org.species == Species::SquirrelMonkey {
                            let vid = rng.gen_range(0..vine_count);
                            self.lianas[vid].apply_tangential_impulse(3.2, 0.25);
                            org.pos = self.lianas[vid].tip_position();
                            org.state = BehaviorState::MonkeyVineSwing {
                                vine_id: vid,
                                swing_time: 0.0,
                                target_branch_id: 3,
                            };
                            self.pending_sounds.push(SoundCue {
                                kind: SoundCueKind::MonkeyChatter,
                                world_pos: org.pos,
                                gain: 0.78,
                                pitch_mul: rng.gen_range(0.94..1.10),
                            });
                            break;
                        }
                    }
                    self.events.record(
                        day,
                        hr,
                        kind.title(),
                        "A squirrel monkey grabs a hanging liana, swings across the mid-canopy gap, and pauses to scan for predators.",
                    );
                }
            }
            StochasticEventKind::PantherUnderbrushProwl => {
                for org in &mut self.organisms {
                    if org.species == Species::BlackPanther {
                        let next_x = if org.pos.x < 0.0 { 9.2 } else { -9.2 };
                        org.state = BehaviorState::PantherUnderbrushStalk {
                            waypoint_x: next_x,
                            waypoint_z: rng.gen_range(-0.2..0.9),
                            stealth_crouch: 0.88,
                        };
                        self.pending_sounds.push(SoundCue {
                            kind: SoundCueKind::PantherLowRumble,
                            world_pos: org.pos,
                            gain: 0.72,
                            pitch_mul: 0.96,
                        });
                        break;
                    }
                }
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "The melanistic black panther lowers its shoulder blades and glides silently through parting sword ferns.",
                );
            }
            StochasticEventKind::BoaAmbushAlert => {
                for org in &mut self.organisms {
                    if org.species == Species::BoaConstrictor {
                        org.pose.tongue_ext = 1.0;
                        self.pending_sounds.push(SoundCue {
                            kind: SoundCueKind::BoaHiss,
                            world_pos: org.pos,
                            gain: 0.65,
                            pitch_mul: 1.0,
                        });
                        break;
                    }
                }
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "Draped motionless over the upper bough, the boa constrictor samples warm prey plumes with its forked tongue.",
                );
            }
            StochasticEventKind::CanopyWindGust => {
                self.events.active_wind_remaining_sec = 8.0;
                self.events.wind_vector_xz = [2.4, 0.55];
                for vine in &mut self.lianas {
                    vine.apply_tangential_impulse(1.5, 0.35);
                }
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "A warm trade-wind gust sways hanging lianas and shakes loose canopy leaves into the decomposer layer.",
                );
            }
            StochasticEventKind::BioluminescentSporePulse => {
                self.pending_sounds.push(SoundCue {
                    kind: SoundCueKind::TreefrogCoqui,
                    world_pos: Vec3::new(-4.0, 1.5, -1.0),
                    gain: 0.62,
                    pitch_mul: 1.02,
                });
                self.events.record(
                    day,
                    hr,
                    kind.title(),
                    "Saprotrophic Mycena fruiting bodies emit emerald luciferin glow across moist decaying roots.",
                );
            }
        }
    }

    pub fn step(&mut self, dt_sec: f32) {
        let dt = dt_sec.clamp(0.001, 0.10);
        self.sim_time_sec += f64::from(dt);
        let t_sec = self.sim_time_sec as f32;

        let mut rng = ChaCha8Rng::seed_from_u64(
            self.rng_seed
                .wrapping_add((self.sim_time_sec * 10_000.0) as u64),
        );

        if self.events.active_rain_remaining_sec <= 0.0 {
            self.diurnal.rain_intensity = (self.diurnal.rain_intensity - dt * 0.12).max(0.0);
            self.diurnal.cloud_cover += (0.16 - self.diurnal.cloud_cover) * (dt * 0.08);
        }
        self.diurnal.step(dt);
        let atmo = self.diurnal.atmosphere_for_biome(self.biome);
        let is_night = self.diurnal.is_night();

        if let Some(ev) = self.events.poll_next_event(
            self.biome,
            dt,
            self.diurnal.hour_of_day,
            is_night,
            &mut rng,
        ) {
            self.trigger_event(ev);
        }

        // Ambient hydrophone / rainforest chorus cues
        self.ambient_chorus_timer -= dt;
        if self.ambient_chorus_timer <= 0.0 {
            self.ambient_chorus_timer = rng.gen_range(8.5..15.0);
            let kind = if self.biome == BiomeKind::RockyIntertidalTidepool {
                if rng.gen::<f32>() < 0.55 { SoundCueKind::SurfSurgeBreaker } else { SoundCueKind::BlackOystercatcherPipe }
            } else if self.biome == BiomeKind::ColdSeepBrinePool {
                if rng.gen::<f32>() < 0.55 { SoundCueKind::MethaneClathrateBubbles } else { SoundCueKind::BrinePoolDensityHum }
            } else if self.biome == BiomeKind::MidwaterPelagicRift {
                let r = rng.gen::<f32>();
                if r < 0.42 {
                    SoundCueKind::SpermWhaleClickCoda
                } else if r < 0.74 {
                    SoundCueKind::PelagicCanyonCurrent
                } else {
                    SoundCueKind::DslBiolumShimmer
                }
            } else if self.biome == BiomeKind::PnwOldGrowthSalmonCreek {
                if is_night {
                    SoundCueKind::SpottedOwlFourNoteHoot
                } else if rng.gen::<f32>() < 0.52 {
                    SoundCueKind::VariedThrushFlute
                } else {
                    SoundCueKind::GravelCreekRiffle
                }
            } else if self.biome == BiomeKind::DeepSeaHydrothermalVent {
                if rng.gen::<f32>() < 0.68 {
                    SoundCueKind::HydrothermalVentRumble
                } else {
                    SoundCueKind::AbyssalBiolumPing
                }
            } else if self.biome == BiomeKind::TemperateFreshwaterLake {
                if is_night {
                    if rng.gen::<f32>() < 0.55 { SoundCueKind::LoonTremoloWail } else { SoundCueKind::BullfrogJugORum }
                } else if rng.gen::<f32>() < 0.5 {
                    SoundCueKind::MallardQuack
                } else {
                    SoundCueKind::KingfisherRattle
                }
            } else if self.biome == BiomeKind::IndoPacificCoralReef {
                SoundCueKind::SnappingShrimpCrackle
            } else if self.biome == BiomeKind::PacificKelpForest {
                if is_night {
                    SoundCueKind::DistantWhaleHydrophone
                } else {
                    SoundCueKind::OceanSwellSurge
                }
            } else if is_night {
                SoundCueKind::TreefrogCoqui
            } else {
                SoundCueKind::CicadaStridulation
            };
            self.pending_sounds.push(SoundCue {
                kind,
                world_pos: Vec3::new(rng.gen_range(-10.0..10.0), 5.0, -6.0),
                gain: 0.40,
                pitch_mul: rng.gen_range(0.92..1.08),
            });
        }

        let wind_x = if self.biome.is_marine() {
            ocean_swell_vx(0.0, 4.0, t_sec, self.diurnal.rain_intensity)
        } else {
            self.events.wind_vector_xz[0]
        };
        let wind_z = self.events.wind_vector_xz[1];

        for vine in &mut self.lianas {
            vine.step(dt, wind_x, wind_z);
        }
        for stalk in &mut self.kelp_stalks {
            stalk.step(dt);
        }

        let par_norm = if self.biome == BiomeKind::ColdSeepBrinePool {
            // 100% Methanotrophic & thiotrophic cold-seep chemosynthesis at 1,050m (PAR = 0.0)!
            0.86 + 0.28 * self.diurnal.rain_intensity
        } else if self.biome == BiomeKind::MidwaterPelagicRift {
            // Epipelagic marine-snow export flux + upwelling phytodetritus feeding the midwater rift!
            0.72 + 0.25 * atmo.night_factor + 0.22 * self.diurnal.rain_intensity
        } else if self.biome == BiomeKind::DeepSeaHydrothermalVent {
            // Lithoautotrophic chemosynthesis (H2S + O2 -> SO4^2- + ATP) — runs 24/7 at PAR = 0!
            0.88 + 0.32 * self.diurnal.rain_intensity
        } else {
            (atmo.par_umol_m2_s / 1750.0).clamp(0.0, 1.2)
        };
        let mut raw_floral_gpp = 0.0_f32;
        let mut vigor_sum = 0.0_f32;
        for plant in &mut self.underbrush {
            let nut = self
                .floor_field
                .consume_nutrients(Vec2::new(plant.root_pos.x, plant.root_pos.z), 0.008 * dt);
            plant.step(dt, wind_x, par_norm, nut * 40.0 + 0.5);
            raw_floral_gpp += 0.045 * par_norm * plant.vigor * dt;
            vigor_sum += plant.vigor;
        }
        let mean_vigor = if self.underbrush.is_empty() {
            1.0
        } else {
            vigor_sum / self.underbrush.len() as f32
        };
        raw_floral_gpp += 0.48 * par_norm * dt * (self.trees.len() + self.kelp_stalks.len()) as f32;

        let gpp_step_kcal = TrophicEnergyLedger::compute_biome_gpp_step(
            self.biome,
            par_norm,
            mean_vigor,
            self.trees.len(),
            self.kelp_stalks.len(),
            self.underbrush.len(),
            self.ledger.mineral_nutrient_pool_mg,
            dt,
        );
        let activity_drive = if matches!(
            self.biome,
            BiomeKind::ColdSeepBrinePool
                | BiomeKind::MidwaterPelagicRift
                | BiomeKind::DeepSeaHydrothermalVent
        ) {
            1.0
        } else {
            0.08 + 0.92 * (par_norm / 0.85).clamp(0.0, 1.0)
        };

        for item in &mut self.falling_detritus {
            if item.step(dt, &self.branches) && item.settled_on_ground {
                self.floor_field
                    .deposit_detritus(Vec2::new(item.pos.x, item.pos.z), 0.85);
                self.ledger.add_external_detritus_g(1.4);
            }
        }
        self.falling_detritus.retain(|d| d.remaining_mass > 0.05);

        self.floor_field.deposit_home_pheromone(
            Vec2::new(self.ant_nest_pos.x, self.ant_nest_pos.z),
            1.2 * dt,
        );
        let mineralized =
            self.floor_field
                .step(dt, self.diurnal.rain_intensity, self.diurnal.foliage_wetness);
        self.ledger.cumulative_mineralized_mg += mineralized * 12.0;
        self.ledger.mineral_nutrient_pool_mg =
            (self.ledger.mineral_nutrient_pool_mg
                + mineralized * 8.0
                + self.ledger.last_resp_rate_kcal_s[0] * 0.012 * dt
                - raw_floral_gpp * 0.35)
                .clamp(25.0, 280.0);
        self.ant_patches.retain(|p| p.remaining_fragments > 0);

        // Snapshots for predator-prey perception across both biomes
        let boa_snapshot = self
            .organisms
            .iter()
            .find(|o| o.species == Species::BoaConstrictor)
            .map(|o| o.pos);
        let panther_snapshot = self
            .organisms
            .iter()
            .find(|o| o.species == Species::BlackPanther)
            .map(|o| o.pos);
        let monkey_positions: Vec<Vec3> = self
            .organisms
            .iter()
            .filter(|o| o.species == Species::SquirrelMonkey)
            .map(|o| o.pos)
            .collect();
        let marine_predators: Vec<Vec3> = self
            .organisms
            .iter()
            .filter(|o| {
                matches!(
                    o.species,
                    Species::PacificHarborSeal | Species::LeopardShark | Species::BlacktipReefShark | Species::GreenSeaTurtle | Species::AbyssalGhostShark | Species::BluntnoseSixgillShark | Species::VulcanoctopusVent
                )
            })
            .map(|o| o.pos)
            .collect();

        let branches_ref = &self.branches;
        let nest_pos = self.ant_nest_pos;
        let is_marine_biome = self.biome.is_marine();
        let is_coral_biome = self.biome == BiomeKind::IndoPacificCoralReef;
        let is_lake_biome = self.biome == BiomeKind::TemperateFreshwaterLake;
        let is_vent_biome = self.biome == BiomeKind::DeepSeaHydrothermalVent;
        let is_pnw_biome = self.biome == BiomeKind::PnwOldGrowthSalmonCreek;
        let is_rift_biome = self.biome == BiomeKind::MidwaterPelagicRift;
        let is_tidepool_biome = self.biome == BiomeKind::RockyIntertidalTidepool;
        let is_brine_biome = self.biome == BiomeKind::ColdSeepBrinePool;
        self.ledger
            .step_trophic_ode(self.biome, gpp_step_kcal, activity_drive, dt);
        let night_f = atmo.night_factor;

        for idx in 0..self.organisms.len() {
            let mut org = self.organisms[idx].clone();
            org.age_hours += (dt * self.diurnal.minutes_per_real_sec) / 60.0;
            org.vocal_cooldown_sec = (org.vocal_cooldown_sec - dt).max(0.0);
            org.pose.tongue_ext = (org.pose.tongue_ext - dt * 2.4).max(0.0);

            let bmr_kcal = 0.0018 * org.species.typical_mass_kg().powf(0.75) * dt;
            org.energy_kcal = (org.energy_kcal - bmr_kcal).max(0.2 * org.max_energy_kcal);

            match org.state.clone() {
                // ================= PACIFIC KELP FOREST ETHOLOGY =================
                BehaviorState::OtterSurfaceRaft {
                    mut remaining_sec,
                    anchor_x,
                    anchor_z,
                } => {
                    remaining_sec -= dt;
                    let swell_x = ocean_swell_vx(anchor_x, KELP_SURFACE_Y, t_sec, self.diurnal.rain_intensity);
                    let bob_y = (t_sec * 1.4 + anchor_x).sin() * 0.14;
                    org.pos = Vec3::new(anchor_x + swell_x * 0.45, KELP_SURFACE_Y - 0.52 + bob_y, anchor_z);
                    org.vel = Vec3::new(swell_x * 0.25, 0.0, 0.0);
                    org.pose.carrying_item = false;
                    org.pose.limb_phase += dt * 1.8;
                    org.pose.head_yaw = (t_sec * 0.9).sin() * 0.35;

                    if remaining_sec <= 0.0 {
                        let tx = rng.gen_range(-6.5..6.5);
                        let tz = rng.gen_range(-0.9..0.8);
                        org.state = BehaviorState::OtterForageDive {
                            target_reef_pos: Vec3::new(tx, reef_height_at(tx, tz) + 0.35, tz),
                            ascending: false,
                        };
                    } else {
                        org.state = BehaviorState::OtterSurfaceRaft {
                            remaining_sec,
                            anchor_x,
                            anchor_z,
                        };
                    }
                }

                BehaviorState::OtterForageDive {
                    target_reef_pos,
                    mut ascending,
                } => {
                    let goal = if ascending {
                        Vec3::new(target_reef_pos.x, KELP_SURFACE_Y - 0.52, target_reef_pos.z)
                    } else {
                        target_reef_pos
                    };
                    let diff = goal - org.pos;
                    let dist = diff.length();
                    let dive_spd = 2.35;

                    if dist > 0.38 {
                        org.vel = (diff / dist) * dive_spd;
                        org.pos += org.vel * dt;
                        org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                        org.pose.body_pitch = (org.vel.y / dive_spd).asin().clamp(-1.1, 1.1);
                        org.pose.spine_phase += dt * 7.5;
                        org.pose.limb_phase += dt * 7.5;
                        for stalk in &mut self.kelp_stalks {
                            let _ = stalk.apply_swimmer_contact(org.pos, org.vel, 0.52);
                        }
                        org.state = BehaviorState::OtterForageDive {
                            target_reef_pos,
                            ascending,
                        };
                    } else if !ascending {
                        ascending = true;
                        org.pose.carrying_item = true; // Collected a purple sea urchin!
                        org.state = BehaviorState::OtterForageDive {
                            target_reef_pos,
                            ascending,
                        };
                    } else {
                        org.vel = Vec3::ZERO;
                        org.pose.body_pitch = 0.0;
                        org.energy_kcal = (org.energy_kcal + 5.5).min(org.max_energy_kcal);
                        org.state = BehaviorState::OtterChestStoneTap {
                            remaining_sec: rng.gen_range(4.5..7.2),
                            tap_cooldown: 0.4,
                        };
                    }
                }

                BehaviorState::OtterChestStoneTap {
                    mut remaining_sec,
                    mut tap_cooldown,
                } => {
                    remaining_sec -= dt;
                    tap_cooldown -= dt;
                    let bob_y = (t_sec * 1.5).sin() * 0.12;
                    org.pos.y = KELP_SURFACE_Y - 0.52 + bob_y;
                    org.vel = Vec3::ZERO;
                    org.pose.carrying_item = true;
                    org.pose.limb_phase += dt * 14.0; // Forepaws tapping urchin on chest stone!

                    if tap_cooldown <= 0.0 {
                        tap_cooldown = 1.6;
                        self.pending_sounds.push(SoundCue {
                            kind: SoundCueKind::OtterChestStoneClick,
                            world_pos: org.pos,
                            gain: 0.62,
                            pitch_mul: rng.gen_range(0.95..1.08),
                        });
                    }

                    if remaining_sec <= 0.0 {
                        org.pose.carrying_item = false;
                        org.state = BehaviorState::OtterSurfaceRaft {
                            remaining_sec: rng.gen_range(6.0..11.0),
                            anchor_x: org.pos.x.clamp(-10.0, 10.0),
                            anchor_z: org.pos.z,
                        };
                    } else {
                        org.state = BehaviorState::OtterChestStoneTap {
                            remaining_sec,
                            tap_cooldown,
                        };
                    }
                }

                BehaviorState::SealKelpGlide { waypoint } => {
                    let diff = waypoint - org.pos;
                    let dist = diff.length();
                    let speed = 1.85;
                    if dist > 0.5 {
                        let dir = diff / dist;
                        org.vel = org.vel.lerp(dir * speed, (dt * 3.0).min(1.0));
                        org.pos += org.vel * dt;
                        org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                        org.pose.body_pitch = (org.vel.y / speed.max(0.1)).clamp(-0.55, 0.55);
                        org.pose.spine_phase += dt * 5.8;
                        org.pose.limb_phase = org.pose.spine_phase;

                        // Part buoyant Giant Kelp stipes & fronds as the harbor seal glides through!
                        let mut max_swish = 0.0_f32;
                        for stalk in &mut self.kelp_stalks {
                            let hit = stalk.apply_swimmer_contact(org.pos, org.vel, 0.72);
                            if hit > max_swish {
                                max_swish = hit;
                            }
                        }
                        if max_swish > 0.45 && org.vocal_cooldown_sec <= 0.0 {
                            org.vocal_cooldown_sec = 4.2;
                            self.pending_sounds.push(SoundCue {
                                kind: SoundCueKind::KelpFrondSwish,
                                world_pos: org.pos,
                                gain: 0.44,
                                pitch_mul: rng.gen_range(0.9..1.1),
                            });
                        }

                        if rng.gen::<f32>() < 0.08 * dt {
                            org.state = BehaviorState::SealMidwaterLookAround {
                                remaining_sec: rng.gen_range(2.5..4.5),
                                scan_seed: rng.gen_range(0.0..6.28),
                            };
                        } else {
                            org.state = BehaviorState::SealKelpGlide { waypoint };
                        }
                    } else {
                        org.state = BehaviorState::SealMidwaterLookAround {
                            remaining_sec: rng.gen_range(2.8..4.8),
                            scan_seed: rng.gen_range(0.0..6.28),
                        };
                    }
                }

                BehaviorState::SealMidwaterLookAround {
                    mut remaining_sec,
                    scan_seed,
                } => {
                    remaining_sec -= dt;
                    let swell_v = ocean_swell_vx(org.pos.x, org.pos.y, t_sec, self.diurnal.rain_intensity);
                    org.vel = Vec3::new(swell_v * 0.35, (t_sec * 1.2).sin() * 0.12, 0.0);
                    org.pos += org.vel * dt;
                    org.pose.body_pitch *= 0.92;
                    let scan = t_sec * 2.2 + scan_seed;
                    org.pose.head_yaw = scan.sin() * 0.72;
                    org.pose.head_pitch = (scan * 1.3).cos() * 0.28;
                    org.pose.spine_phase += dt * 1.6;

                    if remaining_sec <= 0.0 {
                        org.pose.head_yaw = 0.0;
                        org.pose.head_pitch = 0.0;
                        let nx = if org.pos.x > 0.0 {
                            rng.gen_range(-11.0..-4.0)
                        } else {
                            rng.gen_range(4.0..11.0)
                        };
                        org.state = BehaviorState::SealKelpGlide {
                            waypoint: Vec3::new(nx, rng.gen_range(4.8..11.8), rng.gen_range(-2.2..0.8)),
                        };
                    } else {
                        org.state = BehaviorState::SealMidwaterLookAround {
                            remaining_sec,
                            scan_seed,
                        };
                    }
                }

                BehaviorState::SharkReefCruise { mut waypoint } => {
                    let diff = waypoint - org.pos;
                    let dist = diff.length();
                    let speed = 1.42;
                    if dist > 0.55 {
                        let dir = diff / dist;
                        org.vel = org.vel.lerp(dir * speed, (dt * 2.5).min(1.0));
                        org.pos += org.vel * dt;
                        let min_y = reef_height_at(org.pos.x, org.pos.z) + 1.2;
                        org.pos.y = org.pos.y.max(min_y);
                        org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                        org.pose.spine_phase += dt * 6.4; // Sinuous carangiform S-wave!
                        for stalk in &mut self.kelp_stalks {
                            let _ = stalk.apply_swimmer_contact(org.pos, org.vel, 0.65);
                        }
                    } else {
                        let nx = if org.pos.x < 0.0 {
                            rng.gen_range(5.5..11.5)
                        } else {
                            rng.gen_range(-11.5..-5.5)
                        };
                        waypoint = Vec3::new(nx, rng.gen_range(2.2..5.2), rng.gen_range(-2.4..0.6));
                    }
                    org.state = BehaviorState::SharkReefCruise { waypoint };
                }

                BehaviorState::WolfEelCreviceAmbush {
                    den_pos,
                    mut lunge_progress,
                    lunge_target,
                } => {
                    org.pose.spine_phase += dt * 2.2;
                    if lunge_progress > 0.0 {
                        lunge_progress += dt * 2.2;
                        let f = (lunge_progress.min(1.0) * PI).sin();
                        org.pos = den_pos.lerp(lunge_target, f * 0.78);
                        if lunge_progress >= 1.0 {
                            lunge_progress = 0.0;
                            org.pos = den_pos;
                        }
                    } else if rng.gen::<f32>() < 0.03 * dt {
                        lunge_progress = 0.02;
                    }
                    org.state = BehaviorState::WolfEelCreviceAmbush {
                        den_pos,
                        lunge_progress,
                        lunge_target,
                    };
                }

                BehaviorState::OctopusDenCamouflage {
                    den_pos,
                    mut remaining_sec,
                } => {
                    remaining_sec -= dt;
                    org.pos = den_pos;
                    org.vel = Vec3::ZERO;
                    org.pose.chromatophore += (0.12 - org.pose.chromatophore) * (dt * 2.0);
                    org.pose.limb_phase += dt * 1.4;
                    if remaining_sec <= 0.0 {
                        let nx = rng.gen_range(-8.5..6.5);
                        let nz = rng.gen_range(-0.9..0.9);
                        org.state = BehaviorState::OctopusReefJet {
                            target_pos: Vec3::new(nx, reef_height_at(nx, nz) + 0.55, nz),
                        };
                    } else {
                        org.state = BehaviorState::OctopusDenCamouflage {
                            den_pos,
                            remaining_sec,
                        };
                    }
                }

                BehaviorState::OctopusReefJet { target_pos } => {
                    let diff = target_pos - org.pos;
                    let dist = diff.length();
                    // Pulsatile mantle-siphon jetting velocity!
                    let pulse = 0.55 + 0.85 * (t_sec * 3.8).sin().max(0.0);
                    let jet_spd = 1.35 * pulse;
                    org.pose.chromatophore += (0.95 - org.pose.chromatophore) * (dt * 3.0);
                    org.pose.limb_phase += dt * 4.5;

                    if dist > 0.42 {
                        org.vel = (diff / dist) * jet_spd;
                        org.pos += org.vel * dt;
                        org.pos.y = org.pos.y.max(reef_height_at(org.pos.x, org.pos.z) + 0.38);
                        org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                        org.state = BehaviorState::OctopusReefJet { target_pos };
                    } else {
                        let new_den = Vec3::new(
                            org.pos.x,
                            reef_height_at(org.pos.x, org.pos.z) + 0.40,
                            org.pos.z,
                        );
                        org.state = BehaviorState::OctopusDenCamouflage {
                            den_pos: new_den,
                            remaining_sec: rng.gen_range(7.0..13.0),
                        };
                    }
                }

                BehaviorState::ReefFishForage {
                    mut waypoint,
                    mut pause_sec,
                } => {
                    org.pose.wing_phase_amp = [t_sec * 12.0, 1.0]; // Labriform pectoral fin rowing!
                    org.pose.spine_phase += dt * 5.0;
                    if pause_sec > 0.0 {
                        pause_sec -= dt;
                        org.vel *= 0.85;
                    } else {
                        let diff = waypoint - org.pos;
                        let dist = diff.length();
                        let spd = 1.05;
                        if dist > 0.35 {
                            org.vel = (diff / dist) * spd;
                            org.pos += org.vel * dt;
                            org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                        } else {
                            waypoint = Vec3::new(
                                rng.gen_range(-9.5..9.5),
                                rng.gen_range(2.2..7.5),
                                rng.gen_range(-2.2..0.8),
                            );
                            pause_sec = rng.gen_range(1.2..2.8);
                        }
                    }
                    org.state = BehaviorState::ReefFishForage {
                        waypoint,
                        pause_sec,
                    };
                }

                BehaviorState::SeaTurtleReefFlight {
                    mut waypoint,
                    mut surfacing_for_breath,
                } => {
                    let diff = waypoint - org.pos;
                    let dist = diff.length();
                    let spd = if surfacing_for_breath { 1.65 } else { 1.25 };
                    org.pose.wing_phase_amp = [t_sec * 3.4, 1.0];
                    if dist > 0.48 {
                        let dir = diff / dist;
                        org.vel = org.vel.lerp(dir * spd, (dt * 2.5).min(1.0));
                        org.pos += org.vel * dt;
                        org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                        org.pose.body_pitch = (org.vel.y / spd).clamp(-0.55, 0.55);
                        for plant in &mut self.underbrush {
                            let _ = plant.apply_body_contact(org.pos, org.vel, 0.85);
                        }
                    } else if surfacing_for_breath {
                        surfacing_for_breath = false;
                        waypoint = Vec3::new(rng.gen_range(-8.5..8.5), rng.gen_range(3.5..9.5), rng.gen_range(-1.8..0.8));
                    } else if rng.gen::<f32>() < 0.22 {
                        surfacing_for_breath = true;
                        waypoint = Vec3::new(org.pos.x.clamp(-7.0, 7.0), KELP_SURFACE_Y - 0.35, org.pos.z);
                    } else {
                        waypoint = Vec3::new(rng.gen_range(-9.5..9.5), rng.gen_range(3.2..10.2), rng.gen_range(-2.2..0.8));
                    }
                    org.state = BehaviorState::SeaTurtleReefFlight {
                        waypoint,
                        surfacing_for_breath,
                    };
                }

                BehaviorState::ClownfishAnemoneSymbiosis {
                    anemone_pos,
                    mut dart_phase,
                } => {
                    dart_phase += dt * 3.1;
                    let radius = 0.42 + 0.22 * (dart_phase * 0.45).sin().max(0.0);
                    let target = anemone_pos + Vec3::new(
                        dart_phase.cos() * radius,
                        (dart_phase * 1.4).sin() * 0.22 + 0.18,
                        (dart_phase * 0.8).sin() * 0.28,
                    );
                    let v = (target - org.pos) * 3.5;
                    org.vel = v;
                    org.pos += org.vel * dt;
                    org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                    org.pose.spine_phase += dt * 14.0;
                    org.state = BehaviorState::ClownfishAnemoneSymbiosis {
                        anemone_pos,
                        dart_phase,
                    };
                }

                BehaviorState::StingraySandUndulate { mut waypoint } => {
                    let diff = Vec3::new(waypoint.x - org.pos.x, 0.0, waypoint.z - org.pos.z);
                    let dist = diff.length();
                    org.pose.spine_phase += dt * 6.5;
                    if dist > 0.45 {
                        let spd = 1.05;
                        org.vel = (diff / dist) * spd;
                        org.pos.x += org.vel.x * dt;
                        org.pos.z += org.vel.z * dt;
                        org.pos.y = coral_reef_height_at(org.pos.x, org.pos.z) + 0.28 + (t_sec * 1.8).sin() * 0.06;
                        org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                    } else {
                        waypoint = Vec3::new(rng.gen_range(-9.5..9.5), 0.5, rng.gen_range(-1.2..1.2));
                    }
                    org.state = BehaviorState::StingraySandUndulate { waypoint };
                }

                BehaviorState::WaterbirdSurfaceSwim {
                    mut target_x,
                    mut target_z,
                    mut dabble_timer,
                    mut dive_timer,
                } => {
                    org.pose.limb_phase += dt * 7.5;
                    if dive_timer > 0.0 {
                        dive_timer = (dive_timer - dt).max(0.0);
                        let sub_y = LAKE_SURFACE_Y - 4.2 * ((dive_timer / 6.5) * PI).sin();
                        org.vel = Vec3::new((target_x - org.pos.x).signum() * 1.45, 0.0, 0.0);
                        org.pos.x = (org.pos.x + org.vel.x * dt).clamp(-8.0, 8.0);
                        org.pos.y += (sub_y - org.pos.y) * (dt * 3.5).min(1.0);
                        org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                        org.pose.body_pitch = if dive_timer > 3.2 { -0.45 } else { 0.35 };
                    } else {
                        if dabble_timer > 0.0 {
                            dabble_timer = (dabble_timer - dt).max(0.0);
                            org.pose.body_pitch = -1.15;
                        } else {
                            org.pose.body_pitch = 0.0;
                        }
                        let dx = target_x - org.pos.x;
                        if dx.abs() > 0.35 && dabble_timer <= 0.01 {
                            org.vel = Vec3::new(dx.signum() * 0.62, 0.0, 0.0);
                            org.pos.x += org.vel.x * dt;
                            org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                        } else if dabble_timer <= 0.01 {
                            target_x = rng.gen_range(-7.2..7.2);
                            target_z = rng.gen_range(0.15..0.85);
                            if org.species == Species::MallardDuck && rng.gen::<f32>() < 0.35 {
                                dabble_timer = 3.2;
                            } else if org.species == Species::CommonLoon && rng.gen::<f32>() < 0.25 {
                                dive_timer = 6.0;
                            }
                        }
                        org.pos.y = LAKE_SURFACE_Y + 0.04 + (t_sec * 2.1 + org.id as f32).sin() * 0.03;
                    }
                    org.state = BehaviorState::WaterbirdSurfaceSwim {
                        target_x,
                        target_z,
                        dabble_timer,
                        dive_timer,
                    };
                }

                BehaviorState::OspreyAerialSoar {
                    mut orbit_phase,
                    mut plunge_timer,
                    plunge_x,
                } => {
                    org.pose.wing_phase_amp = [t_sec * 5.2, 1.0];
                    if plunge_timer > 0.0 {
                        plunge_timer = (plunge_timer - dt).max(0.0);
                        let frac = 1.0 - plunge_timer / 4.5;
                        let target_y = if frac < 0.45 {
                            LAKE_SURFACE_Y - 0.25
                        } else {
                            12.6
                        };
                        let goal = Vec3::new(plunge_x + (frac - 0.45) * 6.0, target_y, -0.4);
                        org.vel = (goal - org.pos) * 2.8;
                        org.pos += org.vel * dt;
                        org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                        org.pose.body_pitch = (org.vel.y * 0.12).clamp(-0.75, 0.55);
                    } else {
                        orbit_phase += dt * 0.42;
                        let gx = orbit_phase.sin() * 9.5;
                        let gy = 12.2 + (orbit_phase * 2.0).cos() * 0.85;
                        let gz = -1.2 + orbit_phase.cos() * 1.4;
                        let v = Vec3::new(orbit_phase.cos() * 4.0, 0.0, 0.0);
                        org.vel = v;
                        org.pos = org.pos.lerp(Vec3::new(gx, gy, gz), (dt * 2.2).min(1.0));
                        org.pose.body_yaw = if v.x >= 0.0 { 0.0 } else { PI };
                        org.pose.body_pitch = 0.0;
                    }
                    org.state = BehaviorState::OspreyAerialSoar {
                        orbit_phase,
                        plunge_timer,
                        plunge_x,
                    };
                }

                BehaviorState::HeronShalllowStalk {
                    anchor_pos,
                    mut strike_timer,
                } => {
                    if strike_timer > 0.0 {
                        strike_timer = (strike_timer - dt).max(0.0);
                        org.pose.head_pitch = -0.85 * (strike_timer * PI).sin().abs();
                    } else {
                        org.pose.head_pitch = (t_sec * 0.8).sin() * 0.06;
                        if rng.gen::<f32>() < 0.008 {
                            strike_timer = 1.1;
                        }
                    }
                    org.pos = anchor_pos;
                    org.state = BehaviorState::HeronShalllowStalk {
                        anchor_pos,
                        strike_timer,
                    };
                }

                BehaviorState::PikeWeedbedAmbush {
                    hover_pos,
                    mut glide_target,
                    mut gliding,
                } => {
                    org.pose.spine_phase += dt * (if gliding { 14.0 } else { 3.2 });
                    if gliding {
                        let d = glide_target - org.pos;
                        if d.length() > 0.45 {
                            org.vel = d.normalize_or_zero() * 2.6;
                            org.pos += org.vel * dt;
                            org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                        } else {
                            gliding = false;
                        }
                    } else {
                        org.vel = Vec3::ZERO;
                        org.pos.y = hover_pos.y + (t_sec * 1.2).sin() * 0.06;
                        if rng.gen::<f32>() < 0.006 {
                            gliding = true;
                            glide_target = Vec3::new(rng.gen_range(-6.5..6.5), rng.gen_range(2.2..5.5), rng.gen_range(-0.6..0.4));
                        }
                    }
                    org.state = BehaviorState::PikeWeedbedAmbush {
                        hover_pos,
                        glide_target,
                        gliding,
                    };
                }

                BehaviorState::GhostSharkPectoralFlight { mut waypoint } => {
                    org.pose.wing_phase_amp = [t_sec * 3.8, 1.0];
                    org.pose.spine_phase += dt * 5.2;
                    let diff = waypoint - org.pos;
                    let dist = diff.length();
                    if dist > 0.48 {
                        let spd = 1.25;
                        org.vel = org.vel.lerp((diff / dist) * spd, (dt * 2.4).min(1.0));
                        org.pos += org.vel * dt;
                        org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                        org.pose.body_pitch = (org.vel.y / spd).clamp(-0.4, 0.4);
                    } else {
                        waypoint = Vec3::new(rng.gen_range(-9.5..9.5), rng.gen_range(4.2..9.5), rng.gen_range(-1.2..0.8));
                    }
                    org.state = BehaviorState::GhostSharkPectoralFlight { waypoint };
                }

                BehaviorState::YetiCrabBacterialWave { home_pos, mut wave_phase } => {
                    wave_phase += dt * 4.2;
                    org.pose.limb_phase = wave_phase;
                    org.pos = home_pos + Vec3::new((wave_phase * 0.35).sin() * 0.08, 0.0, 0.0);
                    org.state = BehaviorState::YetiCrabBacterialWave { home_pos, wave_phase };
                }

                BehaviorState::ChimneyShrimpSwarm { chimney_pos, mut swirl_phase } => {
                    swirl_phase += dt * 4.8;
                    let r = 0.45 + 0.18 * (swirl_phase * 0.7).sin();
                    let target = chimney_pos + Vec3::new(swirl_phase.cos() * r, (swirl_phase * 1.3).sin() * 0.32, swirl_phase.sin() * r * 0.6);
                    org.vel = (target - org.pos) * 4.0;
                    org.pos += org.vel * dt;
                    org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                    org.state = BehaviorState::ChimneyShrimpSwarm { chimney_pos, swirl_phase };
                }

                BehaviorState::BiolumPelagicPulse { drift_center, mut flash_timer } => {
                    if flash_timer > 0.0 {
                        flash_timer = (flash_timer - dt).max(0.0);
                        org.pose.chromatophore = 1.0;
                    } else {
                        org.pose.chromatophore = 0.35 + 0.35 * (t_sec * 2.4 + org.id as f32).sin().abs();
                    }
                    org.pos = drift_center + Vec3::new((t_sec * 0.45 + org.id as f32).sin() * 0.85, (t_sec * 0.75 + org.id as f32).cos() * 0.45, 0.0);
                    org.state = BehaviorState::BiolumPelagicPulse { drift_center, flash_timer };
                }

                BehaviorState::SculpinTidepoolHop {
                    pool_a,
                    pool_b,
                    mut leap_timer,
                    at_b,
                } => {
                    if leap_timer > 0.0 {
                        leap_timer = (leap_timer - dt).max(0.0);
                        let u = 1.0 - leap_timer / 1.8;
                        let (src_p, dst_p) = if at_b { (pool_a, pool_b) } else { (pool_b, pool_a) };
                        org.pos = src_p.lerp(dst_p, u) + Vec3::new(0.0, (u * PI).sin() * 1.15, 0.0);
                        org.pose.body_yaw = if dst_p.x >= src_p.x { 0.0 } else { PI };
                    } else {
                        let home = if at_b { pool_b } else { pool_a };
                        org.pos = home + Vec3::new((t_sec * 2.2 + org.id as f32).sin() * 0.28, 0.0, 0.0);
                    }
                    org.state = BehaviorState::SculpinTidepoolHop { pool_a, pool_b, leap_timer, at_b };
                }

                BehaviorState::CutthroatEelBrineCruiser {
                    center,
                    mut skim_timer,
                    mut phase,
                } => {
                    phase += dt * 4.2;
                    org.pose.spine_phase = phase;
                    if skim_timer > 0.0 {
                        skim_timer = (skim_timer - dt).max(0.0);
                        let u = 1.0 - skim_timer / 4.5;
                        let dip_y = BRINE_POOL_SURFACE_Y + 0.35 - (u * PI).sin() * 0.32;
                        org.pos = Vec3::new(center.x + (phase * 0.35).sin() * 2.4, dip_y, center.z);
                    } else {
                        org.pos = center + Vec3::new((phase * 0.28).sin() * 2.2, (phase * 0.5).cos() * 0.16, 0.0);
                    }
                    org.pose.body_yaw = if (phase * 0.28).cos() >= 0.0 { 0.0 } else { PI };
                    org.state = BehaviorState::CutthroatEelBrineCruiser { center, skim_timer, phase };
                }

                BehaviorState::DvmVerticalMigrator {
                    day_y,
                    night_y,
                    home_xz,
                    mut phase,
                    mut dvm_boost,
                } => {
                    phase += dt * 4.2;
                    dvm_boost = (dvm_boost - dt).max(0.0);
                    let mig = (night_f + if dvm_boost > 0.0 { 0.85 } else { 0.0 }).clamp(0.0, 1.0);
                    let target_y = day_y + (night_y - day_y) * mig + (phase * 0.45).sin() * 0.28;
                    let drift_x = home_xz[0] + (t_sec * 0.55 + phase).sin() * 1.45;
                    let prev_x = org.pos.x;
                    org.pos = Vec3::new(drift_x, org.pos.y + (target_y - org.pos.y) * (dt * 1.8).min(1.0), home_xz[1]);
                    org.pose.body_yaw = if org.pos.x >= prev_x { 0.0 } else { PI };
                    org.pose.spine_phase = phase * 2.0;
                    org.state = BehaviorState::DvmVerticalMigrator {
                        day_y,
                        night_y,
                        home_xz,
                        phase,
                        dvm_boost,
                    };
                }

                BehaviorState::BarreleyeTubularScan {
                    hover_anchor,
                    mut lock_timer,
                } => {
                    lock_timer = (lock_timer - dt).max(0.0);
                    org.pos = hover_anchor + Vec3::new((t_sec * 0.4).sin() * 0.35, (t_sec * 0.65).cos() * 0.22, 0.0);
                    // Head/eye pitch: +PI/2 upward scan normally, rotating forward (0.0) when locking onto siphonophore prey!
                    org.pose.head_pitch = if lock_timer > 0.0 { 0.05 } else { 1.35 };
                    org.pose.wing_phase_amp = [t_sec * 3.2, 0.25];
                    org.state = BehaviorState::BarreleyeTubularScan { hover_anchor, lock_timer };
                }

                BehaviorState::SpermWhaleEcholocationDive {
                    cruise_y,
                    mut dive_timer,
                    sonar_pulse: _,
                } => {
                    let sonar_pulse = if dive_timer > 0.0 {
                        dive_timer = (dive_timer - dt).max(0.0);
                        let u = 1.0 - dive_timer / 6.5;
                        let dive_depth = (u * PI).sin() * 4.4;
                        org.pos = Vec3::new(-4.5 + u * 9.0, cruise_y - dive_depth, -1.1);
                        org.pose.body_yaw = 0.0;
                        org.pose.body_pitch = -(0.5 - u) * 0.55;
                        (t_sec * 6.0).sin().abs()
                    } else {
                        let sweep = (t_sec * 0.24).sin();
                        let next_x = sweep * 5.2;
                        org.pose.body_yaw = if (t_sec * 0.24).cos() >= 0.0 { 0.0 } else { PI };
                        org.pos = Vec3::new(next_x, cruise_y + (t_sec * 0.6).cos() * 0.32, -1.1);
                        org.pose.body_pitch = 0.0;
                        if rng.gen::<f32>() < 0.004 {
                            dive_timer = 6.5;
                        }
                        0.25 * (t_sec * 2.4).sin().abs()
                    };
                    org.pose.spine_phase = t_sec * 2.2;
                    org.state = BehaviorState::SpermWhaleEcholocationDive { cruise_y, dive_timer, sonar_pulse };
                }

                BehaviorState::ArchiteuthisChasmProwl {
                    home_pos,
                    mut jet_timer,
                    mut phase,
                } => {
                    phase += dt * 3.4;
                    org.pose.spine_phase = phase;
                    if jet_timer > 0.0 {
                        jet_timer = (jet_timer - dt).max(0.0);
                        let u = 1.0 - jet_timer / 4.8;
                        org.pos = home_pos + Vec3::new(-(u * PI * 2.0).sin() * 3.4, (u * PI).sin() * 1.4, 0.0);
                        org.pose.body_yaw = PI;
                    } else {
                        org.pos = home_pos + Vec3::new((phase * 0.28).sin() * 1.6, (phase * 0.4).cos() * 0.38, 0.0);
                        org.pose.body_yaw = if (phase * 0.28).cos() >= 0.0 { 0.0 } else { PI };
                        if is_rift_biome && rng.gen::<f32>() < 0.005 {
                            jet_timer = 4.8;
                        }
                    }
                    org.state = BehaviorState::ArchiteuthisChasmProwl { home_pos, jet_timer, phase };
                }

                BehaviorState::CohoUpstreamReddAndLeap {
                    redd_pos,
                    mut leap_timer,
                    mut phase,
                } => {
                    phase += dt * 9.5;
                    org.pose.spine_phase = phase;
                    if leap_timer > 0.0 {
                        leap_timer = (leap_timer - dt).max(0.0);
                        let u = 1.0 - leap_timer / 2.8;
                        let arc_y = PNW_CREEK_SURFACE_Y - 0.35 + (u * PI).sin() * 2.05;
                        org.pos = Vec3::new(redd_pos.x - (u - 0.5) * 2.6, arc_y, redd_pos.z);
                        org.pose.body_yaw = PI;
                        org.pose.body_pitch = (0.5 - u) * 0.95;
                    } else {
                        org.pos = redd_pos + Vec3::new((phase * 0.22).sin() * 0.45, (phase * 0.35).cos() * 0.12, 0.0);
                        org.pose.body_yaw = PI;
                        org.pose.body_pitch = 0.0;
                        if rng.gen::<f32>() < 0.005 {
                            leap_timer = 2.8;
                        }
                    }
                    org.state = BehaviorState::CohoUpstreamReddAndLeap { redd_pos, leap_timer, phase };
                }

                BehaviorState::DipperRiffleSubmergeWalk {
                    rock_perch,
                    mut walk_timer,
                } => {
                    if walk_timer > 0.0 {
                        walk_timer = (walk_timer - dt).max(0.0);
                        let u = 1.0 - walk_timer / 5.5;
                        let wx = rock_perch.x + (u * PI).sin() * 2.2;
                        let wy = pnw_creek_forest_height_at(wx, rock_perch.z) + 0.14;
                        org.pos = Vec3::new(wx, wy, rock_perch.z);
                        org.pose.body_yaw = if u < 0.5 { 0.0 } else { PI };
                        org.pose.wing_phase_amp = [t_sec * 14.0, 0.35];
                    } else {
                        // Characteristic knee-bending "dipping" bob on creek boulder!
                        let dip = (t_sec * 6.2).sin().abs() * 0.08;
                        org.pos = rock_perch - Vec3::new(0.0, dip, 0.0);
                        org.pose.wing_phase_amp = [0.0, 0.0];
                        if rng.gen::<f32>() < 0.008 {
                            walk_timer = 5.5;
                        }
                    }
                    org.state = BehaviorState::DipperRiffleSubmergeWalk { rock_perch, walk_timer };
                }

                BehaviorState::SpottedOwlCanopyPerchAndGlide {
                    perch_a,
                    perch_b,
                    mut glide_timer,
                    at_b,
                } => {
                    if glide_timer > 0.0 {
                        glide_timer = (glide_timer - dt).max(0.0);
                        let u = 1.0 - glide_timer / 4.2;
                        let (src_p, dst_p) = if at_b { (perch_a, perch_b) } else { (perch_b, perch_a) };
                        org.pos = src_p.lerp(dst_p, u) - Vec3::new(0.0, (u * PI).sin() * 1.15, 0.0);
                        org.pose.body_yaw = if dst_p.x >= src_p.x { 0.0 } else { PI };
                        org.pose.wing_phase_amp = [t_sec * 4.5, 1.0];
                    } else {
                        org.pos = if at_b { perch_b } else { perch_a };
                        org.pose.head_yaw = (t_sec * 0.9).sin() * 0.85; // 160-degree owl head scan!
                        org.pose.wing_phase_amp = [0.0, 0.0];
                    }
                    org.state = BehaviorState::SpottedOwlCanopyPerchAndGlide {
                        perch_a,
                        perch_b,
                        glide_timer,
                        at_b,
                    };
                }

                BehaviorState::SessileMantleFilter { mut open_fraction } => {
                    let mut shadow_near = false;
                    for &p in &marine_predators {
                        if p.distance(org.pos) < 2.8 {
                            shadow_near = true;
                            break;
                        }
                    }
                    let goal = if shadow_near { 0.25 } else { 1.0 };
                    open_fraction += (goal - open_fraction) * (dt * 3.2);
                    org.pose.crouch = 1.0 - open_fraction;
                    org.state = BehaviorState::SessileMantleFilter { open_fraction };
                }

                BehaviorState::PlanktivoreBoidSchool {
                    mut school_center,
                    phase_seed,
                } => {
                    if org.species == Species::BlueGreenChromis {
                        let coral_home = Vec3::new(0.4, 4.4, -0.45);
                        let shark_close = marine_predators.iter().any(|p| p.distance(coral_home) < 4.5);
                        let spread = if shark_close { 0.32 } else { 1.65 };
                        let hover_h = if shark_close { 0.35 } else { 2.0 };
                        let target = coral_home + Vec3::new(
                            (t_sec * 1.3 + phase_seed).cos() * spread,
                            hover_h + (t_sec * 1.7 + phase_seed).sin() * (spread * 0.35),
                            (phase_seed * 2.3).sin() * spread * 0.5,
                        );
                        org.vel = org.vel.lerp((target - org.pos) * 3.2, (dt * 5.0).min(1.0));
                        org.pos += org.vel * dt;
                        org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                        org.pose.spine_phase += dt * 13.0;
                        org.state = BehaviorState::PlanktivoreBoidSchool {
                            school_center: coral_home,
                            phase_seed,
                        };
                    } else {
                    // Cohesive 3D Boids orbit + rapid flash-expansion scatter when Harbor Seal or Leopard Shark nears!
                    school_center.x = (t_sec * 0.28).sin() * 5.2;
                    school_center.y = 8.8 + (t_sec * 0.42).cos() * 1.4;
                    let slot_offset = Vec3::new(
                        (t_sec * 0.9 + phase_seed).cos() * 1.65,
                        (t_sec * 1.3 + phase_seed * 1.7).sin() * 0.85,
                        (phase_seed * 2.1).sin() * 0.9,
                    );
                    let mut desired = (school_center + slot_offset - org.pos) * 1.25;

                    for &pred_p in &marine_predators {
                        let away = org.pos - pred_p;
                        let d = away.length();
                        if d < 3.2 && d > 0.05 {
                            desired += (away / d) * 6.5; // Predator flash-expansion!
                        }
                    }

                    org.vel = org.vel.lerp(desired.clamp_length_max(3.6), (dt * 4.5).min(1.0));
                    org.pos += org.vel * dt;
                    org.pos.y = org.pos.y.clamp(2.5, KELP_SURFACE_Y - 1.2);
                    org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                    org.pose.spine_phase += dt * 11.0;
                        org.state = BehaviorState::PlanktivoreBoidSchool {
                            school_center,
                            phase_seed,
                        };
                    }
                }

                BehaviorState::BenthicGrazerCreep { mut target_xz } => {
                    let to = Vec2::new(target_xz[0] - org.pos.x, target_xz[1] - org.pos.z);
                    let d = to.length();
                    let creep_spd = 0.07;
                    if d > 0.15 {
                        let dir = to / d;
                        org.vel = Vec3::new(dir.x * creep_spd, 0.0, dir.y * creep_spd);
                        org.pos.x += org.vel.x * dt;
                        org.pos.z += org.vel.z * dt;
                        org.pos.y = reef_height_at(org.pos.x, org.pos.z) + 0.05;
                        org.pose.limb_phase += dt * 2.5;
                    } else {
                        target_xz = [
                            (org.pos.x + rng.gen_range(-2.2..2.2)).clamp(-13.0, 13.0),
                            (org.pos.z + rng.gen_range(-0.8..0.8)).clamp(-2.5, 1.5),
                        ];
                    }
                    org.state = BehaviorState::BenthicGrazerCreep { target_xz };
                }

                // ================= NEOTROPICAL RAINFOREST ETHOLOGY =================
                BehaviorState::MonkeyVineSwing {
                    vine_id,
                    mut swing_time,
                    target_branch_id,
                } => {
                    swing_time += dt;
                    let vid = vine_id.min(self.lianas.len().saturating_sub(1));
                    if swing_time < 1.2 {
                        let pump = if self.lianas[vid].omega_rad_s >= 0.0 {
                            0.9 * dt
                        } else {
                            -0.9 * dt
                        };
                        self.lianas[vid].apply_tangential_impulse(pump, 0.0);
                    }
                    let tip = self.lianas[vid].position_along(0.92);
                    let tip_vel = self.lianas[vid].tip_velocity();
                    org.pos = tip;
                    org.vel = tip_vel;
                    org.pose.body_yaw = if tip_vel.x >= 0.0 { 0.0 } else { PI };
                    org.pose.limb_phase += dt * 4.5;
                    org.pose.tail_angle = (-tip_vel.x * 0.22).clamp(-0.9, 0.9);
                    org.pose.head_yaw = (swing_time * 2.2).sin() * 0.25;

                    if swing_time > 2.35 && tip_vel.x.abs() > 0.45 {
                        let next_br = if tip.x < 0.0 { 3 } else { 4 }.min(branches_ref.len() - 1);
                        let target_t = rng.gen_range(0.32..0.72);
                        let target_pt =
                            branches_ref[next_br].point_at(target_t) + Vec3::new(0.0, 0.34, 0.0);
                        let flight_t = 0.72_f32;
                        org.vel = Vec3::new(
                            (target_pt.x - org.pos.x) / flight_t,
                            (target_pt.y - org.pos.y) / flight_t + 0.5 * GRAVITY_M_S2 * flight_t,
                            (target_pt.z - org.pos.z) / flight_t,
                        );
                        org.state = BehaviorState::MonkeyBallisticLeap {
                            target_branch_id: next_br,
                            target_t,
                        };
                    } else {
                        org.state = BehaviorState::MonkeyVineSwing {
                            vine_id: vid,
                            swing_time,
                            target_branch_id,
                        };
                    }
                }

                BehaviorState::MonkeyBallisticLeap {
                    target_branch_id,
                    target_t,
                } => {
                    org.vel.y -= GRAVITY_M_S2 * dt;
                    org.pos += org.vel * dt;
                    org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                    org.pose.body_pitch = (org.vel.y * 0.08).clamp(-0.6, 0.6);
                    org.pose.tail_angle = (-org.vel.y * 0.12).clamp(-0.8, 0.8);

                    let bid = target_branch_id.min(branches_ref.len() - 1);
                    let landing = branches_ref[bid].point_at(target_t) + Vec3::new(0.0, 0.34, 0.0);
                    if org.pos.distance(landing) < 0.55 || org.pos.y < landing.y - 0.25 {
                        org.pos = landing;
                        org.vel = Vec3::ZERO;
                        org.pose.body_pitch = 0.0;
                        org.state = BehaviorState::MonkeyPerchedLookAround {
                            branch_id: bid,
                            branch_t: target_t,
                            remaining_sec: rng.gen_range(2.8..4.8),
                            scan_seed: rng.gen_range(0.0..6.28),
                        };
                    }
                }

                BehaviorState::MonkeyPerchedLookAround {
                    branch_id,
                    branch_t,
                    mut remaining_sec,
                    scan_seed,
                } => {
                    remaining_sec -= dt;
                    let bid = branch_id.min(branches_ref.len() - 1);
                    org.pos = branches_ref[bid].point_at(branch_t) + Vec3::new(0.0, 0.34, 0.0);
                    org.vel = Vec3::ZERO;
                    let t_scan = t_sec * 2.6 + scan_seed;
                    org.pose.head_yaw = (t_scan.sin() * 0.95).clamp(-0.95, 0.95);
                    org.pose.head_pitch = ((t_scan * 1.35).cos() * 0.42).clamp(-0.45, 0.45);
                    org.pose.tail_angle = (t_scan * 0.6).sin() * 0.28;

                    if let Some(boa_p) = boa_snapshot {
                        if boa_p.distance(org.pos) < 4.8 && org.vocal_cooldown_sec <= 0.0 {
                            org.vocal_cooldown_sec = 9.0;
                            self.pending_sounds.push(SoundCue {
                                kind: SoundCueKind::MonkeyChatter,
                                world_pos: org.pos,
                                gain: 0.72,
                                pitch_mul: 1.12,
                            });
                        }
                    }

                    if remaining_sec <= 0.0 {
                        org.pose.head_yaw = 0.0;
                        org.pose.head_pitch = 0.0;
                        let dir = if branch_t > 0.55 { -1.0 } else { 1.0 };
                        org.state = BehaviorState::MonkeyBranchTraverse {
                            branch_id: bid,
                            branch_t,
                            dir,
                        };
                    } else {
                        org.state = BehaviorState::MonkeyPerchedLookAround {
                            branch_id: bid,
                            branch_t,
                            remaining_sec,
                            scan_seed,
                        };
                    }
                }

                BehaviorState::MonkeyBranchTraverse {
                    branch_id,
                    mut branch_t,
                    dir,
                } => {
                    let bid = branch_id.min(branches_ref.len() - 1);
                    let len = branches_ref[bid].length().max(1.0);
                    let speed_m_s = if is_night { 0.45 } else { 1.85 };
                    branch_t += dir * (speed_m_s * dt / len);
                    let next_pos = branches_ref[bid].point_at(branch_t) + Vec3::new(0.0, 0.34, 0.0);
                    org.vel = (next_pos - org.pos) / dt;
                    org.pos = next_pos;
                    org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                    org.pose.limb_phase += dt * speed_m_s * 5.2;
                    org.pose.tail_angle = (t_sec * 3.5).sin() * 0.35;

                    let mut grabbed_vine = None;
                    if !is_night {
                        for (vid, vine) in self.lianas.iter().enumerate() {
                            if vine.tip_position().distance(org.pos) < 2.35 && rng.gen::<f32>() < 0.08 {
                                grabbed_vine = Some(vid);
                                break;
                            }
                        }
                    }

                    if let Some(vid) = grabbed_vine {
                        let impulse_x = if org.pos.x < 0.0 { 2.8 } else { -2.8 };
                        self.lianas[vid].apply_tangential_impulse(impulse_x, 0.15);
                        org.state = BehaviorState::MonkeyVineSwing {
                            vine_id: vid,
                            swing_time: 0.0,
                            target_branch_id: (bid + 1) % branches_ref.len(),
                        };
                    } else if !(0.12..=0.88).contains(&branch_t) && !self.lianas.is_empty() {
                        let vid = rng.gen_range(0..3.min(self.lianas.len()));
                        let impulse_x = if self.lianas[vid].anchor.x < 0.0 {
                            2.9
                        } else {
                            -2.9
                        };
                        self.lianas[vid].apply_tangential_impulse(impulse_x, 0.12);
                        org.state = BehaviorState::MonkeyVineSwing {
                            vine_id: vid,
                            swing_time: 0.0,
                            target_branch_id: 3,
                        };
                    } else {
                        org.state = BehaviorState::MonkeyBranchTraverse {
                            branch_id: bid,
                            branch_t,
                            dir,
                        };
                    }
                }

                BehaviorState::SnakeCoiledAmbush {
                    branch_id,
                    branch_t,
                    mut patience_sec,
                    mut tongue_cooldown,
                } => {
                    patience_sec += dt;
                    tongue_cooldown -= dt;
                    let bid = branch_id.min(branches_ref.len() - 1);
                    let anchor = branches_ref[bid].point_at(branch_t) + Vec3::new(0.0, 0.24, 0.0);
                    org.pos = anchor;
                    org.vel = Vec3::ZERO;
                    org.pose.spine_phase += dt * 0.35;

                    if tongue_cooldown <= 0.0 {
                        tongue_cooldown = rng.gen_range(3.2..6.8);
                        org.pose.tongue_ext = 1.0;
                    }

                    let mut strike_target = None;
                    for &mp in &monkey_positions {
                        if mp.distance(anchor) < 2.9 && patience_sec > 6.0 {
                            strike_target = Some(mp);
                            break;
                        }
                    }

                    if let Some(target) = strike_target {
                        org.pose.body_yaw = (target.x - anchor.x).atan2(target.z - anchor.z);
                        self.pending_sounds.push(SoundCue {
                            kind: SoundCueKind::BoaHiss,
                            world_pos: anchor,
                            gain: 0.75,
                            pitch_mul: 1.05,
                        });
                        org.state = BehaviorState::SnakeBallisticStrike {
                            anchor_pos: anchor,
                            strike_target: target,
                            progress: 0.0,
                        };
                    } else {
                        org.state = BehaviorState::SnakeCoiledAmbush {
                            branch_id: bid,
                            branch_t,
                            patience_sec,
                            tongue_cooldown,
                        };
                    }
                }

                BehaviorState::SnakeBallisticStrike {
                    anchor_pos,
                    strike_target,
                    mut progress,
                } => {
                    progress += dt * 2.8;
                    let lunge = (progress.min(1.0) * PI).sin();
                    let next_pos = anchor_pos.lerp(strike_target, lunge * 0.72);
                    org.vel = (next_pos - org.pos) / dt;
                    org.pos = next_pos;
                    org.pose.tongue_ext = 1.0;

                    if progress >= 1.0 {
                        org.pos = anchor_pos;
                        org.vel = Vec3::ZERO;
                        org.energy_kcal = (org.energy_kcal + 6.5).min(org.max_energy_kcal);
                        org.state = BehaviorState::SnakeBaskingDigest {
                            branch_id: 0,
                            branch_t: 0.55,
                            remaining_sec: 12.0,
                        };
                    } else {
                        org.state = BehaviorState::SnakeBallisticStrike {
                            anchor_pos,
                            strike_target,
                            progress,
                        };
                    }
                }

                BehaviorState::SnakeBaskingDigest {
                    branch_id,
                    branch_t,
                    mut remaining_sec,
                } => {
                    remaining_sec -= dt;
                    let bid = branch_id.min(branches_ref.len() - 1);
                    org.pos = branches_ref[bid].point_at(branch_t) + Vec3::new(0.0, 0.24, 0.0);
                    org.vel = Vec3::ZERO;
                    org.pose.spine_phase += dt * 0.18;
                    if remaining_sec <= 0.0 {
                        org.state = BehaviorState::SnakeCoiledAmbush {
                            branch_id: bid,
                            branch_t,
                            patience_sec: 0.0,
                            tongue_cooldown: 2.5,
                        };
                    } else {
                        org.state = BehaviorState::SnakeBaskingDigest {
                            branch_id: bid,
                            branch_t,
                            remaining_sec,
                        };
                    }
                }

                BehaviorState::PantherUnderbrushStalk {
                    waypoint_x,
                    waypoint_z,
                    stealth_crouch,
                } => {
                    let target = Vec3::new(
                        waypoint_x,
                        ground_height_at(waypoint_x, waypoint_z),
                        waypoint_z,
                    );
                    let to_wp = Vec3::new(target.x - org.pos.x, 0.0, target.z - org.pos.z);
                    let dist = to_wp.length();
                    let stalk_speed = if is_night || self.diurnal.is_crepuscular() {
                        0.98
                    } else {
                        0.72
                    };

                    if dist > 0.35 {
                        let dir = to_wp / dist;
                        org.vel = dir * stalk_speed;
                        org.pos.x += org.vel.x * dt;
                        org.pos.z += org.vel.z * dt;
                        org.pos.y = ground_height_at(org.pos.x, org.pos.z);
                        org.pose.body_yaw = if dir.x >= 0.0 { 0.0 } else { PI };
                        org.pose.crouch += (stealth_crouch - org.pose.crouch) * (dt * 3.0);
                        org.pose.limb_phase += dt * stalk_speed * 2.9;
                        org.pose.spine_phase = org.pose.limb_phase;
                        org.pose.tail_angle = (t_sec * 1.9).sin() * 0.24;
                        org.pose.head_yaw = (t_sec * 0.85).sin() * 0.18;

                        let mut max_rustle = 0.0_f32;
                        for plant in &mut self.underbrush {
                            let imp = plant.apply_body_contact(org.pos, org.vel, 0.68);
                            if imp > max_rustle {
                                max_rustle = imp;
                            }
                        }
                        if max_rustle > 0.42 && org.vocal_cooldown_sec <= 0.0 {
                            org.vocal_cooldown_sec = 3.8;
                            self.pending_sounds.push(SoundCue {
                                kind: SoundCueKind::UnderbrushRustle,
                                world_pos: org.pos,
                                gain: 0.45,
                                pitch_mul: rng.gen_range(0.88..1.06),
                            });
                        }

                        if rng.gen::<f32>() < 0.09 * dt {
                            org.vel = Vec3::ZERO;
                            org.state = BehaviorState::PantherShadowPause {
                                remaining_sec: rng.gen_range(2.6..5.2),
                                look_yaw: rng.gen_range(-0.65..0.65),
                            };
                        } else {
                            org.state = BehaviorState::PantherUnderbrushStalk {
                                waypoint_x,
                                waypoint_z,
                                stealth_crouch,
                            };
                        }
                    } else {
                        org.vel = Vec3::ZERO;
                        org.state = BehaviorState::PantherShadowPause {
                            remaining_sec: rng.gen_range(3.0..5.8),
                            look_yaw: 0.45,
                        };
                    }
                }

                BehaviorState::PantherShadowPause {
                    mut remaining_sec,
                    look_yaw,
                } => {
                    remaining_sec -= dt;
                    org.vel = Vec3::ZERO;
                    org.pose.crouch += (0.88 - org.pose.crouch) * (dt * 2.5);
                    org.pose.head_yaw += (look_yaw - org.pose.head_yaw) * (dt * 2.8);
                    org.pose.tail_angle = (t_sec * 4.6).sin().powi(3) * 0.42;

                    if remaining_sec <= 0.0 {
                        org.pose.head_yaw = 0.0;
                        let next_x = if org.pos.x > 1.5 {
                            rng.gen_range(-11.5..-4.5)
                        } else {
                            rng.gen_range(4.5..11.5)
                        };
                        org.state = BehaviorState::PantherUnderbrushStalk {
                            waypoint_x: next_x,
                            waypoint_z: rng.gen_range(-0.4..1.0),
                            stealth_crouch: rng.gen_range(0.75..0.90),
                        };
                    } else {
                        org.state = BehaviorState::PantherShadowPause {
                            remaining_sec,
                            look_yaw,
                        };
                    }
                }

                BehaviorState::PantherPounce { target_pos } => {
                    let to_t = target_pos - org.pos;
                    if to_t.length() > 0.4 {
                        org.vel = to_t.normalize() * 4.5;
                        org.pos += org.vel * dt;
                        org.pos.y = org.pos.y.max(ground_height_at(org.pos.x, org.pos.z));
                        org.state = BehaviorState::PantherPounce { target_pos };
                    } else {
                        org.vel = Vec3::ZERO;
                        org.state = BehaviorState::PantherShadowPause {
                            remaining_sec: 3.5,
                            look_yaw: 0.0,
                        };
                    }
                }

                BehaviorState::JaguarCanopyLounge {
                    branch_id,
                    branch_t,
                    mut tail_sway_phase,
                    mut remaining_sec,
                } => {
                    tail_sway_phase += dt * 1.4;
                    remaining_sec -= dt;
                    let bid = branch_id.min(branches_ref.len() - 1);
                    org.pos = branches_ref[bid].point_at(branch_t) + Vec3::new(0.0, 0.42, 0.0);
                    org.vel = Vec3::ZERO;
                    org.pose.tail_angle = tail_sway_phase.sin() * 0.35;
                    org.pose.head_yaw = (tail_sway_phase * 0.45).sin() * 0.42;
                    org.pose.head_pitch = -0.18;
                    if remaining_sec <= 0.0 {
                        remaining_sec = rng.gen_range(18.0..32.0);
                    }
                    org.state = BehaviorState::JaguarCanopyLounge {
                        branch_id: bid,
                        branch_t,
                        tail_sway_phase,
                        remaining_sec,
                    };
                }

                BehaviorState::SlothSuspensoryClimb {
                    branch_id,
                    mut branch_t,
                    mut dir,
                } => {
                    let bid = branch_id.min(branches_ref.len() - 1);
                    let len = branches_ref[bid].length().max(1.0);
                    branch_t += dir * (0.085 * dt / len);
                    if branch_t > 0.78 {
                        branch_t = 0.78;
                        dir = -1.0;
                    } else if branch_t < 0.16 {
                        branch_t = 0.16;
                        dir = 1.0;
                    }
                    let next_p = branches_ref[bid].point_at(branch_t) - Vec3::new(0.0, 0.52, 0.0);
                    org.vel = (next_p - org.pos) / dt;
                    org.pos = next_p;
                    org.pose.body_yaw = if dir >= 0.0 { 0.0 } else { PI };
                    org.pose.limb_phase += dt * 0.75;
                    org.pose.head_yaw = (t_sec * 0.35).sin() * 0.32;

                    if rng.gen::<f32>() < 0.05 * dt {
                        org.state = BehaviorState::SlothHangingFoliageRest {
                            branch_id: bid,
                            branch_t,
                            remaining_sec: rng.gen_range(6.0..12.0),
                        };
                    } else {
                        org.state = BehaviorState::SlothSuspensoryClimb {
                            branch_id: bid,
                            branch_t,
                            dir,
                        };
                    }
                }

                BehaviorState::SlothHangingFoliageRest {
                    branch_id,
                    branch_t,
                    mut remaining_sec,
                } => {
                    remaining_sec -= dt;
                    org.vel = Vec3::ZERO;
                    if remaining_sec <= 0.0 {
                        org.state = BehaviorState::SlothSuspensoryClimb {
                            branch_id,
                            branch_t,
                            dir: 1.0,
                        };
                    } else {
                        org.state = BehaviorState::SlothHangingFoliageRest {
                            branch_id,
                            branch_t,
                            remaining_sec,
                        };
                    }
                }

                BehaviorState::AvianPerchedForage {
                    branch_id,
                    branch_t,
                    mut remaining_sec,
                } => {
                    remaining_sec -= dt;
                    let bid = branch_id.min(branches_ref.len() - 1);
                    org.pos = branches_ref[bid].point_at(branch_t) + Vec3::new(0.0, 0.33, 0.0);
                    org.vel = Vec3::ZERO;
                    org.pose.wing_phase_amp = [0.0, 0.0];
                    org.pose.head_yaw = (t_sec * 2.1 + org.id as f32).sin() * 0.45;

                    if remaining_sec <= 0.0 && !is_night {
                        let choices = [2_usize, 3, 5, 6];
                        let next_bid = choices[rng.gen_range(0..choices.len())].min(branches_ref.len() - 1);
                        let next_t = rng.gen_range(0.25..0.75);
                        if org.vocal_cooldown_sec <= 0.0 {
                            org.vocal_cooldown_sec = 9.0;
                            let cue_kind = if org.species == Species::ScarletMacaw {
                                SoundCueKind::MacawSquawk
                            } else {
                                SoundCueKind::ToucanCroak
                            };
                            self.pending_sounds.push(SoundCue {
                                kind: cue_kind,
                                world_pos: org.pos,
                                gain: 0.58,
                                pitch_mul: rng.gen_range(0.95..1.08),
                            });
                        }
                        org.state = BehaviorState::AvianCanopyFlight {
                            target_branch_id: next_bid,
                            target_t: next_t,
                        };
                    } else {
                        if remaining_sec <= 0.0 {
                            remaining_sec = 8.0;
                        }
                        org.state = BehaviorState::AvianPerchedForage {
                            branch_id: bid,
                            branch_t,
                            remaining_sec,
                        };
                    }
                }

                BehaviorState::AvianCanopyFlight {
                    target_branch_id,
                    target_t,
                } => {
                    let bid = target_branch_id.min(branches_ref.len() - 1);
                    let dest = branches_ref[bid].point_at(target_t) + Vec3::new(0.0, 0.33, 0.0);
                    let diff = dest - org.pos;
                    let dist = diff.length();
                    let fly_speed = if org.species == Species::ScarletMacaw { 4.2 } else { 3.4 };
                    if dist > 0.35 {
                        org.vel = diff / dist * fly_speed;
                        org.pos += org.vel * dt;
                        org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                        org.pose.wing_phase_amp[0] += dt * 15.0;
                        org.pose.wing_phase_amp[1] = 1.0;
                        org.state = BehaviorState::AvianCanopyFlight {
                            target_branch_id: bid,
                            target_t,
                        };
                    } else {
                        org.pos = dest;
                        org.vel = Vec3::ZERO;
                        org.pose.wing_phase_amp = [0.0, 0.0];
                        org.state = BehaviorState::AvianPerchedForage {
                            branch_id: bid,
                            branch_t: target_t,
                            remaining_sec: rng.gen_range(6.0..13.0),
                        };
                    }
                }

                BehaviorState::ButterflyPollinationFlutter {
                    mut target_plant_id,
                    mut hover_timer,
                } => {
                    let pid = target_plant_id.min(self.underbrush.len().saturating_sub(1));
                    let flower_top = self.underbrush[pid].root_pos + Vec3::new(0.0, 1.05, 0.0);
                    let diff = flower_top - org.pos;
                    let dist = diff.length();
                    org.pose.wing_phase_amp[0] += dt * 22.0;
                    org.pose.wing_phase_amp[1] = if is_night { 0.1 } else { 1.0 };

                    if is_night {
                        org.vel *= 0.85;
                    } else if dist > 0.45 {
                        let bob = Vec3::new(0.0, (t_sec * 6.5 + org.id as f32).sin() * 0.55, 0.0);
                        org.vel = (diff / dist) * 1.45 + bob;
                        org.pos += org.vel * dt;
                        org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                    } else {
                        hover_timer -= dt;
                        org.vel = Vec3::new((t_sec * 4.0).cos() * 0.2, (t_sec * 7.0).sin() * 0.25, 0.0);
                        org.pos += org.vel * dt;
                        if hover_timer <= 0.0 {
                            target_plant_id = rng.gen_range(0..self.underbrush.len().max(1));
                            hover_timer = rng.gen_range(2.0..4.5);
                        }
                    }
                    org.state = BehaviorState::ButterflyPollinationFlutter {
                        target_plant_id,
                        hover_timer,
                    };
                }

                BehaviorState::FloorHerbivoreBrowse {
                    mut target_x,
                    mut target_z,
                    mut gnawing_timer,
                } => {
                    if org.species == Species::CentralAmericanAgouti {
                        if let Some(pp) = panther_snapshot {
                            if pp.distance(org.pos) < 3.4 {
                                target_x = if pp.x < org.pos.x {
                                    (org.pos.x + 5.5).min(13.5)
                                } else {
                                    (org.pos.x - 5.5).max(-13.5)
                                };
                                gnawing_timer = 0.0;
                            }
                        }
                    }

                    if gnawing_timer > 0.0 {
                        gnawing_timer -= dt;
                        org.vel = Vec3::ZERO;
                        org.pose.carrying_item = org.species == Species::CentralAmericanAgouti;
                        org.pose.head_pitch = (t_sec * 8.0).sin() * 0.16;
                    } else {
                        org.pose.carrying_item = false;
                        let dest = Vec3::new(target_x, ground_height_at(target_x, target_z), target_z);
                        let diff = Vec3::new(dest.x - org.pos.x, 0.0, dest.z - org.pos.z);
                        let dist = diff.length();
                        let walk_speed = if org.species == Species::LowlandTapir { 0.62 } else { 1.55 };
                        if dist > 0.30 {
                            org.vel = (diff / dist) * walk_speed;
                            org.pos.x += org.vel.x * dt;
                            org.pos.z += org.vel.z * dt;
                            org.pos.y = ground_height_at(org.pos.x, org.pos.z);
                            org.pose.body_yaw = if org.vel.x >= 0.0 { 0.0 } else { PI };
                            org.pose.limb_phase += dt * walk_speed * 4.2;
                            for plant in &mut self.underbrush {
                                let r = if org.species == Species::LowlandTapir { 0.75 } else { 0.28 };
                                let _ = plant.apply_body_contact(org.pos, org.vel, r);
                            }
                        } else {
                            target_x = rng.gen_range(-10.5..11.5);
                            target_z = rng.gen_range(-2.6..0.8);
                            gnawing_timer = rng.gen_range(2.5..5.5);
                        }
                    }
                    org.state = BehaviorState::FloorHerbivoreBrowse {
                        target_x,
                        target_z,
                        gnawing_timer,
                    };
                }

                // Shared chemical trail / dissolved scent plume foraging (Leafcutter Ants OR Kelp Crabs!)
                BehaviorState::AntOutwardTrail {
                    mut patch_pos,
                    mut wandering,
                } => {
                    let active_patch = self.ant_patches.first().cloned();
                    let speed = if is_marine_biome { 0.38 } else { 0.55 };
                    org.pose.carrying_item = false;
                    org.pose.limb_phase += dt * 16.0;

                    if let Some(ap) = active_patch {
                        patch_pos = ap.pos;
                        let to_patch = Vec2::new(patch_pos.x - org.pos.x, patch_pos.z - org.pos.z);
                        let dist = to_patch.length();
                        let grad = self
                            .floor_field
                            .pheromone_gradient(Vec2::new(org.pos.x, org.pos.z));
                        let steer = (to_patch.normalize_or_zero() * 0.78
                            + grad.normalize_or_zero() * 0.22)
                            .normalize_or_zero();
                        org.vel = Vec3::new(steer.x * speed, 0.0, steer.y * speed);
                        org.pos.x = (org.pos.x + org.vel.x * dt).clamp(-15.2, 15.2);
                        org.pos.z = (org.pos.z + org.vel.z * dt).clamp(-12.5, 3.2);
                        org.pos.y = if is_pnw_biome {
                            pnw_creek_forest_height_at(org.pos.x, org.pos.z) + 0.04
                        } else if is_pnw_biome {
                            pnw_creek_forest_height_at(org.pos.x, org.pos.z) + 0.04
                        } else if is_vent_biome {
                            vent_field_height_at(org.pos.x, org.pos.z) + 0.04
                        } else if is_vent_biome {
                            vent_field_height_at(org.pos.x, org.pos.z) + 0.04
                        } else if is_lake_biome {
                            lake_bed_height_at(org.pos.x, org.pos.z) + 0.04
                        } else if is_lake_biome {
                            lake_bed_height_at(org.pos.x, org.pos.z) + 0.04
                        } else if is_lake_biome {
                            lake_bed_height_at(org.pos.x, org.pos.z) + 0.04
                        } else if is_coral_biome {
                            coral_reef_height_at(org.pos.x, org.pos.z) + 0.04
                        } else if is_marine_biome {
                            reef_height_at(org.pos.x, org.pos.z) + 0.04
                        } else {
                            ground_height_at(org.pos.x, org.pos.z)
                        };
                        org.pose.body_yaw = org.vel.z.atan2(org.vel.x);

                        self.floor_field
                            .deposit_ant_pheromone(Vec2::new(org.pos.x, org.pos.z), 0.65 * dt);

                        if dist < 0.45 {
                            if let Some(first_mut) = self.ant_patches.first_mut() {
                                first_mut.remaining_fragments =
                                    first_mut.remaining_fragments.saturating_sub(1);
                            }
                            org.pose.carrying_item = true;
                            org.state = BehaviorState::AntReturningWithLeaf { nest_pos };
                        } else {
                            org.state = BehaviorState::AntOutwardTrail {
                                patch_pos,
                                wandering: 0.0,
                            };
                        }
                    } else {
                        wandering += dt * rng.gen_range(-2.4..2.4);
                        let dir = Vec2::new(wandering.cos(), wandering.sin() * 0.5).normalize_or_zero();
                        let pull_home = Vec2::new(nest_pos.x - org.pos.x, nest_pos.z - org.pos.z) * 0.12;
                        let v2 = (dir + pull_home).normalize_or_zero() * (speed * 0.55);
                        org.vel = Vec3::new(v2.x, 0.0, v2.y);
                        org.pos.x = (org.pos.x + org.vel.x * dt).clamp(-15.0, 15.0);
                        org.pos.z = (org.pos.z + org.vel.z * dt).clamp(-6.0, 2.8);
                        org.pos.y = if is_coral_biome {
                            coral_reef_height_at(org.pos.x, org.pos.z) + 0.04
                        } else if is_marine_biome {
                            reef_height_at(org.pos.x, org.pos.z) + 0.04
                        } else {
                            ground_height_at(org.pos.x, org.pos.z)
                        };
                        org.pose.body_yaw = org.vel.z.atan2(org.vel.x);
                        org.state = BehaviorState::AntOutwardTrail {
                            patch_pos,
                            wandering,
                        };
                    }
                }

                BehaviorState::AntReturningWithLeaf { nest_pos } => {
                    let speed = if is_marine_biome { 0.34 } else { 0.48 };
                    org.pose.carrying_item = true;
                    org.pose.limb_phase += dt * 15.0;
                    let to_nest = Vec2::new(nest_pos.x - org.pos.x, nest_pos.z - org.pos.z);
                    let dist = to_nest.length();
                    let dir = to_nest.normalize_or_zero();
                    org.vel = Vec3::new(dir.x * speed, 0.0, dir.y * speed);
                    org.pos.x += org.vel.x * dt;
                    org.pos.z += org.vel.z * dt;
                    org.pos.y = if is_tidepool_biome {
                        tidepool_bench_height_at(org.pos.x, org.pos.z) + 0.04
                    } else if is_brine_biome {
                        brine_pool_height_at(org.pos.x, org.pos.z) + 0.04
                    } else if is_pnw_biome {
                        pnw_creek_forest_height_at(org.pos.x, org.pos.z) + 0.04
                    } else if is_vent_biome {
                        vent_field_height_at(org.pos.x, org.pos.z) + 0.04
                    } else if is_lake_biome {
                        lake_bed_height_at(org.pos.x, org.pos.z) + 0.04
                    } else if is_coral_biome {
                        coral_reef_height_at(org.pos.x, org.pos.z) + 0.04
                    } else if is_marine_biome {
                        reef_height_at(org.pos.x, org.pos.z) + 0.04
                    } else {
                        ground_height_at(org.pos.x, org.pos.z)
                    };
                    org.pose.body_yaw = org.vel.z.atan2(org.vel.x);

                    if !self.ant_patches.is_empty() {
                        self.floor_field
                            .deposit_ant_pheromone(Vec2::new(org.pos.x, org.pos.z), 1.15 * dt);
                    }

                    if dist < 0.42 {
                        org.pose.carrying_item = false;
                        self.floor_field
                            .deposit_detritus(Vec2::new(nest_pos.x, nest_pos.z), 0.18);
                        let next_patch = self
                            .ant_patches
                            .first()
                            .map(|p| p.pos)
                            .unwrap_or(nest_pos + Vec3::new(3.0, 0.0, 0.0));
                        org.state = BehaviorState::AntOutwardTrail {
                            patch_pos: next_patch,
                            wandering: rng.gen_range(0.0..6.28),
                        };
                    } else {
                        org.state = BehaviorState::AntReturningWithLeaf { nest_pos };
                    }
                }

                BehaviorState::FungalFruiting {
                    mut maturity,
                    mut spore_release_timer,
                } => {
                    maturity = (maturity + 0.012 * (0.5 + self.diurnal.foliage_wetness) * dt).min(1.15);
                    org.scale = 0.65 + 0.55 * maturity;
                    spore_release_timer -= dt;
                    if spore_release_timer <= 0.0 {
                        spore_release_timer = rng.gen_range(14.0..26.0);
                    }
                    org.state = BehaviorState::FungalFruiting {
                        maturity,
                        spore_release_timer,
                    };
                }
            }

            self.organisms[idx] = org;
        }
    }
}
