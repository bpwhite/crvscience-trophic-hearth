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

use crate::models_3d::{append_flora_geometry, append_organism_geometry, Tri3D};
use crate::physics::{brine_pool_height_at, coral_reef_height_at, ground_height_at, lake_bed_height_at, pelagic_rift_height_at, pnw_creek_forest_height_at, reef_height_at, tidepool_bench_height_at, tidepool_water_level, vent_field_height_at, BRINE_POOL_SURFACE_Y, KELP_SURFACE_Y, LAKE_SURFACE_Y, PNW_CREEK_SURFACE_Y};
use crate::trophic::{BiomeKind, DiurnalAtmosphere, TrophicFocus, TrophicStratum};
use crate::world::RainforestWorld;
use egui::{Color32, Mesh, Painter, Pos2, Rect, Shape, Stroke, Vec2 as EguiVec2};
use glam::{Mat4, Vec3, Vec4};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraPreset {
    SideDiorama,
    UnderbrushStalkMacro,
    CanopyAmbushPerch,
    SoilDecomposerLens,
    FreeOrbit,
}

#[derive(Debug, Clone)]
pub struct DioramaCamera {
    pub preset: CameraPreset,
    pub target: Vec3,
    pub yaw_rad: f32,
    pub pitch_rad: f32,
    pub distance_m: f32,
    pub fov_y_rad: f32,
    pub breathing_enabled: bool,
    goal_target: Vec3,
    goal_yaw_rad: f32,
    goal_pitch_rad: f32,
    goal_distance_m: f32,
    pub interpolating: bool,
}

impl Default for DioramaCamera {
    fn default() -> Self {
        let (target, yaw_rad, pitch_rad, distance_m) = Self::preset_pose(CameraPreset::SideDiorama);
        Self {
            preset: CameraPreset::SideDiorama,
            target,
            yaw_rad,
            pitch_rad,
            distance_m,
            fov_y_rad: 44.0_f32.to_radians(),
            breathing_enabled: true,
            goal_target: target,
            goal_yaw_rad: yaw_rad,
            goal_pitch_rad: pitch_rad,
            goal_distance_m: distance_m,
            interpolating: false,
        }
    }
}

impl DioramaCamera {
    fn preset_pose(preset: CameraPreset) -> (Vec3, f32, f32, f32) {
        match preset {
            CameraPreset::SideDiorama => (Vec3::new(0.0, 6.0, -1.0), 0.0, 0.04, 23.8),
            CameraPreset::UnderbrushStalkMacro => (Vec3::new(-1.5, 2.2, 0.2), 0.14, 0.08, 9.8),
            CameraPreset::CanopyAmbushPerch => (Vec3::new(-1.0, 11.8, -0.8), -0.08, 0.04, 12.8),
            CameraPreset::SoilDecomposerLens => (Vec3::new(-4.5, 0.4, 0.6), 0.06, 0.16, 8.4),
            CameraPreset::FreeOrbit => (Vec3::new(0.0, 6.0, -1.0), 0.0, 0.04, 23.8),
        }
    }

    /// Starts a smooth exponential transition toward `preset` (or keeps current pose for `FreeOrbit`).
    pub fn apply_preset(&mut self, preset: CameraPreset) {
        self.preset = preset;
        if preset == CameraPreset::FreeOrbit {
            self.cancel_interpolation();
            return;
        }
        let (gt, gy, gp, gd) = Self::preset_pose(preset);
        self.goal_target = gt;
        self.goal_yaw_rad = gy;
        self.goal_pitch_rad = gp;
        self.goal_distance_m = gd;
        self.interpolating = true;
    }

    /// Immediately snaps camera state and interpolation target to `preset` (used for headless captures & biome switches).
    pub fn snap_to_preset(&mut self, preset: CameraPreset) {
        self.preset = preset;
        if preset != CameraPreset::FreeOrbit {
            let (gt, gy, gp, gd) = Self::preset_pose(preset);
            self.target = gt;
            self.yaw_rad = gy;
            self.pitch_rad = gp;
            self.distance_m = gd;
            self.goal_target = gt;
            self.goal_yaw_rad = gy;
            self.goal_pitch_rad = gp;
            self.goal_distance_m = gd;
        }
        self.interpolating = false;
    }

    /// Cancels any active preset glide and anchors the goal to the current pose.
    pub fn cancel_interpolation(&mut self) {
        self.goal_target = self.target;
        self.goal_yaw_rad = self.yaw_rad;
        self.goal_pitch_rad = self.pitch_rad;
        self.goal_distance_m = self.distance_m;
        self.interpolating = false;
    }

    /// Applies scroll zoom without letting an in-progress preset transition override the user's zoom distance.
    pub fn zoom_by(&mut self, delta_m: f32) {
        self.distance_m = (self.distance_m + delta_m).clamp(4.5, 32.0);
        self.goal_distance_m = self.distance_m;
    }

    /// Advances smooth preset interpolation using critically damped exponential smoothing.
    pub fn step(&mut self, dt_sec: f32) {
        if !self.interpolating || self.preset == CameraPreset::FreeOrbit {
            return;
        }
        let dt = dt_sec.clamp(0.0, 0.1);
        if dt <= 0.0 {
            return;
        }
        let alpha = 1.0 - (-dt * 8.5).exp();
        self.target = self.target.lerp(self.goal_target, alpha);
        self.yaw_rad += (self.goal_yaw_rad - self.yaw_rad) * alpha;
        self.pitch_rad += (self.goal_pitch_rad - self.pitch_rad) * alpha;
        self.distance_m += (self.goal_distance_m - self.distance_m) * alpha;

        if (self.target - self.goal_target).length() < 1e-3
            && (self.yaw_rad - self.goal_yaw_rad).abs() < 1e-4
            && (self.pitch_rad - self.goal_pitch_rad).abs() < 1e-4
            && (self.distance_m - self.goal_distance_m).abs() < 1e-3
        {
            self.target = self.goal_target;
            self.yaw_rad = self.goal_yaw_rad;
            self.pitch_rad = self.goal_pitch_rad;
            self.distance_m = self.goal_distance_m;
            self.interpolating = false;
        }
    }

