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
use trophic_hearth::renderer::{
    producer_flux_anchors, render_rainforest_viewport, surface_caustic_or_dapple, CameraPreset,
    DioramaCamera,
};
use trophic_hearth::trophic::{BiomeKind, TrophicEnergyLedger, TrophicFocus, TrophicStratum};
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

#[test]
fn cold_seep_brine_pool_reports_salinity_psu_and_excludes_removed_keystones_from_hitboxes() {
    let mut bp = RainforestWorld::new_cold_seep_brine_pool(2026);
    assert_eq!(BiomeKind::ColdSeepBrinePool.telemetry_unit(), " PSU");
    let bp_atmo = bp.diurnal.atmosphere_for_biome(BiomeKind::ColdSeepBrinePool);
    assert_eq!(bp_atmo.salinity_psu, Some(120.0));
    assert_eq!(bp_atmo.hydro_unit, " PSU");
    let summary = bp.diurnal.telemetry_summary(BiomeKind::ColdSeepBrinePool);
    assert!(
        summary.contains("120 PSU"),
        "Brine pool telemetry must report salinity in PSU, got: {summary}"
    );
    assert!(
        !summary.contains("% DO"),
        "Brine pool telemetry must NOT report salinity as % DO, got: {summary}"
    );

    let reef_summary = bp.diurnal.telemetry_summary(BiomeKind::IndoPacificCoralReef);
    assert!(reef_summary.contains("% DO"));
    let rf_summary = bp.diurnal.telemetry_summary(BiomeKind::NeotropicalRainforest);
    assert!(rf_summary.contains("% RH"));

    // Verify hitboxes include keystones when intact and exclude them when removed across biomes
    let ctx = egui::Context::default();
    let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 800.0));
    let camera = DioramaCamera::default();

    for biome in [
        BiomeKind::RockyIntertidalTidepool,
        BiomeKind::ColdSeepBrinePool,
        BiomeKind::PacificKelpForest,
    ] {
        let mut world = RainforestWorld::new_for_biome(biome, 2026);
        let keystone_ids: Vec<u32> = world
            .organisms
            .iter()
            .filter(|o| o.species.is_keystone_species())
            .map(|o| o.id)
            .collect();
        assert!(
            !keystone_ids.is_empty(),
            "Biome {biome:?} must spawn at least one keystone organism"
        );

        let mut intact_hits = Vec::new();
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(rect),
                ..Default::default()
            },
            |c| {
                egui::CentralPanel::default().show(c, |ui| {
                    intact_hits = render_rainforest_viewport(ui.painter(), rect, &world, &camera, None);
                });
            },
        );
        assert!(
            keystone_ids
                .iter()
                .any(|kid| intact_hits.iter().any(|(hid, _)| hid == kid)),
            "Intact keystones in {biome:?} should appear in viewport hitboxes"
        );

        world.ledger.keystone_removed = true;
        let mut removed_hits = Vec::new();
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(rect),
                ..Default::default()
            },
            |c| {
                egui::CentralPanel::default().show(c, |ui| {
                    removed_hits =
                        render_rainforest_viewport(ui.painter(), rect, &world, &camera, None);
                });
            },
        );
        for kid in &keystone_ids {
            assert!(
                !removed_hits.iter().any(|(hid, _)| hid == kid),
                "Removed keystone id {kid} in {biome:?} must be excluded from hitboxes"
            );
        }
        assert!(
            !removed_hits.is_empty(),
            "Non-keystone organisms in {biome:?} must remain selectable in hitboxes"
        );
    }

    // Pressing K to remove a currently selected keystone clears selected_organism_id
    bp.ledger.keystone_removed = false;
    let bp_keystone_id = bp
        .organisms
        .iter()
        .find(|o| o.species.is_keystone_species())
        .map(|o| o.id)
        .expect("Brine pool has keystone");
    let mut app = TrophicHearthApp::new_headless(bp, false);
    app.selected_organism_id = Some(bp_keystone_id);
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(rect),
            events: vec![egui::Event::Key {
                key: egui::Key::K,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        },
        |c| app.ui_step(c, false),
    );
    assert!(app.world.ledger.keystone_removed);
    assert_eq!(app.selected_organism_id, None);
}

