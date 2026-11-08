//! Put packed files into the game folder, with a backup of the originals.
//!
//! The first install copies the original files to `<game>/bnsedit-backup/`. Later installs
//! never overwrite that backup, so "Put originals back" always restores the first state.
//! Every file is written to a temporary name first, then renamed over the target.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

pub const FILES: [&str; 4] = [
    "datafile.bin",
    "datafile64.bin",
    "localfile.bin",
    "localfile64.bin",
];

pub fn backup_dir(game: &Path) -> PathBuf {
    game.join("bnsedit-backup")
}

/// Files that are in the backup folder.
pub fn backed_up(game: &Path) -> Vec<&'static str> {
    FILES
        .into_iter()
        .filter(|f| backup_dir(game).join(f).is_file())
        .collect()
}

/// Copy `from` to `to` through a temporary file in the target folder.
fn copy_safe(from: &Path, to: &Path) -> Result<(), String> {
    let bytes = fs::read(from).map_err(|e| format!("Cannot read {}: {e}", from.display()))?;
    let tmp = to.with_extension("bnsedit-tmp");
    let write = || -> std::io::Result<()> {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(&bytes)?;
        f.sync_all()
    };
    if let Err(e) = write() {
        let _ = fs::remove_file(&tmp);
        return Err(format!("Cannot write {}: {e}", tmp.display()));
    }
    fs::rename(&tmp, to).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!(
            "Cannot replace {}: {e}. Close the game and try again.",
            to.display()
        )
    })
}

/// Copy the packed files from `packed` into `game`. Returns the names of the files replaced.
pub fn install(game: &Path, packed: &Path) -> Result<Vec<&'static str>, String> {
    let names: Vec<_> = FILES
        .into_iter()
        .filter(|f| packed.join(f).is_file())
        .collect();
    if names.is_empty() {
        return Err("The pack made no files.".into());
    }
    let backup = backup_dir(game);
    fs::create_dir_all(&backup).map_err(|e| format!("Cannot make {}: {e}", backup.display()))?;
    // 1. Back up all originals before any game file changes.
    for f in &names {
        let original = game.join(f);
        if !original.is_file() {
            return Err(format!("{f} is not in the game folder."));
        }
        if !backup.join(f).is_file() {
            copy_safe(&original, &backup.join(f))?;
        }
    }
    // 2. Replace the game files.
    for f in &names {
        copy_safe(&packed.join(f), &game.join(f))?;
    }
    Ok(names)
}

/// Copy the backed-up originals into the game folder. The backup stays.
pub fn restore(game: &Path) -> Result<Vec<&'static str>, String> {
    let names = backed_up(game);
    if names.is_empty() {
        return Err("There is no backup in this game folder.".into());
    }
    for f in &names {
        copy_safe(&backup_dir(game).join(f), &game.join(f))?;
    }
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_keeps_first_backup_and_restores() {
        let root = std::env::temp_dir().join(format!("bnsedit-install-{}", std::process::id()));
        let (game, p1, p2) = (root.join("game"), root.join("p1"), root.join("p2"));
        for d in [&game, &p1, &p2] {
            fs::create_dir_all(d).unwrap();
        }
        fs::write(game.join("datafile.bin"), b"orig32").unwrap();
        fs::write(game.join("localfile.bin"), b"origL").unwrap();
        fs::write(p1.join("datafile.bin"), b"new1").unwrap();
        fs::write(p2.join("datafile.bin"), b"new2").unwrap();
        assert!(restore(&game).is_err());
        assert_eq!(install(&game, &p1).unwrap(), vec!["datafile.bin"]);
        assert_eq!(install(&game, &p2).unwrap(), vec!["datafile.bin"]);
        assert_eq!(fs::read(game.join("datafile.bin")).unwrap(), b"new2");
        assert_eq!(
            fs::read(backup_dir(&game).join("datafile.bin")).unwrap(),
            b"orig32"
        );
        assert_eq!(restore(&game).unwrap(), vec!["datafile.bin"]);
        assert_eq!(fs::read(game.join("datafile.bin")).unwrap(), b"orig32");
        assert_eq!(fs::read(game.join("localfile.bin")).unwrap(), b"origL");
        // A file missing from the game folder stops before any change.
        fs::write(p1.join("datafile64.bin"), b"x").unwrap();
        assert!(install(&game, &p1).is_err());
        assert_eq!(fs::read(game.join("datafile.bin")).unwrap(), b"orig32");
        fs::remove_dir_all(&root).unwrap();
    }
}
