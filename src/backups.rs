//tools for backing up level data

use std::path::{Path, PathBuf};

use super::levelstring::get_local_levels_path;

pub fn back_up_data(backup_name: String) -> Option<String> {
    let gd_path = match get_local_levels_path() {
        Ok(p) => p,
        Err(e) => return Some(e),
    };

    let backup_path = match get_backup_path() {
        Ok(p) => p,
        Err(e) => return Some(e),
    };

    let backup_info = BackupInfo { name: backup_name };

    None
}
pub struct BackupInfo {
    name: String,
}

pub fn get_backup_list() -> Result<Vec<BackupInfo>, String> {
    let backup_path = match get_backup_path() {
        Ok(p) => p,
        Err(e) => return Err(e),
    };

    Ok(Vec::new())
}

pub fn get_backup_path() -> Result<PathBuf, String> {
    Ok(PathBuf::from(match std::env::var("localappdata") {
        Ok(path) => path,
        Err(e) => return Err(e.to_string()),
    })
    .join("LVLShare"))
}
