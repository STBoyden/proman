#![allow(clippy::pedantic, clippy::nursery)]
#![feature(let_chains)]

use std::{collections::BTreeSet, io::stdout, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use crate::{
    config::{
        parse_language_configs, LanguageConfig, LanguageConfigRunner, ProjectType,
        RunningConfigMessage,
    },
    widgets::{StatefulList, StatefulListItem},
};

mod config;
mod consts;
mod widgets;

// The cleanup process for exiting the application.
fn cleanup() -> config::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

enum Message {
    ShouldQuit,
    RunConfiguration(usize),
    NoOp,
}

#[derive(Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
enum InputMode {
    #[default]
    None,
    Text,
    Choice,
}

#[derive(Clone, Debug, Default)]
struct RunningState {
    step_name:              String,
    scroll_back:            Vec<String>,
    input_mode:             InputMode,
    input:                  String,
    project_type_list:      Option<StatefulList<ProjectType>>,
    selected_project_type:  Option<ProjectType>,
    running_config_message: RunningConfigMessage,
}

enum AppState<ListItem>
where
    for<'a> ListItem: StatefulListItem<'a>,
{
    Main(StatefulList<ListItem>),
    Starting(LanguageConfig),
    Running(LanguageConfigRunner, Option<RunningState>),
    Stopping,
}

fn main() -> config::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let language_configs = match parse_language_configs() {
        Ok(c) => c,
        Err(_error) => {
            cleanup().unwrap();
            panic!("could not parse language configs: {_error}")
        },
    };
    let language_list = StatefulList::new(language_configs.clone());
    let mut state = AppState::Main(language_list);

    let language_configs = language_configs.iter().collect::<Vec<_>>();

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|frame: &mut Frame| ui(frame, &mut state))?;

        match handle_events(&mut state)? {
            Message::ShouldQuit => should_quit = true,
            Message::RunConfiguration(index) => match language_configs.get(index) {
                Some(config) => state = AppState::Starting(<LanguageConfig>::clone(config)),
                None => panic!("somehow got an out of bounds index for running a configuration"),
            },
            _ => (),
        }
    }

    cleanup()
}

/// Higher-order scaffolding function for handling key events in [`handle_events`].
fn key_handler<T, F>(param: T, f: Box<F>) -> config::Result<Message>
where
    F: FnOnce(T, KeyCode) -> config::Result<Message>,
{
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                return f(param, key.code);
            }
        }
    }

    Ok(Message::NoOp)
}

/// Handle [`InputMode::Text`] input mode events for use in [`handle_events`].
fn handle_text_input_mode_events(
    key_code: KeyCode,
    state: &mut RunningState,
) -> config::Result<Message> {
    match key_code {
        KeyCode::Char(character) => {
            state.input.push(character);
            Ok(Message::NoOp)
        },
        KeyCode::Esc => Ok(Message::ShouldQuit),
        _ => Ok(Message::NoOp),
    }
}

/// Handle [`InputMode::Choice`] input mode events for use in [`handle_events`].
fn handle_choice_input_mode_events(
    key_code: KeyCode,
    state: &mut RunningState,
) -> config::Result<Message> {
    let RunningConfigMessage::PromptForProjectType { ref channel, .. } =
        state.running_config_message
    else {
        unreachable!("already checked");
    };

    if let Some(ref selected_project_type) = state.selected_project_type {
        channel.send(selected_project_type.clone()).unwrap();

        return Ok(Message::NoOp);
    }

    let mut list = state.project_type_list.clone().unwrap();

    match key_code {
        KeyCode::Char('k') | KeyCode::Up => {
            list.previous_item();
            Ok(Message::NoOp)
        },
        KeyCode::Char('j') | KeyCode::Down => {
            list.next_item();
            Ok(Message::NoOp)
        },
        KeyCode::Enter => {
            let selected_index = list.get_selected_index();
            if let Some(selected_type) = list.get_items().get(selected_index) {
                channel.send(selected_type.clone()).unwrap();

                return Ok(Message::NoOp);
            }

            Ok(Message::NoOp)
        },
        _ => Ok(Message::NoOp),
    }
}

