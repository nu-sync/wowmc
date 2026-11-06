use std::{fs, process::Output};
mod common;
use common::Fixture;

fn success(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn has_color(bytes: &[u8]) -> bool {
    bytes.windows(2).any(|pair| pair == b"\x1b[")
}

#[test]
fn redirected_human_output_is_plain_by_default() {
    let fixture = Fixture::new();
    for args in [vec![], vec!["--help"], vec!["list"]] {
        let output = success(fixture.command().args(&args).output().unwrap());
        assert!(!output.stdout.is_empty(), "{args:?}");
        assert!(!has_color(&output.stdout), "{args:?}");
        assert!(output.stderr.is_empty(), "{args:?}");
    }
    for args in [["show", "Missing"], ["--unknown", "argument"]] {
        let output = fixture.command().args(args).output().unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
        assert!(!has_color(&output.stderr));
    }
}

#[test]
fn forced_color_reaches_help_commands_previews_and_errors() {
    let fixture = Fixture::new();
    fs::write(fixture.join("body.txt"), "/say Ready").unwrap();
    fs::write(
        fixture.join("snapshot.txt"),
        "VER 3 0001 \"Imported\" \"1\"\n/say Hello\nEND\n",
    )
    .unwrap();
    for args in [
        vec![],
        vec!["--help"],
        vec!["add", "Ready", "--file", "body.txt"],
        vec!["list"],
        vec!["show", "Ready"],
        vec!["import", "snapshot.txt"],
    ] {
        let output = success(
            fixture
                .command()
                .env("CLICOLOR_FORCE", "1")
                .args(&args)
                .output()
                .unwrap(),
        );
        assert!(has_color(&output.stdout), "{args:?}");
        assert!(output.stderr.is_empty(), "{args:?}");
    }
    for args in [["show", "Missing"], ["--unknown", "argument"]] {
        let output = fixture
            .command()
            .env("CLICOLOR_FORCE", "1")
            .args(args)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(has_color(&output.stderr));
    }
}

#[test]
fn no_color_overrides_forced_color_for_stdout_and_stderr() {
    let fixture = Fixture::new();
    for args in [
        vec![],
        vec!["--help"],
        vec!["list"],
        vec!["show", "Missing"],
    ] {
        let output = fixture
            .command()
            .env("NO_COLOR", "1")
            .env("CLICOLOR_FORCE", "1")
            .args(&args)
            .output()
            .unwrap();
        assert_eq!(output.status.success(), args.first() != Some(&"show"));
        assert!(!has_color(&output.stdout), "{args:?}");
        assert!(!has_color(&output.stderr), "{args:?}");
    }
}

#[test]
fn forced_color_preserves_export_bytes_and_json_data() {
    let fixture = Fixture::new();
    let body = "/say Ready\n/cast Shoot";
    fs::write(fixture.join("body.txt"), body).unwrap();
    success(fixture.run(&["add", "Ready", "--file", "body.txt"]));

    let exported = success(
        fixture
            .command()
            .env("CLICOLOR_FORCE", "1")
            .args(["export", "Ready"])
            .output()
            .unwrap(),
    );
    assert_eq!(exported.stdout, body.as_bytes());
    assert!(exported.stderr.is_empty());

    let listed = success(
        fixture
            .command()
            .env("CLICOLOR_FORCE", "1")
            .args(["list", "--json"])
            .output()
            .unwrap(),
    );
    assert!(!has_color(&listed.stdout));
    assert!(listed.stderr.is_empty());
    let records: serde_json::Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(records.as_array().unwrap().len(), 1);
    assert_eq!(records[0]["current"]["name"], "Ready");
    assert_eq!(records[0]["current"]["body"], body);
}
