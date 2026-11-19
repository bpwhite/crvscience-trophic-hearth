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

use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

pub const GRAVITY_M_S2: f32 = 9.81;
pub const KELP_SURFACE_Y: f32 = 15.2;
pub const LAKE_SURFACE_Y: f32 = 8.4;
pub const PNW_CREEK_SURFACE_Y: f32 = 2.65;
pub const BRINE_POOL_SURFACE_Y: f32 = 1.65;

/// Structural canopy branch (Rainforest) or rocky reef arch/ledge (Kelp Forest).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchSegment {
    pub id: usize,
    pub tree_id: usize,
    pub start: Vec3,
    pub end: Vec3,
    pub radius_start: f32,
    pub radius_end: f32,
    pub is_perchable: bool,
}

impl BranchSegment {
    pub fn point_at(&self, t: f32) -> Vec3 {
        self.start.lerp(self.end, t.clamp(0.0, 1.0))
    }

    pub fn radius_at(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        self.radius_start + (self.radius_end - self.radius_start) * t
    }

    pub fn length(&self) -> f32 {
        self.start.distance(self.end)
    }
}

/// Hanging rainforest liana vine modeled as a damped nonlinear pendulum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LianaVine {
    pub id: usize,
    pub anchor: Vec3,
    pub length_m: f32,
    pub theta_rad: f32,
    pub phi_z_rad: f32,
    pub omega_rad_s: f32,
    pub omega_z_rad_s: f32,
    pub damping: f32,
    pub occupied_by_entity: Option<u32>,
}

impl LianaVine {
    pub fn tip_position(&self) -> Vec3 {
        self.position_along(1.0)
    }

    pub fn position_along(&self, frac: f32) -> Vec3 {
        let r = self.length_m * frac.clamp(0.0, 1.0);
        let sin_t = self.theta_rad.sin();
        let cos_t = self.theta_rad.cos();
        let sin_p = self.phi_z_rad.sin();
        let bow = (frac * PI).sin() * 0.14 * self.omega_rad_s.clamp(-2.5, 2.5);
        Vec3::new(
            self.anchor.x + r * sin_t + bow,
            self.anchor.y - r * cos_t * self.phi_z_rad.cos(),
            self.anchor.z + r * sin_p,
        )
    }

    pub fn tip_velocity(&self) -> Vec3 {
        let cos_t = self.theta_rad.cos();
        let sin_t = self.theta_rad.sin();
        Vec3::new(
            self.length_m * self.omega_rad_s * cos_t,
            self.length_m * self.omega_rad_s * sin_t,
            self.length_m * self.omega_z_rad_s * self.phi_z_rad.cos(),
        )
    }

    pub fn apply_tangential_impulse(&mut self, vx_boost: f32, vz_boost: f32) {
        let inv_l = 1.0 / self.length_m.max(0.8);
        self.omega_rad_s = (self.omega_rad_s + vx_boost * inv_l).clamp(-3.4, 3.4);
        self.omega_z_rad_s = (self.omega_z_rad_s + vz_boost * inv_l).clamp(-1.8, 1.8);
    }

    pub fn step(&mut self, dt_sec: f32, wind_x: f32, wind_z: f32) {
        let substeps = 2;
        let h = dt_sec / substeps as f32;
        let inv_l = 1.0 / self.length_m.max(0.8);

        for _ in 0..substeps {
            let acc_theta = -(GRAVITY_M_S2 * inv_l) * self.theta_rad.sin()
                - self.damping * self.omega_rad_s
                + wind_x * 0.35 * inv_l;
            self.omega_rad_s += acc_theta * h;
            self.theta_rad += self.omega_rad_s * h;

            let acc_phi = -(GRAVITY_M_S2 * inv_l) * self.phi_z_rad.sin()
                - (self.damping * 1.25) * self.omega_z_rad_s
                + wind_z * 0.25 * inv_l;
            self.omega_z_rad_s += acc_phi * h;
            self.phi_z_rad += self.omega_z_rad_s * h;
        }
    }
}

