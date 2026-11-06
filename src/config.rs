use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub default_target: Option<PathBuf>,
}
impl Config {
    pub fn library_path(explicit: Option<&Path>) -> Result<PathBuf> {
        if let Some(path) = explicit {
            return Ok(path.to_owned());
        }
        if let Some(path) = std::env::var_os("WOWMC_LIBRARY") {
            return Ok(path.into());
        }
        Ok(PathBuf::from(
            std::env::var_os("APPDATA")
                .context("APPDATA is not set; use --library or WOWMC_LIBRARY")?,
        )
        .join("wowmc/library.json"))
    }

    pub fn target(explicit: Option<PathBuf>) -> Result<PathBuf> {
        match explicit {
            Some(target) => Ok(target),
            None => Self::load(&Self::path()?)?
                .default_target
                .context("No default target selected; run wowmc setup or pass --target"),
        }
    }

    pub fn path() -> Result<PathBuf> {
        if let Some(p) = std::env::var_os("WOWMC_CONFIG") {
            return Ok(p.into());
        }
        Ok(PathBuf::from(
            std::env::var_os("APPDATA").context("APPDATA is not set; set WOWMC_CONFIG")?,
        )
        .join("wowmc/config.json"))
    }
    pub fn load(path: &Path) -> Result<Self> {
        match fs::read(path) {
            Ok(data) => Ok(serde_json::from_slice(&data).context("Reading configuration")?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        let data = serde_json::to_vec_pretty(self)?;
        atomicwrites::AtomicFile::new(path, atomicwrites::AllowOverwrite)
            .write(|f| {
                use std::io::Write;
                f.write_all(&data)
            })
            .map_err(|e| anyhow::anyhow!("Saving configuration: {e}"))?;
        Ok(())
    }
}
