use std::fs;
use wowmc::{revision, Library};

fn entry(body: &str) -> wowmc::Revision {
    revision("A".into(), body.into(), None, "account".into()).unwrap()
}

#[test]
fn library_named_lock_does_not_collide_with_its_lock_file() {
    let fixture = tempfile::tempdir().unwrap();
    let path = fixture.path().join("library.lock");
    let mut library = Library::load(&path).unwrap();
    library
        .save_macro(&path, None, entry("/say first"))
        .unwrap();
    assert_eq!(
        Library::load(&path).unwrap().macros[0].current.body,
        "/say first"
    );
}

#[test]
fn backup_named_library_has_a_distinct_recoverable_backup() {
    let fixture = tempfile::tempdir().unwrap();
    let path = fixture.path().join("library.json.bak");
    let mut library = Library::load(&path).unwrap();
    let id = library
        .save_macro(&path, None, entry("/say first"))
        .unwrap();
    let before = fs::read(&path).unwrap();
    library
        .save_macro(&path, Some(id), entry("/say second"))
        .unwrap();
    assert_eq!(
        fs::read(fixture.path().join("library.json.bak.bak")).unwrap(),
        before
    );
    assert_eq!(
        Library::load(&path).unwrap().macros[0].current.body,
        "/say second"
    );
}

#[test]
fn unchanged_save_keeps_the_last_distinct_backup_and_still_checks_conflicts() {
    let fixture = tempfile::tempdir().unwrap();
    let path = fixture.path().join("library.json");
    let backup = fixture.path().join("library.json.bak");
    let mut library = Library::load(&path).unwrap();
    let id = library
        .save_macro(&path, None, entry("/say first"))
        .unwrap();
    let first = fs::read(&path).unwrap();
    library
        .save_macro(&path, Some(id), entry("/say second"))
        .unwrap();
    let second = fs::read(&path).unwrap();
    library
        .save_macro(&path, Some(id), entry("/say second"))
        .unwrap();
    assert_eq!(fs::read(&backup).unwrap(), first);
    assert_eq!(fs::read(&path).unwrap(), second);
    let mut other = Library::load(&path).unwrap();
    other
        .save_macro(&path, Some(id), entry("/say external"))
        .unwrap();
    assert!(library
        .save_macro(&path, Some(id), entry("/say second"))
        .is_err());
    assert_eq!(fs::read(&backup).unwrap(), second);
}

#[test]
fn duplicate_record_ids_are_rejected_before_ambiguous_edits_are_possible() {
    let fixture = tempfile::tempdir().unwrap();
    let path = fixture.path().join("library.json");
    let mut library = Library::load(&path).unwrap();
    library
        .save_macro(&path, None, entry("/say first"))
        .unwrap();
    let original = fs::read(&path).unwrap();
    let mut duplicate = library.macros[0].clone();
    duplicate.current.name = "Different macro".into();
    library.macros.push(duplicate);
    let malformed = fixture.path().join("duplicate.json");
    fs::write(&malformed, serde_json::to_vec(&library).unwrap()).unwrap();
    assert!(
        Library::load(&malformed).is_err(),
        "duplicate IDs must not resolve to an arbitrary record"
    );
    assert!(library.save(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
}

#[test]
fn unchanged_save_preserves_recovery_even_when_json_formatting_differs() {
    let fixture = tempfile::tempdir().unwrap();
    let path = fixture.path().join("library.json");
    let backup = fixture.path().join("library.json.bak");
    let mut library = Library::load(&path).unwrap();
    let id = library
        .save_macro(&path, None, entry("/say first"))
        .unwrap();
    let first = fs::read(&path).unwrap();
    library
        .save_macro(&path, Some(id), entry("/say second"))
        .unwrap();
    let compact = serde_json::to_vec(&library).unwrap();
    fs::write(&path, &compact).unwrap();
    let mut reloaded = Library::load(&path).unwrap();
    reloaded
        .save_macro(&path, Some(id), entry("/say second"))
        .unwrap();
    assert!(
        fs::read(&backup).unwrap() == first,
        "an unchanged save replaced the recovery backup"
    );
    assert!(
        fs::read(&path).unwrap() == compact,
        "an unchanged save rewrote JSON formatting"
    );
}
