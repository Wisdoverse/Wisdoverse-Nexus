use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Plugin initialization failed: {0}")]
    InitFailed(String),

    #[error("Plugin execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Command not recognized: {0}")]
    UnknownCommand(String),

    #[error("Permission denied for plugin: {0}")]
    PermissionDenied(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("{0}")]
    Other(String),
}
