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

use crate::organisms::{BehaviorState, Organism, Species};
use crate::physics::{
    BranchSegment, FallingDetritus, KelpStalk, LianaVine, UnderbrushKind, UnderbrushPlant,
};
use crate::trophic::{BiomeKind, TrophicStratum};
use glam::{Quat, Vec3};
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct Tri3D {
    pub verts: [Vec3; 3],
    pub normal: Vec3,
    pub rgba: [f32; 4],
    pub roughness: f32,
    pub emissive: f32,
    pub stratum: TrophicStratum,
}

impl Tri3D {
    pub fn new(
        v0: Vec3,
        v1: Vec3,
        v2: Vec3,
        rgba: [f32; 4],
        roughness: f32,
        emissive: f32,
        stratum: TrophicStratum,
    ) -> Self {
        let n = (v1 - v0).cross(v2 - v0).normalize_or_zero();
        let normal = if n.length_squared() < 0.2 { Vec3::Y } else { n };
        Self {
            verts: [v0, v1, v2],
            normal,
            rgba,
            roughness,
            emissive,
            stratum,
        }
    }

    pub fn centroid(&self) -> Vec3 {
        (self.verts[0] + self.verts[1] + self.verts[2]) * (1.0 / 3.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpineStation {
    pub pos: Vec3,
    pub half_width: f32,
    pub dorsal_height: f32,
    pub ventral_depth: f32,
    pub dorsal_rgba: [f32; 4],
    pub ventral_rgba: [f32; 4],
}

pub fn push_lofted_spine(
    out: &mut Vec<Tri3D>,
    stations: &[SpineStation],
    ring_verts: usize,
    roughness: f32,
    emissive: f32,
    stratum: TrophicStratum,
) {
    if stations.len() < 2 {
        return;
    }
    let n_ring = ring_verts.max(6);
    let mut rings: Vec<Vec<(Vec3, [f32; 4])>> = Vec::with_capacity(stations.len());

    for (idx, st) in stations.iter().enumerate() {
        let tangent = if idx == 0 {
            (stations[1].pos - stations[0].pos).normalize_or_zero()
        } else if idx + 1 == stations.len() {
            (stations[idx].pos - stations[idx - 1].pos).normalize_or_zero()
        } else {
            (stations[idx + 1].pos - stations[idx - 1].pos).normalize_or_zero()
        };
        let t = if tangent.length_squared() < 0.1 { Vec3::X } else { tangent };
        let up_ref = if t.y.abs() < 0.92 { Vec3::Y } else { Vec3::Z };
        let lateral = t.cross(up_ref).normalize_or_zero();
        let dorsal = lateral.cross(t).normalize_or_zero();

        let mut ring = Vec::with_capacity(n_ring);
        for k in 0..n_ring {
            let ang = (k as f32 / n_ring as f32) * 2.0 * PI;
            let c = ang.cos();
            let s_val = ang.sin();
            let y_rad = if s_val >= 0.0 { st.dorsal_height } else { st.ventral_depth };
            let pt = st.pos + lateral * (c * st.half_width) + dorsal * (s_val * y_rad);
            let blend = (s_val * 0.5 + 0.5).clamp(0.0, 1.0);
            let col = [
                st.ventral_rgba[0] + (st.dorsal_rgba[0] - st.ventral_rgba[0]) * blend,
                st.ventral_rgba[1] + (st.dorsal_rgba[1] - st.ventral_rgba[1]) * blend,
                st.ventral_rgba[2] + (st.dorsal_rgba[2] - st.ventral_rgba[2]) * blend,
                st.dorsal_rgba[3],
            ];
            ring.push((pt, col));
        }
        rings.push(ring);
    }

    for i in 0..(rings.len() - 1) {
        for k in 0..n_ring {
            let kn = (k + 1) % n_ring;
            let (p00, c00) = rings[i][k];
            let (p01, _) = rings[i][kn];
            let (p11, c11) = rings[i + 1][kn];
            let (p10, _) = rings[i + 1][k];
            out.push(Tri3D::new(p00, p01, p11, c00, roughness, emissive, stratum));
            out.push(Tri3D::new(p00, p11, p10, c11, roughness, emissive, stratum));
        }
    }
}

pub fn push_oriented_box(
    out: &mut Vec<Tri3D>,
    center: Vec3,
    half: Vec3,
    rot: Quat,
    rgba: [f32; 4],
    roughness: f32,
    emissive: f32,
    stratum: TrophicStratum,
) {
    let corners_local = [
        Vec3::new(-half.x, -half.y, -half.z),
        Vec3::new(half.x, -half.y, -half.z),
        Vec3::new(half.x, half.y, -half.z),
        Vec3::new(-half.x, half.y, -half.z),
        Vec3::new(-half.x, -half.y, half.z),
        Vec3::new(half.x, -half.y, half.z),
        Vec3::new(half.x, half.y, half.z),
        Vec3::new(-half.x, half.y, half.z),
    ];
    let c: [Vec3; 8] = corners_local.map(|p| center + rot * p);
    let faces: [[usize; 4]; 6] = [
        [4, 5, 6, 7],
        [1, 0, 3, 2],
        [5, 1, 2, 6],
        [0, 4, 7, 3],
        [3, 7, 6, 2],
        [0, 1, 5, 4],
    ];
    for f in faces {
        out.push(Tri3D::new(c[f[0]], c[f[1]], c[f[2]], rgba, roughness, emissive, stratum));
        out.push(Tri3D::new(c[f[0]], c[f[2]], c[f[3]], rgba, roughness, emissive, stratum));
    }
}

pub fn push_tapered_tube(
    out: &mut Vec<Tri3D>,
    p0: Vec3,
    p1: Vec3,
    r0: f32,
    r1: f32,
    sides: usize,
    rgba: [f32; 4],
    roughness: f32,
    emissive: f32,
    stratum: TrophicStratum,
) {
    let axis = p1 - p0;
    let len = axis.length();
    if len < 1e-4 {
        return;
    }
    let dir = axis / len;
    let up_ref = if dir.y.abs() < 0.92 { Vec3::Y } else { Vec3::X };
    let tangent = dir.cross(up_ref).normalize_or_zero();
    let bitangent = dir.cross(tangent).normalize_or_zero();
    let n_sides = sides.max(4);

    for i in 0..n_sides {
        let a0 = (i as f32 / n_sides as f32) * 2.0 * PI;
        let a1 = ((i + 1) as f32 / n_sides as f32) * 2.0 * PI;
        let d0 = tangent * a0.cos() + bitangent * a0.sin();
        let d1 = tangent * a1.cos() + bitangent * a1.sin();
        let b0 = p0 + d0 * r0;
        let b1 = p0 + d1 * r0;
        let t1 = p1 + d1 * r1;
        let t0 = p1 + d0 * r1;
        out.push(Tri3D::new(b0, b1, t1, rgba, roughness, emissive, stratum));
        out.push(Tri3D::new(b0, t1, t0, rgba, roughness, emissive, stratum));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn push_three_segment_limb(
    out: &mut Vec<Tri3D>,
    hip_or_shoulder: Vec3,
    elbow_or_stifle: Vec3,
    wrist_or_hock: Vec3,
    paw_or_hoof: Vec3,
    radii: [f32; 4],
    prox_rgba: [f32; 4],
    dist_rgba: [f32; 4],
    roughness: f32,
    stratum: TrophicStratum,
) {
    push_tapered_tube(out, hip_or_shoulder, elbow_or_stifle, radii[0], radii[1], 6, prox_rgba, roughness, 0.0, stratum);
    push_tapered_tube(out, elbow_or_stifle, wrist_or_hock, radii[1], radii[2], 6, dist_rgba, roughness, 0.0, stratum);
    push_tapered_tube(out, wrist_or_hock, paw_or_hoof, radii[2], radii[3], 5, dist_rgba, roughness, 0.0, stratum);
}

pub fn push_ellipsoid(
    out: &mut Vec<Tri3D>,
    center: Vec3,
    radii: Vec3,
    rot: Quat,
    rgba: [f32; 4],
    roughness: f32,
    emissive: f32,
    stratum: TrophicStratum,
) {
    let rings = 6_usize;
    let segs = 10_usize;
    for r in 0..rings {
        let v0 = (r as f32 / rings as f32) * PI;
        let v1 = ((r + 1) as f32 / rings as f32) * PI;
        for s in 0..segs {
            let u0 = (s as f32 / segs as f32) * 2.0 * PI;
            let u1 = ((s + 1) as f32 / segs as f32) * 2.0 * PI;
            let sph = |v: f32, u: f32| -> Vec3 {
                center + rot * Vec3::new(v.sin() * u.cos() * radii.x, v.cos() * radii.y, v.sin() * u.sin() * radii.z)
            };
            let p00 = sph(v0, u0);
            let p10 = sph(v0, u1);
            let p11 = sph(v1, u1);
            let p01 = sph(v1, u0);
            if r > 0 {
                out.push(Tri3D::new(p00, p10, p11, rgba, roughness, emissive, stratum));
            }
            if r + 1 < rings {
                out.push(Tri3D::new(p00, p11, p01, rgba, roughness, emissive, stratum));
            }
        }
    }
}

/// Builds terrestrial rainforest flora OR marine Giant Kelp (*Macrocystis* & *Nereocystis*), rocky basalt ledges, and coralline algae!
#[allow(clippy::too_many_arguments)]
pub fn append_flora_geometry(
    out: &mut Vec<Tri3D>,
    biome: BiomeKind,
    night_factor: f32,
    sim_time_sec: f32,
    surge_amp: f32,
    kelp_stalks: &[KelpStalk],
    branches: &[BranchSegment],
    lianas: &[LianaVine],
    underbrush: &[UnderbrushPlant],
    detritus: &[FallingDetritus],
    tree_bases: &[(Vec3, f32, f32, f32, u8)],
) {
    let stratum = TrophicStratum::Producers;

    // 1. Marine Giant Kelp & Bull Kelp Columns (with haptera holdfasts, pneumatocysts & corrugated blades!)
    for stalk in kelp_stalks {
        let golden_kelp = match stalk.depth_tier {
            1 => [0.58, 0.52, 0.14, 1.0],
            2 => [0.44, 0.42, 0.13, 1.0],
            _ => [0.28, 0.32, 0.14, 1.0],
        };
        let holdfast_col = [0.34, 0.26, 0.14, 1.0];

        // Conical haptera root-like holdfast gripping the rocky reef
        push_ellipsoid(
            out,
            stalk.holdfast_pos + Vec3::new(0.0, 0.16, 0.0),
            Vec3::new(0.38, 0.24, 0.38),
            Quat::IDENTITY,
            holdfast_col,
            0.82,
            0.0,
            stratum,
        );

        let segs = 12_usize;
        for s in 0..segs {
            let f0 = s as f32 / segs as f32;
            let f1 = (s + 1) as f32 / segs as f32;
            let p0 = stalk.point_along(f0, sim_time_sec, surge_amp);
            let p1 = stalk.point_along(f1, sim_time_sec, surge_amp);
            let r_stipe = if stalk.is_bull_kelp { 0.045 + 0.035 * f0 } else { 0.042 };
            push_tapered_tube(out, p0, p1, r_stipe, r_stipe, 5, golden_kelp, 0.52, 0.0, stratum);

            if !stalk.is_bull_kelp && s >= 1 && stalk.depth_tier <= 2 {
                // Macrocystis pyrifera: alternating pneumatocyst gas bladder + streaming corrugated blade!
                let side = if s % 2 == 0 { 1.0 } else { -1.0 };
                let pneum = p1 + Vec3::new(side * 0.08, 0.0, 0.0);
                push_ellipsoid(out, pneum, Vec3::splat(0.065), Quat::IDENTITY, [0.72, 0.62, 0.16, 1.0], 0.38, 0.0, stratum);
                let wave = (sim_time_sec * 2.2 + s as f32).sin() * 0.18;
                let b_mid = pneum + Vec3::new(side * 0.45 + 0.25, 0.08 + wave * 0.3, 0.12 * side);
                let b_tip = pneum + Vec3::new(side * 0.92 + 0.55, -0.10 + wave, -0.08 * side);
                out.push(Tri3D::new(
                    pneum + Vec3::new(0.0, 0.09, 0.0),
                    b_mid,
                    b_tip,
                    golden_kelp,
                    0.46,
                    0.02,
                    stratum,
                ));
                out.push(Tri3D::new(
                    pneum - Vec3::new(0.0, 0.09, 0.0),
                    b_tip,
                    b_mid,
                    golden_kelp,
                    0.46,
                    0.02,
                    stratum,
                ));
            }
        }

        if stalk.is_bull_kelp {
            // Nereocystis luetkeana: single large terminal bulbous pneumatocyst + crown of streaming blades!
            let crown_pt = stalk.point_along(1.0, sim_time_sec, surge_amp);
            push_ellipsoid(out, crown_pt, Vec3::splat(0.18), Quat::IDENTITY, [0.68, 0.56, 0.15, 1.0], 0.35, 0.0, stratum);
            for b in 0..6 {
                let ang = b as f32 * 0.5 - 1.2;
                let tip = crown_pt + Vec3::new(1.45, ang * 0.35 + (sim_time_sec * 2.0 + b as f32).sin() * 0.2, (b as f32 - 2.5) * 0.18);
                let mid = crown_pt + Vec3::new(0.65, 0.14, (b as f32 - 2.5) * 0.12);
                out.push(Tri3D::new(crown_pt, mid, tip, golden_kelp, 0.45, 0.02, stratum));
            }
        }
    }

    // 2. Rainforest Trees & Boughs OR Kelp Forest Rocky Basalt Reef Ledges
    for &(base, height, r_trunk, r_crown, tier) in tree_bases {
        let bark = match tier {
            1 => [0.36, 0.23, 0.14, 1.0],
            2 => [0.28, 0.19, 0.13, 1.0],
            _ => [0.18, 0.16, 0.14, 1.0],
        };
        let top = base + Vec3::new(0.0, height, 0.0);
        push_tapered_tube(out, base, top, r_trunk * 1.35, r_trunk * 0.52, 7, bark, 0.88, 0.0, stratum);
        for k in 0..4 {
            let ang = k as f32 * PI * 0.5 + 0.4;
            let root_tip = base + Vec3::new(ang.cos() * r_trunk * 2.3, -0.15, ang.sin() * r_trunk * 2.1);
            push_tapered_tube(out, base + Vec3::new(0.0, 2.2, 0.0), root_tip, r_trunk * 0.45, 0.14, 4, bark, 0.90, 0.0, stratum);
        }
        let leaf_col = match tier {
            1 => [0.16, 0.46, 0.19, 1.0],
            2 => [0.13, 0.39, 0.18, 1.0],
            _ => [0.10, 0.31, 0.17, 1.0],
        };
        for lobe in [
            Vec3::new(0.0, 0.2, 0.0),
            Vec3::new(-r_crown * 0.55, -0.6, 0.5),
            Vec3::new(r_crown * 0.55, -0.5, -0.4),
        ] {
            push_ellipsoid(out, top + lobe, Vec3::new(r_crown * 0.65, 2.0, r_crown * 0.55), Quat::IDENTITY, leaf_col, 0.72, 0.0, stratum);
        }
    }

    let bough_col = match biome {
        BiomeKind::RockyIntertidalTidepool => [0.22, 0.24, 0.27, 1.0],
        BiomeKind::ColdSeepBrinePool => [0.24, 0.28, 0.34, 1.0],
        BiomeKind::MidwaterPelagicRift => [0.16, 0.20, 0.30, 1.0], // Overhanging faulted basalt pelagic rift ledges!
        BiomeKind::PnwOldGrowthSalmonCreek => [0.34, 0.38, 0.20, 1.0],
        BiomeKind::DeepSeaHydrothermalVent => [0.28, 0.23, 0.18, 1.0],
        BiomeKind::IndoPacificCoralReef => [0.64, 0.57, 0.52, 1.0],
        BiomeKind::TemperateFreshwaterLake => [0.38, 0.27, 0.18, 1.0],
        BiomeKind::PacificKelpForest => [0.24, 0.25, 0.28, 1.0],
        BiomeKind::NeotropicalRainforest => [0.40, 0.26, 0.15, 1.0],
    };
    for br in branches {
        push_tapered_tube(out, br.start, br.end, br.radius_start, br.radius_end, 6, bough_col, 0.86, 0.0, stratum);
    }

    for vine in lianas {
        for s in 0..6 {
            let f0 = s as f32 / 6.0;
            let f1 = (s + 1) as f32 / 6.0;
            push_tapered_tube(out, vine.position_along(f0), vine.position_along(f1), 0.055, 0.042, 4, [0.22, 0.42, 0.16, 1.0], 0.78, 0.0, stratum);
        }
    }

    for plant in underbrush {
        let bend_rot = Quat::from_rotation_z(-plant.bend_xz[0]) * Quat::from_rotation_x(plant.bend_xz[1]);
        let s = plant.scale;
        match plant.kind {
            UnderbrushKind::SwordFernClump | UnderbrushKind::MonsteraBroadleaf => {
                let col = if plant.kind == UnderbrushKind::SwordFernClump { [0.14, 0.52, 0.22, 1.0] } else { [0.11, 0.44, 0.20, 1.0] };
                for k in 0..7 {
                    let az = (k as f32 / 7.0) * 2.0 * PI + plant.id as f32 * 0.3;
                    let rot = bend_rot * Quat::from_rotation_y(az) * Quat::from_rotation_z(0.58);
                    let tip = plant.root_pos + rot * Vec3::new(0.0, 1.28 * s, 0.0);
                    let mid_l = plant.root_pos + rot * Vec3::new(-0.18 * s, 0.68 * s, 0.08 * s);
                    let mid_r = plant.root_pos + rot * Vec3::new(0.18 * s, 0.68 * s, -0.08 * s);
                    out.push(Tri3D::new(plant.root_pos, mid_l, tip, col, 0.62, 0.0, stratum));
                    out.push(Tri3D::new(plant.root_pos, tip, mid_r, col, 0.62, 0.0, stratum));
                }
            }
            UnderbrushKind::BlueberryMelastomeShrub => {
                let center = plant.root_pos + bend_rot * Vec3::new(0.0, 0.58 * s, 0.0);
                push_ellipsoid(out, center, Vec3::new(0.72 * s, 0.56 * s, 0.68 * s), bend_rot, [0.13, 0.47, 0.21, 1.0], 0.68, 0.0, stratum);
                for b in 0..plant.ripe_berries {
                    let ang = b as f32 * 1.15;
                    let offset = Vec3::new(ang.cos() * 0.56 * s, 0.12 * ((b % 3) as f32 - 1.0), ang.sin() * 0.52 * s);
                    push_oriented_box(out, center + bend_rot * offset, Vec3::splat(0.060), Quat::IDENTITY, [0.18, 0.46, 0.92, 1.0], 0.32, 0.05, stratum);
                }
            }
            UnderbrushKind::HeliconiaFlowerCluster => {
                let stem_top = plant.root_pos + bend_rot * Vec3::new(0.0, 1.05 * s, 0.0);
                push_tapered_tube(out, plant.root_pos, stem_top, 0.05, 0.03, 4, [0.18, 0.54, 0.22, 1.0], 0.65, 0.0, stratum);
                for p in 0..5 {
                    let pa = (p as f32 / 5.0) * 2.0 * PI;
                    let petal = stem_top + bend_rot * Vec3::new(pa.cos() * 0.22, 0.06, pa.sin() * 0.22);
                    push_oriented_box(out, petal, Vec3::new(0.09, 0.03, 0.09), bend_rot, [0.88, 0.16, 0.48, 1.0], 0.45, 0.04, stratum);
                }
            }
            UnderbrushKind::CorallineAlgaeTurf => {
                push_ellipsoid(
                    out,
                    plant.root_pos + Vec3::new(0.0, 0.14 * s, 0.0),
                    Vec3::new(0.42 * s, 0.18 * s, 0.38 * s),
                    bend_rot,
                    [0.78, 0.32, 0.52, 1.0], // Calcified pink-magenta Calliarthron turf
                    0.65,
                    0.0,
                    stratum,
                );
            }
            UnderbrushKind::CrimsonRhodymeniaBlade => {
                for k in 0..5 {
                    let az = (k as f32 / 5.0) * 2.0 * PI;
                    let rot = bend_rot * Quat::from_rotation_y(az) * Quat::from_rotation_z(0.42);
                    let tip = plant.root_pos + rot * Vec3::new(0.0, 0.88 * s, 0.0);
                    let mid_l = plant.root_pos + rot * Vec3::new(-0.16 * s, 0.46 * s, 0.05 * s);
                    let mid_r = plant.root_pos + rot * Vec3::new(0.16 * s, 0.46 * s, -0.05 * s);
                    out.push(Tri3D::new(plant.root_pos, mid_l, tip, [0.68, 0.12, 0.24, 1.0], 0.48, 0.0, stratum));
                    out.push(Tri3D::new(plant.root_pos, tip, mid_r, [0.68, 0.12, 0.24, 1.0], 0.48, 0.0, stratum));
                }
            }
            UnderbrushKind::StaghornAcropora => {
                let gfp_emissive = 0.82 * night_factor;
                let col = match plant.id % 4 {
                    0 => [0.20, 0.90, 0.86, 1.0],
                    1 => [0.96, 0.50, 0.42, 1.0],
                    2 => [0.48, 0.55, 0.98, 1.0],
                    _ => [0.95, 0.76, 0.28, 1.0],
                };
                for b in 0..10 {
                    let az = (b as f32 / 10.0) * 2.0 * PI + plant.id as f32 * 0.7;
                    let mid = plant.root_pos + Vec3::new(az.cos() * 0.34 * s, 0.46 * s, az.sin() * 0.34 * s);
                    let tip = plant.root_pos + Vec3::new(az.cos() * 0.72 * s, (0.86 + (b % 3) as f32 * 0.24) * s, az.sin() * 0.72 * s);
                    push_tapered_tube(out, plant.root_pos, tip, 0.068 * s, 0.024 * s, 5, col, 0.62, gfp_emissive, stratum);
                    let fork = mid + Vec3::new((az + 0.5).cos() * 0.28 * s, 0.36 * s, (az + 0.5).sin() * 0.28 * s);
                    push_tapered_tube(out, mid, fork, 0.042 * s, 0.018 * s, 4, [0.98, 0.96, 0.92, 1.0], 0.55, gfp_emissive, stratum);
                }
            }
            UnderbrushKind::TableAcropora => {
                let gfp_emissive = 0.74 * night_factor;
                let plate_col = if plant.id % 2 == 0 { [0.90, 0.40, 0.66, 1.0] } else { [0.24, 0.86, 0.68, 1.0] };
                let stem = plant.root_pos + Vec3::new(0.0, 0.52 * s, 0.0);
                push_tapered_tube(out, plant.root_pos, stem, 0.20 * s, 0.13 * s, 6, [0.82, 0.74, 0.64, 1.0], 0.75, 0.0, stratum);
                push_ellipsoid(out, stem, Vec3::new(1.12 * s, 0.085 * s, 0.96 * s), Quat::IDENTITY, plate_col, 0.58, gfp_emissive, stratum);
                push_ellipsoid(out, stem + Vec3::new(0.22 * s, 0.22 * s, 0.10 * s), Vec3::new(0.76 * s, 0.075 * s, 0.68 * s), Quat::IDENTITY, plate_col, 0.58, gfp_emissive, stratum);
            }
            UnderbrushKind::FolioseMontipora => {
                let gfp_emissive = 0.78 * night_factor;
                let whorl_col = if plant.id % 2 == 0 { [0.72, 0.30, 0.96, 1.0] } else { [0.92, 0.42, 0.24, 1.0] };
                for w in 0..5 {
                    let a = (w as f32 / 5.0) * 2.0 * PI + plant.id as f32;
                    let c = plant.root_pos + Vec3::new(a.cos() * 0.28 * s, (0.18 + w as f32 * 0.09) * s, a.sin() * 0.28 * s);
                    let q = Quat::from_rotation_y(a) * Quat::from_rotation_z(0.38);
                    push_ellipsoid(out, c, Vec3::new(0.46 * s, 0.06 * s, 0.42 * s), q, whorl_col, 0.60, gfp_emissive, stratum);
                }
            }
            UnderbrushKind::PoritesBoulder => {
                let col = if plant.id % 2 == 0 { [0.84, 0.66, 0.20, 1.0] } else { [0.68, 0.42, 0.76, 1.0] };
                push_ellipsoid(out, plant.root_pos + Vec3::new(0.0, 0.42 * s, 0.0), Vec3::new(0.72 * s, 0.58 * s, 0.66 * s), Quat::IDENTITY, col, 0.76, 0.40 * night_factor, stratum);
                push_ellipsoid(out, plant.root_pos + Vec3::new(0.34 * s, 0.62 * s, 0.12 * s), Vec3::new(0.44 * s, 0.42 * s, 0.42 * s), Quat::IDENTITY, col, 0.76, 0.40 * night_factor, stratum);
            }
            UnderbrushKind::PillarCoral => {
                let col = [0.88, 0.78, 0.48, 1.0];
                for p in 0..6 {
                    let a = (p as f32 / 6.0) * 2.0 * PI;
                    let r = if p == 0 { 0.0 } else { 0.24 * s };
                    let h = (0.75 + (p % 3) as f32 * 0.26) * s;
                    let b0 = plant.root_pos + Vec3::new(a.cos() * r, 0.0, a.sin() * r);
                    push_tapered_tube(out, b0, b0 + Vec3::new(0.0, h, 0.0), 0.085 * s, 0.065 * s, 6, col, 0.68, 0.50 * night_factor, stratum);
                }
            }
            UnderbrushKind::BarrelSponge => {
                let top = plant.root_pos + Vec3::new(0.0, 0.88 * s, 0.0);
                push_tapered_tube(out, plant.root_pos, top, 0.26 * s, 0.36 * s, 8, [0.66, 0.18, 0.36, 1.0], 0.82, 0.22 * night_factor, stratum);
            }
            UnderbrushKind::SunCupTubastraea => {
                for c_i in 0..6 {
                    let a = (c_i as f32 / 6.0) * 2.0 * PI;
                    let base = plant.root_pos + Vec3::new(a.cos() * 0.16 * s, 0.0, a.sin() * 0.16 * s);
                    let tip = base + Vec3::new(a.cos() * 0.08 * s, -0.22 * s, a.sin() * 0.08 * s);
                    push_tapered_tube(out, base, tip, 0.048 * s, 0.056 * s, 5, [0.98, 0.46, 0.08, 1.0], 0.50, 0.92 * night_factor, stratum);
                }
            }
            UnderbrushKind::BrainCoralDome => {
                push_ellipsoid(out, plant.root_pos + Vec3::new(0.0, 0.28 * s, 0.0), Vec3::new(0.58 * s, 0.42 * s, 0.55 * s), Quat::IDENTITY, [0.76, 0.82, 0.34, 1.0], 0.72, 0.55 * night_factor, stratum);
            }
            UnderbrushKind::GorgonianSeaFan => {
                let fan_col = [0.92, 0.24, 0.46, 1.0];
                for r in -3..=3 {
                    let ang = r as f32 * 0.24;
                    let tip = plant.root_pos + bend_rot * Vec3::new(ang.sin() * 1.15 * s, ang.cos() * 1.25 * s, 0.0);
                    push_tapered_tube(out, plant.root_pos, tip, 0.035 * s, 0.012 * s, 4, fan_col, 0.55, 0.35 * night_factor, stratum);
                }
            }
            UnderbrushKind::HostSeaAnemone => {
                push_ellipsoid(out, plant.root_pos + Vec3::new(0.0, 0.22 * s, 0.0), Vec3::new(0.36 * s, 0.24 * s, 0.36 * s), Quat::IDENTITY, [0.68, 0.22, 0.58, 1.0], 0.55, 0.3 * night_factor, stratum);
                for t_i in 0..10 {
                    let a = (t_i as f32 / 10.0) * 2.0 * PI;
                    let base = plant.root_pos + Vec3::new(a.cos() * 0.22 * s, 0.32 * s, a.sin() * 0.22 * s);
                    let tip = base + bend_rot * Vec3::new(a.cos() * 0.16 * s, 0.38 * s, a.sin() * 0.16 * s);
                    push_tapered_tube(out, base, tip, 0.038 * s, 0.026 * s, 5, [0.32, 0.96, 0.76, 1.0], 0.40, 0.85 * night_factor, stratum);
                }
            }
            UnderbrushKind::VallisneriaEelgrass | UnderbrushKind::CoontailMacrophyte => {
                let col = if plant.kind == UnderbrushKind::VallisneriaEelgrass {
                    [0.18, 0.66, 0.32, 1.0]
                } else {
                    [0.14, 0.54, 0.28, 1.0]
                };
                for r_i in 0..6 {
                    let off = Vec3::new((r_i as f32 - 2.5) * 0.09 * s, 0.0, ((r_i % 2) as f32 - 0.5) * 0.08 * s);
                    let h = (2.1 + (r_i % 3) as f32 * 0.75) * s;
                    let sway_x = (sim_time_sec * 1.4 + plant.id as f32 + r_i as f32).sin() * 0.28 * s;
                    push_tapered_tube(out, plant.root_pos + off, plant.root_pos + off + Vec3::new(sway_x, h, 0.0), 0.032 * s, 0.012 * s, 4, col, 0.52, 0.0, stratum);
                }
            }
            UnderbrushKind::FloatingWaterLily => {
                // Tethered stem down to lakebed + floating emerald lily pad & white-gold Nymphaea blossom!
                push_tapered_tube(out, Vec3::new(plant.root_pos.x, 0.5, plant.root_pos.z), plant.root_pos, 0.018, 0.018, 4, [0.24, 0.48, 0.22, 1.0], 0.6, 0.0, stratum);
                push_ellipsoid(out, plant.root_pos, Vec3::new(0.55 * s, 0.025, 0.52 * s), Quat::IDENTITY, [0.20, 0.62, 0.26, 1.0], 0.45, 0.0, stratum);
                if plant.id % 2 == 0 {
                    push_ellipsoid(out, plant.root_pos + Vec3::new(0.08, 0.08, 0.0), Vec3::new(0.14 * s, 0.11 * s, 0.14 * s), Quat::IDENTITY, [0.98, 0.96, 0.88, 1.0], 0.35, 0.15 * night_factor, stratum);
                }
            }
            UnderbrushKind::ShorelineCattail => {
                for st in 0..5 {
                    let ox = (st as f32 - 2.0) * 0.14 * s;
                    let base = plant.root_pos + Vec3::new(ox, 0.0, 0.0);
                    let top = base + bend_rot * Vec3::new(0.0, 4.4 * s, 0.0);
                    push_tapered_tube(out, base, top, 0.032 * s, 0.018 * s, 4, [0.28, 0.58, 0.24, 1.0], 0.62, 0.0, stratum);
                    if st % 2 == 0 {
                        push_tapered_tube(out, top - Vec3::new(0.0, 0.65 * s, 0.0), top - Vec3::new(0.0, 0.18 * s, 0.0), 0.068 * s, 0.065 * s, 6, [0.38, 0.22, 0.10, 1.0], 0.85, 0.0, stratum);
                    }
                }
            }
            UnderbrushKind::SubmergedDriftwoodMoss => {
                push_ellipsoid(out, plant.root_pos + Vec3::new(0.0, 0.14 * s, 0.0), Vec3::new(0.48 * s, 0.20 * s, 0.38 * s), Quat::IDENTITY, [0.18, 0.46, 0.22, 1.0], 0.80, 0.0, stratum);
            }
            UnderbrushKind::RiftiaTubewormCluster => {
                // Giant Riftia pachyptila: curved ivory chitinous tubes + vivid scarlet hemoglobin obturacular plumes!
                for t_i in 0..7 {
                    let a = (t_i as f32 / 7.0) * 2.0 * PI + plant.id as f32;
                    let base = plant.root_pos + Vec3::new(a.cos() * 0.16 * s, 0.0, a.sin() * 0.16 * s);
                    let h = (0.85 + (t_i % 3) as f32 * 0.32) * s;
                    let top = base + bend_rot * Vec3::new(a.cos() * 0.14 * s, h, a.sin() * 0.14 * s);
                    push_tapered_tube(out, base, top, 0.038 * s, 0.044 * s, 5, [0.93, 0.91, 0.86, 1.0], 0.55, 0.08, stratum);
                    push_tapered_tube(out, top, top + Vec3::new(0.0, 0.24 * s, 0.0), 0.048 * s, 0.022 * s, 5, [0.92, 0.08, 0.14, 1.0], 0.38, 0.42, stratum);
                }
            }
            UnderbrushKind::BeggiatoaSulfurMat => {
                push_ellipsoid(out, plant.root_pos + Vec3::new(0.0, 0.04 * s, 0.0), Vec3::new(0.75 * s, 0.05 * s, 0.68 * s), Quat::IDENTITY, [0.95, 0.94, 0.84, 0.90], 0.72, 0.22, stratum);
            }
            UnderbrushKind::SulfidePagodaFlange => {
                // Beehive/pagoda sulfide ledge + shimmering inverted superheated fluid pool underneath!
                push_ellipsoid(out, plant.root_pos, Vec3::new(0.92 * s, 0.14 * s, 0.82 * s), Quat::IDENTITY, [0.42, 0.33, 0.20, 1.0], 0.70, 0.12, stratum);
                push_ellipsoid(out, plant.root_pos - Vec3::new(0.0, 0.06 * s, 0.0), Vec3::new(0.72 * s, 0.03 * s, 0.64 * s), Quat::IDENTITY, [0.98, 0.65, 0.24, 0.85], 0.15, 0.85, stratum);
            }
            UnderbrushKind::VentMusselBed => {
                for m_i in 0..5 {
                    let a = (m_i as f32 / 5.0) * 2.0 * PI;
                    push_ellipsoid(out, plant.root_pos + Vec3::new(a.cos() * 0.22 * s, 0.08 * s, a.sin() * 0.22 * s), Vec3::new(0.16 * s, 0.08 * s, 0.10 * s), Quat::IDENTITY, [0.46, 0.36, 0.22, 1.0], 0.62, 0.0, stratum);
                }
            }
            UnderbrushKind::LicoriceFernNurseMoss => {
                push_ellipsoid(out, plant.root_pos, Vec3::new(0.45 * s, 0.16 * s, 0.36 * s), Quat::IDENTITY, [0.24, 0.56, 0.20, 1.0], 0.78, 0.0, stratum);
                for f_i in 0..5 {
                    let a = (f_i as f32 - 2.0) * 0.35;
                    let tip = plant.root_pos + bend_rot * Vec3::new(a.sin() * 0.42 * s, 0.55 * s, 0.0);
                    push_tapered_tube(out, plant.root_pos, tip, 0.028 * s, 0.008 * s, 4, [0.18, 0.62, 0.24, 1.0], 0.52, 0.0, stratum);
                }
            }
            UnderbrushKind::RedHuckleberryBush => {
                push_ellipsoid(out, plant.root_pos + Vec3::new(0.0, 0.38 * s, 0.0), Vec3::new(0.42 * s, 0.34 * s, 0.38 * s), Quat::IDENTITY, [0.20, 0.54, 0.24, 1.0], 0.62, 0.0, stratum);
                for b_i in 0..4 {
                    let a = (b_i as f32 / 4.0) * 2.0 * PI;
                    push_oriented_box(out, plant.root_pos + Vec3::new(a.cos() * 0.26 * s, 0.36 * s, a.sin() * 0.26 * s), Vec3::splat(0.032 * s), Quat::IDENTITY, [0.92, 0.16, 0.22, 1.0], 0.3, 0.08, stratum);
                }
            }
            UnderbrushKind::BubblegumOctocoral => {
                // Paragorgia arborea coral fan with bulbous magenta-coral polyp tips on canyon wall!
                let coral_pink = [0.94, 0.32, 0.52, 1.0];
                for br in 0..4 {
                    let a = (br as f32 - 1.5) * 0.42;
                    let tip = plant.root_pos + bend_rot * Vec3::new(a.sin() * 0.52 * s, 0.68 * s, (br as f32 - 1.5) * 0.08 * s);
                    push_tapered_tube(out, plant.root_pos, tip, 0.042 * s, 0.022 * s, 5, coral_pink, 0.45, 0.14, stratum);
                    push_ellipsoid(out, tip, Vec3::splat(0.055 * s), Quat::IDENTITY, [0.98, 0.52, 0.68, 1.0], 0.38, 0.22, stratum);
                }
            }
            UnderbrushKind::CloudGlassSponge => {
                // Hexactinellid silica vase sponge (Aphrocallistes vastus)
                let silica = [0.76, 0.90, 0.98, 0.88];
                for v_i in 0..3 {
                    let off = Vec3::new((v_i as f32 - 1.0) * 0.18 * s, 0.0, 0.0);
                    push_tapered_tube(out, plant.root_pos + off, plant.root_pos + off + Vec3::new(0.0, 0.58 * s, 0.0), 0.055 * s, 0.13 * s, 6, silica, 0.32, 0.18, stratum);
                }
            }
            UnderbrushKind::PostelsiaSeaPalm => {
                // Resilient olive-golden Sea Palm (Postelsia palmaeformis) flexing in surf!
                let olive = [0.44, 0.56, 0.18, 1.0];
                let crown = plant.root_pos + bend_rot * Vec3::new(0.0, 0.62 * s, 0.0);
                push_tapered_tube(out, plant.root_pos, crown, 0.052 * s, 0.032 * s, 6, olive, 0.42, 0.0, stratum);
                for f_i in 0..6 {
                    let a = (f_i as f32 / 6.0) * 2.0 * PI;
                    push_tapered_tube(out, crown, crown + Vec3::new(a.cos() * 0.28 * s, -0.18 * s, a.sin() * 0.28 * s), 0.024 * s, 0.006 * s, 4, [0.52, 0.66, 0.20, 1.0], 0.38, 0.0, stratum);
                }
            }
            UnderbrushKind::LamellibrachiaSeepBush => {
                // 250-year cold-seep tubeworm bush (blue-stained chitinous tubes + pale rose plumes)
                for tb in 0..5 {
                    let a = (tb as f32 / 5.0) * 2.0 * PI;
                    let tip = plant.root_pos + bend_rot * Vec3::new(a.cos() * 0.22 * s, 0.88 * s, a.sin() * 0.22 * s);
                    push_tapered_tube(out, plant.root_pos, tip, 0.028 * s, 0.022 * s, 5, [0.42, 0.62, 0.68, 1.0], 0.55, 0.0, stratum);
                    push_ellipsoid(out, tip, Vec3::splat(0.048 * s), Quat::IDENTITY, [0.96, 0.46, 0.62, 1.0], 0.32, 0.35, stratum);
                }
            }
            UnderbrushKind::MethaneClathrateOutcrop => {
                // Golden-amber crystalline Methane Hydrate ("Methane Ice") mound!
                push_ellipsoid(out, plant.root_pos + Vec3::new(0.0, 0.22 * s, 0.0), Vec3::new(0.56 * s, 0.32 * s, 0.46 * s), Quat::IDENTITY, [0.96, 0.76, 0.38, 0.88], 0.24, 0.32, stratum);
            }
        }
    }

    for item in detritus {
        let (col, strt) = match item.kind {
            crate::physics::DetritusKind::RipeCanopyBerry => ([0.20, 0.50, 0.95, 1.0], TrophicStratum::Producers),
            crate::physics::DetritusKind::FigFruit => ([0.78, 0.36, 0.22, 1.0], TrophicStratum::Producers),
            crate::physics::DetritusKind::GreenLeafFall => ([0.24, 0.68, 0.26, 1.0], TrophicStratum::Decomposers),
            crate::physics::DetritusKind::DeadBranchlet => ([0.42, 0.28, 0.16, 1.0], TrophicStratum::Decomposers),
            crate::physics::DetritusKind::DriftKelpBlade => ([0.62, 0.52, 0.14, 1.0], TrophicStratum::Decomposers),
            crate::physics::DetritusKind::CarbonateCoralSand => ([0.96, 0.95, 0.90, 0.88], TrophicStratum::Decomposers),
            crate::physics::DetritusKind::SunkenAlderLeaf => ([0.58, 0.38, 0.16, 1.0], TrophicStratum::Decomposers),
            crate::physics::DetritusKind::SulfideMineralFloc => ([0.94, 0.92, 0.82, 0.90], TrophicStratum::Decomposers),
        };
        push_oriented_box(out, item.pos, Vec3::splat(item.radius), Quat::IDENTITY, col, 0.55, 0.0, strt);
    }
}

/// Builds biologically accurate 3D anatomical meshes for all 40 Coral Reef, Kelp Forest, and Rainforest species.
pub fn append_organism_geometry(out: &mut Vec<Tri3D>, org: &Organism, night_factor: f32) {
    let s = org.scale;
    let stratum = org.stratum();
    let yaw_q = Quat::from_rotation_y(-org.pose.body_yaw) * Quat::from_rotation_z(org.pose.body_pitch);
    let head_q = yaw_q * Quat::from_rotation_y(org.pose.head_yaw) * Quat::from_rotation_z(org.pose.head_pitch);

    match org.species {
        // ================= NE PACIFIC ROCKY INTERTIDAL TIDEPOOL & COLD-SEEP BRINE POOL SPECIES (v0.9.3 SCULPT) =================
        Species::OchreSeaStar => {
            let col = match org.id % 3 {
                0 => [0.46, 0.16, 0.72, 1.0],
                1 => [0.88, 0.36, 0.10, 1.0],
                _ => [0.68, 0.12, 0.28, 1.0],
            };
            let ossicle = [0.98, 0.95, 0.80, 1.0];
            push_ellipsoid(out, org.pos + Vec3::new(0.0, 0.055 * s, 0.0), Vec3::new(0.14 * s, 0.075 * s, 0.14 * s), Quat::IDENTITY, col, 0.45, 0.0, stratum);
            for arm in 0..5 {
                let a = (arm as f32 / 5.0) * 2.0 * PI + 0.22;
                let dir = Vec3::new(a.cos(), 0.0, a.sin());
                let mid = org.pos + dir * (0.18 * s) + Vec3::new(0.0, 0.042 * s, 0.0);
                let tip = org.pos + dir * (0.36 * s) + Vec3::new(0.0, 0.015 * s, 0.0);
                push_tapered_tube(out, org.pos + Vec3::new(0.0, 0.05 * s, 0.0), mid, 0.095 * s, 0.062 * s, 7, col, 0.46, 0.0, stratum);
                push_tapered_tube(out, mid, tip, 0.062 * s, 0.020 * s, 6, col, 0.46, 0.0, stratum);
                push_ellipsoid(out, mid + Vec3::new(0.0, 0.045 * s, 0.0), Vec3::splat(0.015 * s), Quat::IDENTITY, ossicle, 0.35, 0.0, stratum);
            }
        }
        Species::BlackOystercatcher => {
            let soot = [0.16, 0.14, 0.13, 1.0];
            let hood = [0.08, 0.09, 0.11, 1.0];
            let bill_red = [0.98, 0.22, 0.12, 1.0];
            let legs_pink = [0.96, 0.66, 0.72, 1.0];
            let st = [
                SpineStation { pos: org.pos + yaw_q * Vec3::new(-0.24 * s, 0.30 * s, 0.0), half_width: 0.04 * s, dorsal_height: 0.05 * s, ventral_depth: 0.03 * s, dorsal_rgba: hood, ventral_rgba: soot },
                SpineStation { pos: org.pos + yaw_q * Vec3::new(0.02 * s, 0.30 * s, 0.0), half_width: 0.12 * s, dorsal_height: 0.13 * s, ventral_depth: 0.11 * s, dorsal_rgba: soot, ventral_rgba: soot },
                SpineStation { pos: org.pos + yaw_q * Vec3::new(0.20 * s, 0.35 * s, 0.0), half_width: 0.07 * s, dorsal_height: 0.08 * s, ventral_depth: 0.07 * s, dorsal_rgba: hood, ventral_rgba: hood },
            ];
            push_lofted_spine(out, &st, 10, 0.50, 0.0, stratum);
            let hd = org.pos + yaw_q * Vec3::new(0.26 * s, 0.46 * s, 0.0);
            push_ellipsoid(out, hd, Vec3::new(0.09 * s, 0.08 * s, 0.075 * s), head_q, hood, 0.45, 0.0, stratum);
            push_tapered_tube(out, hd, hd + head_q * Vec3::new(0.25 * s, -0.02 * s, 0.0), 0.022 * s, 0.008 * s, 5, bill_red, 0.22, 0.08, stratum);
            for side in [-1.0_f32, 1.0] {
                push_ellipsoid(out, hd + head_q * Vec3::new(0.03 * s, 0.02 * s, side * 0.068 * s), Vec3::splat(0.018 * s), head_q, [0.98, 0.84, 0.12, 1.0], 0.15, 0.15, stratum);
                let hip = org.pos + yaw_q * Vec3::new(0.02 * s, 0.24 * s, side * 0.065 * s);
                push_tapered_tube(out, hip, Vec3::new(hip.x, org.pos.y, hip.z), 0.016 * s, 0.012 * s, 5, legs_pink, 0.45, 0.0, stratum);
            }
        }
        Species::GiantGreenAnemone => {
            let col_wall = [0.26, 0.34, 0.18, 1.0];
            let disc_teal = [0.08, 0.62, 0.48, 1.0];
            let tent_green = [0.22, 0.92, 0.52, 1.0];
            push_tapered_tube(out, org.pos, org.pos + Vec3::new(0.0, 0.15 * s, 0.0), 0.20 * s, 0.18 * s, 10, col_wall, 0.62, 0.0, stratum);
            push_ellipsoid(out, org.pos + Vec3::new(0.0, 0.15 * s, 0.0), Vec3::new(0.18 * s, 0.025 * s, 0.18 * s), Quat::IDENTITY, disc_teal, 0.35, 0.12, stratum);
            for whorl in 0..3 {
                let n = 10 + whorl * 4;
                let r = (0.08 + whorl as f32 * 0.045) * s;
                for t_i in 0..n {
                    let a = (t_i as f32 / n as f32) * 2.0 * PI + whorl as f32 * 0.25;
                    let base = org.pos + Vec3::new(a.cos() * r, 0.155 * s, a.sin() * r);
                    let tip = org.pos + Vec3::new(a.cos() * (r + 0.11 * s), (0.28 - whorl as f32 * 0.025) * s, a.sin() * (r + 0.11 * s));
                    push_tapered_tube(out, base, tip, 0.018 * s, 0.007 * s, 4, tent_green, 0.25, 0.30, stratum);
                }
            }
        }
        Species::TidepoolSculpin => {
            let olive = [0.32, 0.46, 0.16, 1.0];
            let belly = [0.78, 0.82, 0.72, 1.0];
            let st = [
                SpineStation { pos: org.pos + yaw_q * Vec3::new(-0.20 * s, 0.01 * s, 0.0), half_width: 0.014 * s, dorsal_height: 0.024 * s, ventral_depth: 0.018 * s, dorsal_rgba: olive, ventral_rgba: belly },
                SpineStation { pos: org.pos + yaw_q * Vec3::new(0.02 * s, 0.015 * s, 0.0), half_width: 0.056 * s, dorsal_height: 0.062 * s, ventral_depth: 0.034 * s, dorsal_rgba: olive, ventral_rgba: belly },
                SpineStation { pos: org.pos + yaw_q * Vec3::new(0.14 * s, 0.012 * s, 0.0), half_width: 0.072 * s, dorsal_height: 0.056 * s, ventral_depth: 0.030 * s, dorsal_rgba: olive, ventral_rgba: belly },
            ];
            push_lofted_spine(out, &st, 8, 0.36, 0.0, stratum);
            for side in [-1.0_f32, 1.0] {
                push_oriented_box(out, org.pos + yaw_q * Vec3::new(0.06 * s, 0.01 * s, side * 0.085 * s), Vec3::new(0.065 * s, 0.008 * s, 0.065 * s), yaw_q, [0.58, 0.78, 0.24, 0.9], 0.35, 0.0, stratum);
            }
        }
        Species::CaliforniaMusselBed | Species::BrineMusselRing => {
            let col = if org.species == Species::BrineMusselRing { [0.38, 0.26, 0.16, 1.0] } else { [0.12, 0.17, 0.28, 1.0] };
            for m_i in 0..16 {
                let rad = ((m_i as f32 + 0.5) / 16.0).sqrt() * 0.36 * s;
                let a = m_i as f32 * 2.39996;
                let mp = org.pos + Vec3::new(a.cos() * rad, 0.07 * s, a.sin() * rad);
                push_ellipsoid(out, mp, Vec3::new(0.045 * s, 0.11 * s, 0.032 * s), Quat::from_rotation_z(-a.cos() * 0.28), col, 0.35, 0.0, stratum);
            }
        }
        Species::GooseneckBarnacle => {
            for b_i in 0..7 {
                let a = (b_i as f32 / 7.0) * 2.0 * PI;
                let off = Vec3::new(a.cos() * 0.075 * s, 0.0, a.sin() * 0.075 * s);
                let top = org.pos + off + Vec3::new(a.cos() * 0.03 * s, 0.15 * s, a.sin() * 0.03 * s);
                push_tapered_tube(out, org.pos + off, top, 0.028 * s, 0.022 * s, 6, [0.20, 0.17, 0.15, 1.0], 0.68, 0.0, stratum);
                push_ellipsoid(out, top + Vec3::new(0.0, 0.045 * s, 0.0), Vec3::new(0.032 * s, 0.055 * s, 0.020 * s), Quat::IDENTITY, [0.96, 0.96, 0.94, 1.0], 0.30, 0.0, stratum);
            }
        }
        Species::TidepoolHermitCrab => {
            // Coiled Tegula turban shell + emerging crab claws & royal-blue-banded legs!
            for w in 0..6 {
                let wf = w as f32 / 5.0;
                let wa = wf * 3.8;
                let wp = org.pos + yaw_q * Vec3::new(-0.03 * s + wa.cos() * 0.04 * (1.0 - wf) * s, (0.06 + wf * 0.06) * s, wa.sin() * 0.04 * (1.0 - wf) * s);
                push_ellipsoid(out, wp, Vec3::splat(0.052 * (1.0 - wf * 0.55) * s), yaw_q, [0.14, 0.12, 0.20, 1.0], 0.38, 0.0, stratum);
            }
            for side in [-1.0_f32, 1.0] {
                let kn = org.pos + yaw_q * Vec3::new(0.06 * s, 0.05 * s, side * 0.065 * s);
                let ft = org.pos + yaw_q * Vec3::new(0.08 * s, 0.0, side * 0.095 * s);
                push_tapered_tube(out, org.pos + yaw_q * Vec3::new(0.05 * s, 0.04 * s, side * 0.03 * s), kn, 0.010 * s, 0.008 * s, 4, [0.34, 0.42, 0.20, 1.0], 0.42, 0.0, stratum);
                push_tapered_tube(out, kn, ft, 0.008 * s, 0.005 * s, 4, [0.16, 0.42, 0.94, 1.0], 0.28, 0.18, stratum);
            }
        }
        Species::BlackKatyChiton => {
            // Glossy black muscular girdle + 8 white overlapping butterfly shell valves!
            push_ellipsoid(out, org.pos + Vec3::new(0.0, 0.022 * s, 0.0), Vec3::new(0.13 * s, 0.032 * s, 0.082 * s), yaw_q, [0.08, 0.09, 0.11, 1.0], 0.25, 0.0, stratum);
            for pl in 0..8 {
                let px = (-0.084 + pl as f32 * 0.024) * s;
                let w_mul = (((pl as f32 + 0.8) / 8.6) * PI).sin();
                push_oriented_box(out, org.pos + yaw_q * Vec3::new(px, 0.048 * w_mul * s, 0.0), Vec3::new(0.010 * s, 0.006 * s, 0.034 * w_mul * s), yaw_q, [0.95, 0.96, 0.98, 1.0], 0.32, 0.0, stratum);
            }
        }
        Species::CutthroatBrineEel | Species::ColdSeepHagfish => {
            let col = if org.species == Species::CutthroatBrineEel { [0.48, 0.36, 0.52, 1.0] } else { [0.58, 0.42, 0.46, 1.0] };
            let mut prev = org.pos - yaw_q * Vec3::new(0.55 * s, 0.0, 0.0);
            for seg in 1..=6 {
                let f = seg as f32 / 6.0;
                let wave = (org.pose.spine_phase + f * 4.5).sin() * 0.09 * s;
                let nxt = org.pos + yaw_q * Vec3::new((-0.55 + f * 1.10) * s, 0.0, wave);
                push_tapered_tube(out, prev, nxt, 0.042 * s * (1.1 - f * 0.4), 0.036 * s * (1.1 - f * 0.4), 5, col, 0.32, 0.08, stratum);
                prev = nxt;
            }
        }
        Species::SeepGhostChimaera => {
            let pale = [0.70, 0.78, 0.88, 1.0];
            push_tapered_tube(out, org.pos - yaw_q * Vec3::new(0.65 * s, 0.0, 0.0), org.pos + yaw_q * Vec3::new(0.35 * s, 0.0, 0.0), 0.025 * s, 0.14 * s, 7, pale, 0.30, 0.12, stratum);
        }
        Species::DeepSeaRedCrab | Species::SeepSquatLobster => {
            let scarlet = if org.species == Species::DeepSeaRedCrab { [0.88, 0.18, 0.16, 1.0] } else { [0.92, 0.86, 0.78, 1.0] };
            push_ellipsoid(out, org.pos + Vec3::new(0.0, 0.08 * s, 0.0), Vec3::new(0.14 * s, 0.06 * s, 0.12 * s), yaw_q, scarlet, 0.42, 0.06, stratum);
        }
        Species::AlvinocarisSeepShrimp | Species::MethaneIceWorm => {
            let pink = [0.96, 0.56, 0.70, 0.92];
            push_tapered_tube(out, org.pos - yaw_q * Vec3::new(0.08 * s, 0.0, 0.0), org.pos + yaw_q * Vec3::new(0.08 * s, 0.02 * s, 0.0), 0.014 * s, 0.018 * s, 5, pink, 0.28, 0.25, stratum);
        }

        // ================= MIDWATER PELAGIC RIFT CANYON SPECIES (200-1,500M) =================
        Species::DivingSpermWhale => {
            // Physeter macrocephalus: iconic block-shaped spermaceti rostrum, underslung narrow jaw & broad flukes!
            let slate = [0.26, 0.30, 0.36, 1.0];
            let pale_jaw = [0.72, 0.75, 0.80, 1.0];
            let fluke_beat = org.pose.spine_phase.sin() * 0.18 * s;
            let st = [
                SpineStation { pos: org.pos + yaw_q * Vec3::new(-1.55 * s, fluke_beat, 0.0), half_width: 0.08 * s, dorsal_height: 0.12 * s, ventral_depth: 0.10 * s, dorsal_rgba: slate, ventral_rgba: slate },
                SpineStation { pos: org.pos + yaw_q * Vec3::new(-0.35 * s, 0.0, 0.0), half_width: 0.36 * s, dorsal_height: 0.44 * s, ventral_depth: 0.36 * s, dorsal_rgba: slate, ventral_rgba: slate },
                SpineStation { pos: org.pos + yaw_q * Vec3::new(0.85 * s, 0.04 * s, 0.0), half_width: 0.38 * s, dorsal_height: 0.52 * s, ventral_depth: 0.28 * s, dorsal_rgba: slate, ventral_rgba: slate },
                SpineStation { pos: org.pos + yaw_q * Vec3::new(1.55 * s, 0.06 * s, 0.0), half_width: 0.32 * s, dorsal_height: 0.48 * s, ventral_depth: 0.18 * s, dorsal_rgba: slate, ventral_rgba: slate },
            ];
            push_lofted_spine(out, &st, 10, 0.55, 0.0, stratum);
            // Narrow underslung lower jaw + horizontal tail flukes
            push_tapered_tube(out, org.pos + yaw_q * Vec3::new(0.45 * s, -0.26 * s, 0.0), org.pos + yaw_q * Vec3::new(1.38 * s, -0.24 * s, 0.0), 0.07 * s, 0.04 * s, 5, pale_jaw, 0.45, 0.0, stratum);
            for side in [-1.0_f32, 1.0] {
                push_tapered_tube(out, org.pos + yaw_q * Vec3::new(-1.55 * s, fluke_beat, 0.0), org.pos + yaw_q * Vec3::new(-1.82 * s, fluke_beat, side * 0.48 * s), 0.11 * s, 0.03 * s, 4, slate, 0.50, 0.0, stratum);
            }
        }

        Species::ArchiteuthisGiantSquid => {
            // Architeuthis dux: crimson torpedo mantle, arrowhead fins, huge dark eyes & 8 arms + 2 clubbed feeding tentacles!
            let crimson = [0.76, 0.14, 0.22, 1.0];
            let pale_rose = [0.92, 0.42, 0.48, 1.0];
            push_tapered_tube(out, org.pos - yaw_q * Vec3::new(0.85 * s, 0.0, 0.0), org.pos + yaw_q * Vec3::new(0.25 * s, 0.0, 0.0), 0.06 * s, 0.22 * s, 8, crimson, 0.35, 0.06, stratum);
            for arm_i in 0..8 {
                let a = (arm_i as f32 / 8.0) * 2.0 * PI + org.pose.spine_phase * 0.2;
                let wave = (org.pose.spine_phase + arm_i as f32).sin() * 0.10 * s;
                let tip = org.pos + yaw_q * Vec3::new(1.05 * s, a.sin() * 0.18 * s + wave, a.cos() * 0.18 * s);
                push_tapered_tube(out, org.pos + yaw_q * Vec3::new(0.25 * s, 0.0, 0.0), tip, 0.038 * s, 0.010 * s, 5, pale_rose, 0.38, 0.05, stratum);
            }
            for side in [-1.0_f32, 1.0] {
                let club = org.pos + yaw_q * Vec3::new(1.65 * s, (org.pose.spine_phase).sin() * 0.16 * s, side * 0.14 * s);
                push_tapered_tube(out, org.pos + yaw_q * Vec3::new(0.25 * s, 0.0, 0.0), club, 0.020 * s, 0.014 * s, 5, pale_rose, 0.35, 0.08, stratum);
                push_ellipsoid(out, club, Vec3::new(0.09 * s, 0.035 * s, 0.025 * s), yaw_q, crimson, 0.32, 0.10, stratum);
            }
        }

        Species::PacificBarreleye => {
            // Macropinna microstoma: dark body + transparent cranial dome + bright emerald tubular eyes!
            let bronze = [0.20, 0.16, 0.14, 1.0];
            let emerald_eye = [0.18, 0.96, 0.48, 1.0];
            let dome_glass = [0.62, 0.90, 0.98, 0.36];
            push_ellipsoid(out, org.pos, Vec3::new(0.22 * s, 0.12 * s, 0.11 * s), yaw_q, bronze, 0.40, 0.0, stratum);
            for side in [-1.0_f32, 1.0] {
                let eye_base = org.pos + yaw_q * Vec3::new(0.09 * s, 0.03 * s, side * 0.042 * s);
                let eye_dir = Vec3::new(org.pose.head_pitch.cos(), org.pose.head_pitch.sin(), 0.0);
                push_tapered_tube(out, eye_base, eye_base + yaw_q * (eye_dir * 0.085 * s), 0.026 * s, 0.028 * s, 6, emerald_eye, 0.15, 0.85, stratum);
            }
            push_ellipsoid(out, org.pos + yaw_q * Vec3::new(0.09 * s, 0.07 * s, 0.0), Vec3::new(0.13 * s, 0.11 * s, 0.10 * s), yaw_q, dome_glass, 0.08, 0.22, stratum);
        }

        Species::PrayaGiantSiphonophore => {
            // Praya dubia: paired translucent blue nectophores + sinusoidal golden-cyan bioluminescent curtain!
            let bell_c = [0.48, 0.86, 0.98, 0.65];
            let stem_c = [0.98, 0.82, 0.32, 0.92];
            push_ellipsoid(out, org.pos, Vec3::new(0.20 * s, 0.13 * s, 0.12 * s), Quat::IDENTITY, bell_c, 0.15, 0.55, stratum);
            push_ellipsoid(out, org.pos + Vec3::new(-0.22 * s, 0.02 * s, 0.0), Vec3::new(0.18 * s, 0.12 * s, 0.11 * s), Quat::IDENTITY, bell_c, 0.15, 0.55, stratum);
            let mut prev = org.pos + Vec3::new(-0.34 * s, 0.0, 0.0);
            for seg in 1..=8 {
                let f = seg as f32;
                let nxt = org.pos + Vec3::new(-0.34 * s - f * 0.55 * s, (org.pose.spine_phase + f * 0.7).sin() * 0.24 * s - f * 0.08 * s, 0.0);
                push_tapered_tube(out, prev, nxt, 0.022 * s, 0.016 * s, 4, stem_c, 0.20, 0.78, stratum);
                prev = nxt;
            }
        }

        Species::CockEyedJewelSquid => {
            let ruby = [0.84, 0.22, 0.36, 0.92];
            push_ellipsoid(out, org.pos, Vec3::new(0.18 * s, 0.09 * s, 0.09 * s), yaw_q, ruby, 0.28, 0.42, stratum);
            // Enlarged left upward-looking yellow-lensed eye!
            push_ellipsoid(out, org.pos + yaw_q * Vec3::new(0.08 * s, 0.06 * s, 0.065 * s), Vec3::splat(0.042 * s), yaw_q, [0.96, 0.92, 0.24, 1.0], 0.15, 0.75, stratum);
        }

        Species::MyctophidLanternfish | Species::SilverHatchetfish => {
            let is_hatchet = org.species == Species::SilverHatchetfish;
            let silver = [0.68, 0.78, 0.88, 1.0];
            let photophore = if is_hatchet { [0.86, 0.38, 0.98, 1.0] } else { [0.22, 0.88, 0.98, 1.0] };
            let rx = if is_hatchet { 0.11 * s } else { 0.15 * s };
            let ry = if is_hatchet { 0.11 * s } else { 0.045 * s };
            push_ellipsoid(out, org.pos, Vec3::new(rx, ry, 0.028 * s), yaw_q, silver, 0.18, 0.15, stratum);
            push_ellipsoid(out, org.pos - Vec3::new(0.0, ry * 0.78, 0.0), Vec3::new(rx * 0.75, 0.018 * s, 0.022 * s), yaw_q, photophore, 0.10, 0.92, stratum);
        }

        Species::PelagicEuphausiidKrill => {
            push_tapered_tube(out, org.pos - yaw_q * Vec3::new(0.06 * s, 0.0, 0.0), org.pos + yaw_q * Vec3::new(0.06 * s, 0.01 * s, 0.0), 0.012 * s, 0.018 * s, 4, [0.98, 0.48, 0.32, 0.90], 0.28, 0.35, stratum);
        }

        Species::GiantLarvaceanHouse => {
            // Bathochordaeus charon: translucent bilobed mucus filter house + tadpole zooid inside!
            let mucus = [0.58, 0.84, 0.96, 0.26];
            for side in [-1.0_f32, 1.0] {
                push_ellipsoid(out, org.pos + Vec3::new(0.0, 0.0, side * 0.18 * s), Vec3::new(0.38 * s, 0.28 * s, 0.24 * s), Quat::IDENTITY, mucus, 0.15, 0.28, stratum);
            }
            push_tapered_tube(out, org.pos - Vec3::new(0.08 * s, 0.0, 0.0), org.pos + Vec3::new(0.08 * s, 0.0, 0.0), 0.018 * s, 0.032 * s, 5, [0.92, 0.74, 0.86, 0.95], 0.25, 0.45, stratum);
        }

        Species::VampireSquid => {
            // Vampyroteuthis infernalis: velvety maroon cloak, ear-like mantle fins & sapphire eyes!
            let velvet = [0.42, 0.10, 0.16, 1.0];
            push_ellipsoid(out, org.pos, Vec3::new(0.18 * s, 0.14 * s, 0.14 * s), yaw_q, velvet, 0.55, 0.12, stratum);
            push_tapered_tube(out, org.pos + yaw_q * Vec3::new(0.08 * s, 0.0, 0.0), org.pos + yaw_q * Vec3::new(0.32 * s, 0.0, 0.0), 0.15 * s, 0.04 * s, 7, velvet, 0.58, 0.18, stratum);
            push_ellipsoid(out, org.pos + yaw_q * Vec3::new(0.06 * s, 0.03 * s, 0.11 * s), Vec3::splat(0.032 * s), yaw_q, [0.22, 0.68, 0.98, 1.0], 0.12, 0.82, stratum);
        }

        // ================= PNW OLD-GROWTH & SPAWNING SALMON CREEK SPECIES =================
        Species::NorthernSpottedOwl => {
            let choc_brown = [0.36, 0.23, 0.14, 1.0];
            let cream_spot = [0.92, 0.88, 0.78, 1.0];
            push_ellipsoid(out, org.pos + Vec3::new(0.0, 0.18 * s, 0.0), Vec3::new(0.16 * s, 0.26 * s, 0.15 * s), yaw_q, choc_brown, 0.58, 0.0, stratum);
            let head_c = org.pos + Vec3::new(0.0, 0.44 * s, 0.0);
            push_ellipsoid(out, head_c, Vec3::splat(0.13 * s), head_q, choc_brown, 0.55, 0.0, stratum);
            push_ellipsoid(out, head_c + head_q * Vec3::new(0.09 * s, 0.0, 0.0), Vec3::new(0.03 * s, 0.09 * s, 0.10 * s), head_q, cream_spot, 0.50, 0.0, stratum);
            if org.pose.wing_phase_amp[1] > 0.2 {
                let flap = org.pose.wing_phase_amp[0].sin() * 0.38;
                for side in [-1.0_f32, 1.0] {
                    push_tapered_tube(out, org.pos + Vec3::new(0.0, 0.26 * s, 0.0), org.pos + yaw_q * Vec3::new(-0.08 * s, 0.28 * s + flap * 0.22 * s, side * 0.58 * s), 0.09 * s, 0.03 * s, 4, choc_brown, 0.55, 0.0, stratum);
                }
            }
        }

        Species::AmericanBlackBear => {
            let fur = [0.11, 0.10, 0.10, 1.0];
            let muzzle = [0.54, 0.38, 0.24, 1.0];
            let torso = org.pos + Vec3::new(0.0, 0.58 * s, 0.0);
            push_ellipsoid(out, torso, Vec3::new(0.62 * s, 0.38 * s, 0.36 * s), yaw_q, fur, 0.72, 0.0, stratum);
            let hd = torso + yaw_q * Vec3::new(0.58 * s, 0.16 * s, 0.0);
            push_ellipsoid(out, hd, Vec3::splat(0.20 * s), head_q, fur, 0.70, 0.0, stratum);
            push_ellipsoid(out, hd + head_q * Vec3::new(0.16 * s, -0.03 * s, 0.0), Vec3::new(0.10 * s, 0.07 * s, 0.07 * s), head_q, muzzle, 0.60, 0.0, stratum);
            for lx in [-0.36_f32, 0.36] {
                for lz in [-0.20_f32, 0.20] {
                    let hip = torso + yaw_q * Vec3::new(lx * s, -0.18 * s, lz * s);
                    push_tapered_tube(out, hip, Vec3::new(hip.x, org.pos.y, hip.z), 0.12 * s, 0.09 * s, 5, fur, 0.75, 0.0, stratum);
                }
            }
        }

        Species::SpawningCohoSalmon => {
            // Oncorhynchus kisutch in spawning livery: bottle-green back & head + vivid crimson-maroon flank + hooked kype!
            let green_head = [0.14, 0.34, 0.22, 1.0];
            let crimson_side = [0.84, 0.16, 0.22, 1.0];
            let wag = org.pose.spine_phase.sin() * 0.06 * s;
            let st = [
                SpineStation { pos: org.pos + yaw_q * Vec3::new(-0.34 * s, 0.0, wag), half_width: 0.03 * s, dorsal_height: 0.06 * s, ventral_depth: 0.05 * s, dorsal_rgba: green_head, ventral_rgba: crimson_side },
                SpineStation { pos: org.pos, half_width: 0.08 * s, dorsal_height: 0.15 * s, ventral_depth: 0.13 * s, dorsal_rgba: crimson_side, ventral_rgba: crimson_side },
                SpineStation { pos: org.pos + yaw_q * Vec3::new(0.32 * s, 0.0, 0.0), half_width: 0.05 * s, dorsal_height: 0.09 * s, ventral_depth: 0.08 * s, dorsal_rgba: green_head, ventral_rgba: green_head },
            ];
            push_lofted_spine(out, &st, 8, 0.30, 0.0, stratum);
        }

        Species::AmericanDipper | Species::VariedThrush => {
            let is_dipper = org.species == Species::AmericanDipper;
            let body_c = if is_dipper { [0.30, 0.33, 0.38, 1.0] } else { [0.24, 0.30, 0.38, 1.0] };
            push_ellipsoid(out, org.pos + Vec3::new(0.0, 0.08 * s, 0.0), Vec3::new(0.11 * s, 0.08 * s, 0.08 * s), yaw_q, body_c, 0.48, 0.0, stratum);
            if !is_dipper {
                push_ellipsoid(out, org.pos + yaw_q * Vec3::new(0.04 * s, 0.06 * s, 0.0), Vec3::new(0.08 * s, 0.05 * s, 0.07 * s), yaw_q, [0.92, 0.48, 0.12, 1.0], 0.45, 0.0, stratum);
            }
        }

        Species::DouglasSquirrel => {
            let rust_grey = [0.46, 0.34, 0.22, 1.0];
            push_ellipsoid(out, org.pos + Vec3::new(0.0, 0.07 * s, 0.0), Vec3::new(0.12 * s, 0.07 * s, 0.065 * s), yaw_q, rust_grey, 0.55, 0.0, stratum);
            push_tapered_tube(out, org.pos + yaw_q * Vec3::new(-0.10 * s, 0.08 * s, 0.0), org.pos + yaw_q * Vec3::new(-0.22 * s, 0.24 * s, 0.0), 0.045 * s, 0.055 * s, 5, rust_grey, 0.65, 0.0, stratum);
        }

        Species::PacificGiantSalamander => {
            let bronze = [0.42, 0.28, 0.16, 1.0];
            push_tapered_tube(out, org.pos - yaw_q * Vec3::new(0.18 * s, 0.0, 0.0), org.pos + yaw_q * Vec3::new(0.16 * s, 0.02 * s, 0.0), 0.022 * s, 0.038 * s, 6, bronze, 0.38, 0.0, stratum);
        }

        Species::PacificBananaSlug => {
            // Ariolimax columbianus: bright canary-yellow pulmonate slug with sensory tentacles!
            let banana_yellow = [0.95, 0.84, 0.12, 1.0];
            push_tapered_tube(out, org.pos - yaw_q * Vec3::new(0.11 * s, 0.0, 0.0), org.pos + yaw_q * Vec3::new(0.11 * s, 0.01 * s, 0.0), 0.024 * s, 0.032 * s, 6, banana_yellow, 0.28, 0.06, stratum);
            for side in [-1.0_f32, 1.0] {
                push_tapered_tube(out, org.pos + yaw_q * Vec3::new(0.10 * s, 0.03 * s, 0.0), org.pos + yaw_q * Vec3::new(0.15 * s, 0.07 * s, side * 0.025 * s), 0.004 * s, 0.003 * s, 4, banana_yellow, 0.3, 0.0, stratum);
            }
        }

        Species::GoldenChanterelle => {
            let gold = [0.96, 0.68, 0.12, 1.0];
            push_tapered_tube(out, org.pos, org.pos + Vec3::new(0.0, 0.14 * s, 0.0), 0.028 * s, 0.095 * s, 7, gold, 0.48, 0.14, stratum);
        }

        Species::CasedCaddisflyLarva => {
            push_tapered_tube(out, org.pos - Vec3::new(0.05 * s, 0.0, 0.0), org.pos + Vec3::new(0.05 * s, 0.0, 0.0), 0.018 * s, 0.022 * s, 5, [0.46, 0.44, 0.38, 1.0], 0.78, 0.0, stratum);
        }

        // ================= DEEP-SEA HYDROTHERMAL VENT SPECIES (2,600M BLACK SMOKER) =================
        Species::AbyssalGhostShark => {
            // Hydrolagus chimaera: silvery-lavender holocephalan + glowing green tapetum eyes + wing-like pectorals + whip tail!
            let silver_violet = [0.64, 0.66, 0.78, 1.0];
            let pale_belly = [0.86, 0.88, 0.94, 1.0];
            let st = [
                SpineStation { pos: org.pos + yaw_q * Vec3::new(-0.82 * s, 0.0, org.pose.spine_phase.sin() * 0.08 * s), half_width: 0.015 * s, dorsal_height: 0.022 * s, ventral_depth: 0.018 * s, dorsal_rgba: silver_violet, ventral_rgba: pale_belly },
                SpineStation { pos: org.pos + yaw_q * Vec3::new(-0.22 * s, 0.0, 0.0), half_width: 0.095 * s, dorsal_height: 0.13 * s, ventral_depth: 0.10 * s, dorsal_rgba: silver_violet, ventral_rgba: pale_belly },
                SpineStation { pos: org.pos + yaw_q * Vec3::new(0.24 * s, 0.01 * s, 0.0), half_width: 0.13 * s, dorsal_height: 0.16 * s, ventral_depth: 0.13 * s, dorsal_rgba: silver_violet, ventral_rgba: pale_belly },
                SpineStation { pos: org.pos + yaw_q * Vec3::new(0.52 * s, -0.01 * s, 0.0), half_width: 0.075 * s, dorsal_height: 0.10 * s, ventral_depth: 0.08 * s, dorsal_rgba: silver_violet, ventral_rgba: pale_belly },
            ];
            push_lofted_spine(out, &st, 8, 0.28, 0.12, stratum);
            // Tall venomous first dorsal spine + wing-like rowing pectoral fins!
            push_tapered_tube(out, org.pos + yaw_q * Vec3::new(0.16 * s, 0.14 * s, 0.0), org.pos + yaw_q * Vec3::new(0.02 * s, 0.42 * s, 0.0), 0.045 * s, 0.008 * s, 4, silver_violet, 0.30, 0.10, stratum);
            let stroke = org.pose.wing_phase_amp[0].sin() * 0.48;
            for side in [-1.0_f32, 1.0] {
                let f_rot = yaw_q * Quat::from_rotation_x(side * stroke);
                let sh = org.pos + yaw_q * Vec3::new(0.20 * s, -0.05 * s, side * 0.11 * s);
                let tip = sh + f_rot * Vec3::new(-0.14 * s, -0.06 * s, side * 0.52 * s);
                push_tapered_tube(out, sh, tip, 0.11 * s, 0.025 * s, 4, pale_belly, 0.32, 0.12, stratum);
                push_ellipsoid(out, org.pos + yaw_q * Vec3::new(0.38 * s, 0.06 * s, side * 0.085 * s), Vec3::splat(0.036 * s), yaw_q, [0.22, 0.96, 0.58, 1.0], 0.15, 0.95, stratum);
            }
        }

        Species::BluntnoseSixgillShark => {
            let slate = [0.34, 0.36, 0.40, 1.0];
            let st = [
                SpineStation { pos: org.pos + yaw_q * Vec3::new(-1.1 * s, 0.0, org.pose.spine_phase.sin() * 0.14 * s), half_width: 0.05 * s, dorsal_height: 0.08 * s, ventral_depth: 0.07 * s, dorsal_rgba: slate, ventral_rgba: slate },
                SpineStation { pos: org.pos, half_width: 0.22 * s, dorsal_height: 0.24 * s, ventral_depth: 0.20 * s, dorsal_rgba: slate, ventral_rgba: [0.58, 0.60, 0.64, 1.0] },
                SpineStation { pos: org.pos + yaw_q * Vec3::new(0.95 * s, 0.0, 0.0), half_width: 0.14 * s, dorsal_height: 0.12 * s, ventral_depth: 0.10 * s, dorsal_rgba: slate, ventral_rgba: slate },
            ];
            push_lofted_spine(out, &st, 8, 0.42, 0.06, stratum);
        }

        Species::VulcanoctopusVent => {
            let pearl = [0.92, 0.88, 0.94, 1.0];
            push_ellipsoid(out, org.pos + Vec3::new(0.0, 0.16 * s, 0.0), Vec3::new(0.18 * s, 0.20 * s, 0.17 * s), yaw_q, pearl, 0.35, 0.22, stratum);
            for a in 0..8 {
                let ang = (a as f32 / 8.0) * 2.0 * PI + org.pose.spine_phase * 0.2;
                let tip = org.pos + Vec3::new(ang.cos() * 0.36 * s, 0.02 * s, ang.sin() * 0.36 * s);
                push_tapered_tube(out, org.pos + Vec3::new(0.0, 0.08 * s, 0.0), tip, 0.036 * s, 0.010 * s, 5, pearl, 0.38, 0.18, stratum);
            }
        }

        Species::AtollaCrownJelly | Species::BiolumCombJelly => {
            let is_atolla = org.species == Species::AtollaCrownJelly;
            let col = if is_atolla { [0.78, 0.12, 0.28, 0.88] } else { [0.22, 0.88, 0.98, 0.72] };
            let glow = 0.55 + 0.65 * org.pose.chromatophore;
            push_ellipsoid(out, org.pos, Vec3::new(0.24 * s, if is_atolla { 0.10 * s } else { 0.30 * s }, 0.24 * s), Quat::IDENTITY, col, 0.20, glow, stratum);
            if is_atolla {
                for t_i in 0..8 {
                    let a = (t_i as f32 / 8.0) * 2.0 * PI;
                    let base = org.pos + Vec3::new(a.cos() * 0.22 * s, -0.02 * s, a.sin() * 0.22 * s);
                    push_tapered_tube(out, base, base + Vec3::new(a.cos() * 0.12 * s, -0.34 * s, a.sin() * 0.12 * s), 0.012 * s, 0.003 * s, 4, [0.18, 0.76, 1.0, 0.90], 0.2, glow * 1.2, stratum);
                }
            }
        }

        Species::ZoarcidVentEelpout => {
            let pale_pink = [0.92, 0.78, 0.80, 1.0];
            push_tapered_tube(out, org.pos - yaw_q * Vec3::new(0.26 * s, 0.0, 0.0), org.pos + yaw_q * Vec3::new(0.24 * s, 0.0, 0.0), 0.025 * s, 0.042 * s, 6, pale_pink, 0.40, 0.12, stratum);
        }

        Species::KiwaYetiCrab | Species::MunidopsisSquatLobster => {
            let ivory = [0.95, 0.93, 0.88, 1.0];
            push_ellipsoid(out, org.pos + Vec3::new(0.0, 0.07 * s, 0.0), Vec3::new(0.11 * s, 0.06 * s, 0.09 * s), yaw_q, ivory, 0.52, 0.15, stratum);
            let wave = org.pose.limb_phase.sin() * 0.10 * s;
            for side in [-1.0_f32, 1.0] {
                let sh = org.pos + yaw_q * Vec3::new(0.08 * s, 0.08 * s, side * 0.08 * s);
                let claw = sh + yaw_q * Vec3::new(0.22 * s, 0.06 * s + side * wave, side * 0.06 * s);
                push_tapered_tube(out, sh, claw, 0.028 * s, 0.034 * s, 5, ivory, 0.65, 0.18, stratum);
            }
        }

        Species::RimicarisVentShrimp => {
            push_ellipsoid(out, org.pos, Vec3::new(0.055 * s, 0.024 * s, 0.022 * s), yaw_q, [0.96, 0.72, 0.52, 0.92], 0.35, 0.28, stratum);
        }

        Species::ScalyFootSnail => {
            push_ellipsoid(out, org.pos + Vec3::new(0.0, 0.06 * s, 0.0), Vec3::new(0.08 * s, 0.07 * s, 0.07 * s), yaw_q, [0.18, 0.19, 0.22, 1.0], 0.25, 0.08, stratum);
        }

        // ================= TEMPERATE FRESHWATER LAKE SPECIES (UNDERWATER & WATERBIRDS) =================
        Species::Osprey => {
            let dark_brown = [0.24, 0.17, 0.12, 1.0];
            let snow_white = [0.96, 0.96, 0.95, 1.0];
            push_ellipsoid(out, org.pos, Vec3::new(0.32 * s, 0.13 * s, 0.14 * s), yaw_q, snow_white, 0.48, 0.0, stratum);
            push_ellipsoid(out, org.pos + yaw_q * Vec3::new(0.0, 0.05 * s, 0.0), Vec3::new(0.30 * s, 0.09 * s, 0.13 * s), yaw_q, dark_brown, 0.52, 0.0, stratum);
            let flap = org.pose.wing_phase_amp[0].sin() * 0.42;
            for side in [-1.0_f32, 1.0] {
                let wrist = org.pos + yaw_q * Vec3::new(0.06 * s, 0.12 * s + flap * 0.18 * s, side * 0.46 * s);
                let tip = wrist + yaw_q * Vec3::new(-0.14 * s, flap * 0.24 * s, side * 0.48 * s);
                push_tapered_tube(out, org.pos, wrist, 0.075 * s, 0.055 * s, 4, dark_brown, 0.5, 0.0, stratum);
                push_tapered_tube(out, wrist, tip, 0.055 * s, 0.020 * s, 4, dark_brown, 0.5, 0.0, stratum);
            }
        }

        Species::CommonLoon | Species::MallardDuck => {
            let is_loon = org.species == Species::CommonLoon;
            let body_col = if is_loon { [0.10, 0.12, 0.14, 1.0] } else { [0.54, 0.46, 0.38, 1.0] };
            let head_col = if is_loon { [0.08, 0.14, 0.12, 1.0] } else if org.id % 2 == 0 { [0.08, 0.48, 0.26, 1.0] } else { [0.48, 0.36, 0.24, 1.0] };
            push_ellipsoid(out, org.pos, Vec3::new(0.34 * s, 0.14 * s, 0.18 * s), yaw_q, body_col, 0.42, 0.0, stratum);
            let neck_top = org.pos + yaw_q * Vec3::new(0.24 * s, 0.22 * s, 0.0);
            push_tapered_tube(out, org.pos + yaw_q * Vec3::new(0.18 * s, 0.06 * s, 0.0), neck_top, 0.068 * s, 0.055 * s, 6, head_col, 0.38, 0.0, stratum);
            push_ellipsoid(out, neck_top + yaw_q * Vec3::new(0.04 * s, 0.03 * s, 0.0), Vec3::new(0.095 * s, 0.075 * s, 0.070 * s), yaw_q, head_col, 0.35, 0.0, stratum);
            let bill_col = if is_loon { [0.12, 0.12, 0.14, 1.0] } else { [0.92, 0.78, 0.16, 1.0] };
            push_tapered_tube(out, neck_top + yaw_q * Vec3::new(0.10 * s, 0.02 * s, 0.0), neck_top + yaw_q * Vec3::new(0.24 * s, 0.01 * s, 0.0), 0.028 * s, 0.010 * s, 4, bill_col, 0.35, 0.0, stratum);
        }

        Species::GreatBlueHeron => {
            let slate = [0.42, 0.50, 0.60, 1.0];
            let body_c = org.pos + Vec3::new(0.0, 0.62 * s, 0.0);
            for side in [-1.0_f32, 1.0] {
                push_tapered_tube(out, org.pos + Vec3::new(0.0, -0.35 * s, side * 0.07 * s), body_c + Vec3::new(0.0, -0.06 * s, side * 0.07 * s), 0.022 * s, 0.028 * s, 4, [0.52, 0.48, 0.34, 1.0], 0.6, 0.0, stratum);
            }
            push_ellipsoid(out, body_c, Vec3::new(0.32 * s, 0.18 * s, 0.16 * s), yaw_q, slate, 0.50, 0.0, stratum);
            let head_c = body_c + yaw_q * Vec3::new(0.26 * s, 0.48 * s + org.pose.head_pitch * 0.25 * s, 0.0);
            push_tapered_tube(out, body_c + yaw_q * Vec3::new(0.18 * s, 0.10 * s, 0.0), head_c, 0.048 * s, 0.036 * s, 5, [0.86, 0.88, 0.90, 1.0], 0.48, 0.0, stratum);
            push_tapered_tube(out, head_c, head_c + head_q * Vec3::new(0.26 * s, -0.04 * s, 0.0), 0.026 * s, 0.006 * s, 4, [0.94, 0.74, 0.18, 1.0], 0.35, 0.0, stratum);
        }

        Species::BeltedKingfisher => {
            let slate_blue = [0.24, 0.44, 0.68, 1.0];
            push_ellipsoid(out, org.pos, Vec3::new(0.14 * s, 0.09 * s, 0.08 * s), yaw_q, slate_blue, 0.45, 0.0, stratum);
            let hd = org.pos + yaw_q * Vec3::new(0.11 * s, 0.08 * s, 0.0);
            push_ellipsoid(out, hd, Vec3::new(0.085 * s, 0.075 * s, 0.065 * s), head_q, slate_blue, 0.45, 0.0, stratum);
            push_tapered_tube(out, hd, hd + head_q * Vec3::new(0.14 * s, 0.0, 0.0), 0.020 * s, 0.004 * s, 4, [0.15, 0.16, 0.18, 1.0], 0.35, 0.0, stratum);
        }

        Species::NorthernPike | Species::RainbowTrout | Species::LargemouthBass | Species::PumpkinseedSunfish | Species::GoldenShiner => {
            let (dorsal, lateral, len, h_ratio) = match org.species {
                Species::NorthernPike => ([0.22, 0.36, 0.18, 1.0], [0.58, 0.66, 0.34, 1.0], 1.15 * s, 0.18),
                Species::RainbowTrout => ([0.24, 0.42, 0.30, 1.0], [0.92, 0.48, 0.58, 1.0], 0.56 * s, 0.26),
                Species::LargemouthBass => ([0.20, 0.34, 0.18, 1.0], [0.68, 0.74, 0.56, 1.0], 0.52 * s, 0.32),
                Species::PumpkinseedSunfish => ([0.16, 0.52, 0.48, 1.0], [0.94, 0.62, 0.18, 1.0], 0.24 * s, 0.52),
                _ => ([0.58, 0.64, 0.42, 1.0], [0.92, 0.86, 0.54, 1.0], 0.16 * s, 0.28),
            };
            let wag = org.pose.spine_phase.sin() * 0.07 * len;
            let st = [
                SpineStation { pos: org.pos + yaw_q * Vec3::new(-len * 0.5, 0.0, wag), half_width: len * 0.05, dorsal_height: len * h_ratio * 0.35, ventral_depth: len * h_ratio * 0.32, dorsal_rgba: dorsal, ventral_rgba: lateral },
                SpineStation { pos: org.pos, half_width: len * 0.12, dorsal_height: len * h_ratio, ventral_depth: len * h_ratio * 0.85, dorsal_rgba: dorsal, ventral_rgba: lateral },
                SpineStation { pos: org.pos + yaw_q * Vec3::new(len * 0.48, 0.0, 0.0), half_width: len * 0.08, dorsal_height: len * h_ratio * 0.55, ventral_depth: len * h_ratio * 0.45, dorsal_rgba: dorsal, ventral_rgba: lateral },
            ];
            push_lofted_spine(out, &st, 8, 0.32, 0.0, stratum);
        }

        Species::WesternPaintedTurtle => {
            push_ellipsoid(out, org.pos + Vec3::new(0.0, 0.04 * s, 0.0), Vec3::new(0.22 * s, 0.09 * s, 0.18 * s), yaw_q, [0.18, 0.28, 0.16, 1.0], 0.40, 0.0, stratum);
            push_ellipsoid(out, org.pos - Vec3::new(0.0, 0.02 * s, 0.0), Vec3::new(0.20 * s, 0.04 * s, 0.16 * s), yaw_q, [0.90, 0.38, 0.16, 1.0], 0.48, 0.0, stratum);
            push_ellipsoid(out, org.pos + yaw_q * Vec3::new(0.25 * s, 0.03 * s, 0.0), Vec3::splat(0.055 * s), head_q, [0.22, 0.38, 0.20, 1.0], 0.45, 0.0, stratum);
        }

        Species::AmericanBullfrog => {
            push_ellipsoid(out, org.pos, Vec3::new(0.14 * s, 0.09 * s, 0.12 * s), yaw_q, [0.26, 0.52, 0.20, 1.0], 0.42, 0.0, stratum);
        }

        Species::BlueDasherDragonfly => {
            push_tapered_tube(out, org.pos - yaw_q * Vec3::new(0.10 * s, 0.0, 0.0), org.pos + yaw_q * Vec3::new(0.08 * s, 0.0, 0.0), 0.012 * s, 0.016 * s, 4, [0.24, 0.62, 0.94, 1.0], 0.35, 0.0, stratum);
            for side in [-1.0_f32, 1.0] {
                push_ellipsoid(out, org.pos + yaw_q * Vec3::new(0.02 * s, 0.01 * s, side * 0.11 * s), Vec3::new(0.035 * s, 0.004 * s, 0.11 * s), yaw_q, [0.85, 0.94, 0.98, 0.72], 0.2, 0.0, stratum);
            }
        }

        Species::SignalCrayfish => {
            let rust_brown = [0.56, 0.24, 0.16, 1.0];
            push_ellipsoid(out, org.pos + Vec3::new(0.0, 0.04 * s, 0.0), Vec3::new(0.11 * s, 0.045 * s, 0.055 * s), yaw_q, rust_brown, 0.52, 0.0, stratum);
            for side in [-1.0_f32, 1.0] {
                push_tapered_tube(out, org.pos + yaw_q * Vec3::new(0.08 * s, 0.04 * s, side * 0.04 * s), org.pos + yaw_q * Vec3::new(0.19 * s, 0.04 * s, side * 0.08 * s), 0.022 * s, 0.032 * s, 5, rust_brown, 0.48, 0.0, stratum);
            }
        }

        // ================= INDO-PACIFIC CORAL REEF SPECIES =================
        Species::BlacktipReefShark => {
            let bronze = [0.56, 0.52, 0.44, 1.0];
            let white_belly = [0.95, 0.96, 0.96, 1.0];
            let black_tip = [0.07, 0.07, 0.09, 1.0];
            let mut stations = Vec::with_capacity(9);
            for i in 0..9 {
                let u = i as f32 / 8.0;
                let x = (-0.98 + 1.92 * u) * s;
                let wave = ((1.0 - u) * PI * 2.1 + org.pose.spine_phase).sin() * (1.0 - u) * 0.15 * s;
                let rad = (0.19 * (u * PI).sin().powf(0.62)).max(0.032) * s;
                stations.push(SpineStation {
                    pos: org.pos + yaw_q * Vec3::new(x, 0.0, wave),
                    half_width: rad,
                    dorsal_height: rad * 0.96,
                    ventral_depth: rad * 0.78,
                    dorsal_rgba: bronze,
                    ventral_rgba: white_belly,
                });
            }
            push_lofted_spine(out, &stations, 8, 0.30, 0.0, stratum);
            let d1 = org.pos + yaw_q * Vec3::new(0.10 * s, 0.15 * s, 0.0);
            out.push(Tri3D::new(d1 + yaw_q * Vec3::new(0.16 * s, 0.0, 0.0), d1 + yaw_q * Vec3::new(-0.08 * s, 0.24 * s, 0.0), d1 + yaw_q * Vec3::new(-0.14 * s, 0.0, 0.0), bronze, 0.32, 0.0, stratum));
            out.push(Tri3D::new(d1 + yaw_q * Vec3::new(-0.02 * s, 0.18 * s, 0.0), d1 + yaw_q * Vec3::new(-0.11 * s, 0.33 * s, 0.0), d1 + yaw_q * Vec3::new(-0.13 * s, 0.18 * s, 0.0), black_tip, 0.30, 0.0, stratum));
            let tail_p = stations[0].pos;
            push_tapered_tube(out, tail_p, tail_p + yaw_q * Vec3::new(-0.38 * s, 0.28 * s, 0.0), 0.055 * s, 0.015 * s, 4, black_tip, 0.32, 0.0, stratum);
        }

        Species::GiantMorayEel => {
            let mottled = [0.46, 0.33, 0.22, 1.0];
            let mut prev = org.pos;
            for i in 1..=8 {
                let f = i as f32 / 8.0;
                let next = org.pos + yaw_q * Vec3::new(-f * 1.45 * s, (f * 3.0 + org.pose.spine_phase).cos() * 0.09 * s, (f * 4.0 + org.pose.spine_phase).sin() * 0.13 * s);
                push_tapered_tube(out, prev, next, (0.14 - 0.08 * f) * s, (0.13 - 0.08 * f) * s, 6, mottled, 0.44, 0.0, stratum);
                prev = next;
            }
            push_ellipsoid(out, org.pos + yaw_q * Vec3::new(0.14 * s, 0.02 * s, 0.0), Vec3::new(0.19 * s, 0.14 * s, 0.11 * s), yaw_q, mottled, 0.44, 0.0, stratum);
        }

        Species::RedLionfish => {
            let maroon = [0.76, 0.16, 0.18, 1.0];
            let ivory = [0.96, 0.93, 0.88, 1.0];
            push_ellipsoid(out, org.pos, Vec3::new(0.22 * s, 0.14 * s, 0.11 * s), yaw_q, maroon, 0.42, 0.0, stratum);
            for sp in 0..9 {
                let f = sp as f32 / 8.0 - 0.5;
                let base = org.pos + yaw_q * Vec3::new(f * 0.30 * s, 0.08 * s, 0.0);
                let tip = base + yaw_q * Vec3::new(f * 0.16 * s, 0.32 * s, 0.0);
                push_tapered_tube(out, base, tip, 0.012 * s, 0.003 * s, 4, if sp % 2 == 0 { ivory } else { maroon }, 0.38, 0.0, stratum);
            }
            for side in [-1.0_f32, 1.0] {
                for r in 0..5 {
                    let a = r as f32 * 0.25 - 0.5;
                    let tip = org.pos + yaw_q * Vec3::new(a * 0.22 * s, -0.06 * s, side * 0.34 * s);
                    push_tapered_tube(out, org.pos, tip, 0.012 * s, 0.003 * s, 4, ivory, 0.38, 0.0, stratum);
                }
            }
        }

        Species::GreenSeaTurtle => {
            let carapace = [0.44, 0.31, 0.16, 1.0];
            let plastron = [0.88, 0.82, 0.64, 1.0];
            let skin = [0.38, 0.46, 0.30, 1.0];
            push_ellipsoid(out, org.pos + Vec3::new(0.0, 0.05 * s, 0.0), Vec3::new(0.46 * s, 0.18 * s, 0.38 * s), yaw_q, carapace, 0.42, 0.0, stratum);
            push_ellipsoid(out, org.pos - Vec3::new(0.0, 0.04 * s, 0.0), Vec3::new(0.42 * s, 0.09 * s, 0.34 * s), yaw_q, plastron, 0.55, 0.0, stratum);
            let head_c = org.pos + yaw_q * Vec3::new(0.54 * s, 0.04 * s, 0.0);
            push_ellipsoid(out, head_c, Vec3::new(0.13 * s, 0.10 * s, 0.10 * s), head_q, skin, 0.50, 0.0, stratum);
            let stroke = org.pose.wing_phase_amp[0].sin() * 0.55;
            for side in [-1.0_f32, 1.0] {
                let f_rot = yaw_q * Quat::from_rotation_x(side * stroke);
                let sh = org.pos + yaw_q * Vec3::new(0.26 * s, 0.0, side * 0.30 * s);
                let tip = sh + f_rot * Vec3::new(-0.12 * s, 0.0, side * 0.54 * s);
                push_tapered_tube(out, sh, tip, 0.085 * s, 0.025 * s, 4, skin, 0.48, 0.0, stratum);
            }
        }

        Species::BluespottedStingray => {
            let olive_gold = [0.68, 0.58, 0.24, 1.0];
            let blue_spot = [0.10, 0.58, 0.98, 1.0];
            push_ellipsoid(out, org.pos, Vec3::new(0.38 * s, 0.055 * s, 0.42 * s), yaw_q, olive_gold, 0.38, 0.0, stratum);
            for sp in -2..=2 {
                push_oriented_box(out, org.pos + yaw_q * Vec3::new(sp as f32 * 0.11 * s, 0.05 * s, (sp % 2) as f32 * 0.14 * s), Vec3::splat(0.022 * s), yaw_q, blue_spot, 0.3, 0.08, stratum);
            }
            push_tapered_tube(out, org.pos + yaw_q * Vec3::new(-0.34 * s, 0.0, 0.0), org.pos + yaw_q * Vec3::new(-0.92 * s, 0.02 * s, org.pose.spine_phase.sin() * 0.10 * s), 0.032 * s, 0.008 * s, 4, olive_gold, 0.4, 0.0, stratum);
        }

        Species::OcellarisClownfish | Species::MoorishIdol | Species::EmperorAngelfish | Species::SteepheadParrotfish | Species::BlueGreenChromis => {
            let (c_primary, c_band, len) = match org.species {
                Species::OcellarisClownfish => ([0.98, 0.42, 0.06, 1.0], [0.97, 0.97, 0.97, 1.0], 0.22 * s),
                Species::MoorishIdol => ([0.96, 0.94, 0.82, 1.0], [0.10, 0.10, 0.13, 1.0], 0.30 * s),
                Species::EmperorAngelfish => ([0.14, 0.36, 0.88, 1.0], [0.95, 0.80, 0.16, 1.0], 0.36 * s),
                Species::SteepheadParrotfish => ([0.12, 0.78, 0.74, 1.0], [0.92, 0.44, 0.56, 1.0], 0.50 * s),
                _ => ([0.14, 0.88, 0.78, 1.0], [0.18, 0.94, 0.86, 1.0], 0.18 * s),
            };
            let wag = org.pose.spine_phase.sin() * 0.06 * len;
            let st = [
                SpineStation { pos: org.pos + yaw_q * Vec3::new(-len * 0.5, 0.0, wag), half_width: len * 0.08, dorsal_height: len * 0.18, ventral_depth: len * 0.16, dorsal_rgba: c_primary, ventral_rgba: c_primary },
                SpineStation { pos: org.pos, half_width: len * 0.18, dorsal_height: len * 0.42, ventral_depth: len * 0.38, dorsal_rgba: c_band, ventral_rgba: c_band },
                SpineStation { pos: org.pos + yaw_q * Vec3::new(len * 0.48, 0.0, 0.0), half_width: len * 0.13, dorsal_height: len * 0.26, ventral_depth: len * 0.24, dorsal_rgba: c_primary, ventral_rgba: c_primary },
            ];
            push_lofted_spine(out, &st, 8, 0.32, 0.0, stratum);
            if org.species == Species::MoorishIdol {
                push_tapered_tube(out, org.pos + yaw_q * Vec3::new(0.0, len * 0.40, 0.0), org.pos + yaw_q * Vec3::new(-len * 0.85, len * 0.75, wag), 0.015 * s, 0.003 * s, 4, [0.98, 0.98, 0.98, 1.0], 0.3, 0.0, stratum);
            }
        }

        Species::GiantTridacnaClam => {
            let shell = [0.84, 0.80, 0.72, 1.0];
            let gape = (1.0 - 0.65 * org.pose.crouch) * 0.14 * s;
            for side in [-1.0_f32, 1.0] {
                push_ellipsoid(out, org.pos + Vec3::new(0.0, 0.16 * s, side * gape), Vec3::new(0.32 * s, 0.20 * s, 0.07 * s), Quat::IDENTITY, shell, 0.78, 0.0, stratum);
            }
            push_ellipsoid(out, org.pos + Vec3::new(0.0, 0.18 * s, 0.0), Vec3::new(0.28 * s, 0.09 * s, gape * 0.9), Quat::IDENTITY, [0.12, 0.56, 0.96, 1.0], 0.35, 0.45 * night_factor, stratum);
        }

        Species::PacificCleanerShrimp => {
            push_tapered_tube(out, org.pos - yaw_q * Vec3::new(0.08 * s, 0.0, 0.0), org.pos + yaw_q * Vec3::new(0.08 * s, 0.02 * s, 0.0), 0.022 * s, 0.018 * s, 5, [0.94, 0.18, 0.18, 1.0], 0.38, 0.0, stratum);
            for side in [-1.0_f32, 1.0] {
                push_tapered_tube(out, org.pos + yaw_q * Vec3::new(0.08 * s, 0.02 * s, 0.0), org.pos + yaw_q * Vec3::new(0.24 * s, 0.10 * s, side * 0.09 * s), 0.004 * s, 0.002 * s, 4, [0.98, 0.98, 0.98, 1.0], 0.3, 0.0, stratum);
            }
        }

        Species::BlueLinckiaStar => {
            for arm in 0..5 {
                let a = (arm as f32 / 5.0) * 2.0 * PI;
                push_tapered_tube(out, org.pos, org.pos + Vec3::new(a.cos() * 0.19 * s, 0.015 * s, a.sin() * 0.19 * s), 0.032 * s, 0.018 * s, 5, [0.12, 0.42, 0.94, 1.0], 0.48, 0.0, stratum);
            }
        }

        Species::ReefHermitCrab => {
            let p = org.pos + Vec3::new(0.0, 0.05 * s, 0.0);
            push_ellipsoid(out, p + yaw_q * Vec3::new(-0.03 * s, 0.03 * s, 0.0), Vec3::new(0.075 * s, 0.065 * s, 0.065 * s), yaw_q, [0.86, 0.76, 0.62, 1.0], 0.62, 0.0, stratum);
            push_ellipsoid(out, p + yaw_q * Vec3::new(0.05 * s, 0.0, 0.0), Vec3::splat(0.035 * s), yaw_q, [0.16, 0.52, 0.96, 1.0], 0.42, 0.0, stratum);
        }

        // ================= PACIFIC GIANT KELP FOREST SPECIES =================
        Species::SeaOtter => {
            // Enhydra lutris: dense chocolate-brown pelage, pale grizzled blond head & throat,
            // webbed paddle hind flippers, flattened tail, and chest anvil rock + purple sea urchin when supine!
            let supine = matches!(
                org.state,
                BehaviorState::OtterSurfaceRaft { .. } | BehaviorState::OtterChestStoneTap { .. }
            );
            let fur_dark = [0.28, 0.19, 0.13, 1.0];
            let head_blond = [0.78, 0.71, 0.58, 1.0];
            let rot = if supine {
                yaw_q * Quat::from_rotation_x(PI)
            } else {
                yaw_q
            };

            let stations = [
                SpineStation {
                    pos: org.pos + yaw_q * Vec3::new(-0.55 * s, -0.04 * s, 0.0),
                    half_width: 0.07 * s,
                    dorsal_height: 0.05 * s,
                    ventral_depth: 0.05 * s,
                    dorsal_rgba: fur_dark,
                    ventral_rgba: fur_dark,
                },
                SpineStation {
                    pos: org.pos + yaw_q * Vec3::new(-0.24 * s, 0.0, 0.0),
                    half_width: 0.16 * s,
                    dorsal_height: 0.15 * s,
                    ventral_depth: 0.16 * s,
                    dorsal_rgba: fur_dark,
                    ventral_rgba: fur_dark,
                },
                SpineStation {
                    pos: org.pos + yaw_q * Vec3::new(0.18 * s, 0.02 * s, 0.0),
                    half_width: 0.17 * s,
                    dorsal_height: 0.16 * s,
                    ventral_depth: 0.16 * s,
                    dorsal_rgba: fur_dark,
                    ventral_rgba: [0.42, 0.33, 0.24, 1.0],
                },
                SpineStation {
                    pos: org.pos + yaw_q * Vec3::new(0.42 * s, 0.06 * s, 0.0),
                    half_width: 0.13 * s,
                    dorsal_height: 0.13 * s,
                    ventral_depth: 0.13 * s,
                    dorsal_rgba: head_blond,
                    ventral_rgba: head_blond,
                },
            ];
            push_lofted_spine(out, &stations, 8, 0.72, 0.0, stratum);

            let head_c = org.pos + yaw_q * Vec3::new(0.50 * s, if supine { 0.14 * s } else { 0.08 * s }, 0.0);
            push_ellipsoid(out, head_c, Vec3::new(0.14 * s, 0.13 * s, 0.135 * s), head_q, head_blond, 0.68, 0.0, stratum);

            // Webbed hind flippers
            for side in [-1.0_f32, 1.0] {
                let fl = org.pos + yaw_q * Vec3::new(-0.34 * s, 0.06 * s, side * 0.17 * s);
                push_oriented_box(out, fl, Vec3::new(0.10 * s, 0.025 * s, 0.075 * s), rot, fur_dark, 0.65, 0.0, stratum);
            }

            if org.pose.carrying_item {
                // Basalt anvil stone + purple sea urchin on chest!
                let chest_up = if supine { 0.19 * s } else { -0.18 * s };
                let stone_p = org.pos + yaw_q * Vec3::new(0.16 * s, chest_up, 0.0);
                push_ellipsoid(out, stone_p, Vec3::new(0.085 * s, 0.04 * s, 0.075 * s), Quat::IDENTITY, [0.35, 0.36, 0.39, 1.0], 0.8, 0.0, TrophicStratum::Decomposers);
                let tap_lift = (org.pose.limb_phase.sin().abs()) * 0.08 * s;
                push_ellipsoid(
                    out,
                    stone_p + Vec3::new(0.0, 0.06 * s + tap_lift, 0.0),
                    Vec3::splat(0.065 * s),
                    Quat::IDENTITY,
                    [0.48, 0.16, 0.68, 1.0],
                    0.5,
                    0.0,
                    TrophicStratum::PrimaryConsumers,
                );
            }
        }

        Species::PacificHarborSeal => {
            // Phoca vitulina: sleek silver-slate spotted fusiform body, earless rounded head,
            // pectoral foreflippers & alternating bilobed pelvic hind flippers!
            let silver_coat = [0.54, 0.58, 0.62, 1.0];
            let pale_belly = [0.76, 0.79, 0.81, 1.0];
            let sw = org.pose.spine_phase.sin() * 0.09 * s;
            let stations = [
                SpineStation {
                    pos: org.pos + yaw_q * Vec3::new(-0.74 * s, 0.0, sw * 1.4),
                    half_width: 0.08 * s,
                    dorsal_height: 0.09 * s,
                    ventral_depth: 0.08 * s,
                    dorsal_rgba: silver_coat,
                    ventral_rgba: pale_belly,
                },
                SpineStation {
                    pos: org.pos + yaw_q * Vec3::new(-0.32 * s, 0.0, sw * 0.5),
                    half_width: 0.22 * s,
                    dorsal_height: 0.22 * s,
                    ventral_depth: 0.21 * s,
                    dorsal_rgba: silver_coat,
                    ventral_rgba: pale_belly,
                },
                SpineStation {
                    pos: org.pos + yaw_q * Vec3::new(0.16 * s, 0.02 * s, 0.0),
                    half_width: 0.25 * s,
                    dorsal_height: 0.24 * s,
                    ventral_depth: 0.24 * s,
                    dorsal_rgba: silver_coat,
                    ventral_rgba: pale_belly,
                },
                SpineStation {
                    pos: org.pos + yaw_q * Vec3::new(0.54 * s, 0.05 * s, 0.0),
                    half_width: 0.15 * s,
                    dorsal_height: 0.16 * s,
                    ventral_depth: 0.15 * s,
                    dorsal_rgba: silver_coat,
                    ventral_rgba: pale_belly,
                },
            ];
            push_lofted_spine(out, &stations, 8, 0.35, 0.0, stratum);

            let head_c = org.pos + yaw_q * Vec3::new(0.68 * s, 0.08 * s, 0.0);
            push_ellipsoid(out, head_c, Vec3::new(0.18 * s, 0.15 * s, 0.15 * s), head_q, silver_coat, 0.35, 0.0, stratum);

            for side in [-1.0_f32, 1.0] {
                let pec = org.pos + yaw_q * Vec3::new(0.20 * s, -0.14 * s, side * 0.25 * s);
                push_oriented_box(out, pec, Vec3::new(0.12 * s, 0.022 * s, 0.08 * s), yaw_q, silver_coat, 0.38, 0.0, stratum);
                let hind = org.pos + yaw_q * Vec3::new(-0.84 * s, side * 0.05 * s, sw * 1.6 + side * 0.08 * s);
                push_oriented_box(out, hind, Vec3::new(0.14 * s, 0.08 * s, 0.022 * s), yaw_q, silver_coat, 0.38, 0.0, stratum);
            }
        }

        Species::LeopardShark => {
            // Triakis semifasciata: silver-bronze houndshark with dark dorsal saddlebars,
            // two triangular dorsal fins, swept pectoral fins, and asymmetric heterocercal caudal fin!
            let bronze = [0.58, 0.53, 0.44, 1.0];
            let saddle = [0.20, 0.17, 0.14, 1.0];
            let white_belly = [0.90, 0.90, 0.88, 1.0];
            let mut stations = Vec::with_capacity(9);
            for i in 0..9 {
                let u = i as f32 / 8.0; // 0 = caudal peduncle, 1 = snout
                let x = (-0.95 + 1.85 * u) * s;
                let wave = ((1.0 - u) * PI * 2.2 + org.pose.spine_phase).sin() * (1.0 - u) * 0.16 * s;
                let rad = (0.18 * (u * PI).sin().powf(0.65)).max(0.032) * s;
                let d_col = if i % 2 == 0 { saddle } else { bronze };
                stations.push(SpineStation {
                    pos: org.pos + yaw_q * Vec3::new(x, 0.0, wave),
                    half_width: rad,
                    dorsal_height: rad * 0.95,
                    ventral_depth: rad * 0.72,
                    dorsal_rgba: d_col,
                    ventral_rgba: white_belly,
                });
            }
            push_lofted_spine(out, &stations, 8, 0.34, 0.0, stratum);

            // First & Second Dorsal Fins + Heterocercal Upper Caudal Lobe + Swept Pectorals!
            let d1_base = org.pos + yaw_q * Vec3::new(0.12 * s, 0.14 * s, 0.0);
            out.push(Tri3D::new(
                d1_base + yaw_q * Vec3::new(0.14 * s, 0.0, 0.0),
                d1_base + yaw_q * Vec3::new(-0.12 * s, 0.28 * s, 0.0),
                d1_base + yaw_q * Vec3::new(-0.14 * s, 0.0, 0.0),
                bronze,
                0.36,
                0.0,
                stratum,
            ));
            let d2_base = org.pos + yaw_q * Vec3::new(-0.46 * s, 0.09 * s, 0.0);
            out.push(Tri3D::new(
                d2_base + yaw_q * Vec3::new(0.09 * s, 0.0, 0.0),
                d2_base + yaw_q * Vec3::new(-0.08 * s, 0.18 * s, 0.0),
                d2_base + yaw_q * Vec3::new(-0.10 * s, 0.0, 0.0),
                saddle,
                0.36,
                0.0,
                stratum,
            ));
            let tail_p = stations[0].pos;
            push_tapered_tube(
                out,
                tail_p,
                tail_p + yaw_q * Vec3::new(-0.36 * s, 0.24 * s, 0.0),
                0.055 * s,
                0.015 * s,
                4,
                bronze,
                0.36,
                0.0,
                stratum,
            );
            for side in [-1.0_f32, 1.0] {
                out.push(Tri3D::new(
                    org.pos + yaw_q * Vec3::new(0.32 * s, -0.06 * s, side * 0.12 * s),
                    org.pos + yaw_q * Vec3::new(0.06 * s, -0.10 * s, side * 0.44 * s),
                    org.pos + yaw_q * Vec3::new(0.02 * s, -0.06 * s, side * 0.12 * s),
                    bronze,
                    0.36,
                    0.0,
                    stratum,
                ));
            }
        }

        Species::GiantPacificOctopus => {
            // Enteroctopus dofleini: bulbous papillate mantle + siphon + 8 sinusoidal sucker-lined tentacles!
            let c_shift = org.pose.chromatophore.clamp(0.0, 1.0);
            let cryptic = Vec3::new(0.38, 0.29, 0.25);
            let crimson = Vec3::new(0.78, 0.20, 0.16);
            let rgb = cryptic.lerp(crimson, c_shift);
            let skin = [rgb.x, rgb.y, rgb.z, 1.0];
            let sucker_white = [0.93, 0.88, 0.84, 1.0];

            // Bulbous posterior mantle + head with raised ocular ridges
            let mantle = org.pos + yaw_q * Vec3::new(-0.22 * s, 0.24 * s, 0.0);
            push_ellipsoid(out, mantle, Vec3::new(0.28 * s, 0.20 * s, 0.21 * s), yaw_q * Quat::from_rotation_z(-0.35), skin, 0.48, 0.0, stratum);
            push_ellipsoid(out, org.pos + Vec3::new(0.0, 0.14 * s, 0.0), Vec3::splat(0.15 * s), yaw_q, skin, 0.48, 0.0, stratum);

            // 8 articulated tentacles radiating & undulating!
            for arm_i in 0..8 {
                let base_ang = (arm_i as f32 / 8.0) * 2.0 * PI;
                let mut prev = org.pos + Vec3::new(base_ang.cos() * 0.10 * s, 0.08 * s, base_ang.sin() * 0.10 * s);
                for seg in 1..=5 {
                    let f = seg as f32 / 5.0;
                    let wave = (org.pose.limb_phase + arm_i as f32 * 0.8 + f * 3.0).sin() * 0.09 * s;
                    let reach = (0.14 + 0.58 * f) * s;
                    let next = org.pos
                        + Vec3::new(
                            base_ang.cos() * reach + wave,
                            (0.08 * (1.0 - f) + 0.04 * (f * PI).sin()) * s,
                            base_ang.sin() * reach,
                        );
                    let r0 = (0.046 * (1.0 - 0.72 * ((seg - 1) as f32 / 5.0))) * s;
                    let r1 = (0.046 * (1.0 - 0.72 * f)) * s;
                    push_tapered_tube(out, prev, next, r0, r1, 5, skin, 0.50, 0.0, stratum);
                    if seg <= 3 {
                        push_oriented_box(out, (prev + next) * 0.5 - Vec3::new(0.0, r0 * 0.7, 0.0), Vec3::splat(0.014 * s), Quat::IDENTITY, sucker_white, 0.4, 0.0, stratum);
                    }
                    prev = next;
                }
            }
        }

        Species::PacificWolfEel => {
            // Anarrhichthys ocellatus: heavy crushing-jaw head + sinuous grey-blue ribbon body with dark ocelli
            let slate = [0.36, 0.40, 0.46, 1.0];
            let mut prev = org.pos;
            for i in 1..=8 {
                let f = i as f32 / 8.0;
                let next = org.pos
                    + yaw_q
                        * Vec3::new(
                            -f * 1.35 * s,
                            (f * 3.0 + org.pose.spine_phase).cos() * 0.08 * s,
                            (f * 4.0 + org.pose.spine_phase).sin() * 0.12 * s,
                        );
                push_tapered_tube(out, prev, next, (0.13 - 0.08 * f) * s, (0.12 - 0.08 * f) * s, 6, slate, 0.46, 0.0, stratum);
                prev = next;
            }
            push_ellipsoid(out, org.pos + yaw_q * Vec3::new(0.12 * s, 0.02 * s, 0.0), Vec3::new(0.18 * s, 0.15 * s, 0.14 * s), yaw_q, slate, 0.46, 0.0, stratum);
        }

        Species::CaliforniaSheephead | Species::GaribaldiDamselfish | Species::BlacksmithDamselfish => {
            let (head_col, mid_col, tail_col, body_len) = match org.species {
                Species::CaliforniaSheephead => (
                    [0.12, 0.12, 0.15, 1.0], // Jet-black head
                    [0.88, 0.18, 0.24, 1.0], // Vivid crimson saddle!
                    [0.12, 0.12, 0.15, 1.0], // Jet-black caudal peduncle
                    0.56 * s,
                ),
                Species::GaribaldiDamselfish => (
                    [0.98, 0.38, 0.05, 1.0], // Brilliant electric orange!
                    [0.98, 0.42, 0.06, 1.0],
                    [0.96, 0.36, 0.05, 1.0],
                    0.34 * s,
                ),
                _ => (
                    [0.24, 0.34, 0.48, 1.0], // Dusky silver-blue Chromis punctipinnis
                    [0.32, 0.42, 0.56, 1.0],
                    [0.20, 0.28, 0.40, 1.0],
                    0.22 * s,
                ),
            };
            let wag = org.pose.spine_phase.sin() * 0.05 * body_len;
            let stations = [
                SpineStation {
                    pos: org.pos + yaw_q * Vec3::new(-body_len * 0.5, 0.0, wag),
                    half_width: body_len * 0.08,
                    dorsal_height: body_len * 0.16,
                    ventral_depth: body_len * 0.14,
                    dorsal_rgba: tail_col,
                    ventral_rgba: tail_col,
                },
                SpineStation {
                    pos: org.pos,
                    half_width: body_len * 0.18,
                    dorsal_height: body_len * 0.36,
                    ventral_depth: body_len * 0.32,
                    dorsal_rgba: mid_col,
                    ventral_rgba: mid_col,
                },
                SpineStation {
                    pos: org.pos + yaw_q * Vec3::new(body_len * 0.48, 0.0, 0.0),
                    half_width: body_len * 0.12,
                    dorsal_height: body_len * 0.24,
                    ventral_depth: body_len * 0.22,
                    dorsal_rgba: head_col,
                    ventral_rgba: if org.species == Species::CaliforniaSheephead {
                        [0.95, 0.94, 0.92, 1.0] // White sheephead chin!
                    } else {
                        head_col
                    },
                },
            ];
            push_lofted_spine(out, &stations, 8, 0.35, 0.0, stratum);
            // Forked/heart caudal fin
            let t0 = org.pos + yaw_q * Vec3::new(-body_len * 0.48, 0.0, wag);
            out.push(Tri3D::new(
                t0,
                t0 + yaw_q * Vec3::new(-body_len * 0.28, body_len * 0.24, wag * 1.5),
                t0 + yaw_q * Vec3::new(-body_len * 0.28, -body_len * 0.24, wag * 1.5),
                tail_col,
                0.38,
                0.0,
                stratum,
            ));
        }

        Species::PurpleSeaUrchin => {
            let purple = [0.46, 0.14, 0.66, 1.0];
            push_ellipsoid(out, org.pos, Vec3::new(0.13 * s, 0.09 * s, 0.13 * s), Quat::IDENTITY, purple, 0.55, 0.0, stratum);
            for k in 0..12 {
                let a = (k as f32 / 12.0) * 2.0 * PI;
                let elev: f32 = if k % 2 == 0 { 0.35 } else { 0.85 };
                let dir = Vec3::new(a.cos() * elev.cos(), elev.sin(), a.sin() * elev.cos());
                push_tapered_tube(out, org.pos, org.pos + dir * 0.22 * s, 0.012 * s, 0.003 * s, 4, purple, 0.45, 0.0, stratum);
            }
        }

        Species::RedAbalone => {
            push_ellipsoid(
                out,
                org.pos + Vec3::new(0.0, 0.04 * s, 0.0),
                Vec3::new(0.18 * s, 0.065 * s, 0.13 * s),
                yaw_q,
                [0.68, 0.24, 0.18, 1.0],
                0.58,
                0.0,
                stratum,
            );
        }

        Species::BatSeaStar => {
            let vermilion = [0.88, 0.28, 0.14, 1.0];
            for arm in 0..5 {
                let a0 = (arm as f32 / 5.0) * 2.0 * PI;
                let a1 = ((arm as f32 + 0.5) / 5.0) * 2.0 * PI;
                let tip = org.pos + Vec3::new(a0.cos() * 0.18 * s, 0.015 * s, a0.sin() * 0.18 * s);
                let web = org.pos + Vec3::new(a1.cos() * 0.10 * s, 0.02 * s, a1.sin() * 0.10 * s);
                out.push(Tri3D::new(org.pos + Vec3::new(0.0, 0.045 * s, 0.0), tip, web, vermilion, 0.62, 0.0, stratum));
            }
        }

        Species::CaliforniaSeaCucumber => {
            let rust = [0.66, 0.28, 0.14, 1.0];
            push_tapered_tube(
                out,
                org.pos - yaw_q * Vec3::new(0.18 * s, 0.0, 0.0),
                org.pos + yaw_q * Vec3::new(0.18 * s, 0.0, 0.0),
                0.055 * s,
                0.052 * s,
                6,
                rust,
                0.68,
                0.0,
                stratum,
            );
        }

        Species::NorthernKelpCrab => {
            let olive = [0.48, 0.44, 0.14, 1.0];
            let p = org.pos + Vec3::new(0.0, 0.05 * s, 0.0);
            push_ellipsoid(out, p, Vec3::new(0.08 * s, 0.035 * s, 0.075 * s), yaw_q, olive, 0.45, 0.0, stratum);
            for side in [-1.0_f32, 1.0] {
                let claw = p + yaw_q * Vec3::new(0.09 * s, 0.0, side * 0.07 * s);
                push_tapered_tube(out, p, claw, 0.014 * s, 0.018 * s, 4, olive, 0.45, 0.0, stratum);
                for l in -1..=1 {
                    let step = (org.pose.limb_phase + l as f32 + side).sin() * 0.02 * s;
                    let foot = org.pos + yaw_q * Vec3::new((l as f32 * 0.04 + step) * s, 0.0, side * 0.13 * s);
                    push_tapered_tube(out, p, foot, 0.008 * s, 0.004 * s, 4, olive, 0.48, 0.0, stratum);
                }
            }
        }

        // ================= NEOTROPICAL RAINFOREST SPECIES (v0.2.0 Anatomical) =================
        Species::BoaConstrictor => {
            let segs = 28_usize;
            let striking = matches!(org.state, BehaviorState::SnakeBallisticStrike { .. });
            let ventral_cream = [0.88, 0.83, 0.72, 1.0];
            let mut stations = Vec::with_capacity(segs);
            for i in 0..segs {
                let u = i as f32 / (segs - 1) as f32;
                let pt = if striking {
                    let wave = (u * PI * 3.5 + org.pose.spine_phase).sin() * 0.14;
                    org.pos + yaw_q * Vec3::new(-u * 2.35 * s, u * 0.42, wave)
                } else {
                    let coil_ang = u * PI * 4.2 + org.pose.spine_phase * 0.22;
                    let reach_down = (1.0 - u).powf(1.3) * 1.25 * s;
                    org.pos + Vec3::new((u - 0.48) * 1.95 * s + coil_ang.sin() * 0.28 * (1.0 - 0.3 * u), 0.20 - reach_down + (u * PI * 3.0).cos() * 0.22 * u, coil_ang.cos() * 0.30)
                };
                let girth = if u < 0.08 { 0.068 + 0.45 * u } else { 0.145 * (1.0 - ((u - 0.42) / 0.62).powi(2) * 0.78).max(0.18) } * s;
                let saddle = i % 3 == 0;
                let dorsal_col = if u > 0.72 {
                    if saddle { [0.64, 0.11, 0.09, 1.0] } else { [0.84, 0.68, 0.48, 1.0] }
                } else if saddle {
                    [0.24, 0.13, 0.07, 1.0]
                } else {
                    [0.72, 0.50, 0.29, 1.0]
                };
                stations.push(SpineStation { pos: pt, half_width: girth, dorsal_height: girth * 0.92, ventral_depth: girth * 0.58, dorsal_rgba: dorsal_col, ventral_rgba: ventral_cream });
            }
            push_lofted_spine(out, &stations, 8, 0.40, 0.0, stratum);
            let neck_pt = stations[0].pos;
            let skull_back = neck_pt + head_q * Vec3::new(0.05 * s, -0.02 * s, 0.0);
            let snout_tip = skull_back + head_q * Vec3::new(0.24 * s, -0.04 * s, 0.0);
            let head_stations = [
                SpineStation { pos: neck_pt, half_width: 0.065 * s, dorsal_height: 0.055 * s, ventral_depth: 0.045 * s, dorsal_rgba: [0.64, 0.43, 0.24, 1.0], ventral_rgba: ventral_cream },
                SpineStation { pos: skull_back, half_width: 0.125 * s, dorsal_height: 0.078 * s, ventral_depth: 0.055 * s, dorsal_rgba: [0.66, 0.45, 0.25, 1.0], ventral_rgba: ventral_cream },
                SpineStation { pos: snout_tip, half_width: 0.058 * s, dorsal_height: 0.048 * s, ventral_depth: 0.036 * s, dorsal_rgba: [0.58, 0.38, 0.20, 1.0], ventral_rgba: ventral_cream },
            ];
            push_lofted_spine(out, &head_stations, 8, 0.36, 0.0, stratum);
            if org.pose.tongue_ext > 0.08 {
                let fork_base = snout_tip + head_q * Vec3::new(0.15 * org.pose.tongue_ext * s, -0.02 * s, 0.0);
                let crimson = [0.86, 0.10, 0.16, 1.0];
                push_tapered_tube(out, snout_tip, fork_base, 0.011 * s, 0.009 * s, 4, crimson, 0.25, 0.08, stratum);
                for side in [-1.0_f32, 1.0] {
                    let tine = fork_base + head_q * Vec3::new(0.09 * org.pose.tongue_ext * s, -0.02 * s, side * 0.045 * s);
                    push_tapered_tube(out, fork_base, tine, 0.008 * s, 0.003 * s, 4, crimson, 0.25, 0.08, stratum);
                }
            }
        }

        Species::BlackPanther | Species::CanopyJaguar => {
            let is_melanistic = org.species == Species::BlackPanther;
            let dorsal_coat = if is_melanistic { [0.075, 0.078, 0.105, 1.0] } else { [0.85, 0.60, 0.20, 1.0] };
            let ventral_coat = if is_melanistic { [0.06, 0.06, 0.08, 1.0] } else { [0.93, 0.89, 0.78, 1.0] };
            let rosette_ring = if is_melanistic { [0.035, 0.035, 0.048, 1.0] } else { [0.14, 0.09, 0.05, 1.0] };
            let hip_h = (0.64 - 0.22 * org.pose.crouch) * s;
            let origin = org.pos + Vec3::new(0.0, hip_h, org.pose.spine_phase.sin() * 0.035);
            let scapular_rise = 0.055 * org.pose.crouch * s;
            let torso_stations = [
                SpineStation { pos: origin + yaw_q * Vec3::new(-0.56 * s, 0.02 * s, 0.0), half_width: 0.15 * s, dorsal_height: 0.16 * s, ventral_depth: 0.14 * s, dorsal_rgba: dorsal_coat, ventral_rgba: ventral_coat },
                SpineStation { pos: origin + yaw_q * Vec3::new(-0.32 * s, 0.05 * s, 0.0), half_width: 0.19 * s, dorsal_height: 0.20 * s, ventral_depth: 0.16 * s, dorsal_rgba: dorsal_coat, ventral_rgba: ventral_coat },
                SpineStation { pos: origin + yaw_q * Vec3::new(0.04 * s, 0.03 * s, 0.0), half_width: 0.22 * s, dorsal_height: 0.22 * s, ventral_depth: 0.23 * s, dorsal_rgba: dorsal_coat, ventral_rgba: ventral_coat },
                SpineStation { pos: origin + yaw_q * Vec3::new(0.36 * s, 0.06 * s + scapular_rise, 0.0), half_width: 0.23 * s, dorsal_height: 0.24 * s, ventral_depth: 0.22 * s, dorsal_rgba: dorsal_coat, ventral_rgba: ventral_coat },
                SpineStation { pos: origin + yaw_q * Vec3::new(0.58 * s, 0.10 * s, 0.0), half_width: 0.15 * s, dorsal_height: 0.16 * s, ventral_depth: 0.15 * s, dorsal_rgba: dorsal_coat, ventral_rgba: dorsal_coat },
            ];
            push_lofted_spine(out, &torso_stations, 8, if is_melanistic { 0.36 } else { 0.50 }, 0.0, stratum);
            for r_i in 0..5 {
                let rx = (-0.36 + r_i as f32 * 0.18) * s;
                for side in [-1.0_f32, 1.0] {
                    let center = origin + yaw_q * Vec3::new(rx, 0.04 * s, side * 0.215 * s);
                    push_oriented_box(out, center, Vec3::new(0.018 * s, 0.018 * s, 0.008 * s), yaw_q, rosette_ring, 0.55, 0.0, stratum);
                }
            }
            let head_c = origin + yaw_q * Vec3::new(0.60 * s, 0.12 * s - 0.06 * org.pose.crouch, 0.0) + head_q * Vec3::new(0.14 * s, 0.0, 0.0);
            push_ellipsoid(out, head_c, Vec3::new(0.19 * s, 0.155 * s, 0.175 * s), head_q, dorsal_coat, 0.44, 0.0, stratum);
            push_ellipsoid(out, head_c + head_q * Vec3::new(0.17 * s, -0.025 * s, 0.0), Vec3::new(0.115 * s, 0.075 * s, 0.105 * s), head_q, dorsal_coat, 0.46, 0.0, stratum);
            for side in [-1.0_f32, 1.0] {
                let eye_p = head_c + head_q * Vec3::new(0.135 * s, 0.035 * s, side * 0.092 * s);
                push_oriented_box(out, eye_p, Vec3::new(0.022 * s, 0.020 * s, 0.018 * s), head_q, [0.94, 0.80, 0.22, 1.0], 0.18, 0.35 + 0.85 * night_factor, stratum);
                let ear_p = head_c + head_q * Vec3::new(-0.05 * s, 0.155 * s, side * 0.125 * s);
                push_ellipsoid(out, ear_p, Vec3::new(0.038 * s, 0.058 * s, 0.042 * s), head_q, [0.08, 0.07, 0.08, 1.0], 0.6, 0.0, stratum);
            }
            let moving = org.vel.length_squared() > 0.02;
            for (side, ph_off) in [(1.0_f32, 0.0_f32), (-1.0, PI)] {
                let ph = org.pose.limb_phase + ph_off;
                let stride = if moving { ph.sin() * 0.24 * s } else { 0.04 * side * s };
                let lift = if moving { ph.cos().max(0.0) * 0.11 * s } else { 0.0 };
                let sh = origin + yaw_q * Vec3::new(0.36 * s, 0.04 * s, side * 0.16 * s);
                let el = sh + yaw_q * Vec3::new(-0.11 * s + stride * 0.35, -hip_h * 0.46, 0.0);
                let wr = sh + yaw_q * Vec3::new(0.04 * s + stride * 0.85, -hip_h * 0.84 + lift, 0.0);
                let pw = sh + yaw_q * Vec3::new(0.11 * s + stride, -hip_h + 0.03 * s + lift, 0.0);
                push_three_segment_limb(out, sh, el, wr, pw, [0.098 * s, 0.074 * s, 0.056 * s, 0.068 * s], dorsal_coat, dorsal_coat, 0.48, stratum);
            }
            for (side, ph_off) in [(1.0_f32, PI * 0.5), (-1.0, PI * 1.5)] {
                let ph = org.pose.limb_phase + ph_off;
                let stride = if moving { ph.sin() * 0.25 * s } else { -0.05 * side * s };
                let lift = if moving { ph.cos().max(0.0) * 0.11 * s } else { 0.0 };
                let hip = origin + yaw_q * Vec3::new(-0.42 * s, 0.04 * s, side * 0.15 * s);
                let stf = hip + yaw_q * Vec3::new(0.12 * s + stride * 0.4, -hip_h * 0.44, 0.0);
                let hck = hip + yaw_q * Vec3::new(-0.11 * s + stride * 0.75, -hip_h * 0.74 + lift, 0.0);
                let pw = hip + yaw_q * Vec3::new(-0.02 * s + stride, -hip_h + 0.03 * s + lift, 0.0);
                push_three_segment_limb(out, hip, stf, hck, pw, [0.108 * s, 0.080 * s, 0.055 * s, 0.066 * s], dorsal_coat, dorsal_coat, 0.48, stratum);
            }
            let mut tail_st = Vec::with_capacity(6);
            for ts in 0..=5 {
                let f = ts as f32 / 5.0;
                let col = if ts >= 4 { [0.08, 0.07, 0.08, 1.0] } else { dorsal_coat };
                let r = (0.052 - 0.016 * f) * s;
                tail_st.push(SpineStation {
                    pos: origin + yaw_q * Vec3::new((-0.56 - 0.74 * f) * s, (-0.14 * f + 0.16 * f * f) * s, ((f * 1.5).sin() * 0.16 + org.pose.tail_angle * f) * s),
                    half_width: r, dorsal_height: r, ventral_depth: r, dorsal_rgba: col, ventral_rgba: col,
                });
            }
            push_lofted_spine(out, &tail_st, 6, 0.48, 0.0, stratum);
        }

        Species::SquirrelMonkey => {
            let agouti_back = [0.56, 0.52, 0.34, 1.0];
            let cream_belly = [0.92, 0.89, 0.78, 1.0];
            let golden_distal = [0.92, 0.66, 0.16, 1.0];
            let mask_white = [0.95, 0.94, 0.91, 1.0];
            let black_muzzle = [0.11, 0.10, 0.11, 1.0];
            let swinging = matches!(org.state, BehaviorState::MonkeyVineSwing { .. });
            let torso_c = if swinging { org.pos - Vec3::new(0.0, 0.30 * s, 0.0) } else { org.pos + Vec3::new(0.0, 0.13 * s, 0.0) };
            let spine = [
                SpineStation { pos: torso_c + yaw_q * Vec3::new(-0.22 * s, -0.02 * s, 0.0), half_width: 0.095 * s, dorsal_height: 0.10 * s, ventral_depth: 0.09 * s, dorsal_rgba: agouti_back, ventral_rgba: cream_belly },
                SpineStation { pos: torso_c + yaw_q * Vec3::new(0.0, 0.04 * s, 0.0), half_width: 0.115 * s, dorsal_height: 0.125 * s, ventral_depth: 0.11 * s, dorsal_rgba: agouti_back, ventral_rgba: cream_belly },
                SpineStation { pos: torso_c + yaw_q * Vec3::new(0.21 * s, 0.04 * s, 0.0), half_width: 0.11 * s, dorsal_height: 0.115 * s, ventral_depth: 0.105 * s, dorsal_rgba: agouti_back, ventral_rgba: cream_belly },
            ];
            push_lofted_spine(out, &spine, 8, 0.62, 0.0, stratum);
            let head_c = torso_c + yaw_q * Vec3::new(0.25 * s, 0.15 * s, 0.0);
            push_ellipsoid(out, head_c + head_q * Vec3::new(-0.03 * s, 0.015 * s, 0.0), Vec3::new(0.145 * s, 0.115 * s, 0.108 * s), head_q, [0.36, 0.34, 0.28, 1.0], 0.62, 0.0, stratum);
            for side in [-1.0_f32, 1.0] {
                push_ellipsoid(out, head_c + head_q * Vec3::new(0.068 * s, 0.018 * s, side * 0.048 * s), Vec3::new(0.058 * s, 0.072 * s, 0.048 * s), head_q, mask_white, 0.55, 0.0, stratum);
                let sh = torso_c + yaw_q * Vec3::new(0.18 * s, 0.03 * s, side * 0.105 * s);
                let hand = if swinging { org.pos + Vec3::new(0.0, 0.05 * s, side * 0.05 * s) } else { torso_c + yaw_q * Vec3::new(0.17 * s, -0.23 * s, side * 0.09 * s) };
                push_tapered_tube(out, sh, hand, 0.042 * s, 0.030 * s, 5, golden_distal, 0.58, 0.0, stratum);
                let hip = torso_c + yaw_q * Vec3::new(-0.18 * s, -0.02 * s, side * 0.095 * s);
                let foot = torso_c + yaw_q * Vec3::new(-0.16 * s, -0.24 * s, side * 0.095 * s);
                push_tapered_tube(out, hip, foot, 0.046 * s, 0.032 * s, 5, golden_distal, 0.58, 0.0, stratum);
            }
            push_ellipsoid(out, head_c + head_q * Vec3::new(0.115 * s, -0.038 * s, 0.0), Vec3::new(0.052 * s, 0.044 * s, 0.050 * s), head_q, black_muzzle, 0.50, 0.0, stratum);
            let mut tail_st = Vec::with_capacity(7);
            for seg in 0..=6 {
                let f = seg as f32 / 6.0;
                let col = if seg >= 5 { black_muzzle } else { agouti_back };
                let rad = (0.036 - 0.008 * f) * s;
                tail_st.push(SpineStation {
                    pos: torso_c + yaw_q * Vec3::new((-0.22 - 0.68 * f) * s, (0.20 * (f * PI).sin() - 0.30 * f * f) * s, org.pose.tail_angle * f * 0.45 * s),
                    half_width: rad, dorsal_height: rad, ventral_depth: rad, dorsal_rgba: col, ventral_rgba: col,
                });
            }
            push_lofted_spine(out, &tail_st, 6, 0.65, 0.0, stratum);
        }

        Species::ThreeToedSloth => {
            let algal_fur = [0.42, 0.46, 0.34, 1.0];
            let face_disc = [0.85, 0.80, 0.68, 1.0];
            let claw_keratin = [0.86, 0.82, 0.72, 1.0];
            push_ellipsoid(out, org.pos, Vec3::new(0.32 * s, 0.21 * s, 0.21 * s), yaw_q, algal_fur, 0.88, 0.0, stratum);
            let head_c = org.pos + yaw_q * Vec3::new(0.30 * s, 0.09 * s, 0.0);
            push_ellipsoid(out, head_c, Vec3::splat(0.135 * s), head_q, face_disc, 0.80, 0.0, stratum);
            for &(lx, reach_out, ph_off) in &[(0.22_f32, 0.14_f32, PI), (-0.22, -0.08, 0.0)] {
                for side in [-1.0_f32, 1.0] {
                    let sw = (org.pose.limb_phase + ph_off).sin() * 0.06 * s;
                    let mount = org.pos + yaw_q * Vec3::new(lx * s, 0.08 * s, side * 0.14 * s);
                    let elbow = org.pos + yaw_q * Vec3::new((lx + reach_out) * s, 0.32 * s, side * 0.17 * s);
                    let wrist = org.pos + yaw_q * Vec3::new((lx + sw) * s, 0.54 * s, side * 0.06 * s);
                    let hook = wrist + yaw_q * Vec3::new(0.0, 0.08 * s, -side * 0.06 * s);
                    push_three_segment_limb(out, mount, elbow, wrist, hook, [0.055 * s, 0.045 * s, 0.036 * s, 0.022 * s], algal_fur, claw_keratin, 0.84, stratum);
                }
            }
        }

        Species::ScarletMacaw => {
            let scarlet = [0.91, 0.13, 0.13, 1.0];
            let chrome_yellow = [0.97, 0.80, 0.10, 1.0];
            let cobalt = [0.11, 0.37, 0.88, 1.0];
            let body_c = org.pos + Vec3::new(0.0, 0.15 * s, 0.0);
            let tilt = yaw_q * Quat::from_rotation_z(0.52);
            push_ellipsoid(out, body_c, Vec3::new(0.16 * s, 0.27 * s, 0.145 * s), tilt, scarlet, 0.50, 0.0, stratum);
            let head_c = body_c + yaw_q * Vec3::new(0.09 * s, 0.27 * s, 0.0);
            push_ellipsoid(out, head_c, Vec3::splat(0.11 * s), head_q, scarlet, 0.48, 0.0, stratum);
            push_tapered_tube(out, head_c + head_q * Vec3::new(0.09 * s, 0.01 * s, 0.0), head_c + head_q * Vec3::new(0.19 * s, -0.06 * s, 0.0), 0.05 * s, 0.012 * s, 5, [0.95, 0.92, 0.83, 1.0], 0.35, 0.0, stratum);
            for side in [-1.0_f32, 1.0] {
                let sh = body_c + tilt * Vec3::new(0.02 * s, 0.10 * s, side * 0.13 * s);
                let mid = body_c + tilt * Vec3::new(-0.02 * s, -0.04 * s, side * 0.14 * s);
                let tip = body_c + tilt * Vec3::new(-0.06 * s, -0.26 * s, side * 0.11 * s);
                push_tapered_tube(out, sh, mid, 0.075 * s, 0.065 * s, 4, chrome_yellow, 0.48, 0.0, stratum);
                push_tapered_tube(out, mid, tip, 0.062 * s, 0.024 * s, 4, cobalt, 0.48, 0.0, stratum);
            }
            push_tapered_tube(out, body_c + Vec3::new(-0.06 * s, -0.18 * s, 0.0), body_c + yaw_q * Vec3::new(-0.28 * s, -0.86 * s, 0.0), 0.068 * s, 0.015 * s, 4, scarlet, 0.50, 0.0, stratum);
        }

        Species::KeelBilledToucan => {
            let black = [0.08, 0.09, 0.12, 1.0];
            let body_c = org.pos + Vec3::new(0.0, 0.13 * s, 0.0);
            push_ellipsoid(out, body_c, Vec3::new(0.17 * s, 0.23 * s, 0.15 * s), yaw_q * Quat::from_rotation_z(0.38), black, 0.44, 0.0, stratum);
            let head_c = body_c + yaw_q * Vec3::new(0.09 * s, 0.22 * s, 0.0);
            push_ellipsoid(out, head_c, Vec3::splat(0.11 * s), head_q, [0.97, 0.97, 0.95, 1.0], 0.48, 0.0, stratum);
            push_tapered_tube(out, head_c + head_q * Vec3::new(0.09 * s, 0.01 * s, 0.0), head_c + head_q * Vec3::new(0.38 * s, -0.04 * s, 0.0), 0.076 * s, 0.032 * s, 6, [0.98, 0.54, 0.08, 1.0], 0.30, 0.0, stratum);
            push_tapered_tube(out, head_c + head_q * Vec3::new(0.38 * s, -0.04 * s, 0.0), head_c + head_q * Vec3::new(0.45 * s, -0.06 * s, 0.0), 0.032 * s, 0.014 * s, 6, black, 0.28, 0.0, stratum);
        }

        Species::LowlandTapir => {
            let umber = [0.27, 0.22, 0.19, 1.0];
            let origin = org.pos + Vec3::new(0.0, 0.62 * s, 0.0);
            let st = [
                SpineStation { pos: origin + yaw_q * Vec3::new(-0.56 * s, -0.04 * s, 0.0), half_width: 0.24 * s, dorsal_height: 0.28 * s, ventral_depth: 0.24 * s, dorsal_rgba: umber, ventral_rgba: umber },
                SpineStation { pos: origin + yaw_q * Vec3::new(-0.14 * s, 0.06 * s, 0.0), half_width: 0.34 * s, dorsal_height: 0.36 * s, ventral_depth: 0.32 * s, dorsal_rgba: umber, ventral_rgba: umber },
                SpineStation { pos: origin + yaw_q * Vec3::new(0.58 * s, 0.06 * s, 0.0), half_width: 0.21 * s, dorsal_height: 0.25 * s, ventral_depth: 0.21 * s, dorsal_rgba: umber, ventral_rgba: umber },
            ];
            push_lofted_spine(out, &st, 8, 0.72, 0.0, stratum);
            push_oriented_box(out, origin + yaw_q * Vec3::new(0.48 * s, 0.34 * s, 0.0), Vec3::new(0.24 * s, 0.07 * s, 0.025 * s), yaw_q, [0.13, 0.10, 0.09, 1.0], 0.85, 0.0, stratum);
            let head_c = origin + yaw_q * Vec3::new(0.70 * s, 0.11 * s, 0.0);
            push_tapered_tube(out, head_c, head_c + head_q * Vec3::new(0.38 * s, -0.15 * s, 0.0), 0.14 * s, 0.048 * s, 6, umber, 0.70, 0.0, stratum);
            for lx in [0.36_f32, -0.38] {
                for side in [-1.0_f32, 1.0] {
                    let top = origin + yaw_q * Vec3::new(lx * s, -0.06 * s, side * 0.21 * s);
                    let hoof = top + yaw_q * Vec3::new(0.0, -0.56 * s, 0.0);
                    push_tapered_tube(out, top, hoof, 0.095 * s, 0.068 * s, 5, umber, 0.74, 0.0, stratum);
                }
            }
        }

        Species::CentralAmericanAgouti => {
            let rufous = [0.70, 0.35, 0.14, 1.0];
            let upright = org.pose.carrying_item;
            let pitch = if upright { yaw_q * Quat::from_rotation_z(0.70) } else { yaw_q };
            let body_c = org.pos + Vec3::new(0.0, 0.23 * s, 0.0);
            push_ellipsoid(out, body_c, Vec3::new(0.24 * s, 0.15 * s, 0.14 * s), pitch, rufous, 0.65, 0.0, stratum);
            let head_c = body_c + pitch * Vec3::new(0.22 * s, 0.08 * s, 0.0);
            push_ellipsoid(out, head_c, Vec3::new(0.11 * s, 0.08 * s, 0.075 * s), head_q, rufous, 0.62, 0.0, stratum);
            if upright {
                push_ellipsoid(out, head_c + pitch * Vec3::new(0.06 * s, -0.10 * s, 0.0), Vec3::splat(0.048 * s), Quat::IDENTITY, [0.18, 0.46, 0.92, 1.0], 0.32, 0.04, TrophicStratum::Producers);
            }
        }

        Species::MorphoButterfly => {
            let flap = org.pose.wing_phase_amp[0].sin() * 1.12;
            let col = if org.id % 2 == 0 { [0.10, 0.56, 0.98, 1.0] } else { [0.97, 0.86, 0.16, 1.0] };
            for side in [-1.0_f32, 1.0] {
                let w_rot = yaw_q * Quat::from_rotation_x(side * flap);
                out.push(Tri3D::new(org.pos, org.pos + w_rot * Vec3::new(0.16 * s, 0.01 * s, side * 0.25 * s), org.pos + w_rot * Vec3::new(-0.02 * s, 0.01 * s, side * 0.21 * s), col, 0.26, 0.10, stratum));
                out.push(Tri3D::new(org.pos, org.pos + w_rot * Vec3::new(-0.03 * s, 0.01 * s, side * 0.18 * s), org.pos + w_rot * Vec3::new(-0.15 * s, 0.01 * s, side * 0.13 * s), col, 0.28, 0.08, stratum));
            }
        }

        Species::LeafcutterAnt => {
            let ferruginous = [0.62, 0.23, 0.10, 1.0];
            let p = org.pos + Vec3::new(0.0, 0.048 * s, 0.0);
            push_ellipsoid(out, p + yaw_q * Vec3::new(0.068 * s, 0.012 * s, 0.0), Vec3::splat(0.030 * s), yaw_q, ferruginous, 0.42, 0.0, stratum);
            push_ellipsoid(out, p, Vec3::new(0.042 * s, 0.024 * s, 0.022 * s), yaw_q, ferruginous, 0.44, 0.0, stratum);
            push_ellipsoid(out, p + yaw_q * Vec3::new(-0.082 * s, 0.010 * s, 0.0), Vec3::splat(0.034 * s), yaw_q, ferruginous, 0.42, 0.0, stratum);
            if org.pose.carrying_item {
                out.push(Tri3D::new(p + Vec3::new(0.0, 0.04 * s, 0.0), p + yaw_q * Vec3::new(0.12 * s, 0.15 * s, 0.0), p + yaw_q * Vec3::new(-0.04 * s, 0.23 * s, 0.02 * s), [0.24, 0.78, 0.28, 1.0], 0.48, 0.04, TrophicStratum::Producers));
            }
        }

        Species::AmanitaFungus | Species::MycenaBiolumFungus => {
            let biolum = org.species == Species::MycenaBiolumFungus;
            let stipe_top = org.pos + Vec3::new(0.0, 0.27 * s, 0.0);
            push_tapered_tube(out, org.pos, stipe_top, 0.046 * s, 0.030 * s, 6, [0.93, 0.90, 0.82, 1.0], 0.75, if biolum { 0.28 * night_factor } else { 0.0 }, stratum);
            let cap_col = if biolum { [0.20, 0.95, 0.68, 1.0] } else { [0.86, 0.18, 0.14, 1.0] };
            push_ellipsoid(out, stipe_top + Vec3::new(0.0, 0.04 * s, 0.0), Vec3::new(0.185 * s, 0.095 * s, 0.185 * s), Quat::IDENTITY, cap_col, 0.50, if biolum { 0.22 + 1.18 * night_factor } else { 0.0 }, stratum);
        }
    }
}
