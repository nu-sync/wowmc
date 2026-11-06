pub use crate::snapshot::Scope;
use crate::{snapshot::Snapshot, Revision};
use anyhow::{bail, Context, Result};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Component, Path, PathBuf},
};
use uuid::Uuid;

#[derive(Debug)]
pub enum Plan {
    AlreadyPresent,
    Append {
        id: u64,
        bytes: Vec<u8>,
    },
    Replace {
        id: u64,
        previous: String,
        bytes: Vec<u8>,
    },
}

pub fn validate_target_with_scope(path: &Path) -> Result<(PathBuf, Scope)> {
    if !path
        .file_name()
        .is_some_and(|x| x.eq_ignore_ascii_case("macros-cache.txt"))
    {
        bail!("Target must be named macros-cache.txt");
    }
    let parent = fs::canonicalize(path.parent().context("Target has no parent")?)
        .context("Target account or character folder must exist")?;
    let parts: Vec<_> = parent
        .components()
        .filter_map(|c| {
            if let Component::Normal(x) = c {
                x.to_str()
            } else {
                None
            }
        })
        .collect();
    let n = parts.len();
    let scope = if n >= 5
        && parts[n - 5].eq_ignore_ascii_case("WTF")
        && parts[n - 4].eq_ignore_ascii_case("Account")
    {
        Scope::Character
    } else if n >= 3
        && parts[n - 3].eq_ignore_ascii_case("WTF")
        && parts[n - 2].eq_ignore_ascii_case("Account")
    {
        Scope::Account
    } else {
        bail!("Expected WTF/Account/<account>/macros-cache.txt or WTF/Account/<account>/<realm>/<character>/macros-cache.txt");
    };
    let target = parent.join("macros-cache.txt");
    if fs::symlink_metadata(&target).is_ok_and(|m| m.file_type().is_symlink()) {
        bail!("Macro cache cannot be a symbolic link");
    }
    if target.exists() && !target.is_file() {
        bail!("Target is not a file");
    }
    Ok((target, scope))
}
pub fn validate_target(path: &Path) -> Result<PathBuf> {
    let (p, scope) = validate_target_with_scope(path)?;
    if scope != Scope::Character {
        bail!("Expected a character macro cache");
    }
    Ok(p)
}

