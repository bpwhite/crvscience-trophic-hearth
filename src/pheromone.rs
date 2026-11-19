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

use glam::Vec2;
use serde::{Deserialize, Serialize};

pub const GRID_W: usize = 64;
pub const GRID_D: usize = 32;
pub const WORLD_X_MIN: f32 = -16.0;
pub const WORLD_X_MAX: f32 = 16.0;
pub const WORLD_Z_MIN: f32 = -14.0;
pub const WORLD_Z_MAX: f32 = 4.0;

/// 2D reaction-diffusion field on the rainforest floor (`X in [-16, 16]`, `Z in [-14, 4]`)
/// governing volatile Leafcutter Ant trail pheromones (`tau`) and soil microbial
/// nutrient mineralization plumes (`nutrients`) beneath decaying detritus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForestFloorField {
    /// Volatile ant recruitment pheromone concentration `[0.0, 10.0]`.
    pub ant_pheromone: Vec<f32>,
    /// Home-bound nest pheromone concentration `[0.0, 10.0]`.
    pub home_pheromone: Vec<f32>,
    /// Labile organic detritus density on soil surface.
    pub detritus_density: Vec<f32>,
    /// Microbial exo-enzyme & mineralized NH4+/PO4^3- plume concentration.
    pub microbial_nutrients: Vec<f32>,
    /// Scratch buffer for conservative Laplacian diffusion.
    #[serde(skip, default = "zero_grid")]
    scratch: Vec<f32>,
}

fn zero_grid() -> Vec<f32> {
    vec![0.0; GRID_W * GRID_D]
}

impl Default for ForestFloorField {
    fn default() -> Self {
        let n = GRID_W * GRID_D;
        let mut field = Self {
            ant_pheromone: vec![0.0; n],
            home_pheromone: vec![0.0; n],
            detritus_density: vec![0.25; n],
            microbial_nutrients: vec![0.35; n],
            scratch: vec![0.0; n],
        };
        // Seed initial subterranean nest mound aura near (-11.2, 0.8) and fallen log detritus
        field.deposit_home_pheromone(Vec2::new(-11.2, 0.8), 6.0);
        field.deposit_detritus(Vec2::new(-2.5, -1.2), 2.4);
        field.deposit_detritus(Vec2::new(6.8, 0.4), 2.8);
        field
    }
}

impl ForestFloorField {
    pub fn world_to_cell(pos_xz: Vec2) -> (usize, usize) {
        let u = ((pos_xz.x - WORLD_X_MIN) / (WORLD_X_MAX - WORLD_X_MIN)).clamp(0.0, 0.9999);
        let v = ((pos_xz.y - WORLD_Z_MIN) / (WORLD_Z_MAX - WORLD_Z_MIN)).clamp(0.0, 0.9999);
        ((u * GRID_W as f32) as usize, (v * GRID_D as f32) as usize)
    }

    pub fn cell_to_world(ix: usize, iz: usize) -> Vec2 {
        let x = WORLD_X_MIN + ((ix as f32 + 0.5) / GRID_W as f32) * (WORLD_X_MAX - WORLD_X_MIN);
        let z = WORLD_Z_MIN + ((iz as f32 + 0.5) / GRID_D as f32) * (WORLD_Z_MAX - WORLD_Z_MIN);
        Vec2::new(x, z)
    }

    #[inline]
    pub fn idx(ix: usize, iz: usize) -> usize {
        iz * GRID_W + ix
    }

    pub fn deposit_ant_pheromone(&mut self, pos_xz: Vec2, amount: f32) {
        let (ix, iz) = Self::world_to_cell(pos_xz);
        let idx = Self::idx(ix, iz);
        self.ant_pheromone[idx] = (self.ant_pheromone[idx] + amount).min(12.0);
    }

    pub fn deposit_home_pheromone(&mut self, pos_xz: Vec2, amount: f32) {
        let (ix, iz) = Self::world_to_cell(pos_xz);
        for dz in -1_isize..=1 {
            for dx in -1_isize..=1 {
                let nx = (ix as isize + dx).clamp(0, GRID_W as isize - 1) as usize;
                let nz = (iz as isize + dz).clamp(0, GRID_D as isize - 1) as usize;
                let w = if dx == 0 && dz == 0 { 1.0 } else { 0.45 };
                let idx = Self::idx(nx, nz);
                self.home_pheromone[idx] = (self.home_pheromone[idx] + amount * w).min(12.0);
            }
        }
    }

    pub fn deposit_detritus(&mut self, pos_xz: Vec2, amount: f32) {
        let (ix, iz) = Self::world_to_cell(pos_xz);
        for dz in -1_isize..=1 {
            for dx in -1_isize..=1 {
                let nx = (ix as isize + dx).clamp(0, GRID_W as isize - 1) as usize;
                let nz = (iz as isize + dz).clamp(0, GRID_D as isize - 1) as usize;
                let w = if dx == 0 && dz == 0 { 1.0 } else { 0.5 };
                let idx = Self::idx(nx, nz);
                self.detritus_density[idx] = (self.detritus_density[idx] + amount * w).min(8.0);
            }
        }
    }

