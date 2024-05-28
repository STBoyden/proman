mod config;
mod widgets;

use crate::{config::parse_language_configs, widgets::StatefulList};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, prelude::*, Terminal};
use std::{
    io::{self, stdout},
    time,
};
use widgets::StatefulListItem;

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut language_list = StatefulList::new(parse_language_configs()?);

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|frame: &mut Frame| ui(frame, &mut language_list))?;
        should_quit = handle_events(&mut language_list)?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_events<T>(language_list: &mut StatefulList<T>) -> io::Result<bool>
where
    for<'a> T: StatefulListItem<'a>,
{
    if event::poll(time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Char('k') | KeyCode::Up => {
                        language_list.previous_item();
                        return Ok(false);
                    },
                    KeyCode::Char('j') | KeyCode::Down => {
                        language_list.next_item();
                        return Ok(false);
                    },
                    _ => return Ok(false),
                }
            }
        }
    }

    Ok(false)
}

fn ui<T>(frame: &mut Frame, language_list: &mut StatefulList<T>)
where
    for<'a> T: StatefulListItem<'a>,
{
    language_list.draw(
        frame,
        frame.size(),
        String::from("Please choose a language"),
    )
}