pub fn plan_install(
    existing: &[u8],
    macro_: &Revision,
    scope: Scope,
    replace: bool,
) -> Result<Plan> {
    let text = std::str::from_utf8(existing).context("Snapshot is not UTF-8")?;
    let snapshot = Snapshot::parse(text)?;
    let ids = snapshot.validate_for_write(scope)?;
    let max_id = ids.iter().copied().max().unwrap_or(scope.prefix());
    let name = &macro_.name;
    if name.trim().is_empty() || name.chars().count() > 16 || name.contains(['"', '\r', '\n', '\0'])
    {
        bail!("Macro name must be 1–16 characters without quotes or newlines");
    }
    let body = macro_.body.replace("\r\n", "\n");
    if body.trim().is_empty() || body.contains(['\r', '\0']) || body.lines().any(|l| l == "END") {
        bail!("Unsupported macro body");
    }
    if body.len() > 255 {
        bail!("Macro body exceeds the conservative 255-byte limit");
    }
    let icon = macro_.icon.as_deref().unwrap_or("134400");
    if icon.is_empty() || !icon.bytes().all(|b| b.is_ascii_digit()) {
        bail!("Icon must be a numeric file ID");
    }
    let matching: Vec<_> = snapshot
        .records
        .iter()
        .enumerate()
        .filter(|(_, m)| m.entry.name.eq_ignore_ascii_case(name))
        .collect();
    if matching.len() > 1 {
        bail!("Multiple macros named '{name}' exist in the target; no changes made");
    }
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let build = |id: u64| {
        format!(
            "VER 3 {id:016X} \"{name}\" \"{icon}\"{newline}{}{newline}END{newline}",
            body.replace('\n', newline)
        )
        .into_bytes()
    };
    if let Some((i, record)) = matching.first() {
        let old = &record.entry;
        if old.name == *name && old.body == body && old.icon == icon {
            return Ok(Plan::AlreadyPresent);
        }
        if !replace {
            bail!("A different macro named '{name}' exists; preview with --replace to update it");
        }
        let (start, end, id) = (record.span.start, record.span.end, ids[*i]);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&existing[..start]);
        bytes.extend_from_slice(&build(id));
        bytes.extend_from_slice(&existing[end..]);
        return Ok(Plan::Replace {
            id,
            previous: old.body.clone(),
            bytes,
        });
    }
    let id = max_id
        .checked_add(1)
        .context("No available macro identifier")?;
    if id >> 56 != scope.prefix() >> 56 {
        bail!("No available macro identifier");
    }
    let mut bytes = existing.to_vec();
    if !bytes.is_empty() && !bytes.ends_with(b"\n") {
        bytes.extend_from_slice(newline.as_bytes());
    }
    bytes.extend(build(id));
    Ok(Plan::Append { id, bytes })
}
pub fn plan_character_install(existing: &[u8], macro_: &Revision) -> Result<Plan> {
    plan_install(existing, macro_, Scope::Character, false)
}
fn open_existing(path: &Path, write: bool) -> Result<fs::File> {
    let mut o = OpenOptions::new();
    o.read(true).write(write);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        if write {
            o.share_mode(0);
        }
    }
    let f = o
        .open(path)
        .context("Opening target (close other applications holding the file and retry)")?;
    #[cfg(not(windows))]
    if write {
        use fs2::FileExt;
        f.try_lock_exclusive().context("Locking target")?;
    }
    Ok(f)
}
fn backup(path: &Path, original: &[u8]) -> Result<PathBuf> {
    let p = path.with_file_name(format!("macros-cache.wowmc-{}.bak", Uuid::new_v4()));
    let mut f = OpenOptions::new().write(true).create_new(true).open(&p)?;
    f.write_all(original)?;
    f.sync_all()?;
    Ok(p)
}
fn write_existing(
    file: &mut fs::File,
    original: &[u8],
    updated: &[u8],
    backup: &Path,
) -> Result<()> {
    file.seek(SeekFrom::Start(0))?;
    let result = file
        .write_all(updated)
        .and_then(|_| file.set_len(updated.len() as u64))
        .and_then(|_| file.sync_all());
    if let Err(e) = result {
        let rollback = file
            .seek(SeekFrom::Start(0))
            .and_then(|_| file.write_all(original))
            .and_then(|_| file.set_len(original.len() as u64))
            .and_then(|_| file.sync_all());
        bail!(
            "Writing failed: {e}; rollback: {rollback:?}; backup: {}",
            backup.display()
        );
    }
    file.seek(SeekFrom::Start(0))?;
    let mut actual = Vec::new();
    file.read_to_end(&mut actual)?;
    if actual != updated {
        bail!(
            "Verification failed; inspect target and backup at {}",
            backup.display()
        );
    }
    Ok(())
}
pub fn install(
    path: &Path,
    macro_: &Revision,
    apply: bool,
    replace: bool,
) -> Result<(Plan, Option<PathBuf>)> {
    let (path, scope) = validate_target_with_scope(path)?;
    let existed = path.exists();
    let mut file = if existed {
        Some(open_existing(&path, apply)?)
    } else {
        None
    };
    let mut original = Vec::new();
    if let Some(f) = file.as_mut() {
        f.read_to_end(&mut original)?;
    }
    let plan = plan_install(&original, macro_, scope, replace)?;
    if !apply || matches!(plan, Plan::AlreadyPresent) {
        return Ok((plan, None));
    }
    let updated = match &plan {
        Plan::Append { bytes, .. } | Plan::Replace { bytes, .. } => bytes,
        Plan::AlreadyPresent => unreachable!(),
    };
    if let Some(f) = file.as_mut() {
        let b = backup(&path, &original)?;
        write_existing(f, &original, updated, &b)?;
        Ok((plan, Some(b)))
    } else {
        let mut options = OpenOptions::new();
        options.read(true).write(true).create_new(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(0);
        }
        let mut f = options
            .open(&path)
            .context("Target appeared since preview; retry")?;
        if let Err(e) = f.write_all(updated).and_then(|_| f.sync_all()) {
            drop(f);
            let cleanup = fs::remove_file(&path);
            bail!("Creating cache failed: {e}; cleanup: {cleanup:?}");
        }
        f.seek(SeekFrom::Start(0))?;
        let mut actual = Vec::new();
        f.read_to_end(&mut actual)?;
        if &actual != updated {
            bail!("New cache verification failed at {}", path.display());
        }
        Ok((plan, None))
    }
}
pub fn install_character(
    path: &Path,
    macro_: &Revision,
    apply: bool,
) -> Result<(Plan, Option<PathBuf>)> {
    let (_, scope) = validate_target_with_scope(path)?;
    if scope != Scope::Character {
        bail!("Expected a character macro cache");
    }
    install(path, macro_, apply, false)
}
pub fn restore_cache(target: &Path, source: &Path, apply: bool) -> Result<Option<PathBuf>> {
    let (path, scope) = validate_target_with_scope(target)?;
    let source = fs::canonicalize(source).context("Backup not found")?;
    if source.parent() != path.parent()
        || !source
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("macros-cache.wowmc-") && n.ends_with(".bak"))
    {
        bail!("Backup must be a wowmc backup beside the target");
    }
    let replacement = fs::read(&source)?;
    Snapshot::parse(std::str::from_utf8(&replacement).context("Backup is not UTF-8")?)?
        .validate_for_write(scope)
        .context("Backup is not a supported snapshot")?;
    let mut f = open_existing(&path, apply)?;
    let mut original = Vec::new();
    f.read_to_end(&mut original)?;
    if !apply {
        return Ok(None);
    }
    let b = backup(&path, &original)?;
    write_existing(&mut f, &original, &replacement, &b)?;
    Ok(Some(b))
}
