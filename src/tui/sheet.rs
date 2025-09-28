use ratatui::{
    layout::Alignment,
    text::Line,
    widgets::{Clear, Paragraph, StatefulWidget, Widget, Wrap},
};

use crate::{
    misc::globals::theme,
    tui::{
        status_bar::{StatusBar, Tag},
        utils::Scroll,
        widgets::block::Block,
    },
};

#[derive(Debug)]
pub struct SheetSection {
    header: String,
    content: String,
}

impl SheetSection {
    pub fn new(header: String, content: String) -> Self {
        Self { header, content }
    }
}

#[derive(Debug)]
pub struct Sheet {
    sections: Vec<SheetSection>,
}

impl Sheet {
    pub fn new() -> Self {
        Self {
            sections: Default::default(),
        }
    }

    pub fn with_sections(mut self, sections: Vec<SheetSection>) -> Self {
        self.sections = sections;
        self
    }
}

impl Default for Sheet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default)]
pub struct SheetState {
    scroll: Scroll,
}

impl SheetState {
    pub fn scroll_up(&mut self) {
        self.scroll.up();
    }

    pub fn scroll_down(&mut self) {
        self.scroll.down();
    }
}

impl StatefulWidget for Sheet {
    type State = SheetState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        Clear.render(area, buf);

        let pg = Paragraph::new(
            self.sections
                .iter()
                .enumerate()
                .flat_map(|(idx, SheetSection { header, content })| {
                    std::iter::once(Line::raw(header).style(theme().header(idx)))
                        .chain(
                            content
                                .lines()
                                .map(|line| Line::raw(line).style(theme().text())),
                        )
                        .chain(std::iter::once(Line::raw("\n")))
                })
                .collect::<Vec<_>>(),
        )
        .style(theme().text())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .bottom(
                    StatusBar::new()
                        .mono_color()
                        .centered()
                        .tag(Tag::new(" Scroll Up ", " Shift+K | Shift+\u{2191} "))
                        .tag(Tag::new(" Scroll Down ", " Shift+J | Shift+\u{2193} ")),
                )
                .title_alignment(Alignment::Center)
                .into_widget(),
        );

        state
            .scroll
            .adjust(pg.line_count(area.width), area.height.saturating_sub(2));

        pg.scroll((state.scroll.val_u16(), 0)).render(area, buf);
    }
}
