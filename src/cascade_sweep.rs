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

use crate::trophic::{BiomeKind, TrophicEnergyLedger, TrophicStratum};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerturbationRegime {
    BaselineIntact,
    KeystoneRemoved,
    ApexPredatorShock50Pct,
    HerbivoreOutbreakPulse60Pct,
}

impl PerturbationRegime {
    pub const ALL: [Self; 4] = [
        Self::BaselineIntact,
        Self::KeystoneRemoved,
        Self::ApexPredatorShock50Pct,
        Self::HerbivoreOutbreakPulse60Pct,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Self::BaselineIntact => "baseline_intact",
            Self::KeystoneRemoved => "keystone_removed",
            Self::ApexPredatorShock50Pct => "apex_shock_50pct",
            Self::HerbivoreOutbreakPulse60Pct => "herbivore_pulse_60pct",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::BaselineIntact => "Baseline Intact Food Web",
            Self::KeystoneRemoved => "Keystone Guild Removed",
            Self::ApexPredatorShock50Pct => "Apex (L4) -50% Pulse Shock",
            Self::HerbivoreOutbreakPulse60Pct => "Herbivore (L2) +60% Pulse Outbreak",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeRegimeMetrics {
    pub biome_title: String,
    pub biome_short: String,
    pub regime_code: String,
    pub regime_label: String,
    pub initial_l1_kcal: f32,
    pub final_l1_kcal: f32,
    pub l1_producer_ratio_pct: f32,
    pub initial_l2_kcal: f32,
    pub final_l2_kcal: f32,
    pub l2_herbivore_ratio_pct: f32,
    pub final_l3_kcal: f32,
    pub final_l4_kcal: f32,
    pub cascade_amplification_index: f32,
    pub shannon_trophic_evenness: f32,
    pub recovery_resilience_score: f32,
    pub lindeman_l2_efficiency_pct: f32,
    pub lindeman_l4_efficiency_pct: f32,
    pub cumulative_gpp_kcal: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeSensitivityReport {
    pub schema_version: String,
    pub duration_s: f32,
    pub dt_s: f32,
    pub biome_count: usize,
    pub mean_resilience_intact: f32,
    pub mean_resilience_keystone_removed: f32,
    pub keystone_vulnerability_delta: f32,
    pub rows: Vec<CascadeRegimeMetrics>,
}

fn ledger_for_biome(biome: BiomeKind) -> TrophicEnergyLedger {
    let base = TrophicEnergyLedger::baseline_for_biome(biome);
    TrophicEnergyLedger {
        biomass_kcal: base,
        detritus_pool_g: TrophicEnergyLedger::baseline_detritus_g(biome),
        mineral_nutrient_pool_mg: TrophicEnergyLedger::baseline_mineral_mg(biome),
        biomass_history: vec![base; 24],
        ..Default::default()
    }
}

fn shannon_evenness_l1_l4(b: &[f32; 5]) -> f32 {
    let total: f32 = b[1..=4].iter().map(|v| v.max(0.0)).sum();
    if total <= 1e-6 {
        return 0.0;
    }
    let mut h = 0.0_f32;
    for &val in &b[1..=4] {
        let p = (val.max(0.0)) / total;
        if p > 1e-6 {
            h -= p * p.ln();
        }
    }
    (h / (4.0_f32).ln()).clamp(0.0, 1.0)
}

fn evaluate_single_regime(
    biome: BiomeKind,
    regime: PerturbationRegime,
    duration_s: f32,
    dt_s: f32,
) -> CascadeRegimeMetrics {
    let dur = duration_s.clamp(1.0, 120.0);
    let dt = dt_s.clamp(0.01, 0.2);
    let steps = ((dur / dt).ceil() as usize).max(1);
    let base = TrophicEnergyLedger::baseline_for_biome(biome);

    let mut ledger = ledger_for_biome(biome);
    match regime {
        PerturbationRegime::BaselineIntact => {}
        PerturbationRegime::KeystoneRemoved => {
            ledger.keystone_removed = true;
        }
        PerturbationRegime::ApexPredatorShock50Pct => {
            ledger.biomass_kcal[4] *= 0.50;
        }
        PerturbationRegime::HerbivoreOutbreakPulse60Pct => {
            ledger.biomass_kcal[2] *= 1.60;
        }
    }

    let mut max_excursion = 0.0_f32;
    for _ in 0..steps {
        ledger.step_keystone_cascade(biome, dt);
        for k in 1..=4 {
            let rel = ((ledger.biomass_kcal[k] - base[k]).abs()) / base[k].max(1.0);
            if rel > max_excursion {
                max_excursion = rel;
            }
        }
    }

    let final_dev: f32 = (1..=4)
        .map(|k| ((ledger.biomass_kcal[k] - base[k]).abs()) / base[k].max(1.0))
        .sum::<f32>()
        / 4.0;
    let resilience = (100.0 / (1.0 + final_dev * 3.5)).clamp(0.0, 100.0);
    let evenness = shannon_evenness_l1_l4(&ledger.biomass_kcal);

    CascadeRegimeMetrics {
        biome_title: biome.title().to_string(),
        biome_short: biome.short_name().to_string(),
        regime_code: regime.code().to_string(),
        regime_label: regime.label().to_string(),
        initial_l1_kcal: (base[1] * 100.0).round() / 100.0,
        final_l1_kcal: (ledger.biomass_kcal[1] * 100.0).round() / 100.0,
        l1_producer_ratio_pct: ((ledger.biomass_kcal[1] / base[1].max(1.0)) * 1000.0).round() / 10.0,
        initial_l2_kcal: (base[2] * 100.0).round() / 100.0,
        final_l2_kcal: (ledger.biomass_kcal[2] * 100.0).round() / 100.0,
        l2_herbivore_ratio_pct: ((ledger.biomass_kcal[2] / base[2].max(1.0)) * 1000.0).round() / 10.0,
        final_l3_kcal: (ledger.biomass_kcal[3] * 100.0).round() / 100.0,
        final_l4_kcal: (ledger.biomass_kcal[4] * 100.0).round() / 100.0,
        cascade_amplification_index: (max_excursion * 1000.0).round() / 1000.0,
        shannon_trophic_evenness: (evenness * 10000.0).round() / 10000.0,
        recovery_resilience_score: (resilience * 100.0).round() / 100.0,
        lindeman_l2_efficiency_pct: (ledger.transfer_efficiency_pct(TrophicStratum::PrimaryConsumers)
            * 100.0)
            .round()
            / 100.0,
        lindeman_l4_efficiency_pct: (ledger
            .transfer_efficiency_pct(TrophicStratum::TertiaryCarnivores)
            * 100.0)
            .round()
            / 100.0,
        cumulative_gpp_kcal: (ledger.cumulative_gpp_kcal * 100.0).round() / 100.0,
    }
}

/// Runs a trophic cascade resilience & keystone perturbation sensitivity sweep across either
/// a single `biome` or all 9 `BiomeKind::ALL` ecosystems.
pub fn run_trophic_cascade_sensitivity_sweep(
    biome_filter: Option<BiomeKind>,
    duration_s: f32,
    dt_s: f32,
) -> CascadeSensitivityReport {
    let biomes: Vec<BiomeKind> = match biome_filter {
        Some(b) => vec![b],
        None => BiomeKind::ALL.to_vec(),
    };

    let mut rows = Vec::with_capacity(biomes.len() * PerturbationRegime::ALL.len());
    for &biome in &biomes {
        for &regime in &PerturbationRegime::ALL {
            rows.push(evaluate_single_regime(biome, regime, duration_s, dt_s));
        }
    }

    let intact_vals: Vec<f32> = rows
        .iter()
        .filter(|r| r.regime_code == PerturbationRegime::BaselineIntact.code())
        .map(|r| r.recovery_resilience_score)
        .collect();
    let keystone_vals: Vec<f32> = rows
        .iter()
        .filter(|r| r.regime_code == PerturbationRegime::KeystoneRemoved.code())
        .map(|r| r.recovery_resilience_score)
        .collect();

    let mean_intact = if intact_vals.is_empty() {
        0.0
    } else {
        intact_vals.iter().sum::<f32>() / intact_vals.len() as f32
    };
    let mean_keystone = if keystone_vals.is_empty() {
        0.0
    } else {
        keystone_vals.iter().sum::<f32>() / keystone_vals.len() as f32
    };

    CascadeSensitivityReport {
        schema_version: "trophic_hearth_cascade_sweep_v1".to_string(),
        duration_s: duration_s.clamp(1.0, 120.0),
        dt_s: dt_s.clamp(0.01, 0.2),
        biome_count: biomes.len(),
        mean_resilience_intact: (mean_intact * 100.0).round() / 100.0,
        mean_resilience_keystone_removed: (mean_keystone * 100.0).round() / 100.0,
        keystone_vulnerability_delta: ((mean_intact - mean_keystone) * 100.0).round() / 100.0,
        rows,
    }
}

pub fn csv_escape(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// Formats a [`CascadeSensitivityReport`] as an RFC 4180 CSV table.
pub fn format_cascade_sweep_csv(report: &CascadeSensitivityReport) -> String {
    let mut out = String::from(
        "biome_short,biome_title,regime_code,regime_label,initial_l1_kcal,final_l1_kcal,l1_producer_ratio_pct,initial_l2_kcal,final_l2_kcal,l2_herbivore_ratio_pct,final_l3_kcal,final_l4_kcal,cascade_amplification_index,shannon_trophic_evenness,recovery_resilience_score,lindeman_l2_efficiency_pct,lindeman_l4_efficiency_pct,cumulative_gpp_kcal\n",
    );
    for r in &report.rows {
        out.push_str(&format!(
            "{},{},{},{},{:.2},{:.2},{:.1},{:.2},{:.2},{:.1},{:.2},{:.2},{:.3},{:.4},{:.2},{:.2},{:.2},{:.2}\n",
            csv_escape(&r.biome_short),
            csv_escape(&r.biome_title),
            csv_escape(&r.regime_code),
            csv_escape(&r.regime_label),
            r.initial_l1_kcal,
            r.final_l1_kcal,
            r.l1_producer_ratio_pct,
            r.initial_l2_kcal,
            r.final_l2_kcal,
            r.l2_herbivore_ratio_pct,
            r.final_l3_kcal,
            r.final_l4_kcal,
            r.cascade_amplification_index,
            r.shannon_trophic_evenness,
            r.recovery_resilience_score,
            r.lindeman_l2_efficiency_pct,
            r.lindeman_l4_efficiency_pct,
            r.cumulative_gpp_kcal,
        ));
    }
    out
}

/// Runs the sensitivity sweep and writes `trophic_cascade_sweep.json` and `.csv` into `out_dir`.
pub fn export_cascade_sweep_files(
    out_dir: impl AsRef<Path>,
    biome_filter: Option<BiomeKind>,
    duration_s: f32,
    dt_s: f32,
) -> Result<(PathBuf, PathBuf, CascadeSensitivityReport), String> {
    let dir = out_dir.as_ref();
    fs::create_dir_all(dir).map_err(|e| format!("Create export dir failed: {e}"))?;
    let report = run_trophic_cascade_sensitivity_sweep(biome_filter, duration_s, dt_s);
    let json_path = dir.join("trophic_cascade_sweep.json");
    let csv_path = dir.join("trophic_cascade_sweep.csv");

    let json_txt = serde_json::to_string_pretty(&report)
        .map_err(|e| format!("Cascade sweep JSON serialize error: {e}"))?;
    fs::write(&json_path, json_txt).map_err(|e| format!("Write cascade sweep JSON failed: {e}"))?;
    let csv_txt = format_cascade_sweep_csv(&report);
    fs::write(&csv_path, csv_txt).map_err(|e| format!("Write cascade sweep CSV failed: {e}"))?;

    Ok((json_path, csv_path, report))
}
