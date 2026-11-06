use std::{
    ffi::OsStr,
    ops::Deref,
    path::Path,
    process::{Command, Output},
};

pub struct Fixture(tempfile::TempDir);

impl Fixture {
    pub fn new() -> Self {
        Self(tempfile::tempdir().unwrap())
    }

    pub fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_wowmc"));
        command
            .current_dir(self.0.path())
            .env("TEMP", self.0.path())
            .env("TMP", self.0.path())
            .env("APPDATA", self.join("profile"))
            .env("WOWMC_LIBRARY", self.join("library.json"))
            .env("WOWMC_CONFIG", self.join("config.json"))
            .env_remove("NO_COLOR")
            .env_remove("CLICOLOR")
            .env_remove("CLICOLOR_FORCE")
            .env_remove("VISUAL")
            .env_remove("EDITOR");
        command
    }

    pub fn run(&self, args: &[impl AsRef<OsStr>]) -> Output {
        self.command().args(args).output().unwrap()
    }
}

impl Deref for Fixture {
    type Target = Path;
    fn deref(&self) -> &Path {
        self.0.path()
    }
}
