use std::{ffi::OsString, path::PathBuf};

pub(crate) trait RuntimeEnvironment {
    fn var(&self, key: &str) -> Option<String>;
    fn var_os(&self, key: &str) -> Option<OsString>;
    fn current_dir(&self) -> Option<PathBuf>;
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ProcessEnvironment;

impl RuntimeEnvironment for ProcessEnvironment {
    fn var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    fn var_os(&self, key: &str) -> Option<OsString> {
        std::env::var_os(key)
    }

    fn current_dir(&self) -> Option<PathBuf> {
        std::env::current_dir().ok()
    }
}
