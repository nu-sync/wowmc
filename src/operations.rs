use crate::{revision, Library, Revision, SnapshotMacro};
use anyhow::{bail, Result};
use std::path::Path;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportDecision {
    New,
    AlreadyImported,
    NameConflict,
}

impl ImportDecision {
    pub fn label(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::AlreadyImported => "already imported",
            Self::NameConflict => "name already exists; skipped",
        }
    }
}

pub struct ImportItem {
    pub name: String,
    pub decision: ImportDecision,
}

pub struct ImportPlan {
    pub items: Vec<ImportItem>,
    pending: Vec<Revision>,
}

impl ImportPlan {
    pub fn count(&self) -> usize {
        self.pending.len()
    }
}

impl Library {
    fn commit(&mut self, path: &Path, mut next: Self) -> Result<()> {
        if self.version != 1 {
            bail!("Library was not loaded; resolve the load error before saving");
        }
        next.save(path)?;
        *self = next;
        Ok(())
    }

    pub fn save_macro(&mut self, path: &Path, id: Option<Uuid>, current: Revision) -> Result<Uuid> {
        let mut next = self.clone();
        let id = if let Some(id) = id {
            let index = next.resolve(&id.to_string(), false)?;
            let record = &mut next.macros[index];
            let old = &record.current;
            if old.name != current.name
                || old.body != current.body
                || old.icon != current.icon
                || old.scope != current.scope
            {
                record
                    .history
                    .push(std::mem::replace(&mut record.current, current));
            }
            id
        } else {
            next.add(current, None)
        };
        self.commit(path, next)?;
        Ok(id)
    }

    pub fn delete_macro(&mut self, path: &Path, query: &str) -> Result<()> {
        let mut next = self.clone();
        let index = next.resolve(query, false)?;
        next.macros[index].deleted = true;
        self.commit(path, next)
    }

    pub fn restore_macro(
        &mut self,
        path: &Path,
        query: &str,
        revision: Option<usize>,
    ) -> Result<()> {
        let mut next = self.clone();
        let index = next.resolve(query, true)?;
        let record = &mut next.macros[index];
        if let Some(n) = revision {
            let previous = n
                .checked_sub(1)
                .and_then(|n| record.history.get(n))
                .cloned();
            let Some(previous) = previous else {
                bail!("Invalid revision number");
            };
            record
                .history
                .push(std::mem::replace(&mut record.current, previous));
        }
        record.deleted = false;
        self.commit(path, next)
    }

    pub fn plan_import(
        &self,
        entries: &[SnapshotMacro],
        source: &str,
        scope: &str,
    ) -> Result<ImportPlan> {
        let mut plan = ImportPlan {
            items: Vec::new(),
            pending: Vec::new(),
        };
        for entry in entries {
            let exact = self.macros.iter().any(|m| {
                !m.deleted
                    && m.source.as_deref() == Some(source)
                    && m.current.name == entry.name
                    && m.current.body == entry.body
                    && m.current.icon.as_deref() == Some(&entry.icon)
                    && m.current.scope == scope
            });
            let conflict = self
                .macros
                .iter()
                .any(|m| !m.deleted && m.current.name.eq_ignore_ascii_case(&entry.name))
                || plan
                    .pending
                    .iter()
                    .any(|r| r.name.eq_ignore_ascii_case(&entry.name));
            let decision = if exact {
                ImportDecision::AlreadyImported
            } else if conflict {
                ImportDecision::NameConflict
            } else {
                ImportDecision::New
            };
            if decision == ImportDecision::New {
                plan.pending.push(revision(
                    entry.name.clone(),
                    entry.body.clone(),
                    Some(entry.icon.clone()),
                    scope.into(),
                )?);
            }
            plan.items.push(ImportItem {
                name: entry.name.clone(),
                decision,
            });
        }
        Ok(plan)
    }

    pub fn import(
        &mut self,
        path: &Path,
        entries: &[SnapshotMacro],
        source: &str,
        scope: &str,
    ) -> Result<usize> {
        let plan = self.plan_import(entries, source, scope)?;
        let count = plan.count();
        if count > 0 {
            let mut next = self.clone();
            for current in plan.pending {
                next.add(current, Some(source.into()));
            }
            self.commit(path, next)?;
        }
        Ok(count)
    }
}