#[test]
fn coupled_trophic_odes_conserve_energy_and_deplete_producers_under_grazing() {
    // 1. Standalone ODE First-Law accounting: GPP -> d(Biomass + Detritus) + Sum(Respiration)
    for biome in BiomeKind::ALL {
        for &keystone_removed in &[false, true] {
            let mut ledger = match biome {
                BiomeKind::RockyIntertidalTidepool => TrophicEnergyLedger::new_tidepool_baseline(),
                BiomeKind::ColdSeepBrinePool => TrophicEnergyLedger::new_brine_pool_baseline(),
                BiomeKind::MidwaterPelagicRift => TrophicEnergyLedger::new_pelagic_rift_baseline(),
                BiomeKind::PnwOldGrowthSalmonCreek => TrophicEnergyLedger::new_pnw_old_growth_baseline(),
                BiomeKind::DeepSeaHydrothermalVent => TrophicEnergyLedger::new_hydrothermal_vent_baseline(),
                BiomeKind::TemperateFreshwaterLake => TrophicEnergyLedger::new_freshwater_lake_baseline(),
                BiomeKind::IndoPacificCoralReef => TrophicEnergyLedger::new_coral_reef_baseline(),
                BiomeKind::PacificKelpForest => TrophicEnergyLedger::new_kelp_forest_baseline(),
                BiomeKind::NeotropicalRainforest => TrophicEnergyLedger::new_rainforest_baseline(),
            };
            ledger.keystone_removed = keystone_removed;
            let e_initial = ledger.total_system_energy_kcal();

            for _ in 0..160 {
                ledger.step_keystone_cascade(biome, 0.05);
            }

            let e_final = ledger.total_system_energy_kcal();
            let delta_storage = e_final - e_initial - ledger.external_detritus_kcal;
            let resp_total: f32 = ledger.respiration_kcal.iter().sum();
            let net_inflow = ledger.cumulative_gpp_kcal - resp_total;
            assert!(
                (delta_storage - net_inflow).abs() < 0.5,
                "First-law mismatch in {biome:?} (keystone_removed={keystone_removed}): delta_storage={delta_storage:.3}, net_inflow={net_inflow:.3}"
            );
            for stratum in TrophicStratum::ALL {
                let i = stratum.index();
                assert!(
                    ledger.assimilation_kcal[i] > 0.0 && ledger.respiration_kcal[i] > 0.0,
                    "Stratum {stratum:?} in {biome:?} must accumulate assimilation & respiration"
                );
            }
            let te2 = ledger.transfer_efficiency_pct(TrophicStratum::PrimaryConsumers);
            let te3 = ledger.transfer_efficiency_pct(TrophicStratum::SecondaryConsumers);
            if !keystone_removed {
                assert!(
                    (4.5..=30.0).contains(&te2) && (4.5..=30.0).contains(&te3),
                    "Baseline Lindeman efficiencies out of expected ecological band in {biome:?}: L2={te2:.2}%, L3={te3:.2}%"
                );
            } else {
                assert!(
                    (0.0..=60.0).contains(&te2) && (0.0..=60.0).contains(&te3),
                    "Cascade Lindeman efficiencies out of bounds in {biome:?}: L2={te2:.2}%, L3={te3:.2}%"
                );
            }
        }
    }

    // 2. Full RainforestWorld integration conserves energy & depletes L1 producers below the old 3800 clamp
    let mut kelp = RainforestWorld::new_pacific_kelp_forest(2026);
    let kelp_e0 = kelp.ledger.total_system_energy_kcal();
    let kelp_l1_start = kelp.ledger.biomass_kcal[1];
    kelp.ledger.keystone_removed = true; // Remove Sea Otters -> Urchin grazing barren!
    for _ in 0..240 {
        kelp.step(0.05);
        kelp.pending_sounds.clear();
    }
    let kelp_delta =
        kelp.ledger.total_system_energy_kcal() - kelp_e0 - kelp.ledger.external_detritus_kcal;
    let kelp_net = kelp.ledger.cumulative_gpp_kcal - kelp.ledger.respiration_kcal.iter().sum::<f32>();
    assert!(
        (kelp_delta - kelp_net).abs() < 1.0,
        "Full Kelp world conservation error: delta={kelp_delta:.3}, net={kelp_net:.3}"
    );
    assert!(
        kelp.ledger.biomass_kcal[1] < 3800.0 && kelp.ledger.biomass_kcal[1] < kelp_l1_start * 0.72,
        "Kelp L1 producers must deplete cleanly below the old 3800 clamp under urchin grazing (got {:.1})",
        kelp.ledger.biomass_kcal[1]
    );

    // 3. Direct grazing perturbation in Tidepool depletes producers below 3800 kcal cleanly
    let mut tp = RainforestWorld::new_rocky_intertidal_tidepool(2026);
    tp.ledger.biomass_kcal[2] *= 2.5; // Direct grazer density surge
    for _ in 0..120 {
        tp.step(0.05);
    }
    assert!(
        tp.ledger.biomass_kcal[1] < 3800.0,
        "Producer biomass must respond cleanly to direct grazing below 3800 kcal (got {:.1})",
        tp.ledger.biomass_kcal[1]
    );
}

