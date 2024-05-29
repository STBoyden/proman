use std::{io::stdout, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    ExecutableCommand,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, prelude::*, Terminal};

use crate::{
    config::{LanguageConfig, LanguageConfigRunner, parse_language_configs, RunningConfigMessage},
    widgets::{StatefulList, StatefulListItem},
};

#[allow(clippy::pedantic, clippy::nursery)]
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

#[derive(Clone, Debug, Default)]
struct RunningState {
    step_name: String,
    scroll_back: Vec<String>,
    input_mode: bool,
    input: String,
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

/// Handle events that happen during the runtime of the application, can include key
/// events, or other custom-made events that the application should be able to respond to.
fn handle_events<ListItem>(app_state: &mut AppState<ListItem>) -> config::Result<Message>
where
    for<'a> ListItem: StatefulListItem<'a>,
{
    let message = match app_state {
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
                    KeyCode::Enter => Ok(Message::RunConfiguration(list.get_item())),
                    _ => Ok(Message::NoOp),
                },
            ),
        ),
        AppState::Running(_, running_state) => key_handler(
            running_state,
            Box::new(|running_state: &mut Option<RunningState>, key_code| {
                let mut state = if running_state.is_some() {
                    running_state.clone().unwrap()
                } else {
                    RunningState::default()
                };

                if state.input_mode {
                    match key_code {
                        KeyCode::Char(character) => {
                            state.input.push(character);
                            Ok(Message::NoOp)
                        },
                        _ => Ok(Message::NoOp),
                    }
                } else {
                    match key_code {
                        KeyCode::Char('q') => Ok(Message::ShouldQuit),
                        _ => Ok(Message::NoOp),
                    }
                }
            }),
        ),
        _ => Ok(Message::NoOp),
    };

    message
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
            let mut state = if running_state.is_some() {
                running_state.clone().unwrap()
            } else {
                RunningState::default()
            };
            let mut stop = false;

            if let Some(ref rx) = runner.start_or_continue() {
                if let Ok((message, should_stop)) = rx.recv() {
                    if should_stop {
                        stop = should_stop;
                    }

                    // TODO: handle user input for the project name and project type
                    match message {
                        RunningConfigMessage::SetCommandStepText(text) => state.step_name = text,
                        RunningConfigMessage::CommandOutput(output) =>
                            state.scroll_back.push(output),
                        RunningConfigMessage::StartInputPrompt => {
                            state.input_mode = true;
                        },
                        RunningConfigMessage::PromptForProjectName(name_tx) => {},
                        RunningConfigMessage::NoOp => (),
                    }

                    *running_state = Some(state);

                    if stop {
                        *app_state = AppState::Stopping;
                    }
                }
            } else {
                cleanup().unwrap();
                panic!("could not receive from command")
            };
        },
        AppState::Stopping => (),
    }
}
