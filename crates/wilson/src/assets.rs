// SPDX-License-Identifier: GPL-3.0-or-later
//! Loading the **original** Johnny Castaway data (`RESOURCE.MAP` + `RESOURCE.001`).
//!
//! Castaway_Screensaver plays the original game data — there is no bundled/recreated art. The
//! files are located via `--data <dir>`, or auto-detected in the working directory or
//! next to the executable.

use std::path::{Path, PathBuf};

use wilson_dgds::{find_ci, Archive, Palette, ResourceMap};

/// Load the original `RESOURCE.MAP` + its data file from `dir` (file names are matched
/// case-insensitively, so `resource.map` / `RESOURCE.001` etc. all work).
pub fn load(dir: &Path) -> Result<(Archive, Palette), String> {
    let map_path = find_ci(dir, "RESOURCE.MAP")
        .ok_or_else(|| format!("RESOURCE.MAP: not found in {dir:?}"))?;
    let map_bytes = std::fs::read(&map_path).map_err(|e| format!("RESOURCE.MAP: {e}"))?;
    let map = ResourceMap::parse(&map_bytes).map_err(|e| e.to_string())?;
    let data_path = find_ci(dir, &map.data_file_name)
        .ok_or_else(|| format!("{}: not found in {dir:?}", map.data_file_name))?;
    let archive_bytes =
        std::fs::read(&data_path).map_err(|e| format!("{}: {e}", map.data_file_name))?;
    let archive = Archive::parse(&map_bytes, &archive_bytes).map_err(|e| e.to_string())?;
    let palette = archive
        .palette()
        .cloned()
        .ok_or_else(|| "no palette (PAL) found in the data".to_string())?;
    Ok((archive, palette))
}

/// The directories searched for the data, in priority order: the explicit `--data`
/// path, `$CASTAWAY_SCREENSAVER_DATA_DIR`, then the working directory and the executable's directory
/// (each also probed for a `data/` subdirectory).
pub fn data_candidates(explicit: Option<&str>) -> Vec<PathBuf> {
    let mut bases: Vec<PathBuf> = Vec::new();
    if let Some(dir) = explicit {
        bases.push(PathBuf::from(dir));
    }
    if let Some(dir) = std::env::var_os("CASTAWAY_SCREENSAVER_DATA_DIR") {
        bases.push(PathBuf::from(dir));
    }
    if let Ok(cwd) = std::env::current_dir() {
        bases.push(cwd);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            bases.push(parent.to_path_buf());
        }
    }
    // Each base, then a `data/` subdirectory of it.
    let mut out = Vec::with_capacity(bases.len() * 2);
    for base in bases {
        out.push(base.clone());
        out.push(base.join("data"));
    }
    out
}

/// Whether `path` looks like a zip — by `.zip` extension or the `PK\x03\x04` magic.
fn is_zip(path: &Path) -> bool {
    if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("zip"))
    {
        return true;
    }
    use std::io::Read;
    let mut magic = [0u8; 4];
    std::fs::File::open(path)
        .and_then(|mut f| f.read_exact(&mut magic))
        .is_ok()
        && &magic == b"PK\x03\x04"
}

/// True if `dir` has a usable `RESOURCE.MAP` + its named data file (not the installer's
/// 35-byte "has been compressed" stub).
fn dir_has_usable_data(dir: &Path) -> bool {
    let Some(map_path) = find_ci(dir, "RESOURCE.MAP") else {
        return false;
    };
    // The data file's name comes from the MAP (usually RESOURCE.001); also try that name
    // directly, so a quirky/garbage MAP doesn't hide a present RESOURCE.001.
    let big = |name: &str| {
        find_ci(dir, name)
            .and_then(|p| std::fs::metadata(p).ok())
            .is_some_and(|m| m.len() > 1000)
    };
    if big("RESOURCE.001") {
        return true;
    }
    std::fs::read(&map_path)
        .ok()
        .and_then(|b| ResourceMap::parse(&b).ok())
        .is_some_and(|m| big(&m.data_file_name))
}

