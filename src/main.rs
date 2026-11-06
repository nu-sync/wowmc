use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::{
    fs,
    io::{self, IsTerminal, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use wowmc::{candidates, parse_snapshot, revision, snapshots, Library};

#[derive(Parser)]
#[command(version, about = "A local, independent library for WoW macros")]
struct Cli {
    /// Override the library file (also available as WOWMC_LIBRARY).
    #[arg(long, global = true)]
    library: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Action>,
}
#[derive(Subcommand)]
enum Action {
    /// Find Forever installations and inspect available snapshots.
    Setup {
        #[arg(long)]
        path: Option<PathBuf>,
        /// Set the default account or character macro cache.
        #[arg(long)]
        target: Option<PathBuf>,
    },
    /// Preview a snapshot (defaults to setup's target); --apply imports new names.
    Import {
        path: Option<PathBuf>,
        #[arg(long)]
        apply: bool,
        /// Override scope (inferred from game paths, otherwise account).
        #[arg(long)]
        scope: Option<String>,
    },
    /// List library macros.
    List {
        #[arg(long)]
        search: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        json: bool,
    },
    /// Show one macro by name, full ID, or unique ID prefix (8+ characters).
    Show { query: String },
    /// Add a macro in your editor, or from --file PATH / --file - (stdin).
    Add {
        name: String,
        #[arg(long)]
        file: Option<PathBuf>,
        #[arg(long, default_value = "account")]
        scope: String,
        #[arg(long)]
        icon: Option<String>,
    },
    /// Edit a macro body in your configured editor.
    Edit { query: String },
    /// Move a library macro to the recoverable trash.
    Delete { query: String },
    /// Restore a deleted macro or an earlier revision.
    Restore {
        query: String,
        #[arg(long)]
        revision: Option<usize>,
    },
    /// Show earlier revisions (numbered from 1, oldest first).
    History { query: String },
    /// Print a macro body for pasting into WoW.
    Export { query: String },
    /// Copy a macro body to the Windows clipboard.
    Copy { query: String },
    /// Preview or write a library macro to an account or character snapshot.
    Install {
        query: String,
        /// Account or character macros-cache.txt (defaults to the setup selection).
        #[arg(long)]
        target: Option<PathBuf>,
        /// Write the change after previewing it; creates a backup.
        #[arg(long)]
        apply: bool,
        /// Allow replacing a same-named macro after reviewing the preview.
        #[arg(long)]
        replace: bool,
    },
    /// Preview or restore a cache from a nearby wowmc backup.
    RestoreCache {
        #[arg(long)]
        backup: PathBuf,
        #[arg(long)]
        target: Option<PathBuf>,
        #[arg(long)]
        apply: bool,
    },
    /// Compare a library macro with the latest local game snapshot.
    Compare {
        query: String,
        #[arg(long)]
        target: Option<PathBuf>,
    },
}
fn read_text(path: &Path) -> Result<String> {
    if path == Path::new("-") {
        let mut s = String::new();
        io::stdin().read_to_string(&mut s)?;
        Ok(s)
    } else {
        fs::read_to_string(path).with_context(|| format!("Reading {}", path.display()))
    }
}
fn next_after_setup(target: &Path) {
    if target.exists() {
        println!("Next: wowmc import to preview this target's macros, then wowmc import --apply to save new ones.");
    } else {
        println!("No saved cache exists yet. Create a library macro with wowmc add \"My macro\", then preview it with wowmc install \"My macro\".");
    }
}

fn edit_body(initial: &str) -> Result<(String, PathBuf)> {
    let editor = std::env::var_os("VISUAL").or_else(|| std::env::var_os("EDITOR"));
    if editor.is_none() && !io::stdin().is_terminal() {
        bail!(
            "For scripts, use add NAME --file PATH or --file -; set VISUAL or EDITOR for editing."
        );
    }
    let editor = editor
        .or_else(|| cfg!(windows).then(|| "notepad.exe".into()))
        .context("Set VISUAL or EDITOR to an editor executable")?;
    let draft = std::env::temp_dir().join(format!("wowmc-{}.txt", uuid::Uuid::new_v4()));
    fs::write(&draft, initial)?;
    println!(
        "Edit and save your macro, then close the editor. Draft: {}",
        draft.display()
    );
    let status = Command::new(editor)
        .arg(&draft)
        .status()
        .with_context(|| format!("Starting editor; draft retained at {}", draft.display()))?;
    if !status.success() {
        bail!("Editor failed; draft retained at {}", draft.display());
    }
    let body = fs::read_to_string(&draft)?;
    Ok((body, draft))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let Some(command) = cli.command else {
        println!("Keep your WoW macros in a local library.\n\nStart with:\n  wowmc setup          Choose an account or character\n  wowmc import         Preview its saved macros\n  wowmc import --apply Save new macros to your library\n  wowmc list           Find a macro\n\nOr create one: wowmc add \"My macro\"\nUse wowmc --help for all commands.");
        return Ok(());
    };
    let library_path = || wowmc::config::Config::library_path(cli.library.as_deref());
    match command {
        Action::Setup { path, target } => {
            if let Some(target) = target {
                let (target, scope) = wowmc::install::validate_target_with_scope(&target)?;
                let config_path = wowmc::config::Config::path()?;
                let mut config = wowmc::config::Config::load(&config_path)?;
                config.default_target = Some(target.clone());
                config.save(&config_path)?;
                println!("Default {} target: {}", scope.label(), target.display());
                next_after_setup(&target);
                return Ok(());
            }
            let paths = if let Some(p) = path {
                vec![p]
            } else {
                candidates()
            };
            if paths.is_empty() {
                println!("No Forever installation found. Run setup --path <edition folder>, or start with an empty library using add.");
            }
            let mut choices = Vec::new();
            for p in paths {
                println!("Installation: {}", p.display());
                let files = snapshots(&p)?;
                choices.extend(wowmc::character_targets(&p)?);
                if files.is_empty() {
                    println!("  No macro snapshots found. Characters without saved macros may have no file.");
                }
                for f in files {
                    match fs::read_to_string(&f)
                        .context("read failed")
                        .and_then(|s| parse_snapshot(&s))
                    {
                        Ok(ms) => {
                            let modified = fs::metadata(&f)
                                .and_then(|m| m.modified())
                                .ok()
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs());
                            println!(
                                "  {} ({} macros; modified Unix time {:?})",
                                f.display(),
                                ms.len(),
                                modified
                            );
                        }
                        Err(e) => println!("  {}: {e:#}", f.display()),
                    }
                }
            }
            println!("These are local snapshots, which may not reflect current in-game macros. Use import <file> to preview.");
            if choices.len() == 1 || (!choices.is_empty() && io::stdin().is_terminal()) {
                let selected = if choices.len() == 1 {
                    Some(0)
                } else {
                    println!("Choose a default character (0 to skip):");
                    for (i, choice) in choices.iter().enumerate() {
                        println!("  {}. {}", i + 1, choice.display());
                    }
                    print!("Selection: ");
                    io::stdout().flush()?;
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    let n: usize = input.trim().parse().context("Enter a selection number")?;
                    if n > choices.len() {
                        bail!("Selection out of range");
                    }
                    n.checked_sub(1)
                };
                if let Some(i) = selected {
                    let config_path = wowmc::config::Config::path()?;
                    let mut config = wowmc::config::Config::load(&config_path)?;
                    config.default_target = Some(choices[i].clone());
                    config.save(&config_path)?;
                    println!("Default character: {}", choices[i].display());
                    next_after_setup(&choices[i]);
                }
            } else if choices.len() > 1 {
                println!("Multiple characters found; use setup --target <character file> to choose a default.");
            }
        }
        Action::Import {
            path: source,
            apply,
            scope,
        } => {
            let source = wowmc::config::Config::target(source)?;
            let scope = scope.unwrap_or_else(|| {
                wowmc::install::validate_target_with_scope(&source)
                    .map(|(_, scope)| scope.label().to_owned())
                    .unwrap_or_else(|_| "account".into())
            });
            let text = read_text(&source)?;
            let entries = parse_snapshot(&text)?;
            let path = library_path()?;
            let mut lib = Library::load(&path)?;
            let source_name = source.to_string_lossy().to_string();
            println!(
                "Snapshot: {} ({} macros). This may not reflect the current in-game state.",
                source.display(),
                entries.len()
            );
            let plan = lib.plan_import(&entries, &source_name, &scope)?;
            for item in &plan.items {
                println!("  {} [{}] ({})", item.name, scope, item.decision.label());
            }
            if apply {
                let count = lib.import(&path, &entries, &source_name, &scope)?;
                println!("Imported {count} macros into the library. No game files were changed.");
                println!("Next: wowmc list");
            } else {
                println!(
                    "Preview only. Re-run with --apply to import {} new macros.",
                    plan.count()
                );
            }
        }
        Action::List { search, all, json } => {
            let path = library_path()?;
            let lib = Library::load(&path)?;
            let q = search.unwrap_or_default().to_lowercase();
            let items: Vec<_> = lib
                .macros
                .iter()
                .filter(|m| {
                    (all || !m.deleted)
                        && (m.current.name.to_lowercase().contains(&q)
                            || m.current.body.to_lowercase().contains(&q))
                })
                .collect();
            if json {
                println!("{}", serde_json::to_string_pretty(&items)?);
            } else {
                if items.is_empty() {
                    if !q.is_empty() {
                        println!(
                            "No macros match this search. Run wowmc list to see all active macros."
                        );
                    } else if lib.macros.iter().any(|m| m.deleted) && !all {
                        println!("No active macros. Run wowmc list --all to find deleted macros, then wowmc restore <name-or-id>.");
                    } else {
                        println!("Your library is empty. Run wowmc setup, then wowmc import; or create one with wowmc add \"My macro\".");
                    }
                }
                for m in items {
                    println!(
                        "{}  {}  [{}]{}",
                        m.id,
                        m.current.name,
                        m.current.scope,
                        if m.deleted { " (deleted)" } else { "" }
                    );
                }
            }
        }
        Action::Show { query } => {
            let path = library_path()?;
            let lib = Library::load(&path)?;
            let m = &lib.macros[lib.resolve(&query, false)?];
            println!(
                "{}\nID: {}\nScope: {}\nIcon: {}\nSource: {}\nSaved in library\n\n{}",
                m.current.name,
                m.id,
                m.current.scope,
                m.current.icon.as_deref().unwrap_or("unspecified"),
                m.source.as_deref().unwrap_or("library"),
                m.current.body
            );
        }
        Action::Add {
            name,
            file,
            scope,
            icon,
        } => {
            let path = library_path()?;
            let mut lib = Library::load(&path)?;
            let (body, draft) = match file {
                Some(file) => (read_text(&file)?, None),
                None => {
                    let (body, draft) = edit_body("")?;
                    (body, Some(draft))
                }
            };
            let r = revision(
                name,
                body.trim_end_matches(['\r', '\n']).to_owned(),
                icon,
                scope,
            )?;
            if lib
                .macros
                .iter()
                .any(|m| !m.deleted && m.current.name.eq_ignore_ascii_case(&r.name))
            {
                eprintln!(
                    "Warning: a macro with this name already exists; use IDs to distinguish them."
                );
            }
            let id = lib.save_macro(&path, None, r)?;
            if let Some(draft) = draft {
                let _ = fs::remove_file(draft);
            }
            println!("Saved in library: {id}");
        }
        Action::Edit { query } => {
            let path = library_path()?;
            let mut lib = Library::load(&path)?;
            let i = lib.resolve(&query, false)?;
            let (body, temp) = edit_body(&lib.macros[i].current.body)?;
            let old = lib.macros[i].current.clone();
            let new = revision(
                old.name.clone(),
                body.trim_end_matches(['\r', '\n']).to_owned(),
                old.icon.clone(),
                old.scope.clone(),
            )?;
            if new.body != old.body {
                lib.save_macro(&path, Some(lib.macros[i].id), new)
                    .with_context(|| format!("Edited text retained at {}", temp.display()))?;
                println!("Saved in library.");
            } else {
                println!("No changes.");
            }
            let _ = fs::remove_file(temp);
        }
        Action::Delete { query } => {
            let path = library_path()?;
            let mut lib = Library::load(&path)?;
            lib.delete_macro(&path, &query)?;
            println!("Moved to library trash; restore using its ID.");
        }
        Action::Restore {
            query,
            revision: rev,
        } => {
            let path = library_path()?;
            let mut lib = Library::load(&path)?;
            lib.restore_macro(&path, &query, rev)?;
            println!("Restored in library.");
        }
        Action::History { query } => {
            let path = library_path()?;
            let lib = Library::load(&path)?;
            let m = &lib.macros[lib.resolve(&query, true)?];
            for (i, r) in m.history.iter().enumerate() {
                println!("Revision {} (Unix time {}):\n{}\n", i + 1, r.at, r.body);
            }
            if m.history.is_empty() {
                println!("No earlier revisions.");
            }
        }
        Action::Export { query } => {
            let path = library_path()?;
            let lib = Library::load(&path)?;
            print!("{}", lib.macros[lib.resolve(&query, false)?].current.body);
        }
        Action::Install {
            query,
            target,
            apply,
            replace,
        } => {
            let target = wowmc::config::Config::target(target)?;
            let path = library_path()?;
            let lib = Library::load(&path)?;
            let m = &lib.macros[lib.resolve(&query, false)?];
            let (plan, backup) = wowmc::install::install(&target, &m.current, apply, replace)?;
            println!(
                "Target: {}\nMacro: {}\nIntended library scope: {}",
                target.display(),
                m.current.name,
                m.current.scope
            );
            match plan {
                wowmc::install::Plan::AlreadyPresent => {
                    println!("An identical macro is already in the local snapshot; no change made.")
                }
                wowmc::install::Plan::Append { id, .. } if !apply => {
                    println!("Preview: append macro with ID {id:016X}.\n\n{}\n\nRe-run with --apply to write it.", m.current.body);
                }
                wowmc::install::Plan::Replace { id, previous, .. } if !apply => {
                    println!("Preview: replace macro with ID {id:016X}.\n\nExisting:\n{previous}\n\nNew:\n{}\n\nRe-run with --replace --apply to write it.", m.current.body);
                }
                wowmc::install::Plan::Append { .. } | wowmc::install::Plan::Replace { .. } => {
                    if let Some(backup) = backup {
                        println!("Written to local snapshot. Backup: {}", backup.display());
                        println!(
                            "To preview undo: wowmc restore-cache --backup '{}' --target '{}'",
                            backup.display().to_string().replace('\'', "''"),
                            target.display().to_string().replace('\'', "''")
                        );
                    } else {
                        println!("Created local snapshot; no prior file existed.");
                    }
                    println!("Reload WoW, then check in game. A file write is not confirmation the game loaded or retained it.");
                }
            }
        }
        Action::RestoreCache {
            backup,
            target,
            apply,
        } => {
            let target = wowmc::config::Config::target(target)?;
            let saved = wowmc::install::restore_cache(&target, &backup, apply)?;
            println!(
                "Target: {}\nRestore from: {}",
                target.display(),
                backup.display()
            );
            if let Some(saved) = saved {
                println!("Restored local snapshot. Previous contents backed up at {}. Reload and check in game.",saved.display());
            } else {
                println!("Preview only; add --apply to restore this backup.");
            }
        }
        Action::Compare { query, target } => {
            let target = wowmc::config::Config::target(target)?;
            let (target, _) = wowmc::install::validate_target_with_scope(&target)?;
            let path = library_path()?;
            let lib = Library::load(&path)?;
            let m = &lib.macros[lib.resolve(&query, false)?];
            println!(
                "Target: {}\nLibrary: {}\n",
                target.display(),
                m.current.name
            );
            if !target.exists() {
                println!("No local snapshot exists. This does not indicate the in-game state.");
            } else {
                let text = fs::read_to_string(&target)?;
                let entries = parse_snapshot(&text)?;
                let found: Vec<_> = entries
                    .iter()
                    .filter(|e| e.name.eq_ignore_ascii_case(&m.current.name))
                    .collect();
                match found.as_slice() {
                    []=>println!("Not found in local snapshot."),
                    [entry]=> {
                        if entry.name==m.current.name && entry.body==m.current.body.replace("\r\n","\n") && entry.icon==m.current.icon.as_deref().unwrap_or("134400") { println!("Local snapshot matches the library."); }
                        else { println!("Local snapshot differs.\n\nSnapshot (icon {}):\n{}\n\nLibrary (icon {}):\n{}",entry.icon,entry.body,m.current.icon.as_deref().unwrap_or("134400"),m.current.body); }
                    }
                    _=>println!("Multiple same-named macros exist in the snapshot; comparison is ambiguous."),
                }
                if let Ok(t) = fs::metadata(&target)
                    .and_then(|m| m.modified())
                    .and_then(|t| {
                        t.duration_since(std::time::UNIX_EPOCH)
                            .map_err(std::io::Error::other)
                    })
                {
                    println!("Snapshot modified at Unix time {}.", t.as_secs());
                }
                println!("This snapshot is not confirmation of the current in-game state.");
            }
        }
        Action::Copy { query } => {
            let path = library_path()?;
            let lib = Library::load(&path)?;
            let m = &lib.macros[lib.resolve(&query, false)?];
            let mut child = Command::new("clip.exe")
                .stdin(Stdio::piped())
                .spawn()
                .context("Starting Windows clipboard")?;
            child
                .stdin
                .take()
                .context("Clipboard input")?
                .write_all(m.current.body.as_bytes())?;
            if !child.wait()?.success() {
                bail!("Clipboard copy failed");
            }
            println!("Copied body. In WoW, create or edit '{}' in scope '{}' and paste it. This does not install the macro.",m.current.name,m.current.scope);
        }
    }
    Ok(())
}
