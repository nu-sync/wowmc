use std::{fs, path::PathBuf};
mod common;
use common::Fixture;

fn setup() -> (Fixture, PathBuf) {
    let dir = Fixture::new();
    let target = dir.join("WTF/Account/123/Realm/Character/macros-cache.txt");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(
        &target,
        b"VER 3 0100000000000001 \"Existing\" \"123\"\r\n/say old\r\nEND\r\n",
    )
    .unwrap();
    (dir, target)
}
#[test]
fn preview_apply_retry_and_conflict_preserve_original() {
    let (dir, target) = setup();
    let library = dir.join("library.json");
    let body = dir.join("body.txt");
    let original = fs::read(&target).unwrap();
    fs::write(&body, "#showtooltip Shoot\n/cast Shoot").unwrap();
    let add = dir.run(&[
        "--library".as_ref(),
        library.as_os_str(),
        "add".as_ref(),
        "Wand".as_ref(),
        "--file".as_ref(),
        body.as_os_str(),
    ]);
    assert!(add.status.success());
    let args = [
        "--library".as_ref(),
        library.as_os_str(),
        "install".as_ref(),
        "Wand".as_ref(),
        "--target".as_ref(),
        target.as_os_str(),
    ];
    let preview = dir.run(&args);
    assert!(preview.status.success());
    assert!(String::from_utf8_lossy(&preview.stdout).contains("Preview"));
    assert_eq!(fs::read(&target).unwrap(), original);
    let mut apply = args.to_vec();
    apply.push("--apply".as_ref());
    let result = dir.run(&apply);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let after = fs::read(&target).unwrap();
    assert!(after.starts_with(&original));
    let text = String::from_utf8(after.clone()).unwrap();
    assert!(text.contains("VER 3 0100000000000002 \"Wand\" \"134400\"\r\n#showtooltip Shoot\r\n/cast Shoot\r\nEND\r\n"));
    let backup = fs::read_dir(target.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|e| e == "bak"))
        .unwrap();
    assert_eq!(fs::read(backup).unwrap(), original);
    let retry = dir.run(&apply);
    assert!(retry.status.success());
    assert!(String::from_utf8_lossy(&retry.stdout).contains("already"));
    assert_eq!(fs::read(&target).unwrap(), after);
    fs::write(&body, "/say different").unwrap();
    let add2 = dir.run(&[
        "--library".as_ref(),
        library.as_os_str(),
        "add".as_ref(),
        "Wand".as_ref(),
        "--file".as_ref(),
        body.as_os_str(),
    ]);
    assert!(add2.status.success());
    let lib = wowmc::Library::load(&library).unwrap();
    let id = lib.macros[1].id.to_string();
    let conflict = dir.run(&[
        "--library".as_ref(),
        library.as_os_str(),
        "install".as_ref(),
        id.as_ref(),
        "--target".as_ref(),
        target.as_os_str(),
        "--apply".as_ref(),
    ]);
    assert!(!conflict.status.success());
    assert_eq!(fs::read(&target).unwrap(), after);
}
#[test]
fn rejects_bad_paths_malformed_snapshots_and_delimiter_injection() {
    let (dir, target) = setup();
    let revision = wowmc::revision(
        "test".into(),
        "/say yes\nEND\n/say no".into(),
        None,
        "account".into(),
    )
    .unwrap();
    let original = fs::read(&target).unwrap();
    assert!(wowmc::install::install_character(&target, &revision, true).is_err());
    assert_eq!(fs::read(&target).unwrap(), original);
    let invalid = dir.join("macros-cache.txt");
    fs::write(&invalid, &original).unwrap();
    assert!(wowmc::install::install_character(&invalid, &revision, true).is_err());
    fs::write(&target, b"VER 3 0100000000000001 \"x\" \"1\"\nno end\n").unwrap();
    let valid =
        wowmc::revision("test".into(), "/say yes".into(), None, "character".into()).unwrap();
    assert!(wowmc::install::install_character(&target, &valid, true).is_err());
}

#[cfg(windows)]
#[test]
fn refuses_to_write_when_target_is_already_open() {
    let (_dir, target) = setup();
    let before = fs::read(&target).unwrap();
    let open_file = fs::File::open(&target).unwrap();
    let entry =
        wowmc::revision("test".into(), "/say test".into(), None, "character".into()).unwrap();
    assert!(wowmc::install::install_character(&target, &entry, true).is_err());
    drop(open_file);
    assert_eq!(fs::read(&target).unwrap(), before);
}

