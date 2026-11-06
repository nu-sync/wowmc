use std::{
    fs,
    io::Write,
    process::{Output, Stdio},
};
mod common;
use common::Fixture;

fn success(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn empty_library_guides_people_and_keeps_json_machine_readable() {
    let fixture = Fixture::new();
    let welcome = success(fixture.run(&[] as &[&str]));
    assert!(welcome.contains("wowmc setup") && welcome.contains("wowmc import"));
    assert!(success(fixture.run(&["list"])).contains("Your library is empty"));
    let json = success(fixture.run(&["list", "--json"]));
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&json).unwrap(),
        serde_json::json!([])
    );
    assert!(!fixture.join("library.json").exists());
    assert!(!fixture.join("config.json").exists());
    assert!(!fixture.run(&["import"]).status.success());
    let add = fixture.run(&["add", "Example"]);
    assert!(!add.status.success());
    assert!(String::from_utf8_lossy(&add.stderr).contains("--file"));
}

#[test]
fn selected_target_import_install_compare_and_undo_walkthrough() {
    let fixture = Fixture::new();
    let target = fixture.join("WTF/Account/123/Realm/Character/macros-cache.txt");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    let original = b"VER 3 0100000000000001 \"Existing\" \"1\"\r\n/say original\r\nEND\r\n";
    fs::write(&target, original).unwrap();
    let setup = success(fixture.run(&["setup", "--target", target.to_str().unwrap()]));
    assert!(setup.contains("character target") && setup.contains("Next: wowmc import"));
    assert!(success(fixture.run(&["import"])).contains("[character] (new)"));
    assert!(!fixture.join("library.json").exists());
    success(fixture.run(&["import", "--apply"]));
    assert!(success(fixture.run(&["import"])).contains("already imported"));
    let library = wowmc::Library::load(&fixture.join("library.json")).unwrap();
    let prefix = &library.macros[0].id.to_string()[..8];
    assert!(success(fixture.run(&["show", prefix])).contains("/say original"));
    let mut child = fixture
        .command()
        .args(["add", "New", "--file", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"/say new\n")
        .unwrap();
    success(child.wait_with_output().unwrap());
    success(fixture.run(&["install", "New"]));
    assert_eq!(fs::read(&target).unwrap(), original);
    let applied = success(fixture.run(&["install", "New", "--apply"]));
    assert!(applied.contains("To preview undo: wowmc restore-cache"));
    assert!(success(fixture.run(&["compare", "New"])).contains("matches the library"));
    let backup = fs::read_dir(target.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|e| e == "bak"))
        .unwrap();
    let installed = fs::read(&target).unwrap();
    success(fixture.run(&["restore-cache", "--backup", backup.to_str().unwrap()]));
    assert_eq!(fs::read(&target).unwrap(), installed);
    success(fixture.run(&[
        "restore-cache",
        "--backup",
        backup.to_str().unwrap(),
        "--apply",
    ]));
    assert_eq!(fs::read(&target).unwrap(), original);
    success(fixture.run(&["delete", "New"]));
    success(fixture.run(&["restore", "New"]));
    assert_eq!(success(fixture.run(&["export", "New"])), "/say new");
}

#[test]
fn setup_distinguishes_account_scope_and_missing_cache() {
    let fixture = Fixture::new();
    let target = fixture.join("WTF/Account/123/macros-cache.txt");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    let output = success(fixture.run(&["setup", "--target", target.to_str().unwrap()]));
    assert!(output.contains("Default account target") && output.contains("No saved cache exists"));
    fs::write(
        &target,
        "VER 3 0000000000000001 \"Account\" \"1\"\n/say hi\nEND\n",
    )
    .unwrap();
    assert!(success(fixture.run(&["import"])).contains("[account]"));
}

#[test]
fn setup_and_cache_restore_do_not_require_a_library_location() {
    let fixture = Fixture::new();
    let target = fixture.join("WTF/Account/123/macros-cache.txt");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, "").unwrap();
    success(
        fixture
            .command()
            .env_remove("APPDATA")
            .env_remove("WOWMC_LIBRARY")
            .args(["setup", "--target", target.to_str().unwrap()])
            .output()
            .unwrap(),
    );
    let backup = target.with_file_name("macros-cache.wowmc-test.bak");
    fs::write(&backup, "").unwrap();
    success(
        fixture
            .command()
            .env_remove("APPDATA")
            .env_remove("WOWMC_LIBRARY")
            .env_remove("WOWMC_CONFIG")
            .args([
                "restore-cache",
                "--target",
                target.to_str().unwrap(),
                "--backup",
                backup.to_str().unwrap(),
            ])
            .output()
            .unwrap(),
    );
    assert!(!fixture.join("library.json").exists());
}

#[cfg(windows)]
#[test]
fn editor_add_and_edit_share_history_and_recovery_operations() {
    let fixture = Fixture::new();
    let editor = fixture.join("editor.cmd");
    fs::write(&editor, "@echo off\r\n> \"%~1\" echo /say first\r\n").unwrap();
    success(
        fixture
            .command()
            .env("EDITOR", &editor)
            .args(["add", "Edited"])
            .output()
            .unwrap(),
    );
    fs::write(&editor, "@echo off\r\n> \"%~1\" echo /say second\r\n").unwrap();
    success(
        fixture
            .command()
            .env("EDITOR", &editor)
            .args(["edit", "Edited"])
            .output()
            .unwrap(),
    );
    assert!(success(fixture.run(&["history", "Edited"])).contains("/say first"));
    success(fixture.run(&["restore", "Edited", "--revision", "1"]));
    assert_eq!(success(fixture.run(&["export", "Edited"])), "/say first");
}
