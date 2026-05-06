mod config;
mod environment;
mod paths;

#[cfg(test)]
pub(crate) use config::DEFAULT_FAST_OLLAMA_MODEL;
pub(crate) use config::{CloudProviderRuntimeConfig, ModelRuntimeConfig, RuntimeConfig};
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
