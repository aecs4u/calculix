use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RestartState {
    pub schema_version: u32,
    pub step: usize,
    pub increment: usize,
    pub time: f64,
    pub unknowns: Vec<f64>,
    pub metadata: BTreeMap<String, String>,
}

impl Default for RestartState {
    fn default() -> Self {
        Self {
            schema_version: 1,
            step: 1,
            increment: 0,
            time: 0.0,
            unknowns: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }
}

pub fn save_restart(path: impl AsRef<Path>, state: &RestartState) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    let bytes = serde_json::to_vec_pretty(state)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    fs::write(path, bytes)
}

pub fn load_restart(path: impl AsRef<Path>) -> io::Result<RestartState> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn restart_roundtrip_preserves_state() {
        let path = unique_temp_file("ccx_restart_roundtrip", "restart.json");
        let mut metadata = BTreeMap::new();
        metadata.insert("job".to_string(), "beam_static".to_string());
        metadata.insert("solver".to_string(), "ccx-solver".to_string());

        let state = RestartState {
            schema_version: 1,
            step: 3,
            increment: 12,
            time: 1.25,
            unknowns: vec![0.1, -2.3, 9.9],
            metadata,
        };

        save_restart(&path, &state).expect("save should succeed");
        let loaded = load_restart(&path).expect("load should succeed");
        assert_eq!(loaded, state);
    }

    #[test]
    fn load_restart_fails_for_missing_file() {
        let path = unique_temp_file("ccx_restart_missing", "missing.json");
        let err = load_restart(&path).expect_err("missing file should fail");
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn load_restart_fails_for_invalid_payload() {
        let path = unique_temp_file("ccx_restart_invalid", "bad.json");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create temp directory");
        }
        fs::write(&path, "{invalid json").expect("write invalid payload");
        let err = load_restart(&path).expect_err("invalid JSON should fail");
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    fn unique_temp_file(prefix: &str, filename: &str) -> PathBuf {
        let pid = std::process::id();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir()
            .join(format!("{prefix}_{pid}_{nanos}"))
            .join(filename)
    }
}
