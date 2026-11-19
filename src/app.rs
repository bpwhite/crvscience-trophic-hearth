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

use crate::audio::{export_ambient_soundscape_wav, BioacousticEngine};
use crate::events::StochasticEventKind;
use crate::persistence::{PersistenceStore, SaveSlotSummary, TrophicSnapshotV1};
use crate::renderer::{render_rainforest_viewport, CameraPreset, DioramaCamera};
use crate::trophic::{BiomeKind, TrophicEnergyLedger, TrophicFocus, TrophicStratum};
use crate::world::RainforestWorld;
use egui::{Color32, RichText, Sense};
use std::path::PathBuf;
use std::time::Instant;

pub struct TrophicHearthApp {
    pub world: RainforestWorld,
    pub camera: DioramaCamera,
    pub audio: BioacousticEngine,
    pub store: Option<PersistenceStore>,
    pub slots_cache: Vec<SaveSlotSummary>,
    pub paused: bool,
    pub sim_speed: f32,
    pub zen_hearth_mode: bool,
    pub show_inspector_drawer: bool,
    pub show_about_window: bool,
    pub selected_organism_id: Option<u32>,
    pub slot_name_input: String,
    pub status_banner: String,
    last_frame: Instant,
}

impl TrophicHearthApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        world: RainforestWorld,
        db_path: Option<PathBuf>,
        hearth_mode: bool,
    ) -> Self {
        let store = db_path.and_then(|p| PersistenceStore::open(&p).ok());
        let slots_cache = store
            .as_ref()
            .and_then(|s| s.list_slots().ok())
            .unwrap_or_default();

        Self {
            world,
            camera: DioramaCamera::default(),
            audio: BioacousticEngine::new(true),
            store,
            slots_cache,
            paused: false,
            sim_speed: 1.0,
            zen_hearth_mode: hearth_mode,
            show_inspector_drawer: !hearth_mode,
            show_about_window: false,
            selected_organism_id: None,
            slot_name_input: "trophic_snapshot_01".to_string(),
            status_banner: String::new(),
            last_frame: Instant::now(),
        }
    }

    pub fn new_headless(world: RainforestWorld, hearth_mode: bool) -> Self {
        let store = PersistenceStore::open_in_memory().ok();
        let slots_cache = store
            .as_ref()
            .and_then(|s| s.list_slots().ok())
            .unwrap_or_default();
        let mut audio = BioacousticEngine::new(false);
        audio.muted = true;

        Self {
            world,
            camera: DioramaCamera::default(),
            audio,
            store,
            slots_cache,
            paused: true,
            sim_speed: 1.0,
            zen_hearth_mode: hearth_mode,
            show_inspector_drawer: !hearth_mode,
            show_about_window: false,
            selected_organism_id: None,
            slot_name_input: "trophic_snapshot_01".to_string(),
            status_banner: String::new(),
            last_frame: Instant::now(),
        }
    }

    fn switch_biome(&mut self, biome: BiomeKind) {
        if self.world.biome != biome {
            let seed = self.world.rng_seed;
            self.world = RainforestWorld::new_for_biome(biome, seed);
            self.selected_organism_id = None;
            self.camera.apply_preset(CameraPreset::SideDiorama);
        }
    }

    pub fn ui_step(&mut self, ctx: &egui::Context, advance_sim: bool) {
        let now = Instant::now();
        let raw_dt = now.duration_since(self.last_frame).as_secs_f32().min(0.08);
        self.last_frame = now;

        if ctx.input(|i| i.key_pressed(egui::Key::Tab)) {
            self.zen_hearth_mode = !self.zen_hearth_mode;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            self.paused = !self.paused;
        }

        if advance_sim && !self.paused {
            let scaled_dt = raw_dt * self.sim_speed;
            let substeps = ((scaled_dt / 0.04).ceil() as usize).clamp(1, 5);
            let h = scaled_dt / substeps as f32;
            for _ in 0..substeps {
                self.world.step(h);
            }
            for cue in self.world.pending_sounds.drain(..) {
                self.audio.trigger(cue);
            }
        }

        let is_kelp = self.world.biome == BiomeKind::PacificKelpForest;
        let is_coral = self.world.biome == BiomeKind::IndoPacificCoralReef;
        let is_lake = self.world.biome == BiomeKind::TemperateFreshwaterLake;
        let is_vent = self.world.biome == BiomeKind::DeepSeaHydrothermalVent;
        let is_pnw = self.world.biome == BiomeKind::PnwOldGrowthSalmonCreek;
        let is_rift = self.world.biome == BiomeKind::MidwaterPelagicRift;
        let is_tidepool = self.world.biome == BiomeKind::RockyIntertidalTidepool;
        let is_brine = self.world.biome == BiomeKind::ColdSeepBrinePool;
        let is_aquatic = self.world.biome.is_aquatic();

        if !self.zen_hearth_mode {
            egui::TopBottomPanel::top("hearth_top_bar").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new("TrophicHearth v0.9.3")
                            .strong()
                            .color(Color32::from_rgb(135, 232, 212)),
                    );
                    ui.separator();

                    // Nine-Biome Switcher
                    ui.label("Biome:");
                    if ui
                        .selectable_label(is_tidepool, "Tidepool")
                        .clicked()
                    {
                        self.switch_biome(BiomeKind::RockyIntertidalTidepool);
                    }
                    if ui
                        .selectable_label(is_brine, "Brine Pool")
                        .clicked()
                    {
                        self.switch_biome(BiomeKind::ColdSeepBrinePool);
                    }
                    if ui
                        .selectable_label(is_rift, "Pelagic Rift")
                        .clicked()
                    {
                        self.switch_biome(BiomeKind::MidwaterPelagicRift);
                    }
                    if ui
                        .selectable_label(is_pnw, "PNW Old-Growth")
                        .clicked()
                    {
                        self.switch_biome(BiomeKind::PnwOldGrowthSalmonCreek);
                    }
                    if ui
                        .selectable_label(is_vent, "Hydrothermal Vent")
                        .clicked()
                    {
                        self.switch_biome(BiomeKind::DeepSeaHydrothermalVent);
                    }
                    if ui
                        .selectable_label(is_lake, "Freshwater Lake")
                        .clicked()
                    {
                        self.switch_biome(BiomeKind::TemperateFreshwaterLake);
                    }
                    if ui
                        .selectable_label(is_coral, "Coral Reef")
                        .clicked()
                    {
                        self.switch_biome(BiomeKind::IndoPacificCoralReef);
                    }
                    if ui
                        .selectable_label(is_kelp, "Kelp Forest")
                        .clicked()
                    {
                        self.switch_biome(BiomeKind::PacificKelpForest);
                    }
                    if ui
                        .selectable_label(!is_aquatic && !is_pnw, "Rainforest")
                        .clicked()
                    {
                        self.switch_biome(BiomeKind::NeotropicalRainforest);
                    }

                    ui.separator();
                    let ks_on = !self.world.ledger.keystone_removed;
                    if ui
                        .selectable_label(!ks_on, if ks_on { "Keystone: Intact" } else { "[!] Keystone: Removed" })
                        .clicked()
                    {
                        self.world.ledger.keystone_removed = !self.world.ledger.keystone_removed;
                    }
                    if ui
                        .selectable_label(self.world.ledger.show_flux_arcs, "Flux Arcs")
                        .clicked()
                    {
                        self.world.ledger.show_flux_arcs = !self.world.ledger.show_flux_arcs;
                    }
                    if ui
                        .button(if self.paused { "Resume" } else { "Pause" })
                        .clicked()
                    {
                        self.paused = !self.paused;
                    }
                    ui.add(
                        egui::Slider::new(&mut self.sim_speed, 0.25..=4.0)
                            .text("Pace")
                            .fixed_decimals(2),
                    );

                    ui.separator();
                    ui.label("Trophic Lens:");
                    if ui
                        .selectable_label(self.world.focus == TrophicFocus::AllStrata, "All")
                        .clicked()
                    {
                        self.world.focus = TrophicFocus::AllStrata;
                    }
                    for stratum in TrophicStratum::ALL {
                        let [r, g, b] = stratum.badge_rgb();
                        let active = self.world.focus == TrophicFocus::Single(stratum);
                        if ui
                            .selectable_label(
                                active,
                                RichText::new(stratum.short_label())
                                    .color(Color32::from_rgb(r, g, b)),
                            )
                            .on_hover_text(stratum.label_for_biome(self.world.biome))
                            .clicked()
                        {
                            self.world.focus = TrophicFocus::Single(stratum);
                        }
                    }

                    ui.separator();
                    let mute_lbl = if self.audio.muted {
                        "Unmute"
                    } else {
                        "Sound: On"
                    };
                    if ui.button(mute_lbl).clicked() {
                        self.audio.muted = !self.audio.muted;
                    }
                    if ui.button("Hearth Mode (Tab)").clicked() {
                        self.zen_hearth_mode = true;
                    }
                    if ui.button("Telemetry").clicked() {
                        self.show_inspector_drawer = !self.show_inspector_drawer;
                    }
                });

                ui.horizontal_wrapped(|ui| {
                    let atmo = self.world.diurnal.atmosphere_for_biome(self.world.biome);
                    let h = self.world.diurnal.hour_of_day.floor() as u32 % 24;
                    let m = ((self.world.diurnal.hour_of_day.fract() * 60.0) as u32).min(59);
                    let moisture_unit = if is_aquatic { "% DO" } else { "% RH" };
                    ui.label(format!(
                        "Day {} {:02}:{:02} ({}) | {:.1} degC | {:.0}{} | PAR {:.0} umol",
                        self.world.diurnal.day_count,
                        h,
                        m,
                        atmo.phase_name,
                        atmo.air_temp_c,
                        atmo.relative_humidity_pct,
                        moisture_unit,
                        atmo.par_umol_m2_s,
                    ));
                    ui.add(
                        egui::Slider::new(&mut self.world.diurnal.hour_of_day, 0.0..=23.99)
                            .text("Solar Hour"),
                    );
                    if ui.small_button("Dawn").clicked() {
                        self.world.diurnal.hour_of_day = 6.25;
                    }
                    if ui.small_button("Noon").clicked() {
                        self.world.diurnal.hour_of_day = 12.0;
                    }
                    if ui.small_button("Dusk").clicked() {
                        self.world.diurnal.hour_of_day = 17.85;
                    }
                    if ui.small_button("Night").clicked() {
                        self.world.diurnal.hour_of_day = 22.5;
                    }

                    ui.separator();
                    ui.label("Camera:");
                    let cam_labels = if is_tidepool {
                        [
                            (CameraPreset::SideDiorama, "Intertidal Bench"),
                            (CameraPreset::UnderbrushStalkMacro, "Pisaster & Mussels"),
                            (CameraPreset::CanopyAmbushPerch, "Oystercatcher & Surf"),
                            (CameraPreset::SoilDecomposerLens, "Anemones & Sculpins"),
                        ]
                    } else if is_brine {
                        [
                            (CameraPreset::SideDiorama, "Brine Lake & Seep"),
                            (CameraPreset::UnderbrushStalkMacro, "Mussel Shoreline"),
                            (CameraPreset::CanopyAmbushPerch, "Eels & Chimaera"),
                            (CameraPreset::SoilDecomposerLens, "Methane Ice Worms"),
                        ]
                    } else if is_rift {
                        [
                            (CameraPreset::SideDiorama, "Rift Canyon"),
                            (CameraPreset::UnderbrushStalkMacro, "Barreleye & Praya"),
                            (CameraPreset::CanopyAmbushPerch, "Whale & DVM Layer"),
                            (CameraPreset::SoilDecomposerLens, "Giant & Vampire Squid"),
                        ]
                    } else if is_pnw {
                        [
                            (CameraPreset::SideDiorama, "Forest & Creek"),
                            (CameraPreset::UnderbrushStalkMacro, "Salmon & Dipper"),
                            (CameraPreset::CanopyAmbushPerch, "Spotted Owl Boughs"),
                            (CameraPreset::SoilDecomposerLens, "Nurse Log & Hyphae"),
                        ]
                    } else if is_vent {
                        [
                            (CameraPreset::SideDiorama, "Black-Smoker Rift"),
                            (CameraPreset::UnderbrushStalkMacro, "Tubeworms & Yeti Crabs"),
                            (CameraPreset::CanopyAmbushPerch, "Plumes & Ghost Shark"),
                            (CameraPreset::SoilDecomposerLens, "Sulfide Fissure"),
                        ]
                    } else if is_lake {
                        [
                            (CameraPreset::SideDiorama, "Split Waterline"),
                            (CameraPreset::UnderbrushStalkMacro, "Pike & Eelgrass"),
                            (CameraPreset::CanopyAmbushPerch, "Waterbirds & Sky"),
                            (CameraPreset::SoilDecomposerLens, "Crayfish & Cobble"),
                        ]
                    } else if is_coral {
                        [
                            (CameraPreset::SideDiorama, "Side Diorama"),
                            (CameraPreset::UnderbrushStalkMacro, "Coral Fore-Reef"),
                            (CameraPreset::CanopyAmbushPerch, "Lagoon & Sharks"),
                            (CameraPreset::SoilDecomposerLens, "Anemone & Clams"),
                        ]
                    } else if is_kelp {
                        [
                            (CameraPreset::SideDiorama, "Side Diorama"),
                            (CameraPreset::UnderbrushStalkMacro, "Rocky Reef"),
                            (CameraPreset::CanopyAmbushPerch, "Otter Canopy"),
                            (CameraPreset::SoilDecomposerLens, "Holdfast & Stars"),
                        ]
                    } else {
                        [
                            (CameraPreset::SideDiorama, "Side Diorama"),
                            (CameraPreset::UnderbrushStalkMacro, "Underbrush"),
                            (CameraPreset::CanopyAmbushPerch, "Canopy"),
                            (CameraPreset::SoilDecomposerLens, "Soil/Ants"),
                        ]
                    };
                    for (preset, name) in cam_labels {
                        if ui
                            .selectable_label(self.camera.preset == preset, name)
                            .clicked()
                        {
                            self.camera.apply_preset(preset);
                        }
                    }
                });
            });

            if self.show_inspector_drawer {
                egui::SidePanel::right("ecological_inspector")
                    .default_width(310.0)
                    .show(ctx, |ui| {
                        ui.heading("Stochastic Events & Cues");
                        ui.horizontal_wrapped(|ui| {
                            if is_tidepool {
                                if ui.small_button("Surf Surge").clicked() {
                                    self.world.trigger_event(StochasticEventKind::TidepoolHighSurfSurge);
                                }
                                if ui.small_button("Pisaster Hunt").clicked() {
                                    self.world.trigger_event(StochasticEventKind::PisasterMusselForage);
                                }
                                if ui.small_button("Sculpin Leap").clicked() {
                                    self.world.trigger_event(StochasticEventKind::SculpinPoolToPoolLeap);
                                }
                                if ui.small_button("Oystercatcher").clicked() {
                                    self.world.trigger_event(StochasticEventKind::OystercatcherLowTideProbe);
                                }
                                if ui.small_button("Anemone Bloom").clicked() {
                                    self.world.trigger_event(StochasticEventKind::AnemoneSubmergeBloom);
                                }
                                if ui.small_button("Hermit Wrack").clicked() {
                                    self.world.trigger_event(StochasticEventKind::HermitCrabWrackScavenge);
                                }
                            } else if is_brine {
                                if ui.small_button("CH4 Bubbles").clicked() {
                                    self.world.trigger_event(StochasticEventKind::MethaneHydrateBubblePlume);
                                }
                                if ui.small_button("Brine Wave").clicked() {
                                    self.world.trigger_event(StochasticEventKind::BrineShorelineInternalWave);
                                }
                                if ui.small_button("Eel Skim").clicked() {
                                    self.world.trigger_event(StochasticEventKind::CutthroatEelBrineSkim);
                                }
                                if ui.small_button("Ice Worms").clicked() {
                                    self.world.trigger_event(StochasticEventKind::IceWormClathrateGraze);
                                }
                                if ui.small_button("Tubeworms").clicked() {
                                    self.world.trigger_event(StochasticEventKind::LamellibrachiaRootUptake);
                                }
                                if ui.small_button("Red Crabs").clicked() {
                                    self.world.trigger_event(StochasticEventKind::RedCrabMusselPatrol);
                                }
                            } else if is_rift {
                                if ui.small_button("DVM Ascent").clicked() {
                                    self.world.trigger_event(StochasticEventKind::DielVerticalMigrationSurge);
                                }
                                if ui.small_button("Marine Snow").clicked() {
                                    self.world.trigger_event(StochasticEventKind::MarineSnowMucusFall);
                                }
                                if ui.small_button("Vampire Glow").clicked() {
                                    self.world.trigger_event(StochasticEventKind::VampyroteuthisMucusCloud);
                                }
                                if ui.small_button("Barreleye").clicked() {
                                    self.world.trigger_event(StochasticEventKind::BarreleyeDomeTargetLock);
                                }
                                if ui.small_button("Whale Sonar").clicked() {
                                    self.world.trigger_event(StochasticEventKind::SpermWhaleSonarDive);
                                }
                                if ui.small_button("Giant Squid").clicked() {
                                    self.world.trigger_event(StochasticEventKind::ArchiteuthisJetEncounter);
                                }
                            } else if is_pnw {
                                if ui.small_button("Coho Leap").clicked() {
                                    self.world.trigger_event(StochasticEventKind::CohoSalmonReddLeap);
                                }
                                if ui.small_button("15N Hyphae").clicked() {
                                    self.world.trigger_event(StochasticEventKind::MycorrhizalNitrogenPulse);
                                }
                                if ui.small_button("Dipper Walk").clicked() {
                                    self.world.trigger_event(StochasticEventKind::DipperUnderwaterWalk);
                                }
                                if ui.small_button("Banana Slug").clicked() {
                                    self.world.trigger_event(StochasticEventKind::BananaSlugSporeTrail);
                                }
                                if ui.small_button("Spotted Owl").clicked() {
                                    self.world.trigger_event(StochasticEventKind::SpottedOwlDuskGlide);
                                }
                                if ui.small_button("Mist Drizzle").clicked() {
                                    self.world.trigger_event(StochasticEventKind::PacificDrizzleSunbeam);
                                }
                            } else if is_vent {
                                if ui.small_button("360 degC Surge").clicked() {
                                    self.world.trigger_event(StochasticEventKind::BlackSmokerSuperPlume);
                                }
                                if ui.small_button("Ghost Shark").clicked() {
                                    self.world.trigger_event(StochasticEventKind::GhostSharkAbyssalGlide);
                                }
                                if ui.small_button("Atolla Alarm").clicked() {
                                    self.world.trigger_event(StochasticEventKind::AtollaBiolumBurglarAlarm);
                                }
                                if ui.small_button("Yeti Dance").clicked() {
                                    self.world.trigger_event(StochasticEventKind::YetiCrabBacterialDance);
                                }
                                if ui.small_button("Vent Octopus").clicked() {
                                    self.world.trigger_event(StochasticEventKind::VulcanoctopusTubewormHunt);
                                }
                                if ui.small_button("Sulfide Crack").clicked() {
                                    self.world.trigger_event(StochasticEventKind::SulfideFissureFlocRelease);
                                }
                            } else if is_lake {
                                if ui.small_button("Osprey Dive").clicked() {
                                    self.world.trigger_event(StochasticEventKind::OspreyPlungeDive);
                                }
                                if ui.small_button("Loon Dive").clicked() {
                                    self.world.trigger_event(StochasticEventKind::CommonLoonSubmergeDive);
                                }
                                if ui.small_button("Mayfly Rise").clicked() {
                                    self.world.trigger_event(StochasticEventKind::MayflySurfaceHatch);
                                }
                                if ui.small_button("Mallard Dabble").clicked() {
                                    self.world.trigger_event(StochasticEventKind::MallardDabbleRipple);
                                }
                                if ui.small_button("Crayfish Plume").clicked() {
                                    self.world.trigger_event(StochasticEventKind::CrayfishLeafPackPlume);
                                }
                                if ui.small_button("Loon & Fireflies").clicked() {
                                    self.world.trigger_event(StochasticEventKind::FireflyLoonNocturne);
                                }
                            } else if is_coral {
                                if ui.small_button("Tidal Flush").clicked() {
                                    self.world.trigger_event(StochasticEventKind::LagoonTidalFlush);
                                }
                                if ui.small_button("Parrotfish Sand").clicked() {
                                    self.world.trigger_event(StochasticEventKind::ParrotfishSandCloud);
                                }
                                if ui.small_button("Hermit Plume").clicked() {
                                    self.world.trigger_event(StochasticEventKind::HermitCrabMucusPlume);
                                }
                                if ui.small_button("Turtle Breath").clicked() {
                                    self.world.trigger_event(StochasticEventKind::SeaTurtleSurfaceBreath);
                                }
                                if ui.small_button("Shark Pass").clicked() {
                                    self.world.trigger_event(StochasticEventKind::BlacktipSharkPass);
                                }
                                if ui.small_button("Coral GFP").clicked() {
                                    self.world.trigger_event(StochasticEventKind::CoralGfpBiofluorescence);
                                }
                            } else if is_kelp {
                                if ui.small_button("Upwelling").clicked() {
                                    self.world
                                        .trigger_event(StochasticEventKind::CoastalUpwellingPulse);
                                }
                                if ui.small_button("Drift Kelp").clicked() {
                                    self.world
                                        .trigger_event(StochasticEventKind::DriftKelpDetachment);
                                }
                                if ui.small_button("Crab Plume").clicked() {
                                    self.world
                                        .trigger_event(StochasticEventKind::CrabScentPlumeWave);
                                }
                                if ui.small_button("Otter Dive").clicked() {
                                    self.world
                                        .trigger_event(StochasticEventKind::SeaOtterUrchinDive);
                                }
                                if ui.small_button("Seal Glide").clicked() {
                                    self.world
                                        .trigger_event(StochasticEventKind::HarborSealKelpGlide);
                                }
                                if ui.small_button("Octopus Jet").clicked() {
                                    self.world.trigger_event(
                                        StochasticEventKind::OctopusChromatophoreShift,
                                    );
                                }
                                if ui.small_button("Plankton Glow").clicked() {
                                    self.world.trigger_event(
                                        StochasticEventKind::DinoflagellateBiolumSwirl,
                                    );
                                }
                            } else {
                                if ui.small_button("Rain Squall").clicked() {
                                    self.world
                                        .trigger_event(StochasticEventKind::TropicalSunshower);
                                }
                                if ui.small_button("Fruit Drop").clicked() {
                                    self.world
                                        .trigger_event(StochasticEventKind::CanopyFruitMastDrop);
                                }
                                if ui.small_button("Ant Trail").clicked() {
                                    self.world
                                        .trigger_event(StochasticEventKind::AntTrailForagingWave);
                                }
                                if ui.small_button("Monkey Swing").clicked() {
                                    self.world
                                        .trigger_event(StochasticEventKind::MonkeyCanopyTransit);
                                }
                                if ui.small_button("Panther Stalk").clicked() {
                                    self.world
                                        .trigger_event(StochasticEventKind::PantherUnderbrushProwl);
                                }
                                if ui.small_button("Boa Sense").clicked() {
                                    self.world.trigger_event(StochasticEventKind::BoaAmbushAlert);
                                }
                                if ui.small_button("Mycena Glow").clicked() {
                                    self.world.trigger_event(
                                        StochasticEventKind::BioluminescentSporePulse,
                                    );
                                }
                            }
                        });

                        ui.separator();
                        ui.heading("Keystone Cascade & Sparkline");
                        ui.small(format!("Keystone: {}", self.world.biome.keystone_guild_label()));
                        let (spark_rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 52.0), Sense::hover());
                        ui.painter().rect_filled(spark_rect, egui::CornerRadius::same(6), Color32::from_rgb(8, 22, 20));
                        let hist = &self.world.ledger.biomass_history;
                        if hist.len() >= 2 {
                            for &(series_idx, col) in &[
                                (1_usize, Color32::from_rgb(74, 222, 128)),
                                (2_usize, Color32::from_rgb(56, 189, 248)),
                                (4_usize, Color32::from_rgb(251, 113, 133)),
                            ] {
                                let base_v = TrophicEnergyLedger::baseline_for_biome(self.world.biome)[series_idx].max(1.0);
                                let pts: Vec<egui::Pos2> = hist
                                    .iter()
                                    .enumerate()
                                    .map(|(i, sample)| {
                                        let x = spark_rect.left() + 4.0 + (i as f32 / (hist.len() - 1) as f32) * (spark_rect.width() - 8.0);
                                        let ratio = (sample[series_idx] / (base_v * 2.3)).clamp(0.04, 0.96);
                                        let y = spark_rect.bottom() - 4.0 - ratio * (spark_rect.height() - 8.0);
                                        egui::pos2(x, y)
                                    })
                                    .collect();
                                ui.painter().add(egui::Shape::line(pts, egui::Stroke::new(1.6_f32, col)));
                            }
                        }

                        ui.separator();
                        ui.heading("Trophic Energy Pyramid");
                        for stratum in TrophicStratum::ALL.into_iter().rev() {
                            let kcal = self.world.ledger.biomass_kcal[stratum.index()];
                            let eff = self.world.ledger.transfer_efficiency_pct(stratum);
                            let [r, g, b] = stratum.badge_rgb();
                            ui.colored_label(
                                Color32::from_rgb(r, g, b),
                                format!("{}: {:.0} kcal ({:.1}%)", stratum.short_label(), kcal, eff),
                            );
                        }
                        let phero = self.world.floor_field.total_active_pheromone();
                        let active_items: u32 = self
                            .world
                            .ant_patches
                            .iter()
                            .map(|p| p.remaining_fragments)
                            .sum();
                        if is_tidepool {
                            ui.small(format!(
                                "Kelp Wrack Scent: {:.1} u (Fragments: {})",
                                phero, active_items
                            ));
                            ui.small(format!(
                                "Intertidal Wrack: {:.1} g | Upwelled DIN: {:.1} mg",
                                self.world.ledger.detritus_pool_g,
                                self.world.ledger.mineral_nutrient_pool_mg
                            ));
                        } else if is_brine {
                            ui.small(format!(
                                "Dissolved CH4 Plume: {:.1} u (Clathrate bits: {})",
                                phero, active_items
                            ));
                            ui.small(format!(
                                "Seep Carbon Pool: {:.1} g | CH4/H2S Flux: {:.1} mM",
                                self.world.ledger.detritus_pool_g,
                                self.world.ledger.mineral_nutrient_pool_mg
                            ));
                        } else if is_rift {
                            ui.small(format!(
                                "Marine Snow Eddy: {:.1} u (Mucus houses: {})",
                                phero, active_items
                            ));
                            ui.small(format!(
                                "Phytodetritus Flux: {:.1} g | Upwelled NO3/PO4: {:.1} mg",
                                self.world.ledger.detritus_pool_g,
                                self.world.ledger.mineral_nutrient_pool_mg
                            ));
                        } else if is_pnw {
                            ui.small(format!(
                                "Slug Spore Mucus: {:.1} u (Chanterelle bits: {})",
                                phero, active_items
                            ));
                            ui.small(format!(
                                "Nurse-Log Humus: {:.1} g | Mycorrhizal 15N: {:.1} mg",
                                self.world.ledger.detritus_pool_g,
                                self.world.ledger.mineral_nutrient_pool_mg
                            ));
                        } else if is_vent {
                            ui.small(format!(
                                "H2S Sulfide Plume: {:.1} u (Floc bits: {})",
                                phero, active_items
                            ));
                            ui.small(format!(
                                "Chemosynthetic Floc: {:.1} g | H2S/CH4/Fe2+: {:.1} mM",
                                self.world.ledger.detritus_pool_g,
                                self.world.ledger.mineral_nutrient_pool_mg
                            ));
                        } else if is_lake {
                            ui.small(format!(
                                "Leaf-Pack Plume: {:.1} u (Alder bits: {})",
                                phero, active_items
                            ));
                            ui.small(format!(
                                "Lakebed Detritus: {:.1} g | Dissolved N/P: {:.1} mg",
                                self.world.ledger.detritus_pool_g,
                                self.world.ledger.mineral_nutrient_pool_mg
                            ));
                        } else if is_coral {
                            ui.small(format!(
                                "Coral Mucus Plume: {:.1} u (Mucus bits: {})",
                                phero, active_items
                            ));
                            ui.small(format!(
                                "Carbonate Detritus: {:.1} g | Lagoon DIN/DIP: {:.1} mg",
                                self.world.ledger.detritus_pool_g,
                                self.world.ledger.mineral_nutrient_pool_mg
                            ));
                        } else if is_kelp {
                            ui.small(format!(
                                "Benthic Scent Plume: {:.1} u (Drift bits: {})",
                                phero, active_items
                            ));
                            ui.small(format!(
                                "Marine Detritus: {:.1} g | Upwelled NO3/PO4: {:.1} mg",
                                self.world.ledger.detritus_pool_g,
                                self.world.ledger.mineral_nutrient_pool_mg
                            ));
                        } else {
                            ui.small(format!(
                                "Ant Trail Pheromone: {:.1} u (Patch leaves: {})",
                                phero, active_items
                            ));
                            ui.small(format!(
                                "Soil Detritus: {:.1} g | Mineral N/P: {:.1} mg",
                                self.world.ledger.detritus_pool_g,
                                self.world.ledger.mineral_nutrient_pool_mg
                            ));
                        }

                        ui.separator();
                        ui.heading("Selected Organism");
                        if let Some(sel_id) = self.selected_organism_id {
                            if let Some(org) = self.world.organisms.iter().find(|o| o.id == sel_id)
                            {
                                ui.strong(org.species.common_name());
                                ui.label(RichText::new(org.species.scientific_name()).italics());
                                ui.label(format!("Stratum: {}", org.stratum().short_label()));
                                ui.label(format!("Behavior: {}", org.state.description()));
                                ui.label(format!("Velocity: {:.2} m/s", org.speed_m_s()));
                                ui.label(format!(
                                    "Energy Reserve: {:.1} / {:.1} kcal",
                                    org.energy_kcal, org.max_energy_kcal
                                ));
                            }
                        } else {
                            ui.weak("Click any organism in the 3D viewport to inspect its live ethology.");
                        }

                        ui.separator();
                        ui.heading("Field Observation Log");
                        egui::ScrollArea::vertical()
                            .max_height(150.0)
                            .show(ui, |ui| {
                                for entry in self.world.events.log.iter().rev().take(12) {
                                    ui.small(
                                        RichText::new(format!(
                                            "[{}] {}",
                                            entry.formatted_clock(),
                                            entry.title
                                        ))
                                        .strong(),
                                    );
                                    ui.small(&entry.detail);
                                    ui.add_space(4.0);
                                }
                            });

                        ui.separator();
                        ui.heading("Storage, Export & Snapshots");
                        ui.horizontal_wrapped(|ui| {
                            if ui.small_button("Export Snapshot (.json)").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Trophic Snapshot JSON", &["json"])
                                    .set_file_name(format!("{}.trophic.json", self.slot_name_input))
                                    .save_file()
                                {
                                    let snap = TrophicSnapshotV1::from_world(
                                        &self.slot_name_input,
                                        self.world.clone(),
                                    );
                                    match serde_json::to_string_pretty(&snap)
                                        .map_err(|e| format!("JSON serialize error: {e}"))
                                        .and_then(|json| {
                                            std::fs::write(&path, json)
                                                .map_err(|e| format!("Write failed: {e}"))
                                        }) {
                                        Ok(()) => {
                                            self.status_banner =
                                                format!("Exported snapshot to {}", path.display());
                                        }
                                        Err(e) => self.status_banner = e,
                                    }
                                }
                            }
                            if ui.small_button("Import Snapshot (.json)").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Trophic Snapshot JSON", &["json"])
                                    .pick_file()
                                {
                                    match std::fs::read_to_string(&path)
                                        .map_err(|e| format!("Read failed: {e}"))
                                        .and_then(|txt| {
                                            serde_json::from_str::<TrophicSnapshotV1>(&txt)
                                                .map_err(|e| format!("Invalid snapshot JSON: {e}"))
                                        }) {
                                        Ok(snap) => {
                                            self.world = snap.world;
                                            self.selected_organism_id = None;
                                            self.camera.apply_preset(CameraPreset::SideDiorama);
                                            self.status_banner =
                                                format!("Imported snapshot '{}'", snap.slot_name);
                                        }
                                        Err(e) => self.status_banner = e,
                                    }
                                }
                            }
                            if ui.small_button("Export Soundscape (.wav)").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Stereo WAV Audio", &["wav"])
                                    .set_file_name("trophic_soundscape.wav")
                                    .save_file()
                                {
                                    match export_ambient_soundscape_wav(&path, 12.0) {
                                        Ok(()) => {
                                            self.status_banner =
                                                format!("Exported WAV to {}", path.display());
                                        }
                                        Err(e) => {
                                            self.status_banner = format!("WAV export error: {e}");
                                        }
                                    }
                                }
                            }
                            if ui.small_button("Open SQLite DB (.sqlite3)").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("SQLite Database", &["sqlite3", "sqlite", "db"])
                                    .pick_file()
                                {
                                    match PersistenceStore::open(&path) {
                                        Ok(new_store) => {
                                            self.slots_cache =
                                                new_store.list_slots().unwrap_or_default();
                                            self.store = Some(new_store);
                                            self.status_banner =
                                                format!("Opened SQLite DB {}", path.display());
                                        }
                                        Err(e) => {
                                            self.status_banner =
                                                format!("SQLite open error: {e}");
                                        }
                                    }
                                }
                            }
                        });

                        if self.store.is_some() {
                            ui.add_space(4.0);
                            ui.label(RichText::new("SQLite WAL Snapshots").strong());
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(&mut self.slot_name_input);
                                if ui.button("Save").clicked() {
                                    if let Some(ref st) = self.store {
                                        match st.save_world(&self.slot_name_input, &self.world) {
                                            Ok(()) => {
                                                self.status_banner =
                                                    format!("Saved '{}'", self.slot_name_input);
                                                self.slots_cache =
                                                    st.list_slots().unwrap_or_default();
                                            }
                                            Err(e) => self.status_banner = e,
                                        }
                                    }
                                }
                            });
                            let mut load_target = None;
                            for slot in &self.slots_cache {
                                if ui
                                    .small_button(format!(
                                        "Load {} ({}, Day {})",
                                        slot.slot_name, slot.biome_name, slot.day_count
                                    ))
                                    .clicked()
                                {
                                    load_target = Some(slot.slot_name.clone());
                                }
                            }
                            if let (Some(name), Some(ref st)) = (load_target, &self.store) {
                                match st.load_world(&name) {
                                    Ok(w) => {
                                        self.world = w;
                                        self.selected_organism_id = None;
                                        self.status_banner = format!("Restored '{}'", name);
                                    }
                                    Err(e) => {
                                        self.status_banner = format!("Failed loading '{name}': {e}");
                                    }
                                }
                            }
                        }
                        if !self.status_banner.is_empty() {
                            ui.small(&self.status_banner);
                        }
                    });
            }
        }

        egui::TopBottomPanel::bottom("hearth_bottom_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.small("Copyright CRV Science Network  |  GPL-3.0-or-later  |  TrophicHearth v0.9.3");
                ui.separator();
                if ui.small_button("About TrophicHearth").clicked() {
                    self.show_about_window = true;
                }
                ui.hyperlink_to("crvscience.com", "https://crvscience.com");
                if !self.status_banner.is_empty() {
                    ui.separator();
                    ui.weak(&self.status_banner);
                }
            });
        });

        if self.show_about_window {
            let mut open = self.show_about_window;
            egui::Window::new("About TrophicHearth")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.heading("TrophicHearth v0.9.3");
                    ui.label("Copyright CRV Science Network");
                    ui.label("License: GNU General Public License v3.0 or later (GPL-3.0-or-later)");
                    ui.separator();
                    ui.label("Nine-biome 3D living trophic diorama, fireplace simulation,");
                    ui.label("interactive keystone cascade observatory, diurnal atmosphere,");
                    ui.label("and procedural stereo bioacoustic soundscape workstation.");
                    ui.add_space(6.0);
                    ui.hyperlink_to(
                        "CRV Science Network Website (https://crvscience.com)",
                        "https://crvscience.com",
                    );
                    ui.hyperlink_to(
                        "CRV Science Public Apps (https://crvscience.com/apps)",
                        "https://crvscience.com/apps",
                    );
                    ui.add_space(6.0);
                    if ui.button("Close").clicked() {
                        self.show_about_window = false;
                    }
                });
            if !open {
                self.show_about_window = false;
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let rect = ui.available_rect_before_wrap();
                let resp = ui.allocate_rect(rect, Sense::click_and_drag());

                if resp.dragged_by(egui::PointerButton::Primary) {
                    let delta = resp.drag_delta();
                    self.camera.preset = CameraPreset::FreeOrbit;
                    self.camera.yaw_rad =
                        (self.camera.yaw_rad - delta.x * 0.005).clamp(-0.95, 0.95);
                    self.camera.pitch_rad =
                        (self.camera.pitch_rad + delta.y * 0.004).clamp(-0.22, 0.65);
                }
                if resp.dragged_by(egui::PointerButton::Secondary) {
                    let delta = resp.drag_delta();
                    self.camera.preset = CameraPreset::FreeOrbit;
                    self.camera.target.x =
                        (self.camera.target.x - delta.x * 0.025).clamp(-10.0, 10.0);
                    self.camera.target.y =
                        (self.camera.target.y + delta.y * 0.025).clamp(-0.5, 13.5);
                }
                let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                if scroll.abs() > 0.1 {
                    self.camera.distance_m =
                        (self.camera.distance_m - scroll * 0.025).clamp(4.5, 32.0);
                }

                let painter = ui.painter_at(rect);
                let hitboxes = render_rainforest_viewport(
                    &painter,
                    rect,
                    &self.world,
                    &self.camera,
                    self.selected_organism_id,
                );

                if resp.clicked() {
                    if let Some(click_pos) = resp.interact_pointer_pos() {
                        let mut best: Option<(u32, f32)> = None;
                        for (oid, sp) in hitboxes {
                            let d = sp.distance(click_pos);
                            if d < 26.0 && best.map_or(true, |(_, bd)| d < bd) {
                                best = Some((oid, d));
                            }
                        }
                        self.selected_organism_id = best.map(|(id, _)| id);
                    }
                }

                if self.zen_hearth_mode {
                    let pill = egui::Rect::from_min_size(
                        rect.right_top() + egui::vec2(-165.0, 12.0),
                        egui::vec2(152.0, 26.0),
                    );
                    painter.rect_filled(
                        pill,
                        13.0,
                        Color32::from_rgba_unmultiplied(12, 22, 24, 155),
                    );
                    painter.text(
                        pill.center(),
                        egui::Align2::CENTER_CENTER,
                        "Hearth Mode (Tab)",
                        egui::FontId::proportional(12.0),
                        Color32::from_rgb(185, 238, 225),
                    );
                    if resp.clicked() {
                        if let Some(p) = resp.interact_pointer_pos() {
                            if pill.contains(p) {
                                self.zen_hearth_mode = false;
                            }
                        }
                    }
                }
            });
    }
}