/// Buoyant Northeast Pacific Giant Kelp (*Macrocystis pyrifera*) or Bull Kelp (*Nereocystis luetkeana*)
/// anchored by a conical haptera holdfast and swaying in orbital ocean groundswell + swimmer contact!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KelpStalk {
    pub id: usize,
    pub holdfast_pos: Vec3,
    pub height_m: f32,
    pub is_bull_kelp: bool,
    pub depth_tier: u8,
    /// Additional horizontal deflection `(dx, dz)` caused by swimming seals, sharks, or otters!
    pub bend_offset_xz: [f32; 2],
    pub bend_vel_xz: [f32; 2],
    /// Holdfast structural integrity `[0.2, 1.0]` (grazed if sea urchins crowd unchecked).
    pub holdfast_health: f32,
}

impl KelpStalk {
    /// Computes the instantaneous 3D point along the buoyant kelp stipe (`frac in [0, 1]`)
    /// combining orbital wave surge `A * exp(k*(y - H)) * sin(w*t - k*x)` and animal contact deflection.
    pub fn point_along(&self, frac: f32, sim_time_sec: f32, surge_amp: f32) -> Vec3 {
        let f = frac.clamp(0.0, 1.0);
        let y = self.holdfast_pos.y + f * self.height_m;
        let depth_atten = (0.25 + 0.75 * f).powf(1.25);
        let wave_phase = sim_time_sec * 0.82 - self.holdfast_pos.x * 0.24 + self.id as f32 * 0.35;
        let harmonic = (sim_time_sec * 1.65 - self.holdfast_pos.x * 0.42 + f * 2.4).sin() * 0.22;

        let sway_x = (wave_phase.sin() * (0.85 + 0.95 * surge_amp) + harmonic + 0.35) * depth_atten
            + self.bend_offset_xz[0] * f.sqrt();
        let sway_z = ((wave_phase * 0.7).cos() * 0.28 * depth_atten) + self.bend_offset_xz[1] * f.sqrt();

        // Surface canopy layover when Macrocystis reaches the water surface (`y ~ 14.8m`)
        let canopy_stream = if y > KELP_SURFACE_Y - 1.0 {
            (y - (KELP_SURFACE_Y - 1.0)) * 1.35
        } else {
            0.0
        };

        Vec3::new(
            self.holdfast_pos.x + sway_x + canopy_stream,
            y.min(KELP_SURFACE_Y - 0.15),
            self.holdfast_pos.z + sway_z,
        )
    }

    /// Applies hydrodynamic/contact parting impulse when a harbor seal, shark, or otter swims through the kelp!
    pub fn apply_swimmer_contact(&mut self, swimmer_pos: Vec3, swimmer_vel: Vec3, radius: f32) -> f32 {
        if swimmer_pos.y < self.holdfast_pos.y || swimmer_pos.y > self.holdfast_pos.y + self.height_m + 0.5 {
            return 0.0;
        }
        let dx = self.holdfast_pos.x - swimmer_pos.x;
        let dz = self.holdfast_pos.z - swimmer_pos.z;
        let reach = 0.95 + radius;
        let d2 = dx * dx + dz * dz;
        if d2 >= reach * reach {
            return 0.0;
        }
        let d = d2.sqrt().max(0.10);
        let overlap = (reach - d) / reach;
        let speed = swimmer_vel.length();
        let push = overlap * (1.2 + 1.6 * speed);
        self.bend_vel_xz[0] = (self.bend_vel_xz[0] + (dx / d) * push * 2.0).clamp(-3.5, 3.5);
        self.bend_vel_xz[1] = (self.bend_vel_xz[1] + (dz / d) * push * 2.0).clamp(-3.5, 3.5);
        overlap * speed
    }

    pub fn step(&mut self, dt_sec: f32) {
        let stiffness = 6.5; // Buoyant pneumatocyst restoring tension
        let water_drag = 2.8;
        for axis in 0..2 {
            let acc = -stiffness * self.bend_offset_xz[axis] - water_drag * self.bend_vel_xz[axis];
            self.bend_vel_xz[axis] += acc * dt_sec;
            self.bend_offset_xz[axis] =
                (self.bend_offset_xz[axis] + self.bend_vel_xz[axis] * dt_sec).clamp(-1.8, 1.8);
        }
    }
}

