use std::collections::BTreeSet;

use ratatui::{prelude::*, text::Text, widgets::*};

pub(crate) trait StatefulListItem<'a>:
    Clone + Eq + Ord + Into<ListItem<'a>> + Into<Text<'a>>
{
}

impl<'a, T: Clone + Eq + Ord + Into<ListItem<'a>> + Into<Text<'a>>> StatefulListItem<'a> for T {}

#[derive(Debug, Clone)]
pub(crate) struct StatefulList<ListItem>
where
    for<'a> ListItem: StatefulListItem<'a>,
{
    items:          BTreeSet<ListItem>,
    selected_index: usize,
    list_state:     ListState,
}

impl<ListItem> StatefulList<ListItem>
where
    for<'a> ListItem: StatefulListItem<'a>,
{
    pub(crate) fn new(items: BTreeSet<ListItem>) -> Self {
        let selected_index = 0;
        let list_state = ListState::default().with_selected(Some(selected_index));

        Self {
            items,
            selected_index,
            list_state,
        }
    }

    pub(crate) fn set_items(&mut self, items: BTreeSet<ListItem>) {
        if items.len() < self.items.len() {
            self.selected_index = items.len() - 1;
        }

        self.items = items;
    }

    pub(crate) fn get_items(&self) -> Vec<ListItem> {
        self.items.iter().cloned().collect::<Vec<ListItem>>()
    }

    pub(crate) fn next_item(&mut self) {
        if self.selected_index.saturating_add(1) >= self.items.len() {
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
        if self.selected_index.wrapping_sub(1) == usize::MAX {
            self.selected_index = self.items.len() - 1;
        } else {
            self.selected_index -= 1;
        }

        self.list_state = self
            .list_state
            .clone()
            .with_selected(Some(self.selected_index));
    }

    pub(crate) fn get_selected_index(&self) -> usize { self.selected_index }

    pub(crate) fn draw<'b, S: 'b>(&mut self, frame: &mut Frame, area: Rect, title: S)
    where
        Text<'b>: From<S>,
        Line<'b>: From<S>,
    {
        let items = self.items.iter().cloned().collect::<Vec<_>>();

        let list = List::new(items)
            .block(Block::default().title(title).borders(Borders::ALL))
            .direction(ListDirection::TopToBottom)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>")
            .style(Style::default().fg(Color::White));

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
}
