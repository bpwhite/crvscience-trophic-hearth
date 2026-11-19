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

//! Embedded procedural RGBA window icon (`ViewportBuilder::with_icon`) matching `assets/trophic_hearth.ico`.

use eframe::egui;

fn blend_px(buf: &mut [u8], idx: usize, r: f32, g: f32, b: f32, a: f32) {
    let alpha = a.clamp(0.0, 1.0);
    if alpha <= 0.001 {
        return;
    }
    let da = f32::from(buf[idx + 3]) / 255.0;
    let out_a = alpha + da * (1.0 - alpha);
    if out_a <= 1e-5 {
        return;
    }
    let br = f32::from(buf[idx]);
    let bg = f32::from(buf[idx + 1]);
    let bb = f32::from(buf[idx + 2]);
    buf[idx] = ((r * alpha + br * da * (1.0 - alpha)) / out_a).round().clamp(0.0, 255.0) as u8;
    buf[idx + 1] = ((g * alpha + bg * da * (1.0 - alpha)) / out_a).round().clamp(0.0, 255.0) as u8;
    buf[idx + 2] = ((b * alpha + bb * da * (1.0 - alpha)) / out_a).round().clamp(0.0, 255.0) as u8;
    buf[idx + 3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
}

fn sd_rounded_box(px: f32, py: f32, bx: f32, by: f32, r: f32) -> f32 {
    let qx = px.abs() - bx + r;
    let qy = py.abs() - by + r;
    qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - r
}

/// Renders an anti-aliased `size x size` RGBA TrophicHearth badge matching `assets/trophic_hearth.ico`.
pub fn render_trophic_hearth_rgba(size: usize) -> Vec<u8> {
    let mut buf = vec![0u8; size * size * 4];
    let aa = 2.2 / (size as f32).max(1.0);
    let waves: [(f32, f32, f32, [f32; 3]); 3] = [
        (0.02, 0.08, 0.0, [45.0, 212.0, 191.0]),
        (0.28, 0.09, 1.4, [52.0, 211.0, 153.0]),
        (0.54, 0.07, 2.8, [56.0, 189.0, 248.0]),
    ];

    for py in 0..size {
        let y = ((py as f32 + 0.5) / size as f32) * 2.0 - 1.0;
        for px in 0..size {
            let x = ((px as f32 + 0.5) / size as f32) * 2.0 - 1.0;
            let idx = (py * size + px) * 4;
            let box_d = sd_rounded_box(x, y, 0.88, 0.88, 0.24);
            if box_d < aa {
                let bg_a = (0.5 - box_d / aa).clamp(0.0, 1.0);
                let grad = (y + 1.0) * 0.5;
                blend_px(
                    &mut buf,
                    idx,
                    10.0 + 14.0 * (1.0 - grad),
                    26.0 + 22.0 * (1.0 - grad),
                    34.0 + 34.0 * grad,
                    bg_a,
                );
                if box_d.abs() < 0.035 {
                    blend_px(
                        &mut buf,
                        idx,
                        45.0,
                        212.0,
                        191.0,
                        (1.0 - box_d.abs() / 0.035) * 0.85 * bg_a,
                    );
                }
            }
            let d_sun = x.hypot(y + 0.34);
            if d_sun < 0.26 + aa {
                let s_a = ((0.26 - d_sun) / aa).clamp(0.0, 1.0);
                let core = (1.0 - d_sun / 0.26).clamp(0.0, 1.0);
                blend_px(
                    &mut buf,
                    idx,
                    251.0,
                    146.0 + 75.0 * core,
                    60.0 + 60.0 * core,
                    s_a,
                );
            }
            if x.abs() <= 0.72 {
                for &(y0, amp, ph, col) in &waves {
                    let wy = y0 + amp * (x * 4.2 + ph).sin();
                    let d = (y - wy).abs() - 0.048;
                    if d < aa {
                        blend_px(&mut buf, idx, col[0], col[1], col[2], (0.5 - d / aa).clamp(0.0, 1.0));
                    }
                }
            }
        }
    }
    buf
}

/// Returns the `64x64` window/taskbar icon for eframe `ViewportBuilder::with_icon`.
pub fn app_egui_icon() -> egui::IconData {
    let size = 64usize;
    egui::IconData {
        rgba: render_trophic_hearth_rgba(size),
        width: size as u32,
        height: size as u32,
    }
}