/// Handle events that happen during the runtime of the application, can include key
/// events, or other custom-made events that the application should be able to respond to.
fn handle_events<ListItem>(app_state: &mut AppState<ListItem>) -> config::Result<Message>
where
    for<'a> ListItem: StatefulListItem<'a>,
{
    match app_state {
        AppState::Main(ref mut language_list) => key_handler(
            language_list,
            Box::new(
                |list: &mut StatefulList<ListItem>, key_code| match key_code {
                    KeyCode::Char('q') => Ok(Message::ShouldQuit),
                    KeyCode::Char('k') | KeyCode::Up => {
                        list.previous_item();
                        Ok(Message::NoOp)
                    },
                    KeyCode::Char('j') | KeyCode::Down => {
                        list.next_item();
                        Ok(Message::NoOp)
                    },
                    KeyCode::Enter => Ok(Message::RunConfiguration(list.get_selected_index())),
                    _ => Ok(Message::NoOp),
                },
            ),
        ),
        AppState::Running(_, running_state) => {
            let mut state = if let Some(state) = running_state {
                state.clone()
            } else {
                RunningState {
                    project_type_list: Some(StatefulList::new(BTreeSet::<ProjectType>::new())),
                    ..Default::default()
                }
            };

            key_handler(
                running_state,
                Box::new(|running_state: &mut Option<RunningState>, key_code| {
                    let message = match state.input_mode {
                        InputMode::Text => handle_text_input_mode_events(key_code, &mut state),
                        InputMode::Choice
                            if matches!(
                                state,
                                RunningState {
                                    running_config_message:
                                        RunningConfigMessage::PromptForProjectType { .. },
                                    project_type_list: Some(..),
                                    ..
                                }
                            ) =>
                            handle_choice_input_mode_events(key_code, &mut state),
                        _ => match key_code {
                            KeyCode::Char('q') => Ok(Message::ShouldQuit),
                            _ => Ok(Message::NoOp),
                        },
                    };

                    *running_state = Some(state);

                    message
                }),
            )
        },
        _ => Ok(Message::NoOp),
    }
}

fn ui_running<ListItem>(
    frame: &mut Frame,
    runner: &mut LanguageConfigRunner,
    running_state: &mut Option<RunningState>,
) -> Option<AppState<ListItem>>
where
    for<'a> ListItem: StatefulListItem<'a>,
{
    let mut state = if let Some(state) = running_state {
        state.clone()
    } else {
        RunningState {
            project_type_list: Some(StatefulList::new(BTreeSet::<ProjectType>::new())),
            ..Default::default()
        }
    };

    let layout_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(98), Constraint::Fill(2)])
        .margin(2)
        .split(frame.size());

    let mut res = runner.start_or_continue();
    if let Ok(ref mut rx) = res {
        if let Ok((message, should_stop)) = rx.recv() {
            // TODO: handle user input for the project name and project type
            match message {
                RunningConfigMessage::SetCommandStepText(text) => state.step_name = text,
                RunningConfigMessage::CommandOutput(output) => {
                    state.scroll_back.push(output);
                },
                RunningConfigMessage::StartInputPrompt => {
                    state.input_mode = InputMode::Text;
                },
                RunningConfigMessage::StartChoicePrompt => {
                    state.input_mode = InputMode::Choice;
                },
                RunningConfigMessage::PromptForProjectName(name_tx) => {},
                RunningConfigMessage::PromptForProjectType {
                    available_types, ..
                } if available_types.len() > 1 => {
                    let mut list = state
                        .project_type_list
                        .expect("should be populated by this point");

                    list.set_items(available_types.clone());
                    list.draw(frame, layout_chunks[1], "Project types");
                    state.project_type_list = Some(list);
                },
                RunningConfigMessage::PromptForProjectType {
                    available_types, ..
                } => {
                    state.selected_project_type = available_types.first().cloned();
                },
                RunningConfigMessage::NoOp => (),
            }

            let paragraph_text = state.scroll_back.join("\n");
            let scrollback_output = Paragraph::new(paragraph_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Step: {}", state.step_name))
                    .title_alignment(Alignment::Center),
            );

            frame.render_widget(scrollback_output, layout_chunks[0]);

            *running_state = Some(state);

            if should_stop {
                return Some(AppState::Stopping);
            }
        }
    } else if let Err(_error) = res {
        cleanup().unwrap();
        panic!("could not receive from command: {_error}")
    }

    None
}

/// Draw the ui of the application.
fn ui<ListItem>(frame: &mut Frame, app_state: &mut AppState<ListItem>)
where
    for<'a> ListItem: StatefulListItem<'a>,
{
    match app_state {
        AppState::Main(ref mut list) => list.draw(
            frame,
            frame.size(),
            String::from("Please choose a language"),
        ),
        AppState::Starting(ref config) => {
            let runner = config.create_runner();

            *app_state = AppState::Running(runner, None);
        },
        AppState::Running(ref mut runner, ref mut running_state) => {
            if let Some(new_state) = ui_running(frame, runner, running_state) {
                *app_state = new_state;
            }
        },
        AppState::Stopping => (),
    }
}