/// True if `dir` holds the original *installer's* compressed data (e.g. `RESOURCE.00$`),
/// whose proprietary compression we cannot read.
fn dir_has_installer(dir: &Path) -> bool {
    find_ci(dir, "RESOURCE.00$").is_some()
}

/// Length-framed content hashes distinguish changed archives and installer components.
fn content_key(parts: &[&[u8]]) -> String {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    for part in parts {
        hash.update((part.len() as u64).to_le_bytes());
        hash.update(part);
    }
    format!("{:x}", hash.finalize())
}

/// Only completed, usable directories may be reused. New output is staged privately.
fn cache_ready(path: &Path) -> bool {
    path.join(".complete").is_file() && (dir_has_usable_data(path) || dir_has_installer(path))
}

fn publish_cache(stage: tempfile::TempDir, destination: &Path) -> Result<PathBuf, String> {
    std::fs::write(stage.path().join(".complete"), b"v1").map_err(|e| e.to_string())?;
    match std::fs::rename(stage.path(), destination) {
        Ok(()) => Ok(destination.to_path_buf()),
        Err(_) if cache_ready(destination) => Ok(destination.to_path_buf()),
        // An incomplete old cache or a competing process must not destroy our valid output.
        Err(_) => Ok(stage.keep()),
    }
}

/// Hash and extract the same immutable snapshot, not its name or byte count.
fn extract_zip_cached(zip_path: &Path) -> Result<PathBuf, String> {
    let bytes = std::fs::read(zip_path).map_err(|e| format!("{}: {e}", zip_path.display()))?;
    let key = content_key(&[b"zip-v2", &bytes]);
    let destination = std::env::temp_dir().join(format!("castaway-screensaver-zip-{key}"));
    if cache_ready(&destination) {
        return Ok(destination);
    }
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|e| format!("{}: not a readable zip: {e}", zip_path.display()))?;
    let stage = tempfile::Builder::new()
        .prefix("castaway-screensaver-zip-")
        .tempdir()
        .map_err(|e| e.to_string())?;
    zip.extract(stage.path())
        .map_err(|e| format!("extracting {}: {e}", zip_path.display()))?;
    publish_cache(stage, &destination)
}

/// The message shown when no usable data source is found.
const NEED_DATA_MSG: &str = "Castaway_Screensaver needs the original Johnny Castaway data \
    (RESOURCE.MAP + RESOURCE.001). Pass --data <dir-or-zip>, set CASTAWAY_SCREENSAVER_DATA_DIR, or place \
    the files (or the Internet Archive scrantic-run.zip) in the current directory or next to \
    the executable. Get them at \
    https://archive.org/download/johnny-castaway-screensaver/scrantic-run.zip";

/// Turn a candidate path into a ready data directory: a folder with the data is used
/// as-is; a `.zip` (e.g. `scrantic-run.zip`) is extracted to a temp dir; the original
/// installer's compressed data is detected and reported. `Ok(None)` means "not a data
/// source" (skip it); `Err` means "recognised but unusable" (e.g. the installer).
fn prepare(path: &Path) -> Result<Option<PathBuf>, String> {
    if path.is_file() && is_zip(path) {
        let dir = extract_zip_cached(path)?;
        return prepare(&dir); // now a directory
    }
    if path.is_dir() {
        if dir_has_usable_data(path) {
            return Ok(Some(path.to_path_buf()));
        }
        if dir_has_installer(path) {
            return prepare_installer(path).map(Some);
        }
    }
    Ok(None)
}