impl eframe::App for TrophicHearthApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui_step(ctx, true);
        ctx.request_repaint();
    }
}

struct NoopWaker;

impl std::task::Wake for NoopWaker {
    fn wake(self: std::sync::Arc<Self>) {}
}

fn block_on_future<F: std::future::Future>(fut: F) -> F::Output {
    let mut fut = std::pin::pin!(fut);
    let waker = std::task::Waker::from(std::sync::Arc::new(NoopWaker));
    let mut cx = std::task::Context::from_waker(&waker);
    loop {
        match fut.as_mut().poll(&mut cx) {
            std::task::Poll::Ready(val) => return val,
            std::task::Poll::Pending => std::thread::yield_now(),
        }
    }
}

fn write_rgba_png(
    path: &std::path::Path,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let expected = (width as usize) * (height as usize) * 4;
    if rgba.len() != expected {
        return Err(format!(
            "Unexpected RGBA buffer length {} (expected {})",
            rgba.len(),
            expected
        )
        .into());
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let file = std::fs::File::create(path)?;
    let writer = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::High);
    encoder.set_filter(png::Filter::Adaptive);
    let mut png_writer = encoder.write_header()?;
    png_writer.write_image_data(rgba)?;
    png_writer.finish()?;
    Ok(())
}

