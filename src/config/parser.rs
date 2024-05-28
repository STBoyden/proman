use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{BufReader, Read},
};

use super::get_language_plugin_dir;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) struct CommandStepConfig {
    name: String,
    command: Vec<String>,
}

impl CommandStepConfig {
    pub fn name(&self) -> &str { &self.name }
    pub fn command(&self) -> &[String] { &self.command }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) struct LanguageConfig {
    language: String,
    requirements: Vec<String>,
    command_steps: Vec<CommandStepConfig>,
}

impl LanguageConfig {
    pub fn language(&self) -> &str { &self.language }
    pub fn requirements(&self) -> &[String] { &self.requirements }
    pub fn command_steps(&self) -> &[CommandStepConfig] { &self.command_steps }
}

impl<'a> From<LanguageConfig> for ratatui::text::Text<'a> {
    fn from(value: LanguageConfig) -> Self { ratatui::text::Text::raw(value.language.clone()) }
}

pub(crate) fn parse_language_configs() -> anyhow::Result<BTreeSet<LanguageConfig>> {
    let plugin_dir = get_language_plugin_dir()?;
    let mut language_configurations = BTreeSet::new();

    for path in fs::read_dir(plugin_dir)? {
        if path.is_err() {
            continue;
        }
        let path = path.unwrap();

        if path.path().is_dir() {
            continue;
        }

        let file = File::open(path.path())?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();

        _ = reader.read_to_end(&mut buffer);

        let contents = String::from_utf8(buffer)?;
        if let Ok(config) = ron::from_str::<LanguageConfig>(&contents) {
            language_configurations.insert(config);
        } else {
            // ignore error cases, just continue on to the next step.
            // TODO: log to an error file.
            continue;
        }
    }

    Ok(language_configurations)
}
