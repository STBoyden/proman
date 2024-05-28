use std::collections::BTreeSet;

use ratatui::{prelude::*, text::Text, widgets::*};

pub(crate) trait StatefulListItem<'a>:
    Clone + Eq + Ord + Into<ListItem<'a>> + Into<Text<'a>>
{
}

impl<'a, T: Clone + Eq + Ord + Into<ListItem<'a>> + Into<Text<'a>>> StatefulListItem<'a> for T {}

#[derive(Debug, Clone)]
pub(crate) struct StatefulList<T>
where
    for<'a> T: StatefulListItem<'a>,
{
    items: BTreeSet<T>,
    selected_index: usize,
    list_state: ListState,
}

impl<T> StatefulList<T>
where
    for<'a> T: StatefulListItem<'a>,
{
    pub(crate) fn new(items: BTreeSet<T>) -> Self {
        Self {
            items,
            selected_index: 0,
            list_state: ListState::default(),
        }
    }

    pub(crate) fn next_item(&mut self) {
        if self.selected_index + 1 > self.items.len() {
            self.selected_index = 0;
        } else {
            self.selected_index += 1;
        }

        self.list_state = self
            .list_state
            .clone()
            .with_selected(Some(self.selected_index));
    }

    pub(crate) fn previous_item(&mut self) {
        if self.selected_index.saturating_sub(1) == 0 {
            self.selected_index = self.items.len() - 1;
        } else {
            self.selected_index -= 1;
        }

        self.list_state = self
            .list_state
            .clone()
            .with_selected(Some(self.selected_index));
    }

    pub(crate) fn draw<'b, S: 'b>(&mut self, frame: &mut Frame, area: Rect, title: S)
    where
        Text<'b>: From<S>,
        Line<'b>: From<S>,
    {
        let items = self.items.iter().cloned().collect::<Vec<_>>();

        let list = List::new(items)
            .block(Block::default().title(title).borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>")
            .direction(ListDirection::TopToBottom);

        frame.render_stateful_widget(list, area, &mut self.list_state)
    }
}