/// Interactive understory plant (Rainforest ferns/shrubs or Kelp Forest crimson Rhodymenia / coralline turf).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnderbrushPlant {
    pub id: usize,
    pub root_pos: Vec3,
    pub scale: f32,
    pub kind: UnderbrushKind,
    pub bend_xz: [f32; 2],
    pub bend_vel_xz: [f32; 2],
    pub ripe_berries: u8,
    pub growth_progress: f32,
    pub vigor: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnderbrushKind {
    SwordFernClump,
    BlueberryMelastomeShrub,
    HeliconiaFlowerCluster,
    MonsteraBroadleaf,
    // Kelp Forest benthic understory flora
    CorallineAlgaeTurf,
    CrimsonRhodymeniaBlade,
    // Indo-Pacific Coral Reef holobionts, octocorals & sponges (10 morphotypes)
    StaghornAcropora,
    TableAcropora,
    BrainCoralDome,
    GorgonianSeaFan,
    HostSeaAnemone,
    FolioseMontipora,
    PoritesBoulder,
    PillarCoral,
    BarrelSponge,
    SunCupTubastraea,
    // Temperate Freshwater Lake submerged, floating & emergent macrophytes
    VallisneriaEelgrass,
    CoontailMacrophyte,
    FloatingWaterLily,
    ShorelineCattail,
    SubmergedDriftwoodMoss,
    // Deep-Sea Hydrothermal Vent chemosynthetic holobionts & mineral edifices
    RiftiaTubewormCluster,
    BeggiatoaSulfurMat,
    SulfidePagodaFlange,
    VentMusselBed,
    // PNW Old-Growth Nurse-Log & Understory Flora
    LicoriceFernNurseMoss,
    RedHuckleberryBush,
    // Midwater Pelagic Rift Canyon Wall Suspension Flora/Porifera
    BubblegumOctocoral,
    CloudGlassSponge,
    // Rocky Intertidal & Cold-Seep Brine Pool Flora/Holobionts
    PostelsiaSeaPalm,
    LamellibrachiaSeepBush,
    MethaneClathrateOutcrop,
}

impl UnderbrushPlant {
    pub fn apply_body_contact(&mut self, body_pos: Vec3, body_vel: Vec3, body_radius: f32) -> f32 {
        let reach = 0.85 * self.scale + body_radius;
        let dx = self.root_pos.x - body_pos.x;
        let dz = self.root_pos.z - body_pos.z;
        let dy = (body_pos.y - self.root_pos.y).abs();
        if dy > 1.8 * self.scale {
            return 0.0;
        }
        let dist_sq = dx * dx + dz * dz;
        if dist_sq >= reach * reach {
            return 0.0;
        }
        let dist = dist_sq.sqrt().max(0.08);
        let overlap = (reach - dist) / reach;
        let speed = Vec3::new(body_vel.x, 0.0, body_vel.z).length();
        let push = overlap * (1.6 + 1.9 * speed);

        let nx = dx / dist;
        let nz = dz / dist;
        self.bend_vel_xz[0] = (self.bend_vel_xz[0] + nx * push * 2.4).clamp(-5.5, 5.5);
        self.bend_vel_xz[1] = (self.bend_vel_xz[1] + nz * push * 2.4).clamp(-5.5, 5.5);
        overlap * speed
    }

