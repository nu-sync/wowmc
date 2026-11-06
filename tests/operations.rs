use wowmc::{operations::ImportDecision, revision, Library, SnapshotMacro};

fn entry(name: &str, body: &str) -> SnapshotMacro {
    SnapshotMacro {
        name: name.into(),
        body: body.into(),
        icon: "1".into(),
    }
}

#[test]
fn import_preview_failure_retry_and_duplicate_decisions_agree() {
    let fixture = tempfile::tempdir().unwrap();
    let path = fixture.path().join("library.json");
    let blocker = fixture.path().join("blocked");
    std::fs::write(&blocker, "not a directory").unwrap();
    let mut library = Library::load(&path).unwrap();
    let entries = vec![entry("A", "/say first"), entry("a", "/say other")];
    let plan = library
        .plan_import(&entries, "snapshot", "account")
        .unwrap();
    assert_eq!(plan.count(), 1);
    assert_eq!(plan.items[0].decision, ImportDecision::New);
    assert_eq!(plan.items[1].decision, ImportDecision::NameConflict);
    assert!(!path.exists());
    assert!(library
        .import(
            &blocker.join("library.json"),
            &entries,
            "snapshot",
            "account"
        )
        .is_err());
    assert!(library.macros.is_empty());
    assert_eq!(
        library
            .import(&path, &entries, "snapshot", "account")
            .unwrap(),
        1
    );
    let persisted = std::fs::read(&path).unwrap();
    let retry = library
        .plan_import(&entries, "snapshot", "account")
        .unwrap();
    assert_eq!(retry.items[0].decision, ImportDecision::AlreadyImported);
    assert_eq!(retry.items[1].decision, ImportDecision::NameConflict);
    assert_eq!(
        library
            .import(&path, &entries, "snapshot", "account")
            .unwrap(),
        0
    );
    assert_eq!(std::fs::read(&path).unwrap(), persisted);
    assert_eq!(library.macros[0].current.body, "/say first");
}

#[test]
fn shared_edit_trash_and_restore_preserve_history_and_failed_writes() {
    let fixture = tempfile::tempdir().unwrap();
    let path = fixture.path().join("library.json");
    let mut library = Library::load(&path).unwrap();
    let make = |body: &str| revision("A".into(), body.into(), None, "account".into()).unwrap();
    let id = library.save_macro(&path, None, make("/say one")).unwrap();
    library
        .save_macro(&path, Some(id), make("/say two"))
        .unwrap();
    library
        .save_macro(&path, Some(id), make("/say two"))
        .unwrap();
    assert_eq!(library.macros[0].history.len(), 1);
    let mut stale = library.clone();
    library.delete_macro(&path, "A").unwrap();
    assert!(stale.save_macro(&path, Some(id), make("/say two")).is_err());
    assert!(stale
        .save_macro(&path, Some(id), make("/say lost"))
        .is_err());
    assert_eq!(stale.macros[0].current.body, "/say two");
    assert_eq!(stale.macros[0].history.len(), 1);
    assert!(library.macros[0].deleted);
    assert!(library.restore_macro(&path, "A", Some(0)).is_err());
    assert!(library.macros[0].deleted);
    library.restore_macro(&path, "A", Some(1)).unwrap();
    let reloaded = Library::load(&path).unwrap();
    assert!(!reloaded.macros[0].deleted);
    assert_eq!(reloaded.macros[0].current.body, "/say one");
    assert_eq!(reloaded.macros[0].history[1].body, "/say two");
}

#[test]
fn id_prefixes_require_uniqueness_and_exact_names_take_priority() {
    let fixture = tempfile::tempdir().unwrap();
    let mut library = Library::load(&fixture.path().join("library.json")).unwrap();
    for (id, name) in [
        ("12345678-1111-4111-8111-111111111111", "Same"),
        ("12345678-2222-4222-8222-222222222222", "Same"),
        ("abcdefab-1111-4111-8111-111111111111", "12345678"),
    ] {
        library.add(
            revision(name.into(), "/say hi".into(), None, "account".into()).unwrap(),
            None,
        );
        library.macros.last_mut().unwrap().id = uuid::Uuid::parse_str(id).unwrap();
    }
    assert_eq!(library.resolve("12345678", false).unwrap(), 2);
    library.macros[2].deleted = true;
    assert!(library.resolve("12345678", false).is_err());
    assert!(library.resolve("Same", false).is_err());
    assert_eq!(library.resolve("12345678-1", false).unwrap(), 0);
    assert_eq!(library.resolve("ABCDEFAB", true).unwrap(), 2);
    assert!(library.resolve("1234567", false).is_err());
}

#[test]
fn imported_same_names_remain_conflicts_across_sources_and_scopes() {
    let fixture = tempfile::tempdir().unwrap();
    let path = fixture.path().join("library.json");
    let mut library = Library::load(&path).unwrap();
    let entries = vec![entry("A", "/say hi")];
    library.import(&path, &entries, "one", "account").unwrap();
    for (source, scope) in [("two", "account"), ("one", "character")] {
        let plan = library.plan_import(&entries, source, scope).unwrap();
        assert_eq!(plan.items[0].decision, ImportDecision::NameConflict);
    }
    library.delete_macro(&path, "A").unwrap();
    assert_eq!(
        library.import(&path, &entries, "two", "character").unwrap(),
        1
    );
}
