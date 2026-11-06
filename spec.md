# wowmc: product specification

Status: proposal · 2026-11-05

## Vision

wowmc is a local library for World of Warcraft macros. It helps players collect, find, edit, organize, and recover macros across characters, then move them into the game with a clear understanding of what has and has not happened.

Start with WoW Forever on Windows. A Rust core serves both a command-line interface (CLI) and, later, a graphical interface (GUI). The library works offline and does not require an account.

## The problem

Macros are short, but managing them across characters, guides, and revisions is awkward. The game’s local macro files provide useful snapshots, but a successful file write does not establish that the game accepted or retained a change. wowmc must not confuse a library save, a local snapshot, and confirmation in game.

A community [Forever migration guide](https://us.forums.blizzard.com/en/wow/t/from-beta-to-live-wow-forever/2386486) documents account and character cache locations and cautions that its file-copy test was performed in beta. On November 5, 2026, the user reported that a character macro appended while Forever build 1.60.1.70790 was running worked after `/reload`. This is one live observation; persistence after restart and other configurations remain unverified.

## Users and outcomes

- A player with existing macros can bring them into a searchable library without modifying game files.
- A player copying a macro from a guide can save and edit it, then transfer it into WoW with clear instructions.
- A player managing several characters can find the right macro, compare it with a local snapshot, and recover a previous library version.
- A terminal user can perform the same core library operations interactively or from scripts.

## Product principles

1. **Clarity over implied synchronization.** Show the source, scope, and observation time of game snapshots. Never describe a file write or copy as an in-game installation.
2. **A useful library first.** Core editing, search, and recovery must work even if automatic installation proves unreliable.
3. **Safe by default.** Reading game files is separate from editing them. Preview imports and preserve library history.
4. **One core, two interfaces.** CLI and GUI share behavior and storage; neither needs the other to run.
5. **Small, polished loop.** Optimize finding, editing, saving, and transferring a macro before adding advanced features.

## Onboarding

Onboarding is optional and repeatable. A user can start an empty library without WoW installed and connect an installation later.

### 1. Welcome

Explain the benefit in one sentence: “Keep your macros organized and ready to use.” Offer **Find my WoW installation** and **Start with an empty library**. Briefly explain that importing reads local files and does not change macros in the game.

### 2. Find an installation

Search likely Windows installation locations and show each candidate with its path and detected edition. Let the user browse to a folder or enter a path. Do not assume a single install or account. If nothing is found, explain how to select the folder manually and allow the user to continue with an empty library.

### 3. Choose what to inspect

Show discovered accounts, realms, and characters with account-wide and character-specific scopes. Allow multiple selections and an optional default context. A missing character macro file is an empty or unavailable snapshot, not an error: the file may not yet have been created. Display readable names when available and preserve directory identifiers when they are not.

### 4. Review the import

Read selected files without modifying them. Show each source, last modified time, macro count, and parse warnings. Preview macros and let the user select what to import. Duplicate names are not enough to identify a macro; show scope and body when resolving possible duplicates. A file that cannot be read or parsed should have a clear error and a retry or skip option; do not silently discard unrecognized content.

### 5. Finish with a useful first action

After confirmation, create library records and show the library with a brief explanation: “These are copies from your local WoW files. They may not reflect the current in-game state.” Offer **Open a macro**, **Add a macro**, and **Copy a macro for WoW**. Re-running import previews changes and avoids creating duplicates silently; the exact matching and conflict rules must be specified and tested before implementation.

The CLI follows the same sequence with prompts, explicit paths, and non-interactive options. It should explain failures and allow setup to be resumed. No setup step writes to the game installation.

## Everyday experience

### Find and inspect

Search names and bodies; filter by intended scope, character, and tags. Each result shows its name, scope, and library status. The detail view shows the full body, icon metadata if known, last library edit, and any associated snapshot with its observation time. Duplicate names remain distinguishable by scope and stable ID.

### Add and edit

Create a macro by pasting text, importing a file, or editing an existing library record. Provide a multiline editor, name, intended scope, optional icon, and tags. Show a length indicator and known validation issues. Edition-specific limits and how length is counted must be verified against the target client before being enforced. Preserve unfamiliar or potentially valid syntax and distinguish errors from advisory warnings.

Saving updates the library only. Preserve the prior revision so it can be compared or restored. In the CLI, editing opens the configured editor; file and standard-input paths support scripting. Deleting a library record must require an explicit action and offer recovery through history or a documented trash mechanism.

### Transfer to the game

The CLI can remember a character target and preview an append or explicit replacement in an account or character cache. Existing caches are backed up and can be restored; new caches can be created in existing account or character folders. After the write, the player reloads and checks it in game; a local write alone does not confirm that the game loaded it. Copy and paste remains available as a fallback.

Status describes evidence, not a presumed lifecycle: **Saved in library**, **Ready to paste**, and **Observed in local snapshot** are distinct facts. A matching snapshot is an observation at a recorded time, not proof of current server state. Use **Confirmed in game** only if a future, tested integration can actually establish it.

### Refresh and compare

A manual refresh rereads selected snapshots and shows additions, changes, and missing entries. It does not overwrite library edits. Present a side-by-side comparison and explicit choices to import a new record, update a library record while retaining history, or ignore the observation. Do not infer deletion from an unavailable or unreadable source.

## Interface sketches

GUI main view:

```text
Characters & filters     Macro library          Macro editor
────────────────────     ─────────────          ─────────────────────
All macros               Emergency Heal         Emergency Heal
Account-wide             Focus Interrupt        Character: MyCharacter
MyCharacter              Travel                 [multiline macro body]
  Healing                                       Saved in library
  Utility                                       [Save] [Copy for WoW]
```

Illustrative CLI (subject to implementation design):

```text
wowmc setup
wowmc list --character MyCharacter
wowmc show "Emergency Heal"
wowmc add --file heal.txt
wowmc edit "Emergency Heal"
wowmc diff "Emergency Heal"
wowmc copy "Emergency Heal"
```

Ambiguous names prompt for selection in interactive use; scripts can use stable IDs. Provide machine-readable output and meaningful exit codes for automation, without making the default output difficult to read.

## Data and architecture

A library record has a stable ID, name, body, optional icon, intended scope, tags, and revision history. A snapshot is a separate observation with installation, account and character context, source path, and file modification or observation time. A relationship between a library record and a snapshot must not imply synchronization.

The Rust core owns library operations, validation, history, comparison, and import/export. A WoW integration layer discovers installations and reads snapshots. CLI and GUI are independent clients of the same core. Start with a versioned, human-readable local format if a small prototype confirms it supports reliable saves, recovery, and migrations. Decide the GUI toolkit after testing editor usability and Windows packaging.

Keep storage and game paths configurable. Library writes should be crash-safe, and backups or recovery should be tested. Preserve original macro text on import, including line breaks. Never execute macro text as code in wowmc.

## Delivery stages and acceptance

### Stage 1: discovery and feasibility

- On Windows, discover or manually select a Forever installation and enumerate available account and character snapshots without writing to it.
- Parse representative copied files, including empty, missing, malformed, and unusual inputs; report unsupported data without losing it.
- Separately test a harmless direct cache edit with WoW closed, a backup, an in-game check, and a restart check. Record the client version and result. A successful write is not a successful installation.

### Stage 2: library and CLI

- Complete setup or start empty; import selected snapshots after preview.
- Add, list, search, inspect, edit, export, and recover library macros; disambiguate duplicate names.
- Copy or output a macro for manual in-game entry. Explain what the action did.
- Refresh and compare snapshots without overwriting edits. Verify behavior with realistic fixtures and an actual local installation when available.

### Stage 3: GUI

- Complete the same first-run flow and core editing and transfer journey through the GUI.
- Test empty, missing-installation, malformed-file, duplicate, and conflict states as well as the happy path.
- Verify that GUI and CLI see the same library and preserve revisions.

### Stage 4: integration research

Prototype a companion addon only after testing the handoff across game load/save boundaries, the user's in-game action, conflicts, and the return of any confirmation. Character-cache appends have one reported live-client success after reload; other cases and persistence after restart remain unverified.

## Out of scope for the initial release

Cloud accounts and synchronization, automatic gameplay, guaranteed macro correctness, bindings or full UI migration, and support claims for untested WoW editions. Advanced syntax assistance, shared collections, and addon integration are candidates for later work, not commitments.

## Decisions and open questions

- Which Forever installation and characters should be used for the first real-world test?
- What are the live client's actual macro limits, counting rules, file formats, and persistence behavior?
- What matching rules make repeated imports predictable without merging unrelated macros?
- Which GUI toolkit gives the best Windows editing and distribution experience?
- Can an addon provide a practical, observable handoff without overstating confirmation?

These are research or implementation decisions; none should block an empty-library and manual-copy workflow.
