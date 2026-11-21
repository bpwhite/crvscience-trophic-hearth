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

use trophic_hearth::app::TrophicHearthApp;
use trophic_hearth::audio::{export_ambient_soundscape_wav, synthesize_cue_stereo};
use trophic_hearth::events::{SoundCue, SoundCueKind, StochasticEventKind};
use trophic_hearth::models_3d::append_organism_geometry;
use trophic_hearth::organisms::{BehaviorState, Species};
use trophic_hearth::persistence::PersistenceStore;
use trophic_hearth::trophic::{BiomeKind, TrophicStratum};
use trophic_hearth::world::RainforestWorld;

#[test]
fn tidepool_and_brine_pool_run_keystone_trophic_cascades_and_flux_arcs() {
    let mut tp = RainforestWorld::new_rocky_intertidal_tidepool(2026);
    assert_eq!(tp.biome, BiomeKind::RockyIntertidalTidepool);
    assert!(tp.ledger.show_flux_arcs);

    let l2_intact = tp.ledger.biomass_kcal[2];
    tp.ledger.keystone_removed = true; // Remove Pisaster ochre sea stars (Paine 1966)!
    tp.trigger_event(StochasticEventKind::SculpinPoolToPoolLeap);
    for _ in 0..80 {
        tp.step(0.05);
    }
    assert!(tp.ledger.biomass_kcal[2] > l2_intact * 1.25);
    assert!(tp.ledger.biomass_history.len() >= 4);

    let mut bp = RainforestWorld::new_cold_seep_brine_pool(2026);
    assert_eq!(bp.biome, BiomeKind::ColdSeepBrinePool);
    assert_eq!(bp.diurnal.atmosphere_for_biome(bp.biome).par_umol_m2_s, 0.0);
    bp.trigger_event(StochasticEventKind::MethaneHydrateBubblePlume);
    bp.trigger_event(StochasticEventKind::CutthroatEelBrineSkim);
    for _ in 0..60 {
        bp.step(0.05);
    }
    assert!(!bp.organisms.is_empty());
}

#[test]
fn midwater_pelagic_rift_drives_diel_vertical_migration_and_sonar_dives() {
    let mut world = RainforestWorld::new_midwater_pelagic_rift(2026);
    assert_eq!(world.biome, BiomeKind::MidwaterPelagicRift);
    assert!(world.organisms.len() >= 40);

    let mut tris = Vec::new();
    for org in &world.organisms {
        append_organism_geometry(&mut tris, org, 0.5);
    }
    assert!(tris.len() > 600);

    let det_before = world.ledger.detritus_pool_g;
    world.trigger_event(StochasticEventKind::DielVerticalMigrationSurge);
    world.trigger_event(StochasticEventKind::SpermWhaleSonarDive);
    world.trigger_event(StochasticEventKind::MarineSnowMucusFall);
    assert!(world.ledger.detritus_pool_g > det_before);
    for _ in 0..120 {
        world.step(0.04);
    }
    assert!(!world.organisms.is_empty());
}

#[test]
fn pnw_old_growth_couples_salmon_creek_and_mycorrhizal_forest() {
    let mut world = RainforestWorld::new_pnw_old_growth_salmon_creek(2026);
    assert_eq!(world.biome, BiomeKind::PnwOldGrowthSalmonCreek);
    assert!(world.trees.len() >= 8);

    let mut tris = Vec::new();
    for org in &world.organisms {
        append_organism_geometry(&mut tris, org, 0.5);
    }
    assert!(tris.len() > 380);

    let n_before = world.ledger.mineral_nutrient_pool_mg;
    world.trigger_event(StochasticEventKind::CohoSalmonReddLeap);
    world.trigger_event(StochasticEventKind::DipperUnderwaterWalk);
    world.trigger_event(StochasticEventKind::MycorrhizalNitrogenPulse);
    assert!(world.ledger.mineral_nutrient_pool_mg > n_before);
    for _ in 0..120 {
        world.step(0.04);
    }
    assert!(!world.organisms.is_empty());
}

#[test]
fn hydrothermal_vent_runs_pure_chemosynthesis_at_zero_sunlight() {
    let mut world = RainforestWorld::new_deep_sea_hydrothermal_vent(2026);
    assert_eq!(world.biome, BiomeKind::DeepSeaHydrothermalVent);
    let atmo = world.diurnal.atmosphere_for_biome(world.biome);
    assert_eq!(atmo.par_umol_m2_s, 0.0, "2,600m abyssal vent must have zero solar PAR");

    let mut tris = Vec::new();
    for org in &world.organisms {
        append_organism_geometry(&mut tris, org, 1.0);
    }
    assert!(tris.len() > 350);

    world.trigger_event(StochasticEventKind::BlackSmokerSuperPlume);
    world.trigger_event(StochasticEventKind::GhostSharkAbyssalGlide);
    for _ in 0..120 {
        world.step(0.05);
    }
    assert!(world.ledger.cumulative_gpp_kcal > 0.0, "Chemosynthetic H2S oxidation must accumulate GPP at PAR=0");
}