    pub fn eye_position(&self, sim_time_sec: f64) -> Vec3 {
        let breath_x = if self.breathing_enabled && self.preset != CameraPreset::FreeOrbit {
            (sim_time_sec as f32 * 0.16).sin() * 0.42
        } else {
            0.0
        };
        let breath_y = if self.breathing_enabled && self.preset != CameraPreset::FreeOrbit {
            (sim_time_sec as f32 * 0.11).cos() * 0.18
        } else {
            0.0
        };
        let cp = self.pitch_rad.cos();
        let offset = Vec3::new(
            self.distance_m * cp * self.yaw_rad.sin() + breath_x,
            self.distance_m * self.pitch_rad.sin() + breath_y,
            self.distance_m * cp * self.yaw_rad.cos(),
        );
        self.target + offset
    }

    pub fn view_proj(&self, aspect: f32, sim_time_sec: f64) -> (Mat4, Vec3) {
        let eye = self.eye_position(sim_time_sec);
        let view = Mat4::look_at_rh(eye, self.target, Vec3::Y);
        let proj = Mat4::perspective_rh(self.fov_y_rad, aspect.max(0.25), 0.4, 95.0);
        (proj * view, eye)
    }
}

fn project_point(vp: &Mat4, rect: Rect, p: Vec3) -> Option<(Pos2, f32)> {
    let clip: Vec4 = *vp * Vec4::new(p.x, p.y, p.z, 1.0);
    if clip.w <= 0.15 {
        return None;
    }
    let ndc = clip.truncate() / clip.w;
    if ndc.x.abs() > 12.0 || ndc.y.abs() > 12.0 || ndc.z < 0.0 || ndc.z > 1.0 {
        return None;
    }
    let sx = rect.left() + (ndc.x * 0.5 + 0.5) * rect.width();
    let sy = rect.top() + (0.5 - ndc.y * 0.5) * rect.height();
    Some((Pos2::new(sx, sy), clip.w))
}

