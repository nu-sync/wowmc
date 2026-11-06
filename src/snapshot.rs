use anyhow::{bail, Context, Result};
use std::{collections::HashSet, ops::Range};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Account,
    Character,
}

impl Scope {
    pub(crate) fn prefix(self) -> u64 {
        match self {
            Self::Account => 0,
            Self::Character => 1u64 << 56,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Account => "account",
            Self::Character => "character",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotMacro {
    pub name: String,
    pub icon: String,
    pub body: String,
}

pub(crate) struct Record {
    pub entry: SnapshotMacro,
    pub identifier: String,
    header_supported: bool,
    pub span: Range<usize>,
}

pub(crate) struct Snapshot<'a> {
    pub text: &'a str,
    pub records: Vec<Record>,
}

impl<'a> Snapshot<'a> {
    pub fn parse(text: &'a str) -> Result<Self> {
        let mut records = Vec::new();
        let mut lines = text.split_inclusive('\n').enumerate();
        let mut offset = 0;
        while let Some((n, raw)) = lines.next() {
            let start = offset;
            offset += raw.len();
            let line = raw.strip_suffix('\n').unwrap_or(raw);
            let line = line.strip_suffix('\r').unwrap_or(line);
            if line.trim().is_empty() {
                continue;
            }
            if !line.starts_with("VER 3 ") {
                bail!("Unsupported header at line {}", n + 1);
            }
            let quote = line.find('"').context("Missing quoted name")?;
            let parts: Vec<_> = line[quote..].split('"').collect();
            if parts.len() != 5
                || !parts[0].is_empty()
                || !parts[2].trim().is_empty()
                || !parts[4].trim().is_empty()
            {
                bail!("Invalid header at line {}", n + 1);
            }
            let identifier = line[..quote]
                .split_whitespace()
                .nth(2)
                .unwrap_or("")
                .to_owned();
            let header_supported = line[..quote].split_whitespace().count() == 3
                && line[..quote].ends_with(char::is_whitespace);
            let mut body = Vec::new();
            let mut ended = false;
            for (_, raw) in lines.by_ref() {
                offset += raw.len();
                let line = raw.strip_suffix('\n').unwrap_or(raw);
                let line = line.strip_suffix('\r').unwrap_or(line);
                if line == "END" {
                    ended = true;
                    break;
                }
                body.push(line);
            }
            if !ended {
                bail!("Missing END for macro at line {}", n + 1);
            }
            records.push(Record {
                entry: SnapshotMacro {
                    name: parts[1].into(),
                    icon: parts[3].into(),
                    body: body.join("\n"),
                },
                identifier,
                header_supported,
                span: start..offset,
            });
        }
        Ok(Self { text, records })
    }

    pub fn validate_for_write(&self, scope: Scope) -> Result<Vec<u64>> {
        if self.text.replace("\r\n", "").contains('\r') {
            bail!("Unsupported line endings in snapshot");
        }
        let mut seen = HashSet::new();
        self.records
            .iter()
            .map(|record| {
                if !record.header_supported {
                    bail!("Unsupported snapshot header");
                }
                let raw = &record.identifier;
                if raw.len() != 16 || !raw.bytes().all(|b| b.is_ascii_hexdigit()) {
                    bail!("Unsupported macro identifier");
                }
                let id = u64::from_str_radix(raw, 16)?;
                if id >> 56 != scope.prefix() >> 56 || id == scope.prefix() {
                    bail!("Macro identifier does not match target scope");
                }
                if !seen.insert(id) {
                    bail!("Duplicate macro identifiers in snapshot");
                }
                Ok(id)
            })
            .collect()
    }
}

pub fn parse_snapshot(text: &str) -> Result<Vec<SnapshotMacro>> {
    Ok(Snapshot::parse(text)?
        .records
        .into_iter()
        .map(|r| r.entry)
        .collect())
}