    pub fn step(&mut self, dt_sec: f32, wind_or_surge_x: f32, par_norm: f32, soil_nutrients: f32) {
        let stiffness = 16.5;
        let damping = 4.2;
        let sway = (wind_or_surge_x * 0.16) + (self.id as f32 * 1.7).sin() * 0.03 * wind_or_surge_x;

        let acc_x = -stiffness * (self.bend_xz[0] - sway) - damping * self.bend_vel_xz[0];
        let acc_z = -stiffness * self.bend_xz[1] - damping * self.bend_vel_xz[1];

        self.bend_vel_xz[0] += acc_x * dt_sec;
        self.bend_vel_xz[1] += acc_z * dt_sec;
        self.bend_xz[0] = (self.bend_xz[0] + self.bend_vel_xz[0] * dt_sec).clamp(-0.85, 0.85);
        self.bend_xz[1] = (self.bend_xz[1] + self.bend_vel_xz[1] * dt_sec).clamp(-0.85, 0.85);

        let photo_rate = (0.008 + 0.036 * par_norm) * (0.65 + 0.55 * soil_nutrients.min(2.0));
        self.vigor = (self.vigor + (photo_rate - 0.006) * dt_sec).clamp(0.35, 1.35);
        if self.kind == UnderbrushKind::BlueberryMelastomeShrub && self.ripe_berries < 10 {
            self.growth_progress += photo_rate * dt_sec * 0.65;
            if self.growth_progress >= 1.0 {
                self.growth_progress -= 1.0;
                self.ripe_berries += 1;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallingDetritus {
    pub id: u32,
    pub pos: Vec3,
    pub vel: Vec3,
    pub radius: f32,
    pub energy_kcal: f32,
    pub kind: DetritusKind,
    pub settled_on_ground: bool,
    pub remaining_mass: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetritusKind {
    RipeCanopyBerry,
    FigFruit,
    GreenLeafFall,
    DeadBranchlet,
    DriftKelpBlade,
    CarbonateCoralSand,
    SunkenAlderLeaf,
    SulfideMineralFloc,
}

pub fn ground_height_at(x: f32, z: f32) -> f32 {
    let macro_slope = (x * 0.14).sin() * 0.18 + (z * 0.22).cos() * 0.14;
    let root_mounds = ((x + 9.5) * 0.55).cos() * ((z + 1.5) * 0.55).sin() * 0.10;
    macro_slope + root_mounds
}

/// Spur-and-groove carbonate reef wall, terraced buttresses & foreground aragonite sand channel `y(x, z)`.
pub fn coral_reef_height_at(x: f32, z: f32) -> f32 {
    let sand_ripples = (x * 2.1 + z * 0.6).sin() * 0.045;
    if z >= 0.35 {
        // Foreground sunlit white aragonite sand channel
        0.16 + sand_ripples + ((x * 0.42).cos().max(0.0)) * 0.14
    } else {
        // Rising spur-and-groove carbonate reef wall with 5 coral spurs & sand chutes
        let back = (0.35 - z).max(0.0);
        let spur_wave = 0.5 + 0.5 * (x * 0.56 + 0.4).cos();
        let wall_rise = (back.powf(1.10) * (0.46 + 0.62 * spur_wave)).min(6.4);
        let terrace_steps = ((wall_rise * 1.4).floor() * 0.22).min(1.4);
        0.22 + wall_rise * 0.78 + terrace_steps + sand_ripples
    }
}

/// Semidiurnal NE Pacific tide height `y` (`1.45m` minus low tide to `4.65m` high surf surge).
pub fn tidepool_water_level(hour_of_day: f32, sim_time_sec: f32, surge_boost: f32) -> f32 {
    let semi = ((hour_of_day - 3.0) / 12.0 * 2.0 * std::f32::consts::PI).cos();
    let swash = (sim_time_sec * 1.65).sin() * (0.14 + 0.22 * surge_boost);
    2.95 + 1.48 * semi + swash
}

/// Pure 1D X-profile flat side-cutaway basalt bench (`x = -16.5..+16.5`) with 3 stepped U-shaped tidepool bowls!
pub fn tidepool_bench_height_at(x: f32, z: f32) -> f32 {
    let s1 = 0.5 * (1.0 - ((x + 10.3) * 1.15).tanh());
    let s2 = 0.5 * (1.0 - ((x + 4.9) * 1.15).tanh());
    let s3 = 0.5 * (1.0 - ((x - 1.7) * 1.15).tanh());
    let s4 = 0.5 * (1.0 - ((x - 7.7) * 1.05).tanh());
    let mut h = 0.22 + 1.38 * s4 + 0.82 * s3 + 0.84 * s2 + 0.58 * s1 + (x * 0.55).sin() * 0.03;
    for &(bx, hw, depth) in &[(-7.6_f32, 2.35_f32, 1.48_f32), (-1.6, 2.65, 1.52), (4.7, 2.45, 1.38)] {
        let u = (x - bx).abs() / hw;
        if u < 1.0 {
            let w = 1.0 - u * u;
            h -= depth * w * w;
        }
    }
    let base = h.max(0.18);
    if z < -2.0 {
        let bt = ((-2.0 - z) / 1.65).clamp(0.0, 1.0);
        base + bt * bt * 0.55
    } else {
        base
    }
}

/// Continental-slope Cold-Seep bathymetry (`1,050m`): left Methane Hydrate mounds (`x < -4.5`) & central/right Hypersaline Brine Lake basin (`x in -3.8..8.8`, surface `1.65m`).
pub fn brine_pool_height_at(x: f32, z: f32) -> f32 {
    let micro = (x * 0.9).sin() * (z * 0.8).cos() * 0.09;
    let d_brine = ((x - 2.4) / 6.2).powi(2) + ((z + 0.4) / 4.4).powi(2);
    if d_brine < 1.0 {
        // Anoxic hypersaline brine lake basin (`y ~ 0.45..1.65m`, shoreline berm at `d_brine ~ 1.0`)
        0.48 + 1.28 * d_brine.powf(1.4) + micro * 0.4
    } else if x < -4.8 {
        // Outcropping golden methane hydrate ("methane ice") mounds on the left!
        let m1 = (-(((x + 8.6) / 2.6).powi(2) + ((z - 0.2) / 2.2).powi(2))).exp() * 1.85;
        1.82 + m1 + micro
    } else {
        1.78 + micro
    }
}

/// Stepped faulted basalt Midwater Pelagic Rift Canyon walls framing a central bottomless bathypelagic chasm (`x in -5.2..7.2`, `y ~ -1.95m`).
pub fn pelagic_rift_height_at(x: f32, z: f32) -> f32 {
    let strata_step = (x * 1.4).sin() * 0.35 + (z * 0.8).cos() * 0.22;
    if x < -5.2 {
        let wall_t = ((-x - 5.2) / 9.5).clamp(0.0, 1.0);
        let terrace = (wall_t * 4.0).floor() * 2.35 + (wall_t * 4.0).fract() * 1.65;
        (1.15 + terrace + strata_step).clamp(0.4, 12.8)
    } else if x > 7.2 {
        let wall_t = ((x - 7.2) / 8.2).clamp(0.0, 1.0);
        let terrace = (wall_t * 3.5).floor() * 2.25 + (wall_t * 3.5).fract() * 1.55;
        (0.95 + terrace + strata_step).clamp(0.4, 10.8)
    } else {
        // Central bathypelagic rift chasm drop-off!
        -1.95 + (x * 0.8 + z * 0.5).sin() * 0.14
    }
}

/// Split-level PNW Old-Growth forest bank (`x < 1.8`, `y ~ 2.9..3.5m`) dropping into a clear gravel Salmon Creek (`x >= 1.8`, `y ~ 0.4..1.1m`, waterline `2.65m`).
pub fn pnw_creek_forest_height_at(x: f32, z: f32) -> f32 {
    let micro = (x * 0.75).sin() * (z * 0.65).cos() * 0.11;
    if x < 1.2 {
        3.05 + ((-x - 2.0).max(0.0) * 0.04).min(0.45) + micro
    } else if x < 2.6 {
        let t = ((x - 1.2) / 1.4).clamp(0.0, 1.0);
        (3.05 * (1.0 - t) + 0.68 * t) + micro * 0.5
    } else {
        let far_bar = ((x - 11.2).max(0.0) * 0.52).min(2.2);
        0.58 + far_bar + (x * 1.6 + z).sin().abs() * 0.09
    }
}

/// Mid-ocean ridge pillow-basalt & sulfide-mound hydrothermal vent bathymetry `y(x, z)` at `2,600m` depth.
pub fn vent_field_height_at(x: f32, z: f32) -> f32 {
    let central_mound = (-(x * x * 0.045 + (z + 1.2) * (z + 1.2) * 0.18)).exp() * 1.55;
    let west_mound = (-((x + 6.8).powi(2) * 0.08 + (z + 1.5).powi(2) * 0.20)).exp() * 1.25;
    let east_mound = (-((x - 5.4).powi(2) * 0.07 + (z + 1.4).powi(2) * 0.18)).exp() * 1.35;
    let pillows = (x * 0.95).sin().abs() * (z * 0.85).cos().abs() * 0.24;
    0.16 + central_mound + west_mound + east_mound + pillows
}

/// Concave glacial-cobble & littoral reed-bank lakebed bathymetry `y(x, z)` (waterline at `LAKE_SURFACE_Y = 8.4m`).
pub fn lake_bed_height_at(x: f32, z: f32) -> f32 {
    // Rises softly on far left/right littoral reed banks (`|x| > 8.5`) toward cattail shallows!
    let bank_rise = ((x.abs() - 8.2).max(0.0) * 0.42).powf(1.25).min(5.8);
    let back_slope = ((-z - 1.5).max(0.0) * 0.22).min(2.2);
    let cobble = (x * 0.65).sin() * (z * 0.55).cos() * 0.14;
    0.18 + bank_rise + back_slope + cobble
}

/// Rocky basalt & granite reef bathymetry `y(x, z)` for the Pacific Kelp Forest.
pub fn reef_height_at(x: f32, z: f32) -> f32 {
    let ledges = (x * 0.19).sin() * 0.34 + (z * 0.27).cos() * 0.26;
    let boulders = ((x - 3.2) * 0.62).cos() * ((z + 1.2) * 0.58).sin() * 0.24;
    ledges + boulders
}

/// Orbital horizontal water velocity `u_x(x, y, t)` in `m/s` driven by Pacific groundswell.
pub fn ocean_swell_vx(x: f32, y: f32, sim_time_sec: f32, surge_boost: f32) -> f32 {
    let atten = (0.30 + 0.70 * (y / KELP_SURFACE_Y).clamp(0.0, 1.0)).powf(1.2);
    let phase = sim_time_sec * 0.82 - x * 0.24;
    phase.cos() * (0.45 + 0.65 * surge_boost) * atten
}

impl FallingDetritus {
    pub fn step(&mut self, dt_sec: f32, branches: &[BranchSegment]) -> bool {
        let mut just_impacted = false;
        if !self.settled_on_ground {
            let is_marine = matches!(self.kind, DetritusKind::DriftKelpBlade | DetritusKind::CarbonateCoralSand | DetritusKind::SunkenAlderLeaf | DetritusKind::SulfideMineralFloc);
            let eff_g = if is_marine { 1.35 } else { GRAVITY_M_S2 }; // Buoyancy-reduced sinking in seawater!
            self.vel.y -= eff_g * dt_sec;
            let drag = match self.kind {
                DetritusKind::DriftKelpBlade | DetritusKind::CarbonateCoralSand => 3.4,
                DetritusKind::GreenLeafFall => 2.8,
                _ => 0.18,
            };
            if matches!(self.kind, DetritusKind::GreenLeafFall | DetritusKind::DriftKelpBlade) {
                self.vel.x += (self.pos.y * 2.1).sin() * 1.4 * dt_sec;
            }
            self.vel *= (1.0 - drag * dt_sec).max(0.1);
            self.pos += self.vel * dt_sec;

            for br in branches {
                let ab = br.end - br.start;
                let len_sq = ab.length_squared().max(1e-5);
                let t = ((self.pos - br.start).dot(ab) / len_sq).clamp(0.0, 1.0);
                let closest = br.start + ab * t;
                let r_sum = br.radius_at(t) + self.radius;
                let diff = self.pos - closest;
                let d_sq = diff.length_squared();
                if d_sq < r_sum * r_sum && self.vel.y < -0.2 {
                    let n = diff.normalize_or_zero();
                    let n = if n.length_squared() < 0.5 { Vec3::Y } else { n };
                    self.pos = closest + n * (r_sum + 0.01);
                    let vn = self.vel.dot(n);
                    if vn < 0.0 {
                        self.vel -= n * (1.35 * vn);
                        self.vel.x += 0.25;
                        just_impacted = true;
                    }
                }
            }

            let floor_y = if is_marine {
                reef_height_at(self.pos.x, self.pos.z)
            } else {
                ground_height_at(self.pos.x, self.pos.z)
            } + self.radius * 0.65;

            if self.pos.y <= floor_y {
                self.pos.y = floor_y;
                if self.vel.y.abs() > 1.2 && !matches!(self.kind, DetritusKind::GreenLeafFall | DetritusKind::DriftKelpBlade) {
                    self.vel.y = -self.vel.y * 0.28;
                    self.vel.x *= 0.55;
                    self.vel.z *= 0.55;
                    just_impacted = true;
                } else {
                    self.vel = Vec3::ZERO;
                    self.settled_on_ground = true;
                    just_impacted = true;
                }
            }
        }
        just_impacted
    }
}