fn to_color32(rgba: [f32; 4]) -> Color32 {
    let r = (rgba[0].clamp(0.0, 1.0) * 255.0) as u8;
    let g = (rgba[1].clamp(0.0, 1.0) * 255.0) as u8;
    let b = (rgba[2].clamp(0.0, 1.0) * 255.0) as u8;
    let a = (rgba[3].clamp(0.0, 1.0) * 255.0) as u8;
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

/// Computes surface wave caustics (in sunlit aquatic zones) or forest canopy dapple.
/// Strictly returns `1.0` when `atmo.par_umol_m2_s <= 0.0` (e.g. aphotic vents, cold seeps, and night).
pub fn surface_caustic_or_dapple(
    biome: BiomeKind,
    atmo: &DiurnalAtmosphere,
    center: Vec3,
    t_sec: f32,
) -> f32 {
    if atmo.par_umol_m2_s <= 0.0 {
        return 1.0;
    }
    let solar_atten = (atmo.par_umol_m2_s / 600.0).clamp(0.0, 1.0);
    let submerged = match biome {
        BiomeKind::PacificKelpForest
        | BiomeKind::IndoPacificCoralReef
        | BiomeKind::MidwaterPelagicRift => true,
        BiomeKind::TemperateFreshwaterLake => center.y <= LAKE_SURFACE_Y + 0.15,
        BiomeKind::RockyIntertidalTidepool => center.y <= 3.6,
        BiomeKind::PnwOldGrowthSalmonCreek => center.y <= PNW_CREEK_SURFACE_Y + 0.15 && center.x > 1.2,
        BiomeKind::DeepSeaHydrothermalVent
        | BiomeKind::ColdSeepBrinePool
        | BiomeKind::NeotropicalRainforest => false,
    };
    if submerged {
        let c1 = (center.x * 1.45 + t_sec * 1.8).sin() * (center.z * 1.35 - t_sec * 1.4).cos();
        let c2 = ((center.x - center.z) * 2.2 + t_sec * 2.3).sin();
        let caustic = 0.82 + 0.34 * (c1 * 0.6 + c2 * 0.4).max(0.0);
        1.0 + (caustic - 1.0) * solar_atten
    } else {
        let dapple =
            0.76 + 0.24 * ((center.x * 0.8 + t_sec * 0.3).sin() * (center.z * 0.7 - t_sec * 0.2).cos());
        1.0 + (dapple - 1.0) * solar_atten
    }
}

/// Selects up to 4 well-spaced 3D anchor points on actual `L1` producer stands
/// (`kelp_stalks`, `trees`, or `underbrush`) for calm food-web energy-flux arcs.
pub fn producer_flux_anchors(world: &RainforestWorld, t_sec: f32) -> Vec<Vec3> {
    let mut anchors = Vec::with_capacity(4);
    if !world.kelp_stalks.is_empty() {
        let step = (world.kelp_stalks.len() / 4).max(1);
        for stalk in world.kelp_stalks.iter().step_by(step).take(4) {
            anchors.push(stalk.point_along(0.62, t_sec, 0.35));
        }
    } else if !world.trees.is_empty() {
        let step = (world.trees.len() / 4).max(1);
        for tree in world.trees.iter().step_by(step).take(4) {
            anchors.push(
                tree.base_pos + Vec3::new(0.0, (tree.height_m * 0.58).min(11.0), 0.0),
            );
        }
    }
    if anchors.len() < 4 && !world.underbrush.is_empty() {
        let needed = 4 - anchors.len();
        let step = (world.underbrush.len() / needed.max(1)).max(1);
        for plant in world.underbrush.iter().step_by(step).take(needed) {
            anchors.push(
                plant.root_pos
                    + Vec3::new(
                        plant.bend_xz[0] * 0.3,
                        (0.45 * plant.scale).max(0.32),
                        plant.bend_xz[1] * 0.3,
                    ),
            );
        }
    }
    anchors
}

pub fn render_rainforest_viewport(
    painter: &Painter,
    rect: Rect,
    world: &RainforestWorld,
    camera: &DioramaCamera,
    selected_id: Option<u32>,
) -> Vec<(u32, Pos2)> {
    let is_kelp = world.biome == BiomeKind::PacificKelpForest;
    let is_coral = world.biome == BiomeKind::IndoPacificCoralReef;
    let is_lake = world.biome == BiomeKind::TemperateFreshwaterLake;
    let is_vent = world.biome == BiomeKind::DeepSeaHydrothermalVent;
    let is_pnw = world.biome == BiomeKind::PnwOldGrowthSalmonCreek;
    let is_rift = world.biome == BiomeKind::MidwaterPelagicRift;
    let is_tidepool = world.biome == BiomeKind::RockyIntertidalTidepool;
    let is_brine = world.biome == BiomeKind::ColdSeepBrinePool;
    let is_marine = world.biome.is_marine();
    let is_aquatic = world.biome.is_aquatic();
    let atmo = world.diurnal.atmosphere_for_biome(world.biome);
    let aspect = rect.width() / rect.height().max(1.0);
    let (vp, eye) = camera.view_proj(aspect, world.sim_time_sec);
    let t_sec = world.sim_time_sec as f32;

    // 1. Sky / Pelagic Water-Column Backdrop
    let mut bg_mesh = Mesh::default();
    let top_c = to_color32([atmo.sky_top_rgb[0], atmo.sky_top_rgb[1], atmo.sky_top_rgb[2], 1.0]);
    let hor_c = to_color32([
        atmo.sky_horizon_rgb[0],
        atmo.sky_horizon_rgb[1],
        atmo.sky_horizon_rgb[2],
        1.0,
    ]);
    let floor_c = if is_marine {
        to_color32([0.04, 0.06, 0.08, 1.0])
    } else {
        to_color32([0.07, 0.05, 0.04, 1.0])
    };
    let mid_y = rect.top() + rect.height() * 0.78;

    let idx0 = bg_mesh.vertices.len() as u32;
    bg_mesh.colored_vertex(rect.left_top(), top_c);
    bg_mesh.colored_vertex(rect.right_top(), top_c);
    bg_mesh.colored_vertex(Pos2::new(rect.right(), mid_y), hor_c);
    bg_mesh.colored_vertex(Pos2::new(rect.left(), mid_y), hor_c);
    bg_mesh.colored_vertex(rect.right_bottom(), floor_c);
    bg_mesh.colored_vertex(rect.left_bottom(), floor_c);
    bg_mesh.add_triangle(idx0, idx0 + 1, idx0 + 2);
    bg_mesh.add_triangle(idx0, idx0 + 2, idx0 + 3);
    bg_mesh.add_triangle(idx0 + 3, idx0 + 2, idx0 + 4);
    bg_mesh.add_triangle(idx0 + 3, idx0 + 4, idx0 + 5);
    painter.add(Shape::mesh(bg_mesh));

    // 2. Assemble 3D Scene Triangles
    let mut tris: Vec<Tri3D> = Vec::with_capacity(7500);

    let gx_steps = 20;
    let gz_steps = 10;
    for iz in 0..gz_steps {
        let z0 = -15.5 + (iz as f32 / gz_steps as f32) * 18.5;
        let z1 = -15.5 + ((iz + 1) as f32 / gz_steps as f32) * 18.5;
        for ix in 0..gx_steps {
            let x0 = -16.5 + (ix as f32 / gx_steps as f32) * 33.0;
            let x1 = -16.5 + ((ix + 1) as f32 / gx_steps as f32) * 33.0;
            let h_fn: fn(f32, f32) -> f32 = if is_tidepool { tidepool_bench_height_at } else if is_brine { brine_pool_height_at } else if is_rift { pelagic_rift_height_at } else if is_pnw { pnw_creek_forest_height_at } else if is_vent { vent_field_height_at } else if is_lake { lake_bed_height_at } else if is_coral { coral_reef_height_at } else if is_kelp { reef_height_at } else { ground_height_at };
            let p00 = Vec3::new(x0, h_fn(x0, z0), z0);
            let p10 = Vec3::new(x1, h_fn(x1, z0), z0);
            let p11 = Vec3::new(x1, h_fn(x1, z1), z1);
            let p01 = Vec3::new(x0, h_fn(x0, z1), z1);

            let (substrate, sub_emissive) = if is_vent {
                if ix == 9 || ix == 10 {
                    ([0.85, 0.28, 0.06, 1.0], 0.72) // Glowing axial hydrothermal basalt crack!
                } else if (ix + iz) % 2 == 0 {
                    ([0.09, 0.11, 0.14, 1.0], 0.0) // Glassy black pillow basalt
                } else {
                    ([0.22, 0.14, 0.09, 1.0], 0.0) // Iron-oxyhydroxide hydrothermal rust
                }
            } else {
                ([0.0; 4], 0.0)
            };
            let substrate = if is_tidepool {
                if (ix + iz) % 2 == 0 {
                    [0.18, 0.20, 0.24, 1.0] // Wet wave-cut dark Pacific basalt
                } else {
                    [0.32, 0.24, 0.30, 1.0] // Coralline-encrusted intertidal rock
                }
            } else if is_brine {
                if p00.y < 1.55 {
                    [0.06, 0.05, 0.16, 1.0] // Dense 120-PSU hypersaline brine lake basin
                } else if x0 < -5.2 {
                    [0.42, 0.34, 0.18, 1.0] // Methane hydrate sediment mound
                } else {
                    [0.16, 0.18, 0.22, 1.0] // Continental-slope cold-seep mud
                }
            } else if is_rift {
                if p00.y < -1.2 {
                    [0.02, 0.05, 0.13, 1.0] // Bottomless bathypelagic trench abyss!
                } else if (ix + iz) % 2 == 0 {
                    [0.10, 0.15, 0.25, 1.0] // Stepped faulted pelagic basalt wall
                } else {
                    [0.14, 0.20, 0.32, 1.0] // Marine-snow dusted canyon ledge
                }
            } else if is_pnw {
                if x0 > 1.8 && p00.y < 2.2 {
                    [0.34, 0.36, 0.38, 1.0] // River cobble salmon-redd streambed!
                } else if (ix + iz) % 2 == 0 {
                    [0.16, 0.36, 0.14, 1.0] // Deep old-growth feather-moss carpet
                } else {
                    [0.24, 0.20, 0.13, 1.0] // Rich Douglas-fir needle humus
                }
            } else if is_vent {
                substrate
            } else if is_lake {
                if p00.y > 2.2 {
                    [0.32, 0.42, 0.24, 1.0] // Littoral mossy reed bank
                } else if (ix + iz) % 2 == 0 {
                    [0.36, 0.33, 0.26, 1.0] // Glacial cobble & golden silt
                } else {
                    [0.28, 0.27, 0.22, 1.0] // Submerged organic sediment
                }
            } else if is_coral {
                if p00.y > 1.2 {
                    [0.62, 0.52, 0.54, 1.0] // Coralline-encrusted carbonate reef wall!
                } else if (ix + iz) % 2 == 0 {
                    [0.88, 0.85, 0.76, 1.0] // White aragonite coral sand channel!
                } else {
                    [0.78, 0.74, 0.66, 1.0] // Carbonate reef rubble
                }
            } else if is_kelp {
                if (ix + iz) % 2 == 0 {
                    [0.20, 0.23, 0.26, 1.0]
                } else {
                    [0.28, 0.26, 0.24, 1.0]
                }
            } else if (ix + iz) % 2 == 0 {
                [0.14, 0.28, 0.13, 1.0]
            } else {
                [0.17, 0.24, 0.12, 1.0]
            };
            tris.push(Tri3D::new(p00, p01, p11, substrate, 0.88, sub_emissive, TrophicStratum::Decomposers));
            tris.push(Tri3D::new(p00, p11, p10, substrate, 0.88, sub_emissive, TrophicStratum::Decomposers));

            if iz + 1 == gz_steps {
                let b0 = Vec3::new(x0, -2.1, z1);
                let b1 = Vec3::new(x1, -2.1, z1);
                let wall_col = if is_coral {
                    [0.54, 0.50, 0.44, 1.0]
                } else if is_kelp {
                    [0.13, 0.15, 0.19, 1.0]
                } else {
                    [0.24, 0.15, 0.09, 1.0]
                };
                tris.push(Tri3D::new(p01, b0, b1, wall_col, 0.95, 0.0, TrophicStratum::Decomposers));
                tris.push(Tri3D::new(p01, b1, p11, wall_col, 0.95, 0.0, TrophicStratum::Decomposers));
            }
        }
    }

    // Ocean Surface Shimmer Ceiling in Kelp Forest!
    if is_pnw {
        // Clear Coast Range Spawning Salmon Creek Waterline (`x = 1.6..12.8`, `y = 2.65m`)!
        for ix in 0..6 {
            let x0 = 1.6 + ix as f32 * 1.85;
            let x1 = x0 + 1.85;
            let y0 = PNW_CREEK_SURFACE_Y + (t_sec * 3.2 - x0 * 0.9).sin() * 0.04;
            let y1 = PNW_CREEK_SURFACE_Y + (t_sec * 3.2 - x1 * 0.9).sin() * 0.04;
            let s00 = Vec3::new(x0, y0, -8.0);
            let s10 = Vec3::new(x1, y1, -8.0);
            let s11 = Vec3::new(x1, y1, 2.8);
            let s01 = Vec3::new(x0, y0, 2.8);
            tris.push(Tri3D::new(s00, s11, s01, [0.46, 0.88, 0.92, 0.32], 0.15, 0.16, TrophicStratum::Producers));
            tris.push(Tri3D::new(s00, s10, s11, [0.46, 0.88, 0.92, 0.32], 0.15, 0.16, TrophicStratum::Producers));
        }
    }
    if is_tidepool {
        let tide_y = tidepool_water_level(world.diurnal.hour_of_day, t_sec, world.diurnal.rain_intensity) - 0.45;
        let eff_y = |x: f32| -> f32 {
            let mut lv = tide_y + (t_sec * 2.4 - x * 0.55).sin() * 0.05;
            if (-10.2..=-5.0).contains(&x) { lv = lv.max(3.12); }
            else if (-5.0..=1.7).contains(&x) { lv = lv.max(2.28); }
            else if (1.7..=7.7).contains(&x) { lv = lv.max(1.46); }
            lv
        };
        for ix in 0..48 {
            let x0 = -16.0 + ix as f32 * (32.0 / 48.0);
            let x1 = x0 + (32.0 / 48.0);
            let r0 = tidepool_bench_height_at(x0, 2.8);
            let r1 = tidepool_bench_height_at(x1, 2.8);
            let w0 = eff_y(x0);
            let w1 = eff_y(x1);
            if w0 > r0 + 0.01 || w1 > r1 + 0.01 {
                let wy0 = w0.max(r0);
                let wy1 = w1.max(r1);
                // Vertical front-glass cutaway water pane!
                let f00 = Vec3::new(x0, r0, 2.82);
                let f10 = Vec3::new(x1, r1, 2.82);
                let f11 = Vec3::new(x1, wy1, 2.82);
                let f01 = Vec3::new(x0, wy0, 2.82);
                tris.push(Tri3D::new(f00, f10, f11, [0.08, 0.72, 0.84, 0.34], 0.10, 0.18, TrophicStratum::Producers));
                tris.push(Tri3D::new(f00, f11, f01, [0.08, 0.72, 0.84, 0.34], 0.10, 0.18, TrophicStratum::Producers));
                // Horizontal top water sheet
                let s00 = Vec3::new(x0, wy0, -4.0);
                let s10 = Vec3::new(x1, wy1, -4.0);
                let s11 = Vec3::new(x1, wy1, 2.82);
                let s01 = Vec3::new(x0, wy0, 2.82);
                tris.push(Tri3D::new(s00, s11, s01, [0.22, 0.80, 0.88, 0.28], 0.12, 0.14, TrophicStratum::Producers));
                tris.push(Tri3D::new(s00, s10, s11, [0.22, 0.80, 0.88, 0.28], 0.12, 0.14, TrophicStratum::Producers));
            }
        }
    }
    if is_brine {
        // Silvery-violet 120-PSU Hypersaline Brine Lake ("Underwater Lake") surface at Y = 1.65m!
        for ix in 0..8 {
            let x0 = -3.6 + ix as f32 * 1.5;
            let x1 = x0 + 1.5;
            let y0 = BRINE_POOL_SURFACE_Y + (t_sec * 1.1 - x0 * 0.5).sin() * 0.03;
            let y1 = BRINE_POOL_SURFACE_Y + (t_sec * 1.1 - x1 * 0.5).sin() * 0.03;
            let s00 = Vec3::new(x0, y0, -4.4);
            let s10 = Vec3::new(x1, y1, -4.4);
            let s11 = Vec3::new(x1, y1, 3.2);
            let s01 = Vec3::new(x0, y0, 3.2);
            tris.push(Tri3D::new(s00, s11, s01, [0.46, 0.42, 0.86, 0.62], 0.08, 0.32, TrophicStratum::Producers));
            tris.push(Tri3D::new(s00, s10, s11, [0.46, 0.42, 0.86, 0.62], 0.08, 0.32, TrophicStratum::Producers));
        }
    }
    if is_aquatic && !is_vent && !is_rift && !is_tidepool && !is_brine {
        let surf_base_y = if is_lake { LAKE_SURFACE_Y } else { KELP_SURFACE_Y };
        let wave_amp = if is_lake { 0.06 } else { 0.18 };
        for ix in 0..16 {
            let x0 = -16.0 + ix as f32 * 2.0;
            let x1 = x0 + 2.0;
            let y0 = surf_base_y + (t_sec * 1.4 - x0 * 0.35).sin() * wave_amp;
            let y1 = surf_base_y + (t_sec * 1.4 - x1 * 0.35).sin() * wave_amp;
            let s00 = Vec3::new(x0, y0, -12.0);
            let s10 = Vec3::new(x1, y1, -12.0);
            let s11 = Vec3::new(x1, y1, 2.8);
            let s01 = Vec3::new(x0, y0, 2.8);
            let surf_col = [0.42, 0.88, 0.96, 0.24];
            tris.push(Tri3D::new(s00, s11, s01, surf_col, 0.15, 0.18, TrophicStratum::Producers));
            tris.push(Tri3D::new(s00, s10, s11, surf_col, 0.15, 0.18, TrophicStratum::Producers));
        }
    }

    // Ant Pheromone Trail OR Benthic Dissolved Scent Plume
    for (xz, conc) in world.floor_field.visible_pheromone_cells(0.18) {
        let y = (if is_tidepool {
            tidepool_bench_height_at(xz.x, xz.y)
        } else if is_brine {
            brine_pool_height_at(xz.x, xz.y)
        } else if is_rift {
            2.2
        } else if is_pnw {
            pnw_creek_forest_height_at(xz.x, xz.y)
        } else if is_vent {
            vent_field_height_at(xz.x, xz.y)
        } else if is_lake {
            lake_bed_height_at(xz.x, xz.y)
        } else if is_coral {
            coral_reef_height_at(xz.x, xz.y)
        } else if is_kelp {
            reef_height_at(xz.x, xz.y)
        } else {
            ground_height_at(xz.x, xz.y)
        }) + 0.03;
        let alpha = (conc * 0.20).clamp(0.10, 0.62);
        let r = 0.20;
        let col = if is_marine {
            [0.32, 0.88, 0.92, alpha]
        } else {
            [0.62, 0.92, 0.34, alpha]
        };
        let p0 = Vec3::new(xz.x - r, y, xz.y - r);
        let p1 = Vec3::new(xz.x + r, y, xz.y - r);
        let p2 = Vec3::new(xz.x + r, y, xz.y + r);
        let p3 = Vec3::new(xz.x - r, y, xz.y + r);
        tris.push(Tri3D::new(p0, p2, p1, col, 0.4, 0.32, TrophicStratum::Decomposers));
        tris.push(Tri3D::new(p0, p3, p2, col, 0.4, 0.32, TrophicStratum::Decomposers));
    }

    let tree_tuples: Vec<(Vec3, f32, f32, f32, u8)> = world
        .trees
        .iter()
        .map(|t| (t.base_pos, t.height_m, t.trunk_radius_m, t.crown_radius_m, t.depth_tier))
        .collect();

    append_flora_geometry(
        &mut tris,
        world.biome,
        atmo.night_factor,
        t_sec,
        world.diurnal.rain_intensity,
        &world.kelp_stalks,
        &world.branches,
        &world.lianas,
        &world.underbrush,
        &world.falling_detritus,
        &tree_tuples,
    );

    for org in &world.organisms {
        if world.ledger.keystone_removed && org.species.is_keystone_species() {
            continue;
        }
        append_organism_geometry(&mut tris, org, atmo.night_factor);
    }

    // 3. Shade & Depth-Sort 3D Triangles (with PAR-Gated Caustics, Rim Separation & Depth Cues)
    let sun_dir = Vec3::from_array(atmo.sun_direction).normalize_or_zero();
    let sun_rgb = Vec3::from_array(atmo.sun_rgb);
    let fog_rgb = Vec3::from_array(atmo.fog_rgb);
    let sky_amb = Vec3::from_array(atmo.sky_horizon_rgb) * 0.48;
    let floor_amb = match world.biome {
        BiomeKind::IndoPacificCoralReef => Vec3::new(0.18, 0.28, 0.30),
        BiomeKind::PacificKelpForest => Vec3::new(0.05, 0.14, 0.18),
        BiomeKind::DeepSeaHydrothermalVent => Vec3::new(0.12, 0.05, 0.03),
        BiomeKind::ColdSeepBrinePool => Vec3::new(0.04, 0.06, 0.12),
        BiomeKind::MidwaterPelagicRift => Vec3::new(0.02, 0.05, 0.11),
        BiomeKind::RockyIntertidalTidepool => Vec3::new(0.10, 0.13, 0.16),
        BiomeKind::TemperateFreshwaterLake => Vec3::new(0.09, 0.13, 0.09),
        BiomeKind::PnwOldGrowthSalmonCreek => Vec3::new(0.08, 0.12, 0.07),
        BiomeKind::NeotropicalRainforest => Vec3::new(0.14, 0.10, 0.07),
    };
    let biolum_boost = 0.65 + 0.75 * atmo.night_factor;

    struct ProjectedTri {
        depth: f32,
        pts: [Pos2; 3],
        color: Color32,
    }

    let mut projected: Vec<ProjectedTri> = Vec::with_capacity(tris.len());
    for tri in &tris {
        let Some((s0, w0)) = project_point(&vp, rect, tri.verts[0]) else { continue };
        let Some((s1, w1)) = project_point(&vp, rect, tri.verts[1]) else { continue };
        let Some((s2, w2)) = project_point(&vp, rect, tri.verts[2]) else { continue };

        let center = tri.centroid();
        let view_vec = (eye - center).normalize_or_zero();
        let n = if tri.normal.dot(view_vec) < 0.0 { -tri.normal } else { tri.normal };

        let hemi = n.y * 0.5 + 0.5;
        let ambient = floor_amb.lerp(sky_amb, hemi);
        let n_dot_l = (n.dot(sun_dir) * 0.75 + 0.25).clamp(0.0, 1.0);
        let dapple_or_caustic = surface_caustic_or_dapple(world.biome, &atmo, center, t_sec);

        let half_v = (sun_dir + view_vec).normalize_or_zero();
        let wetness = if is_marine { 0.85 } else { world.diurnal.foliage_wetness };
        let spec_pow = (1.0 - tri.roughness * (1.0 - 0.5 * wetness)).clamp(0.1, 0.95) * 48.0;
        let spec = n.dot(half_v).max(0.0).powf(spec_pow) * (0.08 + 0.28 * wetness) * atmo.sun_intensity;

        let ndotv = n.dot(view_vec).clamp(0.0, 1.0);
        let rim = (1.0 - ndotv).powf(3.0) * 0.16 * sky_amb;

        let vent_glow = if is_vent {
            let d2 = (center - Vec3::new(-0.4, 5.2, -1.1)).length_squared();
            let atten = 1.0 / (1.0 + 0.055 * d2);
            Vec3::new(0.55, 0.19, 0.05) * atten
        } else {
            Vec3::ZERO
        };

        let base = Vec3::new(tri.rgba[0], tri.rgba[1], tri.rgba[2]);
        let mut lit = base * (ambient + sun_rgb * (atmo.sun_intensity * n_dot_l * dapple_or_caustic) + vent_glow)
            + sun_rgb * spec
            + rim
            + base * (tri.emissive * biolum_boost);

        if is_aquatic && atmo.par_umol_m2_s > 0.0 && center.y < KELP_SURFACE_Y {
            let sub_depth = (KELP_SURFACE_Y - center.y).clamp(0.0, 14.0);
            let atten = (-sub_depth * 0.018).exp();
            lit *= Vec3::new(0.84 + 0.16 * atten, 0.94 + 0.06 * atten, 1.0);
        }

        if let TrophicFocus::Single(target_s) = world.focus {
            if tri.stratum != target_s {
                let luma = lit.dot(Vec3::new(0.2126, 0.7152, 0.0722));
                lit = lit.lerp(Vec3::splat(luma * 0.30), 0.72);
            }
        }

        let dist = (eye - center).length();
        let fd = dist * atmo.fog_density;
        let fog_f = (1.0 - (-fd * fd).exp()).clamp(0.0, 0.92);
        let final_rgb = lit.lerp(fog_rgb, fog_f);

        projected.push(ProjectedTri {
            depth: (w0 + w1 + w2) * (1.0 / 3.0),
            pts: [s0, s1, s2],
            color: to_color32([final_rgb.x, final_rgb.y, final_rgb.z, tri.rgba[3]]),
        });
    }

    projected.sort_by(|a, b| b.depth.total_cmp(&a.depth));

    let mut scene_mesh = Mesh::default();
    scene_mesh.vertices.reserve(projected.len() * 3);
    scene_mesh.indices.reserve(projected.len() * 3);
    for pt in &projected {
        let base_idx = scene_mesh.vertices.len() as u32;
        scene_mesh.colored_vertex(pt.pts[0], pt.color);
        scene_mesh.colored_vertex(pt.pts[1], pt.color);
        scene_mesh.colored_vertex(pt.pts[2], pt.color);
        scene_mesh.add_triangle(base_idx, base_idx + 1, base_idx + 2);
    }
    painter.add(Shape::mesh(scene_mesh));

    // 4C. Midwater Pelagic Rift: Sinking Marine-Snow Particles & Sperm Whale Spermaceti Sonar Arcs!
    if is_rift {
        for s_i in 0..28 {
            let sx = -12.0 + (s_i as f32 * 1.93) % 24.0 + (t_sec * 0.35 + s_i as f32).sin() * 0.4;
            let sy = 13.5 - ((t_sec * 0.42 + s_i as f32 * 1.7) % 14.2);
            let sz = -3.0 + (s_i % 5) as f32 * 1.1;
            if let Some((sp, _)) = project_point(&vp, rect, Vec3::new(sx, sy, sz)) {
                painter.circle_filled(sp, 1.8, Color32::from_rgba_unmultiplied(195, 232, 255, 120));
            }
        }
        for org in &world.organisms {
            if let crate::organisms::BehaviorState::SpermWhaleEcholocationDive { dive_timer, .. } = org.state {
                if dive_timer > 0.05 {
                    for ring in 1..=3 {
                        let d = ring as f32 * 1.6 + (t_sec * 4.0 % 1.6);
                        let p = org.pos + Vec3::new(d * 0.7, -d * 0.65, 0.0);
                        if let Some((cp, _)) = project_point(&vp, rect, p) {
                            painter.circle_stroke(cp, 8.0 + ring as f32 * 6.0, Stroke::new(1.4_f32, Color32::from_rgba_unmultiplied(110, 225, 255, 115)));
                        }
                    }
                }
            }
        }
    }

    // 4B. Subterranean Cutaway: Glowing Ectomycorrhizal Root-Hyphae Network & Marine 15N Pulses in PNW Old-Growth!
    if is_pnw {
        let nodes = [
            Vec3::new(2.2, 0.8, 2.85), Vec3::new(0.2, 1.3, 2.85), Vec3::new(-1.8, 1.7, 2.85),
            Vec3::new(-4.2, 2.1, 2.85), Vec3::new(-6.8, 1.5, 2.85), Vec3::new(-9.5, 1.9, 2.85),
            Vec3::new(-12.0, 2.2, 2.85),
        ];
        for seg in 0..(nodes.len() - 1) {
            if let (Some((a, _)), Some((b, _))) = (project_point(&vp, rect, nodes[seg]), project_point(&vp, rect, nodes[seg + 1])) {
                painter.line_segment([a, b], Stroke::new(1.8_f32, Color32::from_rgba_unmultiplied(110, 245, 200, 145)));
                let u = (t_sec * 0.65 + seg as f32 * 0.25) % 1.0;
                let pulse_pos = nodes[seg].lerp(nodes[seg + 1], u);
                if let Some((pp, _)) = project_point(&vp, rect, pulse_pos) {
                    painter.circle_filled(pp, 3.2, Color32::from_rgba_unmultiplied(255, 235, 120, 225));
                }
            }
        }
    }

    // 4A. Buoyant Turbulent Black-Smoker Sulfide Plumes & 360°C Orifice Glow in Hydrothermal Vent!
    if is_vent {
        for &(cx, cy, cz, is_white) in &[(-0.4_f32, 8.6_f32, -1.1_f32, false), (-6.5, 6.8, -1.4, false), (5.2, 7.4, -1.25, false), (-9.6, 4.4, -0.85, true)] {
            if let Some((op, _)) = project_point(&vp, rect, Vec3::new(cx, cy, cz)) {
                let glow_c = if is_white { Color32::from_rgba_unmultiplied(220, 245, 255, 160) } else { Color32::from_rgba_unmultiplied(255, 95, 20, 195) };
                painter.circle_filled(op, 7.5, glow_c);
            }
            for puff in 0..16 {
                let u = ((puff as f32 / 16.0) + (t_sec * 0.28)) % 1.0;
                let py = cy + u * 6.8;
                let drift = u * (1.2 + 1.8 * world.diurnal.rain_intensity) + (t_sec * 2.1 + puff as f32).sin() * 0.35 * u;
                if let Some((sp, _)) = project_point(&vp, rect, Vec3::new(cx + drift, py, cz)) {
                    let rad = 5.5 + u * 24.0;
                    let alpha = ((1.0 - u) * 165.0) as u8;
                    let sm_c = if is_white {
                        Color32::from_rgba_unmultiplied(210, 225, 235, alpha / 2)
                    } else {
                        Color32::from_rgba_unmultiplied(18, 20, 26, alpha)
                    };
                    painter.circle_filled(sp, rad, sm_c);
                }
            }
        }
    }

    // 4. Volumetric Caustic / Sunbeam Shafts (gated on solar PAR > 0)
    if atmo.par_umol_m2_s > 1.0 && atmo.sun_elevation_rad > 0.05 && (is_marine || world.diurnal.rain_intensity < 0.45) {
        let beam_col = if is_marine {
            Color32::from_rgba_unmultiplied(125, 240, 255, 26)
        } else {
            Color32::from_rgba_unmultiplied(255, 236, 175, 24)
        };
        for b in 0..5 {
            let bx = -10.0 + b as f32 * 5.0 + (t_sec * 0.4 + b as f32).sin() * 0.5;
            let top_l = Vec3::new(bx - 0.9, 15.1, -2.5);
            let top_r = Vec3::new(bx + 0.9, 15.1, -2.5);
            let bot_r = Vec3::new(bx + 2.8, 0.2, 0.5);
            let bot_l = Vec3::new(bx + 0.4, 0.2, 0.5);
            if let (Some((p0, _)), Some((p1, _)), Some((p2, _)), Some((p3, _))) = (
                project_point(&vp, rect, top_l),
                project_point(&vp, rect, top_r),
                project_point(&vp, rect, bot_r),
                project_point(&vp, rect, bot_l),
            ) {
                let mut bm = Mesh::default();
                bm.colored_vertex(p0, beam_col);
                bm.colored_vertex(p1, beam_col);
                bm.colored_vertex(p2, Color32::TRANSPARENT);
                bm.colored_vertex(p3, Color32::TRANSPARENT);
                bm.add_triangle(0, 1, 2);
                bm.add_triangle(0, 2, 3);
                painter.add(Shape::mesh(bm));
            }
        }
    }

    // 5. Marine Snow & Nocturnal Dinoflagellate Bioluminescence OR Rainforest Rain & Fireflies
    if is_marine {
        let count = if atmo.night_factor > 0.35 { 55 } else { 40 };
        for p in 0..count {
            let px = ((p as f32 * 5.7 + (t_sec * 0.45).sin() * 1.2) % 28.0) - 14.0;
            let py = 14.8 - ((p as f32 * 2.9 + t_sec * 0.42) % 14.4);
            let pz = ((p as f32 * 4.3) % 12.0) - 9.0;
            if let Some((sp, _)) = project_point(&vp, rect, Vec3::new(px, py, pz)) {
                let col = if atmo.night_factor > 0.35 {
                    let glow = (((t_sec * 3.0 + p as f32).sin() * 0.5 + 0.5) * 210.0) as u8;
                    Color32::from_rgba_unmultiplied(65, 235, 255, glow) // Cyan dinoflagellates!
                } else {
                    Color32::from_rgba_unmultiplied(210, 240, 240, 95) // Drifting marine snow
                };
                painter.circle_filled(sp, 2.1, col);
            }
        }
    } else if world.diurnal.rain_intensity > 0.05 {
        let count = (140.0 * world.diurnal.rain_intensity) as usize;
        let rain_col = Color32::from_rgba_unmultiplied(195, 225, 245, 110);
        for r in 0..count {
            let rx = ((r as f32 * 7.31 + t_sec * 2.2) % 30.0) - 15.0;
            let ry = 15.5 - ((r as f32 * 3.17 + t_sec * 18.5) % 16.5);
            let rz = ((r as f32 * 5.13) % 14.0) - 11.0;
            if let (Some((a, _)), Some((b, _))) = (
                project_point(&vp, rect, Vec3::new(rx, ry, rz)),
                project_point(&vp, rect, Vec3::new(rx - 0.15, ry - 0.75, rz)),
            ) {
                painter.line_segment([a, b], Stroke::new(1.1_f32, rain_col));
            }
        }
    } else if atmo.night_factor > 0.35 {
        for f in 0..28 {
            let fx = ((f as f32 * 4.7) % 26.0) - 13.0 + (t_sec * 0.7 + f as f32).sin() * 0.9;
            let fy = 1.2 + ((f as f32 * 2.3) % 7.5) + (t_sec * 1.1 + f as f32).cos() * 0.4;
            let fz = ((f as f32 * 3.9) % 10.0) - 7.5;
            let pulse = ((t_sec * 2.8 + f as f32 * 1.9).sin() * 0.5 + 0.5).powi(2);
            if let Some((sp, _)) = project_point(&vp, rect, Vec3::new(fx, fy, fz)) {
                let a = (pulse * atmo.night_factor * 210.0) as u8;
                painter.circle_filled(sp, 2.6, Color32::from_rgba_unmultiplied(180, 255, 110, a));
            }
        }
    }

    // 5B. Calm, Non-Strobing 3D Food-Web Energy-Flux Arcs (L0 -> L1 -> L2 -> L3 -> L4)!
    if world.ledger.show_flux_arcs {
        let active_orgs: Vec<&crate::organisms::Organism> = world
            .organisms
            .iter()
            .filter(|o| !(world.ledger.keystone_removed && o.species.is_keystone_species()))
            .collect();
        let prod_anchors = producer_flux_anchors(world, t_sec);

        let anchors_for_stratum = |stratum: TrophicStratum| -> Vec<(Option<u32>, Vec3)> {
            if stratum == TrophicStratum::Producers {
                prod_anchors.iter().map(|&p| (None, p)).collect()
            } else {
                active_orgs
                    .iter()
                    .filter(|o| o.stratum() == stratum)
                    .take(4)
                    .map(|o| (Some(o.id), o.pos + Vec3::new(0.0, 0.25, 0.0)))
                    .collect()
            }
        };

        for s_idx in 0..4_usize {
            let src_stratum = TrophicStratum::ALL[s_idx];
            let dst_stratum = TrophicStratum::ALL[s_idx + 1];
            let srcs = anchors_for_stratum(src_stratum);
            let dsts = anchors_for_stratum(dst_stratum);
            if srcs.is_empty() || dsts.is_empty() {
                continue;
            }
            for (k, &(src_id, p0)) in srcs.iter().enumerate() {
                if let Some(&(dst_id, p2)) = dsts.get(k % dsts.len()) {
                    let is_sel = selected_id.is_some()
                        && (src_id == selected_id || dst_id == selected_id);
                    let mid = (p0 + p2) * 0.5 + Vec3::new(0.0, 1.35 + 0.25 * s_idx as f32, 0.35);
                    let [r, g, b] = dst_stratum.badge_rgb();
                    let arc_alpha = if is_sel { 175 } else { 72 };
                    let stroke = Stroke::new(if is_sel { 2.1_f32 } else { 1.15_f32 }, Color32::from_rgba_unmultiplied(r, g, b, arc_alpha));
                    let mut prev_pt = None;
                    for seg in 0..=12 {
                        let u = seg as f32 / 12.0;
                        let om = 1.0 - u;
                        let q = p0 * (om * om) + mid * (2.0 * om * u) + p2 * (u * u);
                        if let Some((sp, _)) = project_point(&vp, rect, q) {
                            if let Some(pp) = prev_pt {
                                painter.line_segment([pp, sp], stroke);
                            }
                            prev_pt = Some(sp);
                        }
                    }
                    // Gentle, tonic slow-gliding kcal bead along the Bezier curve (zero strobing)
                    let u_bead = (t_sec * 0.16 + k as f32 * 0.31 + s_idx as f32 * 0.23) % 1.0;
                    let om = 1.0 - u_bead;
                    let qb = p0 * (om * om) + mid * (2.0 * om * u_bead) + p2 * (u_bead * u_bead);
                    if let Some((bp, _)) = project_point(&vp, rect, qb) {
                        painter.circle_filled(bp, if is_sel { 3.6 } else { 2.4 }, Color32::from_rgba_unmultiplied(r, g, b, if is_sel { 225 } else { 135 }));
                    }
                }
            }
        }
    }

    let mut hitboxes = Vec::with_capacity(world.organisms.len());
    for org in &world.organisms {
        if world.ledger.keystone_removed && org.species.is_keystone_species() {
            continue;
        }
        if let Some((sp, _)) = project_point(&vp, rect, org.pos + Vec3::new(0.0, 0.25, 0.0)) {
            hitboxes.push((org.id, sp));
            if selected_id == Some(org.id) {
                painter.circle_stroke(sp, 16.0, Stroke::new(2.0_f32, Color32::from_rgb(250, 224, 110)));
                painter.text(
                    sp + EguiVec2::new(0.0, -22.0),
                    egui::Align2::CENTER_BOTTOM,
                    org.species.common_name(),
                    egui::FontId::proportional(12.5),
                    Color32::from_rgb(255, 244, 195),
                );
            }
        }
    }

    hitboxes
}
