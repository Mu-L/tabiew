use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Flex, Layout},
    widgets::{Clear, StatefulWidget, Widget},
};

use crate::tui::widgets::{
    block::Block,
    input::{Input, InputState, InputType},
};

#[derive(Debug)]
pub struct GoToLineState {
    rollback: usize,
    input: InputState,
}

impl GoToLineState {
    pub fn new(rollback: usize) -> Self {
        Self {
            input: InputState::default().with_input_type(InputType::Numeric),
            rollback,
        }
    }

    pub fn with_value(self, value: usize) -> Self {
        Self {
            rollback: self.rollback,
            input: InputState::default().with_value(value.to_string()),
        }
    }

    pub fn handle(&mut self, event: KeyEvent) {
        self.input.handle(event);
    }

    pub fn rollback(&self) -> usize {
        self.rollback
    }

    pub fn value(&self) -> usize {
        self.input.value().parse().unwrap_or(1)
    }
}

#[derive(Debug, Default)]
pub struct GoToLine {}

impl StatefulWidget for GoToLine {
    type State = GoToLineState;

    fn render(
        self,
        _: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let [_, area] = Layout::horizontal([Constraint::Length(1), Constraint::Length(32)])
            .flex(Flex::End)
            .areas(buf.area);
        let [_, area] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(3)]).areas(area);
        Clear.render(area, buf);

        Input::default()
            .block(Block::default().title("Go to line"))
            .render(area, buf, &mut state.input);
    }
}
