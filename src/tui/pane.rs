use polars::frame::DataFrame;
use ratatui::{
    layout::{Constraint, Flex, Layout, Margin, Rect},
    widgets::{StatefulWidget, Widget},
};

use super::{
    data_frame_table::{DataFrameTable, DataFrameTableState},
    popups::help_modal::HelpModal,
    search_bar::{SearchBar, SearchBarState},
    sheet::{Sheet, SheetState},
};
use crate::{
    misc::{
        globals::sql,
        polars_ext::GetSheetSections,
        sql::{Source, TableInfo},
    },
    tui::{
        plots::{
            histogram_plot::{HistogramPlot, HistogramPlotState},
            scatter_plot::{ScatterPlot, ScatterPlotState},
        },
        popups::inline_query::{InlineQuery, InlineQueryState, InlineQueryType},
        schema::data_frame_info::{DataFrameInfo, DataFrameInfoState},
    },
};

#[derive(Debug, Default)]
pub enum Modal {
    Sheet(SheetState),
    SearchBar(SearchBarState),
    DataFrameInfo(DataFrameInfoState),
    ScatterPlot(ScatterPlotState),
    HistogramPlot(HistogramPlotState),
    InlineQuery(InlineQueryState),
    Help,
    #[default]
    None,
}

impl Modal {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableType {
    Help,
    Name(String),
    Query(String),
}

impl AsRef<str> for TableType {
    fn as_ref(&self) -> &str {
        match self {
            TableType::Help => "Help",
            TableType::Name(name) => name.as_str(),
            TableType::Query(query) => query.as_str(),
        }
    }
}

#[derive(Debug)]
pub struct PaneState {
    table: DataFrameTableState,
    modal: Modal,
    table_type: TableType,
}

impl PaneState {
    /// Constructs a new instance of [`App`].
    pub fn new(data_frame: DataFrame, table_type: TableType) -> Self {
        Self {
            table: DataFrameTableState::new(data_frame.clone()),
            modal: Modal::None,
            table_type,
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        match &mut self.modal {
            Modal::SearchBar(search_bar) => {
                if let Some(df) = search_bar.search().latest() {
                    self.table.set_data_frame(df);
                }
            }
            Modal::Sheet(_) => (),
            Modal::DataFrameInfo(_) => (),
            Modal::None => (),
            Modal::ScatterPlot(_) => (),
            Modal::HistogramPlot(_) => (),
            Modal::InlineQuery(_) => (),
            Modal::Help => (),
        }
    }

    pub fn table(&self) -> &DataFrameTableState {
        &self.table
    }

    pub fn table_mut(&mut self) -> &mut DataFrameTableState {
        &mut self.table
    }

    pub fn table_type(&self) -> &TableType {
        &self.table_type
    }

    pub fn show_sheet(&mut self) {
        self.modal = Modal::Sheet(Default::default());
    }

    pub fn show_fuzzy_search(&mut self) {
        self.modal = Modal::SearchBar(SearchBarState::fuzzy(self.table.data_frame().clone()));
    }

    pub fn show_exact_search(&mut self) {
        self.modal = Modal::SearchBar(SearchBarState::exact(self.table.data_frame().clone()));
    }

    pub fn show_data_frame_info(&mut self) {
        self.modal = Modal::DataFrameInfo(Default::default())
    }

    pub fn show_scatter_plot(&mut self, scatter: ScatterPlotState) {
        self.modal = Modal::ScatterPlot(scatter)
    }

    pub fn show_histogram_plot(&mut self, hist: HistogramPlotState) {
        self.modal = Modal::HistogramPlot(hist)
    }

    pub fn show_inline_query(&mut self, query_type: InlineQueryType) {
        self.modal = Modal::InlineQuery(InlineQueryState::new(query_type));
    }

    pub fn show_help(&mut self) {
        self.modal = Modal::Help
    }

    pub fn modal(&self) -> &Modal {
        &self.modal
    }

    pub fn modal_mut(&mut self) -> &mut Modal {
        &mut self.modal
    }

    pub fn modal_take(&mut self) -> Modal {
        std::mem::replace(&mut self.modal, Modal::None)
    }
}

#[derive(Debug, Default)]
pub struct Pane;

impl StatefulWidget for Pane {
    type State = PaneState;

    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        let (search_bar_area, table_area) = match state.modal {
            Modal::SearchBar(_) => {
                let [a0, a1] =
                    Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(area);
                (a0, a1)
            }
            _ => (Rect::default(), area),
        };
        DataFrameTable::new().render(table_area, buf, &mut state.table);

        match &mut state.modal {
            Modal::Sheet(sheet_state) => {
                let area = area.inner(Margin::new(13, 3));
                let sections = state
                    .table
                    .data_frame()
                    .get_sheet_sections(state.table.selected());
                Sheet::new()
                    .with_sections(sections)
                    .render(area, buf, sheet_state);
            }
            Modal::SearchBar(search_bar_state) => {
                SearchBar::new().with_selection(true).render(
                    search_bar_area,
                    buf,
                    search_bar_state,
                );
            }
            Modal::DataFrameInfo(data_frame_info) => {
                //
                let [area] = Layout::horizontal([Constraint::Length(100)])
                    .flex(Flex::Center)
                    .areas(area);
                let [_, area] = Layout::vertical([Constraint::Length(3), Constraint::Length(25)])
                    // .flex(Flex::Center)
                    .areas(area);
                match &state.table_type {
                    TableType::Help => todo!(),
                    TableType::Name(name) => {
                        if let Some(tbl_info) = sql().schema().get(name) {
                            let source = tbl_info.source().clone();
                            DataFrameInfo::new(&TableInfo::new(
                                source,
                                state.table.data_frame_mut(),
                            ))
                            .render(area, buf, data_frame_info);
                        }
                    }
                    TableType::Query(_) => {
                        DataFrameInfo::new(&TableInfo::new(
                            Source::User,
                            state.table.data_frame_mut(),
                        ))
                        .render(area, buf, data_frame_info);
                    }
                };
            }
            Modal::ScatterPlot(state) => {
                let [area] = Layout::horizontal([Constraint::Length(120)])
                    .flex(Flex::Center)
                    .areas(area);
                let [_, area] = Layout::vertical([Constraint::Length(3), Constraint::Length(40)])
                    // .flex(Flex::Center)
                    .areas(area);
                ScatterPlot::default().render(area, buf, state);
            }
            Modal::HistogramPlot(state) => {
                let [area] = Layout::horizontal([Constraint::Length(122)])
                    .flex(Flex::Center)
                    .areas(area);
                let [_, area] = Layout::vertical([Constraint::Length(3), Constraint::Length(40)])
                    // .flex(Flex::Center)
                    .areas(area);
                HistogramPlot.render(area, buf, state);
            }
            Modal::InlineQuery(state) => {
                InlineQuery::default().render(area, buf, state);
            }
            Modal::Help => {
                let [area] = Layout::horizontal([Constraint::Length(90)])
                    .flex(Flex::Center)
                    .areas(area);
                let [_, area] =
                    Layout::vertical([Constraint::Length(2), Constraint::Length(50)]).areas(area);
                HelpModal::new().render(area, buf);
            }
            Modal::None => (),
        }
    }
}