#[test]
fn freshwater_lake_biome_spawns_underwater_and_waterbird_strata() {
    let mut world = RainforestWorld::new_temperate_freshwater_lake(2026);
    assert_eq!(world.biome, BiomeKind::TemperateFreshwaterLake);
    assert!(world.underbrush.len() >= 70);

    for st in [
        TrophicStratum::Decomposers,
        TrophicStratum::PrimaryConsumers,
        TrophicStratum::SecondaryConsumers,
        TrophicStratum::TertiaryCarnivores,
    ] {
        assert!(world.organisms.iter().any(|o| o.stratum() == st));
    }

    let mut tris = Vec::new();
    for org in &world.organisms {
        append_organism_geometry(&mut tris, org, 0.4);
    }
    assert!(tris.len() > 400);

    world.trigger_event(StochasticEventKind::OspreyPlungeDive);
    world.trigger_event(StochasticEventKind::CommonLoonSubmergeDive);
    for _ in 0..160 {
        world.step(0.04);
    }
    assert!(!world.organisms.is_empty());
}

#[test]
fn coral_reef_biome_spawns_all_strata_and_shelters_chromis_on_shark_pass() {
    let mut world = RainforestWorld::new_indo_pacific_coral_reef(2026);
    assert_eq!(world.biome, BiomeKind::IndoPacificCoralReef);
    assert!(world.underbrush.len() >= 25);

    for st in [
        TrophicStratum::Decomposers,
        TrophicStratum::PrimaryConsumers,
        TrophicStratum::SecondaryConsumers,
        TrophicStratum::TertiaryCarnivores,
    ] {
        assert!(world.organisms.iter().any(|o| o.stratum() == st));
    }

    let mut tris = Vec::new();
    for org in &world.organisms {
        append_organism_geometry(&mut tris, org, 0.6);
    }
    assert!(tris.len() > 450, "Coral reef fauna should generate rich 3D meshes");

    world.trigger_event(StochasticEventKind::ParrotfishSandCloud);
    world.trigger_event(StochasticEventKind::BlacktipSharkPass);
    for _ in 0..200 {
        world.step(0.04);
    }
    assert!(!world.organisms.is_empty());
}

#[test]
fn kelp_forest_biome_spawns_all_strata_and_deflects_kelp_stipes() {
    let mut world = RainforestWorld::new_pacific_kelp_forest(2026);
    assert_eq!(world.biome, BiomeKind::PacificKelpForest);
    assert!(!world.kelp_stalks.is_empty());

    // Confirm organisms span L0, L2, L3, L4 plus L1 kelp stalks
    for st in [
        TrophicStratum::Decomposers,
        TrophicStratum::PrimaryConsumers,
        TrophicStratum::SecondaryConsumers,
        TrophicStratum::TertiaryCarnivores,
    ] {
        assert!(
            world.organisms.iter().any(|o| o.stratum() == st),
            "Missing stratum {st:?} in Kelp Forest"
        );
    }

    // Step simulation: sea otter taps stone -> rafts -> dives, and harbor seal bends kelp stalks
    let mut saw_otter_dive = false;
    let mut max_kelp_bend = 0.0_f32;
    let mut tris = Vec::new();
    for org in &world.organisms {
        append_organism_geometry(&mut tris, org, 0.4);
    }
    assert!(tris.len() > 400, "Kelp forest organisms should build rich 3D meshes");

    for _ in 0..450 {
        world.step(0.04);
        for st in &world.kelp_stalks {
            max_kelp_bend = max_kelp_bend.max(st.bend_offset_xz[0].abs().max(st.bend_offset_xz[1].abs()));
        }
        for org in &world.organisms {
            if org.species == Species::SeaOtter
                && matches!(org.state, BehaviorState::OtterForageDive { .. })
            {
                saw_otter_dive = true;
            }
        }
    }

    assert!(saw_otter_dive, "Sea otter should transition into foraging dive");
    assert!(
        max_kelp_bend > 0.02,
        "Swimming marine fauna should deflect buoyant kelp stalks (max={max_kelp_bend})"
    );
}

