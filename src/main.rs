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

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(unsafe_code)]

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use trophic_hearth::app::{render_headless_screenshot, TrophicHearthApp};
use trophic_hearth::audio::export_ambient_soundscape_wav;
use trophic_hearth::icon::app_egui_icon;
use trophic_hearth::persistence::TrophicSnapshotV1;
use trophic_hearth::trophic::{BiomeKind, TrophicFocus, TrophicStratum};
use trophic_hearth::world::RainforestWorld;

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum CliBiome {
    Tidepool,
    Brine,
    Rift,
    Pnw,
    Vent,
    Lake,
    Coral,
    Kelp,
    Rainforest,
}

impl From<CliBiome> for BiomeKind {
    fn from(v: CliBiome) -> Self {
        match v {
            CliBiome::Tidepool => BiomeKind::RockyIntertidalTidepool,
            CliBiome::Brine => BiomeKind::ColdSeepBrinePool,
            CliBiome::Rift => BiomeKind::MidwaterPelagicRift,
            CliBiome::Pnw => BiomeKind::PnwOldGrowthSalmonCreek,
            CliBiome::Vent => BiomeKind::DeepSeaHydrothermalVent,
            CliBiome::Lake => BiomeKind::TemperateFreshwaterLake,
            CliBiome::Coral => BiomeKind::IndoPacificCoralReef,
            CliBiome::Kelp => BiomeKind::PacificKelpForest,
            CliBiome::Rainforest => BiomeKind::NeotropicalRainforest,
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "trophic-hearth",
    version = "0.9.3",
    about = "Nine-Biome 3D Living Trophic Diorama, Fireplace, and Bioacoustic Ecosystem Simulator"
)]
struct Cli {
    /// Initial biome (tidepool, brine, rift, pnw, vent, lake, coral, kelp, rainforest)
    #[arg(long, value_enum, default_value_t = CliBiome::Tidepool)]
    biome: CliBiome,

    /// Optional trophic stratum focus index (0=Decomposers .. 4=Tertiary Carnivores)
    #[arg(long)]
    stratum: Option<usize>,

    /// Launch directly into borderless-HUD Zen Hearth Mode
    #[arg(long, default_value_t = false)]
    hearth_mode: bool,

    /// SQLite WAL persistence database path
    #[arg(long, default_value = "trophic_hearth.sqlite3")]
    db: PathBuf,

    /// Deterministic world RNG seed
    #[arg(long, default_value_t = 2026_1116)]
    seed: u64,

    /// Render an offscreen 1600x1000 RGBA PNG screenshot without opening a GUI window
    #[arg(long)]
    headless_screenshot: Option<PathBuf>,

    /// Select preset view (1=Tidepool, 2=Coral Reef, 3=Hydrothermal Vent) for --headless-screenshot
    #[arg(long, default_value_t = 1)]
    screenshot_view: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Launch the interactive 3D Living Diorama Fireplace GUI (default)
    Gui {
        /// Initial biome (tidepool, brine, rift, pnw, vent, lake, coral, kelp, rainforest)
        #[arg(long, value_enum, default_value_t = CliBiome::Tidepool)]
        biome: CliBiome,

        /// Optional trophic stratum focus index (0=Decomposers .. 4=Tertiary Carnivores)
        #[arg(long)]
        stratum: Option<usize>,

        /// Launch directly into borderless-HUD Zen Hearth Mode
        #[arg(long, default_value_t = false)]
        hearth_mode: bool,

        /// SQLite WAL persistence database path
        #[arg(long, default_value = "trophic_hearth.sqlite3")]
        db: PathBuf,

        /// Deterministic world RNG seed
        #[arg(long, default_value_t = 2026_1116)]
        seed: u64,
    },

    /// Run a headless diurnal ecological simulation and write JSON telemetry
    Simulate {
        /// Biome to simulate (tidepool, brine, rift, pnw, vent, lake, coral, kelp, rainforest)
        #[arg(long, value_enum, default_value_t = CliBiome::Tidepool)]
        biome: CliBiome,

        /// Simulated ecological hours to run
        #[arg(long, default_value_t = 24.0)]
        hours: f32,

        /// Physics integration time step in real-equivalent seconds
        #[arg(long, default_value_t = 0.05)]
        dt: f32,

        /// Output JSON snapshot path
        #[arg(long, default_value = "trophic_simulation_report.json")]
        output: PathBuf,

        /// Deterministic world RNG seed
        #[arg(long, default_value_t = 2026_1116)]
        seed: u64,
    },

    /// Synthesize a stereo procedural bioacoustic soundscape WAV file
    ExportWav {
        /// Duration in seconds
        #[arg(long, default_value_t = 18.0)]
        seconds: f32,

        /// Output WAV path
        #[arg(long, default_value = "trophic_hearth.wav")]
        output: PathBuf,
    },
}

#[cfg(windows)]
#[allow(unsafe_code)]
fn attach_parent_console_if_cli_args() {
    if std::env::args_os().len() > 1 {
        unsafe {
            let _ = windows_sys::Win32::System::Console::AttachConsole(u32::MAX);
        }
    }
}

fn launch_gui(
    biome: CliBiome,
    stratum: Option<usize>,
    hearth_mode: bool,
    db: PathBuf,
    seed: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut world = RainforestWorld::new_for_biome(biome.into(), seed);
    if let Some(idx) = stratum {
        if let Some(st) = TrophicStratum::from_index(idx) {
            world.focus = TrophicFocus::Single(st);
        }
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1360.0, 820.0])
            .with_min_inner_size([920.0, 580.0])
            .with_icon(app_egui_icon())
            .with_title("TrophicHearth v0.9.3 - Nine-Biome 3D Living Trophic Diorama"),
        ..Default::default()
    };

    eframe::run_native(
        "TrophicHearth v0.9.3",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(TrophicHearthApp::new(
                cc,
                world,
                Some(db),
                hearth_mode,
            )))
        }),
    )
    .map_err(|e| format!("eframe error: {e}"))?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    attach_parent_console_if_cli_args();

    let cli = Cli::parse();

    if let Some(out_path) = cli.headless_screenshot {
        render_headless_screenshot(&out_path, cli.screenshot_view)?;
        return Ok(());
    }

    match cli.command {
        None => launch_gui(cli.biome, cli.stratum, cli.hearth_mode, cli.db, cli.seed)?,
        Some(Commands::Gui {
            biome,
            stratum,
            hearth_mode,
            db,
            seed,
        }) => launch_gui(biome, stratum, hearth_mode, db, seed)?,

        Some(Commands::Simulate {
            biome,
            hours,
            dt,
            output,
            seed,
        }) => {
            let mut world = RainforestWorld::new_for_biome(biome.into(), seed);
            world.diurnal.minutes_per_real_sec = 10.0;
            let total_real_sec = (hours * 60.0) / world.diurnal.minutes_per_real_sec;
            let steps = (total_real_sec / dt).ceil() as usize;

            for _ in 0..steps {
                world.step(dt);
                world.pending_sounds.clear();
            }

            let snap = TrophicSnapshotV1::from_world("headless_run", world);
            let json = serde_json::to_string_pretty(&snap)?;
            std::fs::write(&output, json)?;
            println!(
                "Completed {:.1}h {:?} trophic simulation ({} steps) -> {}",
                hours,
                biome,
                steps,
                output.display()
            );
        }

        Some(Commands::ExportWav { seconds, output }) => {
            export_ambient_soundscape_wav(&output, seconds)?;
            println!(
                "Synthesized {:.1}s stereo bioacoustic WAV -> {}",
                seconds,
                output.display()
            );
        }
    }

    Ok(())
}
