use wowmc::{
    install::{plan_install, Plan, Scope},
    parse_snapshot, revision,
};

#[test]
fn permissive_reads_and_strict_writes_share_records_without_losing_bytes() {
    let short_id = "VER 3 0001 \"A\" \"1\"\n/say hi\nEND\n";
    let macro_ = revision(
        "A".into(),
        "/say changed".into(),
        Some("1".into()),
        "character".into(),
    )
    .unwrap();
    assert_eq!(parse_snapshot(short_id).unwrap()[0].body, "/say hi");
    assert!(plan_install(short_id.as_bytes(), &macro_, Scope::Character, true).is_err());
    for newline in ["\n", "\r\n"] {
        let prefix = format!("  {newline}VER 3 0100000000000001 \"Other\" \"2\"{newline}/say café{newline}END{newline}{newline}");
        let edited =
            format!("VER 3 0100000000000002 \"A\" \"1\"{newline}/say old{newline}END{newline}");
        let suffix =
            format!("{newline}VER 3 0100000000000003 \"Last\" \"3\"{newline}/say last{newline}END");
        let original = format!("{prefix}{edited}{suffix}");
        let Plan::Replace { id, bytes, .. } =
            plan_install(original.as_bytes(), &macro_, Scope::Character, true).unwrap()
        else {
            panic!("expected replacement")
        };
        assert_eq!(id, 0x0100000000000002);
        assert!(bytes.starts_with(prefix.as_bytes()));
        assert!(bytes.ends_with(suffix.as_bytes()));
        let updated = String::from_utf8(bytes).unwrap();
        let entries = parse_snapshot(&updated).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[1].body, "/say changed");
        assert!(matches!(
            plan_install(updated.as_bytes(), &macro_, Scope::Character, false).unwrap(),
            Plan::AlreadyPresent
        ));
    }
}

#[test]
fn strict_validation_rejects_duplicate_wrong_scope_and_malformed_records() {
    let macro_ = revision("New".into(), "/say hi".into(), None, "character".into()).unwrap();
    let good = "VER 3 0100000000000001 \"A\" \"1\"\n/say hi\nEND\n";
    for text in [
        good.repeat(2),
        good.replace("0100000000000001", "0000000000000001"),
        good.replace("END\n", ""),
        good.replace('\n', "\r"),
    ] {
        assert!(plan_install(text.as_bytes(), &macro_, Scope::Character, false).is_err());
    }
}

#[test]
fn unrecognized_header_fields_are_readable_but_never_rewritten() {
    let macro_ = revision("New".into(), "/say hi".into(), None, "character".into()).unwrap();
    for header in [
        "VER 3 0100000000000001\"A\" \"1\"",
        "VER 3 0100000000000001 unknown \"A\" \"1\"",
    ] {
        let snapshot = format!("{header}\n/say old\nEND\n");
        assert_eq!(parse_snapshot(&snapshot).unwrap()[0].name, "A");
        assert!(
            plan_install(snapshot.as_bytes(), &macro_, Scope::Character, false).is_err(),
            "accepted unsupported header: {header}"
        );
    }
}