pub fn render_headless_screenshot(
    out_path: &std::path::Path,
    view: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    use eframe::egui_wgpu::{wgpu, Renderer, ScreenDescriptor};

    let width: u32 = 1600;
    let height: u32 = 1000;
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = block_on_future(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok_or("No GPU adapter available for offscreen rendering")?;
    let (device, queue) = block_on_future(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("trophic-hearth-offscreen-device"),
            ..Default::default()
        },
        None,
    ))?;

    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("trophic-hearth-offscreen-target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let bytes_per_row = width * 4;
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("trophic-hearth-offscreen-staging"),
        size: (bytes_per_row * height) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut renderer = Renderer::new(&device, format, None, 1, false);
    let egui_ctx = egui::Context::default();
    egui_ctx.set_pixels_per_point(1.0);
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = Color32::from_rgb(12, 18, 20);
    visuals.panel_fill = Color32::from_rgb(14, 21, 24);
    egui_ctx.set_visuals(visuals);

    let (biome, hour, cam_preset, trigger_vent_plume) = match view {
        2 => (
            BiomeKind::IndoPacificCoralReef,
            11.5_f32,
            CameraPreset::UnderbrushStalkMacro,
            false,
        ),
        3 => (
            BiomeKind::DeepSeaHydrothermalVent,
            2.0_f32,
            CameraPreset::SideDiorama,
            true,
        ),
        _ => (
            BiomeKind::RockyIntertidalTidepool,
            10.5_f32,
            CameraPreset::SideDiorama,
            false,
        ),
    };

    let mut world = RainforestWorld::new_for_biome(biome, 2026_1116);
    world.diurnal.hour_of_day = hour;
    world.ledger.show_flux_arcs = true;
    if trigger_vent_plume {
        world.trigger_event(StochasticEventKind::BlackSmokerSuperPlume);
    }
    for _ in 0..60 {
        world.step(0.05);
        world.pending_sounds.clear();
    }

    let first_org = world.organisms.first().map(|o| o.id);
    let mut app = TrophicHearthApp::new_headless(world, false);
    app.camera.apply_preset(cam_preset);
    if view == 1 {
        app.selected_organism_id = first_org;
    }
    app.show_inspector_drawer = true;

    let warmup_delays_ms = [40u64, 20, 20, 20];
    for (frame_idx, delay_ms) in warmup_delays_ms.into_iter().enumerate() {
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width as f32, height as f32),
            )),
            viewports: std::iter::once((
                egui::ViewportId::ROOT,
                egui::ViewportInfo {
                    native_pixels_per_point: Some(1.0),
                    ..Default::default()
                },
            ))
            .collect(),
            time: Some(frame_idx as f64 / 60.0),
            predicted_dt: 1.0 / 60.0,
            ..Default::default()
        };
        let full_output = egui_ctx.run(raw_input, |ctx| {
            app.ui_step(ctx, false);
        });
        for (id, image_delta) in &full_output.textures_delta.set {
            renderer.update_texture(&device, &queue, *id, image_delta);
        }
        let paint_jobs = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: 1.0,
        };
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("trophic-hearth-offscreen-encoder"),
        });
        let user_cmd_bufs = renderer.update_buffers(
            &device,
            &queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("trophic-hearth-offscreen-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            let mut render_pass = render_pass.forget_lifetime();
            renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }
        if frame_idx == 3 {
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &staging,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(bytes_per_row),
                        rows_per_image: Some(height),
                    },
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
        }
        queue.submit(user_cmd_bufs.into_iter().chain(std::iter::once(encoder.finish())));
        device.poll(wgpu::Maintain::Wait);
        for id in &full_output.textures_delta.free {
            renderer.free_texture(id);
        }
    }

    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |res| {
        let _ = tx.send(res);
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv()??;
    let mapped = slice.get_mapped_range();
    write_rgba_png(out_path, width, height, &mapped)?;
    drop(mapped);
    staging.unmap();
    Ok(())
}