    pub fn sample_ant_pheromone(&self, pos_xz: Vec2) -> f32 {
        let (ix, iz) = Self::world_to_cell(pos_xz);
        self.ant_pheromone[Self::idx(ix, iz)]
    }

    pub fn sample_nutrients(&self, pos_xz: Vec2) -> f32 {
        let (ix, iz) = Self::world_to_cell(pos_xz);
        self.microbial_nutrients[Self::idx(ix, iz)]
    }

    pub fn consume_nutrients(&mut self, pos_xz: Vec2, max_uptake: f32) -> f32 {
        let (ix, iz) = Self::world_to_cell(pos_xz);
        let idx = Self::idx(ix, iz);
        let taken = self.microbial_nutrients[idx].min(max_uptake);
        self.microbial_nutrients[idx] -= taken;
        taken
    }

    /// Computes local gradient of the volatile ant trail pheromone so foraging ants steer along active trails.
    pub fn pheromone_gradient(&self, pos_xz: Vec2) -> Vec2 {
        let (ix, iz) = Self::world_to_cell(pos_xz);
        let xl = ix.saturating_sub(1);
        let xr = (ix + 1).min(GRID_W - 1);
        let zd = iz.saturating_sub(1);
        let zu = (iz + 1).min(GRID_D - 1);

        let gx = self.ant_pheromone[Self::idx(xr, iz)] - self.ant_pheromone[Self::idx(xl, iz)];
        let gz = self.ant_pheromone[Self::idx(ix, zu)] - self.ant_pheromone[Self::idx(ix, zd)];
        Vec2::new(gx, gz)
    }

    /// Advances volatile trail diffusion/evaporation and microbial mineralization.
    /// Returns total newly mineralized nutrients across the grid in this step.
    pub fn step(&mut self, dt_sec: f32, rain_intensity: f32, soil_moisture: f32) -> f32 {
        if self.scratch.len() != GRID_W * GRID_D {
            self.scratch.resize(GRID_W * GRID_D, 0.0);
        }

        // 1. Ant trail pheromone diffusion & exponential volatilization (rain accelerates washout!)
        let diff_tau = (0.55 * dt_sec).min(0.20);
        let evap_rate = 0.24 + 0.85 * rain_intensity; // Ensures trails cleanly dissipate once recruitment stops
        let retention = (-evap_rate * dt_sec).exp();

        for iz in 0..GRID_D {
            for ix in 0..GRID_W {
                let c = self.ant_pheromone[Self::idx(ix, iz)];
                let l = self.ant_pheromone[Self::idx(ix.saturating_sub(1), iz)];
                let r = self.ant_pheromone[Self::idx((ix + 1).min(GRID_W - 1), iz)];
                let d = self.ant_pheromone[Self::idx(ix, iz.saturating_sub(1))];
                let u = self.ant_pheromone[Self::idx(ix, (iz + 1).min(GRID_D - 1))];
                let lap = (l + r + d + u) * 0.25 - c;
                let next = (c + diff_tau * lap) * retention;
                self.scratch[Self::idx(ix, iz)] = if next < 1e-4 { 0.0 } else { next };
            }
        }
        self.ant_pheromone.copy_from_slice(&self.scratch);

        // Gentle decay of nest aura (replenished continuously by colony entrance)
        let home_ret = (-0.06 * dt_sec).exp();
        for v in &mut self.home_pheromone {
            *v *= home_ret;
        }

        // 2. Saprotrophic bacterial & fungal mineralization of detritus -> soluble nutrients
        let moisture_boost = 0.65 + 0.75 * soil_moisture.clamp(0.0, 1.0);
        let mineral_rate = 0.032 * moisture_boost * dt_sec;
        let mut total_mineralized = 0.0;

        for i in 0..(GRID_W * GRID_D) {
            let converted = (self.detritus_density[i] * mineral_rate).min(self.detritus_density[i]);
            self.detritus_density[i] -= converted;
            self.microbial_nutrients[i] = (self.microbial_nutrients[i] + converted * 1.45).min(6.0);
            total_mineralized += converted;
        }

        total_mineralized
    }

    pub fn total_active_pheromone(&self) -> f32 {
        self.ant_pheromone.iter().sum()
    }

    /// Returns sampled trail cells `(world_xz, concentration)` above visibility threshold for 3D rendering.
    pub fn visible_pheromone_cells(&self, threshold: f32) -> Vec<(Vec2, f32)> {
        let mut out = Vec::new();
        for iz in 0..GRID_D {
            for ix in 0..GRID_W {
                let v = self.ant_pheromone[Self::idx(ix, iz)];
                if v >= threshold {
                    out.push((Self::cell_to_world(ix, iz), v));
                }
            }
        }
        out
    }
}