#[test]
fn creates_missing_character_cache_and_installs_account_macro() {
    let (dir, target) = setup();
    let entry = wowmc::revision("New".into(), "/say new".into(), None, "character".into()).unwrap();
    let missing = target
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("Other/macros-cache.txt");
    fs::create_dir(missing.parent().unwrap()).unwrap();
    wowmc::install::install(&missing, &entry, false, false).unwrap();
    assert!(!missing.exists());
    let (_, backup) = wowmc::install::install(&missing, &entry, true, false).unwrap();
    assert!(backup.is_none());
    assert!(fs::read_to_string(&missing)
        .unwrap()
        .contains("VER 3 0100000000000001"));
    let account = dir.join("WTF/Account/123/macros-cache.txt");
    fs::write(
        &account,
        "VER 3 0000000000000001 \"General\" \"1\"\n/say old\nEND\n",
    )
    .unwrap();
    let (_, backup) = wowmc::install::install(&account, &entry, true, false).unwrap();
    assert!(backup.is_some());
    assert!(fs::read_to_string(&account)
        .unwrap()
        .contains("VER 3 0000000000000002"));
}
#[test]
fn replace_requires_explicit_choice_and_restore_backs_up_current() {
    let (_dir, target) = setup();
    let original = fs::read(&target).unwrap();
    let edit = wowmc::revision(
        "Existing".into(),
        "/say updated".into(),
        Some("123".into()),
        "character".into(),
    )
    .unwrap();
    assert!(wowmc::install::install(&target, &edit, true, false).is_err());
    assert_eq!(fs::read(&target).unwrap(), original);
    let (plan, backup) = wowmc::install::install(&target, &edit, false, true).unwrap();
    assert!(matches!(plan, wowmc::install::Plan::Replace { .. }));
    assert!(backup.is_none());
    assert_eq!(fs::read(&target).unwrap(), original);
    let (_, backup) = wowmc::install::install(&target, &edit, true, true).unwrap();
    let backup = backup.unwrap();
    let updated = fs::read(&target).unwrap();
    assert!(String::from_utf8_lossy(&updated)
        .contains("VER 3 0100000000000001 \"Existing\" \"123\"\r\n/say updated"));
    assert_eq!(fs::read(&backup).unwrap(), original);
    wowmc::install::restore_cache(&target, &backup, false).unwrap();
    assert_eq!(fs::read(&target).unwrap(), updated);
    let undo = wowmc::install::restore_cache(&target, &backup, true)
        .unwrap()
        .unwrap();
    assert_eq!(fs::read(&target).unwrap(), original);
    assert_eq!(fs::read(undo).unwrap(), updated);
}

#[test]
fn setup_remembers_a_character_and_install_uses_it() {
    let (dir, target) = setup();
    let library = dir.join("library.json");
    let body = dir.join("body.txt");
    fs::write(&body, "/say hello").unwrap();
    let cli = |args: &[&std::ffi::OsStr]| dir.command().args(args).output().unwrap();
    assert!(
        cli(&["setup".as_ref(), "--target".as_ref(), target.as_os_str()])
            .status
            .success()
    );
    assert!(cli(&[
        "--library".as_ref(),
        library.as_os_str(),
        "add".as_ref(),
        "Hello".as_ref(),
        "--file".as_ref(),
        body.as_os_str()
    ])
    .status
    .success());
    let result = cli(&[
        "--library".as_ref(),
        library.as_os_str(),
        "install".as_ref(),
        "Hello".as_ref(),
        "--apply".as_ref(),
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(fs::read_to_string(&target).unwrap().contains("\"Hello\""));
}

#[test]
fn compare_reports_matches_and_differences_without_editing_snapshot() {
    let (dir, target) = setup();
    let library = dir.join("library.json");
    let body = dir.join("body.txt");
    fs::write(&body, "/say old").unwrap();
    let add = dir.run(&[
        "--library".as_ref(),
        library.as_os_str(),
        "add".as_ref(),
        "Existing".as_ref(),
        "--file".as_ref(),
        body.as_os_str(),
        "--icon".as_ref(),
        "123".as_ref(),
    ]);
    assert!(add.status.success());
    let args = [
        "--library".as_ref(),
        library.as_os_str(),
        "compare".as_ref(),
        "Existing".as_ref(),
        "--target".as_ref(),
        target.as_os_str(),
    ];
    let before = fs::read(&target).unwrap();
    let result = dir.run(&args);
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("matches the library"));
    fs::write(
        &target,
        b"VER 3 0100000000000001 \"Existing\" \"123\"\n/say changed\nEND\n",
    )
    .unwrap();
    let changed = fs::read(&target).unwrap();
    let result = dir.run(&args);
    assert!(result.status.success());
    let output = String::from_utf8_lossy(&result.stdout);
    assert!(
        output.contains("differs")
            && output.contains("/say changed")
            && output.contains("/say old")
    );
    assert_eq!(fs::read(&target).unwrap(), changed);
    assert_ne!(before, changed);
}

#[test]
fn restore_accepts_valid_backups_independent_of_macro_names() {
    let (_dir, target) = setup();
    let original = fs::read(&target).unwrap();
    let backup = target.with_file_name("macros-cache.wowmc-regression.bak");
    let replacement = b"VER 3 0100000000000001 \"wowmc-check\" \"134400\"\r\n/say user\r\nEND\r\n";
    fs::write(&backup, replacement).unwrap();
    assert!(wowmc::install::restore_cache(&target, &backup, false)
        .unwrap()
        .is_none());
    assert_eq!(fs::read(&target).unwrap(), original);
    let undo = wowmc::install::restore_cache(&target, &backup, true)
        .unwrap()
        .unwrap();
    assert_eq!(fs::read(&target).unwrap(), replacement);
    assert_eq!(fs::read(undo).unwrap(), original);
}

#[test]
fn restore_validates_format_without_requiring_space_for_another_macro() {
    let (_dir, target) = setup();
    let before = fs::read(&target).unwrap();
    let backup = target.with_file_name("macros-cache.wowmc-validation.bak");
    for replacement in [
        "VER 3 0001 \"A\" \"1\"\n/say hi\nEND\n",
        "VER 3 0000000000000001 \"A\" \"1\"\n/say hi\nEND\n",
        "VER 3 0100000000000001 \"A\" \"1\"\n/say hi\n",
    ] {
        fs::write(&backup, replacement).unwrap();
        assert!(wowmc::install::restore_cache(&target, &backup, true).is_err());
        assert_eq!(fs::read(&target).unwrap(), before);
    }
    fs::write(
        &backup,
        "VER 3 01FFFFFFFFFFFFFF \"Last\" \"1\"\n/say hi\nEND\n",
    )
    .unwrap();
    assert!(wowmc::install::restore_cache(&target, &backup, false)
        .unwrap()
        .is_none());
    assert_eq!(fs::read(&target).unwrap(), before);
}