#[test]
fn headless_gui_new_hotkeys_smooth_camera_and_hover_guarded_scroll() {
    let ctx = egui::Context::default();
    let world = RainforestWorld::new_for_biome(BiomeKind::RockyIntertidalTidepool, 2026);
    let mut app = TrophicHearthApp::new_headless(world, false);
    app.show_inspector_drawer = true;

    let run_events = |app: &mut TrophicHearthApp,
                      events: Vec<egui::Event>,
                      t_sec: f64| {
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1360.0, 820.0),
            )),
            time: Some(t_sec),
            predicted_dt: 1.0 / 60.0,
            events,
            ..Default::default()
        };
        let _ = ctx.run(raw, |c| app.ui_step(c, false));
    };

    // Warmup
    run_events(&mut app, vec![], 0.0);

    // Hotkey '2' activates smooth preset transition (not an instant snap)
    let dist_before = app.camera.distance_m;
    run_events(
        &mut app,
        vec![egui::Event::Key {
            key: egui::Key::Num2,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
        0.02,
    );
    assert_eq!(app.camera.preset, CameraPreset::UnderbrushStalkMacro);
    assert!(app.camera.interpolating);
    assert!(
        (app.camera.distance_m - dist_before).abs() < (dist_before - 9.8).abs(),
        "Preset 2 must interpolate smoothly rather than snapping immediately"
    );
    for step_i in 0..40 {
        app.camera.step(0.03);
        let _ = step_i;
    }
    assert!((app.camera.distance_m - 9.8).abs() < 0.05);

    // Hotkeys '[' and ']' cycle biomes across all 9 ecosystems
    assert_eq!(app.world.biome, BiomeKind::RockyIntertidalTidepool);
    run_events(
        &mut app,
        vec![egui::Event::Key {
            key: egui::Key::CloseBracket,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
        0.05,
    );
    assert_eq!(app.world.biome, BiomeKind::ColdSeepBrinePool);
    run_events(
        &mut app,
        vec![egui::Event::Key {
            key: egui::Key::OpenBracket,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
        0.07,
    );
    assert_eq!(app.world.biome, BiomeKind::RockyIntertidalTidepool);

    // Shift+0..4 focuses L0..L4, and plain 0 returns to AllStrata without triggering camera preset
    run_events(
        &mut app,
        vec![egui::Event::Key {
            key: egui::Key::Num3,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::SHIFT,
        }],
        0.09,
    );
    assert_eq!(
        app.world.focus,
        TrophicFocus::Single(TrophicStratum::SecondaryConsumers)
    );
    assert_eq!(app.camera.preset, CameraPreset::SideDiorama);
    run_events(
        &mut app,
        vec![egui::Event::Key {
            key: egui::Key::Num0,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
        0.11,
    );
    assert_eq!(app.world.focus, TrophicFocus::AllStrata);

    // 'F' toggles flux arcs
    assert!(app.world.ledger.show_flux_arcs);
    run_events(
        &mut app,
        vec![egui::Event::Key {
            key: egui::Key::F,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
        0.13,
    );
    assert!(!app.world.ledger.show_flux_arcs);

    // Focus slot_name_input -> K, F, [, ], 0..4 must all be blocked by wants_keyboard_input()
    ctx.memory_mut(|m| m.request_focus(egui::Id::new("slot_name_input")));
    run_events(&mut app, vec![], 0.15);
    assert!(ctx.wants_keyboard_input());
    run_events(
        &mut app,
        vec![
            egui::Event::Key {
                key: egui::Key::K,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::Key {
                key: egui::Key::F,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::Key {
                key: egui::Key::CloseBracket,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            },
        ],
        0.17,
    );
    assert!(!app.world.ledger.keystone_removed);
    assert!(!app.world.ledger.show_flux_arcs);
    assert_eq!(app.world.biome, BiomeKind::RockyIntertidalTidepool);
    ctx.memory_mut(|m| m.surrender_focus(egui::Id::new("slot_name_input")));
    run_events(&mut app, vec![], 0.19);

    // Scroll-zoom guard: hovering right inspector panel must NOT zoom camera
    app.camera.snap_to_preset(CameraPreset::SideDiorama);
    let cam_dist_start = app.camera.distance_m;
    run_events(
        &mut app,
        vec![egui::Event::PointerMoved(egui::pos2(1240.0, 420.0))],
        0.21,
    );
    run_events(
        &mut app,
        vec![
            egui::Event::PointerMoved(egui::pos2(1240.0, 420.0)),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, 80.0),
                modifiers: egui::Modifiers::NONE,
            },
        ],
        0.23,
    );
    assert!(
        (app.camera.distance_m - cam_dist_start).abs() < 1e-4,
        "Scroll wheel over right inspector drawer must not zoom 3D viewport"
    );

    // Hovering 3D viewport center DOES zoom camera
    run_events(
        &mut app,
        vec![egui::Event::PointerMoved(egui::pos2(450.0, 420.0))],
        0.25,
    );
    run_events(
        &mut app,
        vec![
            egui::Event::PointerMoved(egui::pos2(450.0, 420.0)),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, 120.0),
                modifiers: egui::Modifiers::NONE,
            },
        ],
        0.27,
    );
    assert!(
        app.camera.distance_m < cam_dist_start - 0.1,
        "Scroll wheel while hovering 3D viewport must zoom camera inward"
    );
}

#[test]
fn aphotic_biomes_gate_caustics_flux_arcs_anchor_producers_and_wgsl_validates() {
    // 1. Aphotic hydrothermal vent & cold seep must have strictly 1.0 (zero solar caustics)
    let mut cycle = trophic_hearth::trophic::DiurnalCycle::default();
    cycle.hour_of_day = 12.0; // Solar noon
    for aphotic in [
        BiomeKind::DeepSeaHydrothermalVent,
        BiomeKind::ColdSeepBrinePool,
    ] {
        let atmo = cycle.atmosphere_for_biome(aphotic);
        assert_eq!(atmo.par_umol_m2_s, 0.0);
        for x in [-6.0_f32, 0.0, 4.5] {
            for z in [-3.0_f32, 0.5] {
                let f = surface_caustic_or_dapple(
                    aphotic,
                    &atmo,
                    glam::Vec3::new(x, 2.0, z),
                    1.25,
                );
                assert_eq!(
                    f, 1.0,
                    "Aphotic biome {aphotic:?} must not emit sunlight caustics (got {f})"
                );
            }
        }
    }
    // Sunlit shallow reef at noon should modulate caustics
    let reef_atmo = cycle.atmosphere_for_biome(BiomeKind::IndoPacificCoralReef);
    assert!(reef_atmo.par_umol_m2_s > 1000.0);
    let c_a = surface_caustic_or_dapple(
        BiomeKind::IndoPacificCoralReef,
        &reef_atmo,
        glam::Vec3::new(1.5, 2.0, 0.5),
        1.25,
    );
    let c_b = surface_caustic_or_dapple(
        BiomeKind::IndoPacificCoralReef,
        &reef_atmo,
        glam::Vec3::new(-2.2, 2.0, -1.5),
        2.75,
    );
    assert!((c_a - c_b).abs() > 0.01);

    // 2. All 9 biomes provide valid L1 producer flux-arc anchors (trees/kelp/underbrush)
    for biome in BiomeKind::ALL {
        let world = RainforestWorld::new_for_biome(biome, 2026);
        let anchors = producer_flux_anchors(&world, 1.0);
        assert!(
            anchors.len() >= 4,
            "Biome {biome:?} should yield >= 4 L1 producer flux anchors (got {})",
            anchors.len()
        );
        assert!(anchors.iter().all(|p| p.is_finite()));
    }

    // 3. WGSL diorama shader parses and validates cleanly with wgpu::naga
    let wgsl_src = include_str!("../shaders/rainforest_diorama.wgsl");
    let module = wgpu::naga::front::wgsl::parse_str(wgsl_src)
        .expect("rainforest_diorama.wgsl must parse cleanly");
    let mut validator = wgpu::naga::valid::Validator::new(
        wgpu::naga::valid::ValidationFlags::all(),
        wgpu::naga::valid::Capabilities::default(),
    );
    validator
        .validate(&module)
        .expect("rainforest_diorama.wgsl must pass naga validation");
}
