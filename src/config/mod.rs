#[cfg(not(debug_assertions))]
use std::io::Error;
use std::{fs, io, path::PathBuf};

#[cfg(not(debug_assertions))]
use directories::ProjectDirs;

pub(crate) use parser::*;

mod parser;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub(crate) enum Error {
    // std errors
    #[error("an std::io::Error occurred: {0}")]
    IO(#[from] io::Error),
    #[error("an error occurred when attempting to coerce a string from UTF-8 bytes: {0}")]
    StringFromUTF8(#[from] std::string::FromUtf8Error),

    // plugin directory errors
    #[error("could not create the directory for language plugins: {0}")]
    FailedPluginDirectoryCreation(String),
    #[error("could not find plugin directory for language plugins")]
    FailedFindingPluginDirectory,

    // configuration errors
    #[error("could not parse default plugins: {0}")]
    CouldNotReadDefaultPlugins(String),
    #[error("no configurations found on the filesystem")]
    NoConfigurations,

    // runner errors
    #[error("an occurred in the language configuration runner: {0}")]
    Runner(#[from] RunnerError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(debug_assertions)]
pub(crate) fn get_language_plugin_dir() -> Result<PathBuf> {
    let path = PathBuf::from("./plugins");

    if !path.exists() {
        fs::create_dir_all(&path)
            .map_err(|error| Error::FailedPluginDirectoryCreation(error.to_string()))?;
    }

    Ok(path)
}

#[cfg(not(debug_assertions))]
pub(crate) fn get_language_plugin_dir() -> Result<PathBuf> {
    let config_dirs = if let Some(project_dirs) = ProjectDirs::from("com", "stboyden", "proman") {
        project_dirs.config_dir().to_owned()
    } else {
        return Err(Error::FailedFindingPluginDirectory);
    };

    let path = config_dirs.join("plugins");

    if !path.exists() {
        fs::create_dir_all(&path)
            .map_err(|error| Error::FailedPluginDirectoryCreation(error.to_string()))?;
    }

    Ok(path)
}
