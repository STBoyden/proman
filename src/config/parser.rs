use std::{
    collections::BTreeSet,
    fmt,
    fs::{self, File},
    io::{BufReader, Read},
};

use super::get_language_plugin_dir;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) enum CommandType {
    PromptProjectType,
    Command(String),
}

impl fmt::Display for CommandType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PromptProjectType => f.write_str("Prompting project type (binary, library)"),
            Self::Command(command) => f.write_fmt(format_args!("Running \"{command}\"...")),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) struct CommandStep {
    name: String,
    command: CommandType,
}

impl CommandStep {
    pub fn name(&self) -> &str { &self.name }
    pub fn command_string(&self) -> String { self.command.to_string() }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) struct LanguageConfig {
    language: String,
    requirements: Vec<String>,
    command_steps: Vec<CommandStep>,
}

impl LanguageConfig {
    pub fn language(&self) -> &str { &self.language }
    pub fn requirements(&self) -> &[String] { &self.requirements }
    pub fn command_steps(&self) -> &[CommandStep] { &self.command_steps }
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
