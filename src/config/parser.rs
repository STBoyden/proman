use std::{
    collections::BTreeSet,
    fmt,
    fs::{self, File},
    io::{BufReader, Read},
    sync::{Arc, mpsc, Mutex, RwLock},
};

use bus::{Bus, BusReader};

use super::{Error, get_language_plugin_dir, Result};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) enum CommandType {
    PromptProjectType,
    PromptProjectName,
    #[serde(rename = "ShellCommand")]
    Command(String, String),
}

impl fmt::Display for CommandType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PromptProjectType => f.write_str("Prompting project type (binary, library)"),
            Self::PromptProjectName => f.write_fmt(format_args!("Prompting project name")),
            Self::Command(command, arguments) =>
                f.write_fmt(format_args!("Running \"{command} {arguments}\"...")),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, PartialOrd, Eq, Ord)]
#[serde(rename = "Step")]
pub(crate) struct CommandStep {
    name: String,
    command: CommandType,
}

impl CommandStep {
    pub fn name(&self) -> &str { &self.name }
    pub fn command_string(&self) -> String { self.command.to_string() }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub enum ProjectType {
    Binary,
    Library,
    Workspace,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct LanguageConfig {
    language: String,
    requirements: Vec<String>,
    project_types: Vec<ProjectType>,
    command_steps: Vec<CommandStep>,
}

impl LanguageConfig {
    pub fn language(&self) -> &str { &self.language }
    pub fn requirements(&self) -> &[String] { &self.requirements }
    pub fn command_steps(&self) -> &[CommandStep] { &self.command_steps }

    pub fn create_runner(&self) -> LanguageConfigRunner {
        LanguageConfigRunner::new(self.command_steps.clone())
    }
}

impl<'a> From<LanguageConfig> for ratatui::text::Text<'a> {
    fn from(value: LanguageConfig) -> Self { ratatui::text::Text::raw(value.language.clone()) }
}

/// Parses the default language configurations from
/// [`crate::consts::DEFAULT_PLUGINS_BYTES`] which is configured and set at compile-time
/// in the build script. Returns a [`Result<BTreeSet<LanguageConfig>>`] but should not
/// error.
fn parse_default_language_configs() -> Result<BTreeSet<LanguageConfig>> {
    let mut language_configurations = BTreeSet::new();

    for bytes in crate::consts::DEFAULT_PLUGINS_BYTES {
        let contents = String::from_utf8(Vec::from(bytes))?;

        match ron::from_str::<LanguageConfig>(&contents) {
            Ok(config) => language_configurations.insert(config),
            Err(error) => return Err(Error::CouldNotReadDefaultPlugins(error.to_string())),
        };
    }

    Ok(language_configurations)
}

/// Parse the plugins in the plugins directory, specified at runtime and return the
/// available language configurations that could be parsed.
pub(crate) fn parse_language_configs() -> Result<BTreeSet<LanguageConfig>> {
    let plugin_dir = get_language_plugin_dir()?;
    let mut language_configurations = parse_default_language_configs()?;

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

    if language_configurations.is_empty() {
        Err(Error::NoConfigurations)
    } else {
        Ok(language_configurations)
    }
}

#[derive(Clone, Debug)]
pub(crate) enum RunningConfigMessage {
    SetCommandStepText(String),
    StartInputPrompt,
    PromptForProjectName(mpsc::Sender<String>),
    CommandOutput(String),
    NoOp,
}

#[derive(Copy, Clone, Debug, thiserror::Error)]
pub enum RunnerError {
    #[error("the runner has already been started, but there seems to be no bus to send from?")]
    AlreadyStartedButNoBus,
}

type CommandBusType = Option<Arc<Mutex<Bus<(RunningConfigMessage, bool)>>>>;

pub(crate) struct LanguageConfigRunner {
    commands: Vec<CommandStep>,
    project_name: Arc<RwLock<String>>,
    project_type: Arc<RwLock<ProjectType>>,
    has_started: bool,
    command_bus: CommandBusType,
}

impl LanguageConfigRunner {
    fn new(commands: Vec<CommandStep>) -> LanguageConfigRunner {
        LanguageConfigRunner {
            commands,
            project_name: Arc::new(RwLock::new(String::new())),
            project_type: Arc::new(RwLock::new(ProjectType::Binary)),
            has_started: false,
            command_bus: None,
        }
    }

    /// Start or continue the current runner. If the current runner is already running and
    /// has a set [`Self::command_receiver`] for the recipient, then the function returns
    /// early, a cloned version of the reference-counted [`Self::command_receiver`].
    pub fn start_or_continue(
        &mut self,
    ) -> std::result::Result<BusReader<(RunningConfigMessage, bool)>, RunnerError> {
        if let Some(ref mut bus) = self.command_bus
            && self.has_started
        {
            return Ok(bus.lock().unwrap().add_rx());
        } else if self.has_started {
            return Err(RunnerError::AlreadyStartedButNoBus);
        }

        self.has_started = true;

        let command_tx = Arc::new(Mutex::new(Bus::new(4096)));
        let command_rx = command_tx.lock().unwrap().add_rx();
        self.command_bus = Some(command_tx.clone());

        let commands = self.commands.clone();
        let name_lock = self.project_name.clone();
        std::thread::spawn(move || {
            commands.iter().for_each(|step| {
                command_tx.lock().unwrap().broadcast((
                    RunningConfigMessage::SetCommandStepText(step.name.clone()),
                    false,
                ));

                match &step.command {
                    CommandType::PromptProjectName => {
                        command_tx
                            .lock()
                            .unwrap()
                            .broadcast((RunningConfigMessage::StartInputPrompt, false));

                        let (name_tx, name_rx) = mpsc::channel();
                        command_tx.lock().unwrap().broadcast((
                            RunningConfigMessage::PromptForProjectName(name_tx),
                            false,
                        ));

                        if let Ok(name) = name_rx.recv() {
                            *name_lock.write().unwrap() = name;
                        }
                    },
                    CommandType::Command(command, arguments) => (),
                    _ => {},
                }
            });

            command_tx
                .lock()
                .unwrap()
                .broadcast((RunningConfigMessage::NoOp, true));
        });

        Ok(command_rx)
    }
}
