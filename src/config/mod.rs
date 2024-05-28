mod parser;
pub(crate) use parser::*;

#[cfg(not(debug_assertions))]
use std::io::Error;
use std::{fs, io, path::PathBuf};

#[cfg(not(debug_assertions))]
use directories::ProjectDirs;

#[cfg(debug_assertions)]
pub(crate) fn get_language_plugin_dir() -> io::Result<PathBuf> {
    let path = PathBuf::from("./plugins");

    if !path.exists() {
        fs::create_dir_all(&path)?;
    }

    Ok(path)
}

#[cfg(not(debug_assertions))]
pub(crate) fn get_language_plugin_dir() -> io::Result<PathBuf> {
    let config_dirs = if let Some(project_dirs) = ProjectDirs::from("com", "stboyden", "proman") {
        project_dirs.config_dir().to_owned()
    } else {
        return Err(Error::from(io::ErrorKind::NotFound));
    };

    let path = config_dirs.join("plugins");

    if !path.exists() {
        fs::create_dir_all(&path)?;
    }

    Ok(path)
}