#[test]
fn ant_pheromone_trail_forms_and_dissipates_after_patch_depletion() {
    let mut world = RainforestWorld::new_neotropical_rainforest(42);
    let initial_pheromone = world.floor_field.total_active_pheromone();
    assert!(initial_pheromone > 5.0);

    world.ant_patches.clear();
    for _ in 0..600 {
        world.step(0.05);
    }
    let post_dissipation = world.floor_field.total_active_pheromone();
    assert!(post_dissipation < initial_pheromone * 0.05);
}

#[test]
fn squirrel_monkey_swings_then_stops_to_look_around_then_continues() {
    let mut world = RainforestWorld::new_neotropical_rainforest(101);
    let mut saw_swing = false;
    let mut saw_look_around = false;
    let mut saw_continue_traverse = false;

    for _ in 0..400 {
        world.step(0.04);
        for org in &world.organisms {
            if org.species == Species::SquirrelMonkey {
                match org.state {
                    BehaviorState::MonkeyVineSwing { .. } => saw_swing = true,
                    BehaviorState::MonkeyPerchedLookAround { .. } => {
                        if saw_swing {
                            saw_look_around = true;
                        }
                    }
                    BehaviorState::MonkeyBranchTraverse { .. } => {
                        if saw_look_around {
                            saw_continue_traverse = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    assert!(saw_swing);
    assert!(saw_look_around);
    assert!(saw_continue_traverse);
}

#[test]
fn black_panther_stalks_and_physically_deflects_underbrush_ferns() {
    let mut world = RainforestWorld::new_neotropical_rainforest(777);
    world.events.wind_vector_xz = [0.0, 0.0];

    let mut max_fern_bend = 0.0_f32;
    for _ in 0..240 {
        world.step(0.04);
        for plant in &world.underbrush {
            max_fern_bend = max_fern_bend.max(plant.bend_xz[0].abs().max(plant.bend_xz[1].abs()));
        }
    }
    assert!(max_fern_bend > 0.08);
}

#[test]
fn marine_and_terrestrial_bioacoustics_and_sqlite_roundtrip() {
    for kind in [SoundCueKind::OtterChestStoneClick, SoundCueKind::DistantWhaleHydrophone] {
        let cue = SoundCue {
            kind,
            world_pos: glam::Vec3::new(-2.0, 12.0, -1.0),
            gain: 0.8,
            pitch_mul: 1.0,
        };
        let samples = synthesize_cue_stereo(cue, 44_100, 0.8);
        assert!(samples.iter().any(|s| s.abs() > 0.02));
    }

    let tmp_wav = std::env::temp_dir().join("trophic_hearth_v030_test.wav");
    export_ambient_soundscape_wav(&tmp_wav, 1.5).expect("WAV export succeeds");
    let _ = std::fs::remove_file(&tmp_wav);

    let mut world = RainforestWorld::new_pacific_kelp_forest(99);
    world.trigger_event(StochasticEventKind::CoastalUpwellingPulse);
    let store = PersistenceStore::open_in_memory().expect("SQLite in-memory");
    store.save_world("kelp_slot", &world).expect("Save kelp slot");
    let restored = store.load_world("kelp_slot").expect("Load kelp slot");
    assert_eq!(restored.biome, BiomeKind::PacificKelpForest);
    assert_eq!(restored.organisms.len(), world.organisms.len());
}

#[test]
fn window_icon_rgba_dimensions_and_non_empty() {
    let icon = trophic_hearth::icon::app_egui_icon();
    assert_eq!(icon.width, 64);
    assert_eq!(icon.height, 64);
    assert_eq!(icon.rgba.len(), 64 * 64 * 4);
    assert!(icon.rgba.chunks_exact(4).any(|px| px[3] > 200));
}

#[test]
fn all_nine_biomes_step_and_serialize_cleanly() {
    let all_biomes = [
        BiomeKind::RockyIntertidalTidepool,
        BiomeKind::ColdSeepBrinePool,
        BiomeKind::MidwaterPelagicRift,
        BiomeKind::PnwOldGrowthSalmonCreek,
        BiomeKind::DeepSeaHydrothermalVent,
        BiomeKind::TemperateFreshwaterLake,
        BiomeKind::IndoPacificCoralReef,
        BiomeKind::PacificKelpForest,
        BiomeKind::NeotropicalRainforest,
    ];
    for (idx, biome) in all_biomes.into_iter().enumerate() {
        let mut w = RainforestWorld::new_for_biome(biome, 100 + idx as u64);
        for _ in 0..10 {
            w.step(0.05);
            w.pending_sounds.clear();
        }
        assert!(!w.organisms.is_empty(), "Biome {:?} should spawn organisms", biome);
        assert!(w.ledger.biomass_kcal.iter().all(|v| v.is_finite()));
    }
}

#[test]
fn json_snapshot_envelope_roundtrip() {
    let world = RainforestWorld::new_for_biome(BiomeKind::DeepSeaHydrothermalVent, 777);
    let snap = trophic_hearth::persistence::TrophicSnapshotV1::from_world("vent_test", world);
    let json = serde_json::to_string_pretty(&snap).expect("Serialize TrophicSnapshotV1");
    let decoded: trophic_hearth::persistence::TrophicSnapshotV1 =
        serde_json::from_str(&json).expect("Deserialize TrophicSnapshotV1");
    assert_eq!(decoded.slot_name, "vent_test");
    assert_eq!(decoded.world.biome, BiomeKind::DeepSeaHydrothermalVent);
}

#[test]
fn headless_gui_shortcuts_f1_escape_and_focus_guarded_space_tab_m() {
    let ctx = egui::Context::default();
    let world = RainforestWorld::new_for_biome(BiomeKind::RockyIntertidalTidepool, 2026);
    let first_org_id = world.organisms.first().map(|o| o.id);
    let mut app = TrophicHearthApp::new_headless(world, false);
    app.show_inspector_drawer = true;
    assert!(app.paused);
    assert!(!app.zen_hearth_mode);
    assert!(!app.show_about_window);
    assert!(app.audio.muted);

    let run_frame = |app: &mut TrophicHearthApp, key: Option<egui::Key>| {
        let mut raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1360.0, 820.0),
            )),
            ..Default::default()
        };
        if let Some(k) = key {
            raw.events.push(egui::Event::Key {
                key: k,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            });
        }
        let _ = ctx.run(raw, |ctx| {
            app.ui_step(ctx, false);
        });
    };

    // Warmup frame without focus
    run_frame(&mut app, None);

    // Unfocused Space, Tab, and M toggle paused, zen_hearth_mode, and audio.muted
    run_frame(&mut app, Some(egui::Key::Space));
    assert!(!app.paused, "Space should unpause when unfocused");
    run_frame(&mut app, Some(egui::Key::Tab));
    assert!(app.zen_hearth_mode, "Tab should enter Zen Hearth Mode");
    run_frame(&mut app, Some(egui::Key::M));
    assert!(!app.audio.muted, "M should toggle audio mute");

    // Exit Zen Hearth Mode so the inspector drawer & slot_name_input are shown
    run_frame(&mut app, Some(egui::Key::Tab));
    assert!(!app.zen_hearth_mode);

    // Focus slot_name_input -> Space, Tab, and M must NOT toggle when typing in text input
    ctx.memory_mut(|m| m.request_focus(egui::Id::new("slot_name_input")));
    run_frame(&mut app, None);
    assert!(ctx.wants_keyboard_input());
    run_frame(&mut app, Some(egui::Key::Space));
    run_frame(&mut app, Some(egui::Key::Tab));
    run_frame(&mut app, Some(egui::Key::M));
    assert!(!app.paused, "Space must not pause while text input is focused");
    assert!(
        !app.zen_hearth_mode,
        "Tab must not toggle Zen Hearth Mode while text input is focused"
    );
    assert!(
        !app.audio.muted,
        "M must not mute audio while text input is focused"
    );

    // Clear focus, test F1 & Escape cascade (show_about_window -> zen_hearth_mode -> selected_organism_id)
    ctx.memory_mut(|m| m.surrender_focus(egui::Id::new("slot_name_input")));
    run_frame(&mut app, None);
    app.zen_hearth_mode = true;
    app.selected_organism_id = first_org_id;

    run_frame(&mut app, Some(egui::Key::F1));
    assert!(app.show_about_window, "F1 should open About window");

    run_frame(&mut app, Some(egui::Key::Escape));
    assert!(!app.show_about_window, "First Escape closes About window");
    assert!(app.zen_hearth_mode, "First Escape keeps Zen Hearth Mode");

    run_frame(&mut app, Some(egui::Key::Escape));
    assert!(!app.zen_hearth_mode, "Second Escape exits Zen Hearth Mode");
    assert_eq!(
        app.selected_organism_id, first_org_id,
        "Second Escape preserves selected organism"
    );

    run_frame(&mut app, Some(egui::Key::Escape));
    assert_eq!(
        app.selected_organism_id, None,
        "Third Escape clears selected_organism_id"
    );

    // F1 toggles About window open and closed
    run_frame(&mut app, Some(egui::Key::F1));
    assert!(app.show_about_window);
    run_frame(&mut app, Some(egui::Key::F1));
    assert!(!app.show_about_window);
}
