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

use crate::world::RainforestWorld;
use crate::SCHEMA_VERSION;
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Cross-compatible save envelope shared between the native Rust SQLite engine
/// and the interactive Three.js browser preview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrophicSnapshotV1 {
    pub schema: String,
    pub slot_name: String,
    pub saved_unix_sec: u64,
    pub world: RainforestWorld,
}

impl TrophicSnapshotV1 {
    pub fn from_world(slot_name: impl Into<String>, world: RainforestWorld) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            schema: SCHEMA_VERSION.to_string(),
            slot_name: slot_name.into(),
            saved_unix_sec: now,
            world,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SaveSlotSummary {
    pub slot_name: String,
    pub biome_name: String,
    pub day_count: u32,
    pub hour_of_day: f32,
    pub saved_unix_sec: u64,
}

/// Bundled SQLite WAL persistence store for saving and restoring living diorama ecosystems.
pub struct PersistenceStore {
    conn: Connection,
}

impl PersistenceStore {
    pub fn open(path: &Path) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS save_slots (
                slot_name TEXT PRIMARY KEY,
                biome_name TEXT NOT NULL,
                day_count INTEGER NOT NULL,
                hour_of_day REAL NOT NULL,
                saved_unix_sec INTEGER NOT NULL,
                snapshot_json TEXT NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS save_slots (
                slot_name TEXT PRIMARY KEY,
                biome_name TEXT NOT NULL,
                day_count INTEGER NOT NULL,
                hour_of_day REAL NOT NULL,
                saved_unix_sec INTEGER NOT NULL,
                snapshot_json TEXT NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }

    pub fn save_world(&self, slot_name: &str, world: &RainforestWorld) -> Result<(), String> {
        let snap = TrophicSnapshotV1::from_world(slot_name, world.clone());
        let json = serde_json::to_string(&snap).map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "INSERT INTO save_slots (slot_name, biome_name, day_count, hour_of_day, saved_unix_sec, snapshot_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(slot_name) DO UPDATE SET
                    biome_name = excluded.biome_name,
                    day_count = excluded.day_count,
                    hour_of_day = excluded.hour_of_day,
                    saved_unix_sec = excluded.saved_unix_sec,
                    snapshot_json = excluded.snapshot_json",
                params![
                    slot_name,
                    world.biome_name,
                    world.diurnal.day_count,
                    world.diurnal.hour_of_day,
                    snap.saved_unix_sec as i64,
                    json
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_world(&self, slot_name: &str) -> Result<RainforestWorld, String> {
        let json: String = self
            .conn
            .query_row(
                "SELECT snapshot_json FROM save_slots WHERE slot_name = ?1",
                params![slot_name],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        let snap: TrophicSnapshotV1 = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        Ok(snap.world)
    }

    pub fn list_slots(&self) -> SqlResult<Vec<SaveSlotSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT slot_name, biome_name, day_count, hour_of_day, saved_unix_sec
             FROM save_slots ORDER BY saved_unix_sec DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(SaveSlotSummary {
                slot_name: r.get(0)?,
                biome_name: r.get(1)?,
                day_count: r.get(2)?,
                hour_of_day: r.get(3)?,
                saved_unix_sec: r.get::<_, i64>(4)? as u64,
            })
        })?;
        let mut out = Vec::new();
        for item in rows {
            out.push(item?);
        }
        Ok(out)
    }
}
