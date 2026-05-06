mod config;
mod environment;
mod paths;

#[cfg(test)]
pub(crate) use config::{
    DEFAULT_CLAUDE_CODE_MODEL, DEFAULT_CODEX_MODEL, DEFAULT_FAST_OLLAMA_MODEL,
};
pub(crate) use config::{ModelRuntimeConfig, RuntimeConfig};
pub(crate) use paths::RuntimePaths;

use environment::{ProcessEnvironment, RuntimeEnvironment};

#[derive(Clone, Debug)]
pub(crate) struct Runtime {
    config: RuntimeConfig,
    paths: RuntimePaths,
}

impl Runtime {
    pub(crate) fn load() -> Self {
        Self::from_environment(&ProcessEnvironment)
    }

    #[cfg(test)]
    pub(crate) fn for_tests() -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

        let base = std::env::temp_dir().join(format!(
            "ai-suite-runtime-test-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));

        Self {
            config: RuntimeConfig::from_environment(&ProcessEnvironment),
            paths: RuntimePaths::from_parts(std::env::temp_dir(), base.clone(), Some(base)),
        }
    }

    pub(crate) fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    pub(crate) fn paths(&self) -> &RuntimePaths {
        &self.paths
    }

    fn from_environment<E>(environment: &E) -> Self
    where
        E: RuntimeEnvironment + ?Sized,
    {
        Self {
            config: RuntimeConfig::from_environment(environment),
            paths: RuntimePaths::from_environment(environment),
        }
    }
}
