use anyhow::{bail, Context, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Revision {
    pub name: String,
    pub body: String,
    pub icon: Option<String>,
    pub scope: String,
    pub at: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Macro {
    pub id: Uuid,
    pub current: Revision,
    pub history: Vec<Revision>,
    pub source: Option<String>,
    pub deleted: bool,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Library {
    pub version: u32,
    pub macros: Vec<Macro>,
    #[serde(skip)]
    original: Option<Vec<u8>>,
}
pub mod operations;
pub mod snapshot;
pub use snapshot::{parse_snapshot, SnapshotMacro};

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub fn revision(
    name: String,
    body: String,
    icon: Option<String>,
    scope: String,
) -> Result<Revision> {
    if name.trim().is_empty() {
        bail!("Name cannot be empty")
    }
    if body.trim().is_empty() {
        bail!("Macro body cannot be empty")
    }
    Ok(Revision {
        name,
        body,
        icon,
        scope,
        at: now(),
    })
}
impl Library {
    fn validate(&self) -> Result<()> {
        if self.version != 1 {
            bail!("Unsupported library version {}", self.version);
        }
        let mut ids = std::collections::HashSet::new();
        for record in &self.macros {
            if !ids.insert(record.id) {
                bail!(
                    "Duplicate macro ID {}; repair the library before editing",
                    record.id
                );
            }
        }
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self {
                version: 1,
                macros: vec![],
                original: None,
            });
        }
        let bytes = fs::read(path).with_context(|| format!("Reading {}", path.display()))?;
        let mut lib: Self = serde_json::from_slice(&bytes)?;
        lib.validate()?;
        lib.original = Some(bytes);
        Ok(lib)
    }
    pub fn save(&mut self, path: &Path) -> Result<()> {
        self.validate()?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        let lock_path = if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("lock"))
        {
            appended_path(path, ".lock")
        } else {
            path.with_extension("lock")
        };
        let lock = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(lock_path)?;
        lock.lock_exclusive().context("Locking library")?;
        let current = match fs::read(path) {
            Ok(bytes) => Some(bytes),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        if current != self.original {
            bail!("Library changed in another process; reload it before saving");
        }
        if let Some(bytes) = current.as_deref() {
            let previous: Self = serde_json::from_slice(bytes)?;
            if previous.version == self.version && previous.macros == self.macros {
                return Ok(());
            }
        }
        let data = serde_json::to_vec_pretty(self)?;
        let backup = appended_path(path, ".bak");
        if path.exists() {
            fs::copy(path, &backup).context("Backing up library")?;
        }
        let file = atomicwrites::AtomicFile::new(path, atomicwrites::AllowOverwrite);
        file.write(|f| {
            use std::io::Write;
            f.write_all(&data)
        })
        .map_err(|e| anyhow::anyhow!("Saving library: {e}"))?;
        self.original = Some(data);
        Ok(())
    }
    pub fn add(&mut self, current: Revision, source: Option<String>) -> Uuid {
        let id = Uuid::new_v4();
        self.macros.push(Macro {
            id,
            current,
            history: vec![],
            source,
            deleted: false,
        });
        id
    }
    pub fn resolve(&self, query: &str, include_deleted: bool) -> Result<usize> {
        let active: Vec<_> = self
            .macros
            .iter()
            .enumerate()
            .filter(|(_, m)| include_deleted || !m.deleted)
            .collect();
        if let Some((i, _)) = active
            .iter()
            .find(|(_, m)| m.id.to_string().eq_ignore_ascii_case(query))
        {
            return Ok(*i);
        }
        let names: Vec<_> = active
            .iter()
            .copied()
            .filter(|(_, m)| m.current.name.eq_ignore_ascii_case(query))
            .collect();
        let matches = if names.is_empty()
            && query.len() >= 8
            && query.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-')
        {
            let prefix = query.to_ascii_lowercase();
            active
                .into_iter()
                .filter(|(_, m)| m.id.to_string().starts_with(&prefix))
                .collect()
        } else {
            names
        };
        match matches.as_slice() {
            [(i, _)] => Ok(*i),
            [] => bail!("No macro found: {query}. Run wowmc list to see names and IDs."),
            _ => {
                let choices = matches
                    .iter()
                    .map(|(_, m)| format!("{} ({})", m.id, m.current.name))
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!("Multiple macros match {query}; use a unique ID prefix (8+ characters):\n  {choices}")
            }
        }
    }
}

fn appended_path(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

pub fn snapshots(edition: &Path) -> Result<Vec<PathBuf>> {
    let root = edition.join("WTF").join("Account");
    let mut found = vec![];
    if !root.exists() {
        return Ok(found);
    }
    fn walk(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir).with_context(|| format!("Reading {}", dir.display()))? {
            let e = entry?;
            let ty = e.file_type()?;
            let path = e.path();
            if ty.is_file() && e.file_name() == "macros-cache.txt" {
                out.push(path);
            } else if ty.is_dir() && depth < 3 {
                walk(&path, depth + 1, out)?;
            }
        }
        Ok(())
    }
    walk(&root, 0, &mut found)?;
    found.sort();
    Ok(found)
}
pub fn character_targets(edition: &Path) -> Result<Vec<PathBuf>> {
    let root = edition.join("WTF").join("Account");
    if !root.exists() {
        return Ok(vec![]);
    }
    let mut result = vec![];
    for account in fs::read_dir(root)? {
        let account = account?;
        if !account.file_type()?.is_dir() {
            continue;
        }
        for realm in fs::read_dir(account.path())? {
            let realm = realm?;
            if !realm.file_type()?.is_dir()
                || realm
                    .file_name()
                    .to_string_lossy()
                    .eq_ignore_ascii_case("SavedVariables")
            {
                continue;
            }
            for character in fs::read_dir(realm.path())? {
                let character = character?;
                if !character.file_type()?.is_dir() {
                    continue;
                }
                result.push(character.path().join("macros-cache.txt"));
            }
        }
    }
    result.sort();
    Ok(result)
}
pub fn candidates() -> Vec<PathBuf> {
    let mut paths = vec![];
    for drive in b'C'..=b'Z' {
        for prefix in [
            "battlenet\\World of Warcraft",
            "Games\\World of Warcraft",
            "Program Files (x86)\\World of Warcraft",
            "Program Files\\World of Warcraft",
            "World of Warcraft",
        ] {
            let p = PathBuf::from(format!("{}:\\{}\\_forever_", drive as char, prefix));
            if p.is_dir() {
                paths.push(p);
            }
        }
    }
    paths
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_multiple_and_rejects_incomplete() {
        let s = "VER 3 0001 \"A\" \"123\"\n/one\n/two\nEND\nVER 3 0002 \"B\" \"456\"\n/x\nEND\n";
        let m = parse_snapshot(s).unwrap();
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].body, "/one\n/two");
        assert!(parse_snapshot("VER 3 0001 \"A\" \"123\"\n/x\n").is_err());
        assert!(parse_snapshot("unknown\n").is_err());
    }
    #[test]
    fn ambiguous_names_require_id() {
        let mut l = Library {
            version: 1,
            macros: vec![],
            original: None,
        };
        let r = revision("A".into(), "/x".into(), None, "account".into()).unwrap();
        let id = l.add(r.clone(), None);
        l.add(r, None);
        assert!(l.resolve("A", false).is_err());
        assert_eq!(l.resolve(&id.to_string(), false).unwrap(), 0);
    }
    #[test]
    fn stale_writer_cannot_overwrite_another_save() {
        let fixture = tempfile::tempdir().unwrap();
        let dir = fixture.path();
        let path = dir.join("library.json");
        let mut first = Library::load(&path).unwrap();
        let mut second = Library::load(&path).unwrap();
        first.add(
            revision("First".into(), "/say one".into(), None, "account".into()).unwrap(),
            None,
        );
        first.save(&path).unwrap();
        second.add(
            revision("Second".into(), "/say two".into(), None, "account".into()).unwrap(),
            None,
        );
        assert!(second
            .save(&path)
            .unwrap_err()
            .to_string()
            .contains("changed in another process"));
        let saved = Library::load(&path).unwrap();
        assert_eq!(saved.macros.len(), 1);
        assert_eq!(saved.macros[0].current.name, "First");
    }
    #[test]
    fn concurrent_writers_cannot_both_commit_the_same_version() {
        let fixture = tempfile::tempdir().unwrap();
        let dir = fixture.path();
        let path = dir.join("library.json");
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let threads: Vec<_> = ["First", "Second"]
            .into_iter()
            .map(|name| {
                let path = path.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    let mut lib = Library::load(&path).unwrap();
                    lib.add(
                        revision(name.into(), "/say hi".into(), None, "account".into()).unwrap(),
                        None,
                    );
                    barrier.wait();
                    lib.save(&path).is_ok()
                })
            })
            .collect();
        let results: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
        assert_eq!(results.iter().filter(|&&success| success).count(), 1);
        assert_eq!(Library::load(&path).unwrap().macros.len(), 1);
    }
}

pub mod install;

pub mod config;
