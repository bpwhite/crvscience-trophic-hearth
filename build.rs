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

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_windows_kits_rc() -> Option<PathBuf> {
    let kits_root = Path::new(r"C:\Program Files (x86)\Windows Kits\10\bin");
    let entries = fs::read_dir(kits_root).ok()?;
    let mut versions: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|x| x.path()))
        .filter(|p| p.is_dir())
        .collect();
    versions.sort();
    for ver_dir in versions.into_iter().rev() {
        let candidate = ver_dir.join("x64").join("rc.exe");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn find_program(candidates: &[&str]) -> Option<PathBuf> {
    for &prog in candidates {
        let p = PathBuf::from(prog);
        if p.is_file() {
            return Some(p);
        }
        if let Ok(path_var) = env::var("PATH") {
            for dir in env::split_paths(&path_var) {
                let full = dir.join(prog);
                if full.is_file() {
                    return Some(full);
                }
                #[cfg(windows)]
                {
                    let with_exe = dir.join(format!("{prog}.exe"));
                    if with_exe.is_file() {
                        return Some(with_exe);
                    }
                }
            }
        }
    }
    None
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/trophic_hearth.ico");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let ico_path = manifest_dir.join("assets").join("trophic_hearth.ico");
    if !ico_path.is_file() {
        println!(
            "cargo:warning=Missing Windows icon asset at {}; skipping PE resource embed",
            ico_path.display()
        );
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let rc_path = out_dir.join("trophic_hearth.rc");
    let ico_rc_str = ico_path.to_string_lossy().replace('\\', "/");

    let rc_content = format!(
        r#"1 ICON "{ico_rc_str}"

1 VERSIONINFO
FILEVERSION 0,9,3,0
PRODUCTVERSION 0,9,3,0
FILEFLAGSMASK 0x3fL
FILEFLAGS 0x0L
FILEOS 0x40004L
FILETYPE 0x1L
FILESUBTYPE 0x0L
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904B0"
        BEGIN
            VALUE "CompanyName", "CRV Science Network\0"
            VALUE "FileDescription", "TrophicHearth 9-Biome 3D Living Trophic Diorama & Bioacoustic Simulator\0"
            VALUE "FileVersion", "0.9.3.0\0"
            VALUE "InternalName", "trophic-hearth\0"
            VALUE "LegalCopyright", "Copyright CRV Science Network (GPL-3.0-or-later)\0"
            VALUE "OriginalFilename", "19_Trophic-Hearth.exe\0"
            VALUE "ProductName", "TrophicHearth\0"
            VALUE "ProductVersion", "0.9.3.0\0"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x409, 1200
    END
END
"#
    );

    if let Err(e) = fs::write(&rc_path, rc_content) {
        println!("cargo:warning=Failed to write {}: {e}", rc_path.display());
        return;
    }

    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_env == "gnu" {
        let windres_env = env::var("WINDRES").ok();
        let windres = windres_env
            .as_deref()
            .and_then(|s| find_program(&[s]))
            .or_else(|| find_program(&["windres", "x86_64-w64-mingw32-windres"]));
        let Some(windres_bin) = windres else {
            println!("cargo:warning=windres not found; skipping Windows PE icon/VersionInfo embed");
            return;
        };
        let obj_path = out_dir.join("app_res.o");
        match Command::new(&windres_bin)
            .arg(&rc_path)
            .arg("-O")
            .arg("coff")
            .arg("-o")
            .arg(&obj_path)
            .status()
        {
            Ok(st) if st.success() => {
                println!("cargo:rustc-link-arg-bins={}", obj_path.display());
            }
            Ok(st) => {
                println!(
                    "cargo:warning=windres exited with status {st}; skipping PE resource embed"
                );
            }
            Err(e) => {
                println!("cargo:warning=Failed to run windres: {e}");
            }
        }
    } else if target_env == "msvc" {
        let rc_env = env::var("RC").ok();
        let rc_bin = rc_env
            .as_deref()
            .and_then(|s| find_program(&[s]))
            .or_else(|| find_program(&["rc.exe"]))
            .or_else(find_windows_kits_rc);
        let Some(rc_bin) = rc_bin else {
            println!("cargo:warning=rc.exe not found; skipping Windows PE icon/VersionInfo embed");
            return;
        };
        let res_path = out_dir.join("app_res.res");
        match Command::new(&rc_bin)
            .arg("/nologo")
            .arg("/fo")
            .arg(&res_path)
            .arg(&rc_path)
            .status()
        {
            Ok(st) if st.success() => {
                println!("cargo:rustc-link-arg-bins={}", res_path.display());
            }
            Ok(st) => {
                println!(
                    "cargo:warning=rc.exe exited with status {st}; skipping PE resource embed"
                );
            }
            Err(e) => {
                println!("cargo:warning=Failed to run rc.exe: {e}");
            }
        }
    }
}
