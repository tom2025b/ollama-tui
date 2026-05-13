//! Abstractions over process state used while assembling runtime config and
//! paths.

use std::{ffi::OsString, path::PathBuf};

use crate::{Error, Result};

/// Process-level inputs needed to resolve runtime paths and config.
pub(crate) trait RuntimeEnvironment {
    /// Read a UTF-8 environment variable, returning `None` when unset or not
    /// valid UTF-8.
    fn var(&self, key: &str) -> Option<String>;

    /// Read an environment variable without forcing UTF-8 decoding.
    fn var_os(&self, key: &str) -> Option<OsString>;

    /// Resolve the current working directory.
    fn current_dir(&self) -> Result<PathBuf>;
}

/// Production environment backed by `std::env`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ProcessEnvironment;

impl RuntimeEnvironment for ProcessEnvironment {
    fn var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    fn var_os(&self, key: &str) -> Option<OsString> {
        std::env::var_os(key)
    }

    fn current_dir(&self) -> Result<PathBuf> {
        std::env::current_dir()
            .map_err(|source| Error::io_operation("resolve current working directory", source))
    }
}