/// Decompress an original *installer* directory (its `RESOURCE.00$` is PKWARE-DCL-imploded)
/// into a temp dir holding a plain `RESOURCE.MAP` + `RESOURCE.001` (and `SCRANTIC.SCR` for
/// sound). Reused across runs.
fn prepare_installer(dir: &Path) -> Result<PathBuf, String> {
    let map = find_ci(dir, "RESOURCE.MAP")
        .ok_or_else(|| "installer is missing RESOURCE.MAP".to_string())?;
    let r00 = find_ci(dir, "RESOURCE.00$")
        .ok_or_else(|| "installer is missing RESOURCE.00$".to_string())?;
    let map_bytes = std::fs::read(map).map_err(|e| format!("RESOURCE.MAP: {e}"))?;
    let comp = std::fs::read(r00).map_err(|e| format!("RESOURCE.00$: {e}"))?;
    let sound = find_ci(dir, "SCRANTIC.SC$")
        .map(std::fs::read)
        .transpose()
        .map_err(|e| format!("SCRANTIC.SC$: {e}"))?;
    let key = content_key(&[
        b"installer-v2",
        &map_bytes,
        &comp,
        sound.as_deref().unwrap_or(&[]),
    ]);
    let destination = std::env::temp_dir().join(format!("castaway-screensaver-installer-{key}"));
    if cache_ready(&destination) {
        return Ok(destination);
    }
    let data = wilson_dgds::decompress_installer(&comp).ok_or_else(|| {
        "RESOURCE.00$: unexpected installer compression (could not decompress)".to_string()
    })?;
    let stage = tempfile::Builder::new()
        .prefix("castaway-screensaver-installer-")
        .tempdir()
        .map_err(|e| e.to_string())?;
    std::fs::write(stage.path().join("RESOURCE.MAP"), map_bytes).map_err(|e| e.to_string())?;
    std::fs::write(stage.path().join("RESOURCE.001"), data).map_err(|e| e.to_string())?;
    if let Some(exe) = sound.and_then(|c| wilson_dgds::decompress_installer(&c)) {
        std::fs::write(stage.path().join("SCRANTIC.SCR"), exe).map_err(|e| e.to_string())?;
    }
    publish_cache(stage, &destination)
}

/// Candidate data paths (folders and zips), in priority order. Reuses [`data_candidates`]
/// (the explicit `--data` path — which may be a `.zip` — plus the dir search) and also
/// probes for the Internet Archive zips next to each.
fn resolve_candidates(explicit: Option<&str>) -> Vec<PathBuf> {
    let dirs = data_candidates(explicit);
    let mut zips: Vec<PathBuf> = Vec::new();
    for d in &dirs {
        zips.push(d.join("scrantic-run.zip"));
        zips.push(d.join("scrantic-installer.zip"));
    }
    let mut v = dirs;
    v.extend(zips);
    v
}

