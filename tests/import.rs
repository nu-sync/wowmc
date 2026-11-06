use std::fs;
mod common;
use common::Fixture;

#[test]
fn import_preview_and_apply_skip_duplicate_names_within_snapshot() {
    let dir = Fixture::new();
    let snapshot = dir.join("snapshot.txt");
    let library = dir.join("library.json");
    fs::write(
        &snapshot,
        "VER 3 0001 \"Repeat\" \"1\"\n/say one\nEND\nVER 3 0002 \"Repeat\" \"1\"\n/say two\nEND\n",
    )
    .unwrap();
    let invoke = |apply: bool| {
        let mut cmd = dir.command();
        cmd.arg("--library")
            .arg(&library)
            .arg("import")
            .arg(&snapshot);
        if apply {
            cmd.arg("--apply");
        }
        let output = cmd.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    };
    assert!(dir.run(&["list", "--json"]).status.success());
    assert!(invoke(false).contains("name already exists; skipped"));
    assert!(!library.exists());
    assert!(invoke(true).contains("Imported 1 macros"));
    assert!(invoke(true).contains("Imported 0 macros"));
    let lib = wowmc::Library::load(&library).unwrap();
    assert_eq!(lib.macros.len(), 1);
    assert_eq!(lib.macros[0].current.body, "/say one");
}