/// Resolve `--data`/auto-detect into a ready data directory. Accepts a **folder** or a
/// **`.zip`** (extracted to a temp dir). Returns a clear error if nothing usable is found
/// (preferring a specific reason, e.g. "that's the installer", over the generic message).
pub fn resolve_data_dir(explicit: Option<&str>) -> Result<PathBuf, String> {
    let mut reason: Option<String> = None;
    for cand in resolve_candidates(explicit) {
        match prepare(&cand) {
            Ok(Some(dir)) => return Ok(dir),
            Ok(None) => {}
            Err(e) => {
                if reason.is_none() {
                    reason = Some(e);
                }
            }
        }
    }
    Err(reason.unwrap_or_else(|| NEED_DATA_MSG.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// A temp dir unique to this test process + `tag`.
    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("wilson-test-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn missing_data_dir_reports_resource_map() {
        let err = load(Path::new("/nonexistent/wilson/data")).unwrap_err();
        assert!(err.contains("RESOURCE.MAP"), "unexpected error: {err}");
    }

    #[test]
    fn detects_usable_dir_and_installer_and_zip_magic() {
        // A usable dir: RESOURCE.MAP + a >1000-byte RESOURCE.001.
        let good = tmp("good");
        std::fs::write(good.join("RESOURCE.MAP"), [0u8; 32]).unwrap();
        std::fs::write(good.join("RESOURCE.001"), vec![0u8; 2000]).unwrap();
        assert!(dir_has_usable_data(&good));
        assert!(!dir_has_installer(&good));

        // The installer: a compressed RESOURCE.00$ + the 35-byte stub.
        let inst = tmp("inst");
        std::fs::write(inst.join("RESOURCE.MAP"), [0u8; 32]).unwrap();
        std::fs::write(
            inst.join("RESOURCE.001"),
            b"Resource #1 has been compressed.\r\n",
        )
        .unwrap();
        std::fs::write(inst.join("RESOURCE.00$"), vec![0u8; 4000]).unwrap();
        assert!(!dir_has_usable_data(&inst), "stub data file is not usable");
        assert!(dir_has_installer(&inst));
        // The installer is recognised and decompression is attempted; on this garbage
        // RESOURCE.00$ that fails with a clear error (real data is covered by a gated test).
        let err = prepare(&inst).unwrap_err();
        assert!(
            err.contains("RESOURCE.00$") || err.to_lowercase().contains("installer"),
            "{err}"
        );

        // is_zip by magic, regardless of extension.
        let zpath = tmp("z").join("data.bin");
        std::fs::write(&zpath, b"PK\x03\x04rest").unwrap();
        assert!(is_zip(&zpath));
        assert!(!is_zip(&good.join("RESOURCE.MAP")));
    }

    #[test]
    fn data_zip_is_accepted_directly() {
        // Build a scrantic-run-style zip: RESOURCE.MAP + a >1000-byte RESOURCE.001.
        let dir = tmp("zipsrc");
        let zip_path = dir.join("scrantic-run.zip");
        {
            let file = std::fs::File::create(&zip_path).unwrap();
            let mut w = zip::ZipWriter::new(file);
            let opts = zip::write::SimpleFileOptions::default();
            w.start_file("RESOURCE.MAP", opts).unwrap();
            w.write_all(&[0u8; 32]).unwrap();
            w.start_file("RESOURCE.001", opts).unwrap();
            w.write_all(&vec![7u8; 2000]).unwrap();
            w.finish().unwrap();
        }
        // resolve_data_dir extracts the zip and returns a usable directory.
        let got = resolve_data_dir(Some(zip_path.to_str().unwrap())).expect("zip accepted");
        assert!(dir_has_usable_data(&got), "extracted dir has usable data");
        assert!(find_ci(&got, "RESOURCE.001").is_some());
    }

    #[test]
    fn candidates_prioritise_explicit_then_probe_data_subdir() {
        let c = data_candidates(Some("/explicit"));
        // The explicit dir and its data/ subdir come first, in that order.
        assert_eq!(c[0], PathBuf::from("/explicit"));
        assert_eq!(c[1], PathBuf::from("/explicit/data"));
        // The working directory is searched too (after the explicit path).
        assert!(c.iter().any(|p| p.ends_with("data")));
        assert!(c.len() >= 4);
    }

    #[test]
    fn zip_cannot_write_outside_extraction_directory() {
        let dir = tmp("zip-traversal");
        let zip_path = dir.join("traversal.zip");
        let marker = format!("castaway-escape-{}.txt", std::process::id());
        let escaped = std::env::temp_dir().join(&marker);
        assert!(!escaped.exists(), "test marker already exists");
        {
            let file = std::fs::File::create(&zip_path).unwrap();
            let mut writer = zip::ZipWriter::new(file);
            writer
                .start_file(
                    format!("../{marker}"),
                    zip::write::SimpleFileOptions::default(),
                )
                .unwrap();
            writer
                .write_all(b"must stay inside extraction directory")
                .unwrap();
            writer.finish().unwrap();
        }
        // Rejection or safe containment are both acceptable; escaping is not.
        let _ = extract_zip_cached(&zip_path);
        assert!(
            !escaped.exists(),
            "archive escaped its extraction directory"
        );
    }

    #[test]
    fn changed_same_size_zip_does_not_reuse_old_data() {
        let first = tmp("cache-first").join("same.zip");
        let second = tmp("cache-second").join("same.zip");
        for (path, value) in [(&first, 3u8), (&second, 9u8)] {
            let file = std::fs::File::create(path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            zip.start_file("RESOURCE.MAP", options).unwrap();
            zip.write_all(&[0; 32]).unwrap();
            zip.start_file("RESOURCE.001", options).unwrap();
            zip.write_all(&vec![value; 2000]).unwrap();
            zip.finish().unwrap();
        }
        assert_eq!(
            std::fs::metadata(&first).unwrap().len(),
            std::fs::metadata(&second).unwrap().len()
        );
        let a = extract_zip_cached(&first).unwrap();
        let b = extract_zip_cached(&second).unwrap();
        assert_ne!(a, b);
        assert_eq!(
            std::fs::read(a.join("RESOURCE.001")).unwrap(),
            vec![3; 2000]
        );
        assert_eq!(
            std::fs::read(b.join("RESOURCE.001")).unwrap(),
            vec![9; 2000]
        );
        assert_eq!(b, extract_zip_cached(&second).unwrap());
    }

    #[test]
    fn installer_cache_key_covers_map_data_and_sound() {
        let original = content_key(&[b"map", b"data", b"sound"]);
        for parts in [
            [&b"MAP"[..], &b"data"[..], &b"sound"[..]],
            [&b"map"[..], &b"DATA"[..], &b"sound"[..]],
            [&b"map"[..], &b"data"[..], &b"SOUND"[..]],
        ] {
            assert_ne!(original, content_key(&parts));
        }
        assert_ne!(content_key(&[b"ab", b"c"]), content_key(&[b"a", b"bc"]));
    }

    #[test]
    fn finds_real_data_when_present() {
        // Gated: only runs when WILSON_DATA_DIR points at real data.
        if std::env::var_os("WILSON_DATA_DIR").is_some() {
            let found = resolve_data_dir(None).expect("auto-detect via WILSON_DATA_DIR");
            assert!(find_ci(&found, "RESOURCE.MAP").is_some());
            assert!(load(&found).is_ok());
        }
    }

    #[test]
    fn resolves_real_scrantic_zip_if_present() {
        // Gated end-to-end: `WILSON_TEST_ZIP=/path/scrantic-run.zip` → accept the zip
        // directly, load the graphics, and extract the 23 sounds from the embedded EXE.
        let Some(z) = std::env::var_os("WILSON_TEST_ZIP") else {
            return;
        };
        let dir = resolve_data_dir(Some(z.to_str().unwrap())).expect("resolve real zip");
        assert!(load(&dir).is_ok(), "load graphics from the extracted zip");
        let exe = find_ci(&dir, "SCRANTIC.EXE")
            .or_else(|| find_ci(&dir, "SCRANTIC.SCR"))
            .expect("zip carries SCRANTIC.EXE/.SCR");
        let bytes = std::fs::read(exe).unwrap();
        let n = wilson_dgds::sounds_from_scrantic_exe(&bytes)
            .iter()
            .filter(|s| s.is_some())
            .count();
        assert_eq!(n, 23, "23 sounds extracted from the EXE inside the zip");
    }

    #[test]
    fn resolves_real_installer_if_present() {
        // Gated: `WILSON_TEST_INSTALLER=<dir-or-zip>` (the scrantic-installer with the
        // compressed RESOURCE.00$) → decompress and load the graphics.
        let Some(p) = std::env::var_os("WILSON_TEST_INSTALLER") else {
            return;
        };
        let dir = resolve_data_dir(Some(p.to_str().unwrap())).expect("resolve installer");
        assert!(
            load(&dir).is_ok(),
            "load graphics from the decompressed installer"
        );
    }
}
