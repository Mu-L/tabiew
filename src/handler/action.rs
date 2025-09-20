use std::{fs, ops::Div};

use anyhow::{Ok, anyhow};
use crossterm::event::KeyEvent;
use itertools::Itertools;
use polars::frame::DataFrame;
use rand::Rng;

use crate::{
    AppResult,
    app::{App, Content},
    misc::{
        globals::{config, sql},
        paths::config_path,
        polars_ext::{IntoString, PlotData},
        type_inferer::TypeInferer,
    },
    reader::{
        ArrowIpcToDataFrame, CsvToDataFrame, FwfToDataFrame, JsonLineToDataFrame, JsonToDataFrame,
        ParquetToDataFrame, ReadToDataFrames, Source, SqliteToDataFrames,
    },
    tui::{
        TabContentState, TableType,
        data_frame_table::DataFrameTableState,
        plots::{histogram_plot::HistogramPlotState, scatter_plot::ScatterPlotState},
        schema::data_frame_info::DataFrameInfoState,
        search_bar::SearchBarState,
        tab_content::Modal,
        theme::Theme,
    },
    writer::{
        Destination, JsonFormat, WriteToArrow, WriteToCsv, WriteToFile, WriteToJson, WriteToParquet,
    },
};

use super::command::{commands_help_data_frame, parse_into_action};

#[derive(Debug, Clone)]
pub enum AppAction {
    NoAction,
    ToggleBorders,
    DismissError,
    DismissErrorAndShowPalette,
    GotoLine(usize),
    SwitchToSchema,
    SwitchToTabulars,

    TableInferColumns(TypeInferer),
    TableDismissModal,
    TableScrollRight,
    TableScrollLeft,
    TableScrollRightColumn,
    TableScrollLeftColumn,
    TableScrollStart,
    TableScrollEnd,
    TableToggleExpansion,
    TableGotoFirst,
    TableGotoLast,
    TableGotoRandom,
    TableGoUp(usize),
    TableGoUpHalfPage,
    TableGoUpFullPage,
    TableGoDown(usize),
    TableGoDownHalfPage,
    TableGoDownFullPage,
    TableSelect(String),
    TableOrder(String),
    TableFilter(String),
    TableQuery(String),
    TableSetDataFrame(DataFrame),
    TableReset,

    SheetShow,
    SheetScrollUp,
    SheetScrollDown,

    PaletteGotoNext,
    PaletteGotoPrev,
    PaletteGotoNextWord,
    PaletteGotoPrevWord,
    PaletteGotoStart,
    PaletteGotoEnd,
    PaletteDeleteNext,
    PaletteDeletePrev,
    PaletteDeleteNextWord,
    PaletteDeletePrevWord,
    PaletteInsert(char),
    PaletteInsertSelectedOrCommit,
    PaletteShow(String),
    PaletteDeselectOrDismiss,
    PaletteSelectPrevious,
    PaletteSelectNext,

    SearchFuzzyShow,
    SearchExactShow,
    SearchGotoNext,
    SearchGotoPrev,
    SearchGotoNextWord,
    SearchGotoPrevWord,
    SearchGotoStart,
    SearchGotoEnd,
    SearchDeleteNext,
    SearchDeletePrev,
    SearchDeleteNextWord,
    SearchDeletePrevWord,
    SearchInsert(char),
    SearchRollback,
    SearchCommit,

    TabNewQuery(String),
    TabSelect(usize),
    TabRemove(usize),
    TabPrev,
    TabNext,
    TabRemoveOrQuit,
    TabRename(usize, String),
    TabShowPanel,
    TabHidePanel,
    TabPanelPrev,
    TabPanelNext,
    TabPanelSelect,

    ExportDsv {
        destination: Destination,
        separator: char,
        quote: char,
        header: bool,
    },
    ExportParquet(Destination),
    ExportJson(Destination, JsonFormat),
    ExportArrow(Destination),
    ImportDsv {
        source: Source,
        separator: char,
        has_header: bool,
        quote: char,
    },

    ImportParquet(Source),
    ImportJson(Source, JsonFormat),
    ImportArrow(Source),
    ImportSqlite(Source),
    ImportFwf {
        source: Source,
        widths: Vec<usize>,
        separator_length: usize,
        flexible_width: bool,
        has_header: bool,
    },

    SchemaNamesSelectPrev,
    SchemaNamesSelectNext,
    SchemaNamesSelectFirst,
    SchemaNamesSelectLast,
    SchemaFieldsScrollUp,
    SchemaFieldsScrollDown,
    SchemaOpenTable,
    SchemaUnloadTable,

    DataFrameInfoScrollUp,
    DataFrameInfoScrollDown,
    DataFrameInfoShow,

    ScatterPlot(String, String, Vec<String>),

    HistogramPlot(String, usize),
    HistogramScrollUp,
    HistogramScrollDown,

    PreviewTheme(Theme),
    StoreConfig,

    ThemeSelectorSelectPrev,
    ThemeSelectorSelectNext,
    ThemeSelectorRollback,
    ThemeSelectorCommit,
    ThemeSelectorHandleEvent(KeyEvent),

    RegisterDataFrame(String),
    Help,
    Quit,
}

pub fn execute(action: AppAction, app: &mut App) -> AppResult<Option<AppAction>> {
    match action {
        AppAction::NoAction => Ok(None),
        AppAction::DismissError => {
            app.dismiss_error();
            Ok(None)
        }
        AppAction::ToggleBorders => {
            app.toggle_borders();
            Ok(None)
        }
        AppAction::SwitchToSchema => {
            app.switch_schema();
            Ok(None)
        }
        AppAction::SwitchToTabulars => {
            app.switch_tabular();
            Ok(None)
        }
        AppAction::DismissErrorAndShowPalette => {
            app.dismiss_error();
            app.show_palette("");
            Ok(None)
        }
        AppAction::TableDismissModal => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                tab.modal_take();
            }
            Ok(None)
        }
        AppAction::SheetShow => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                tab.show_sheet()
            }
            Ok(None)
        }
        AppAction::SearchFuzzyShow => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                tab.show_fuzzy_search();
            }
            Ok(None)
        }
        AppAction::SearchExactShow => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                tab.show_exact_search();
            }
            Ok(None)
        }
        AppAction::GotoLine(line) => {
            match app.context() {
                crate::app::Context::Table => {
                    if let Some(tabular) = app.tabs_mut().selected_mut() {
                        tabular.table_mut().select(line)
                    }
                }
                crate::app::Context::Schema => {
                    app.schema_mut().names_mut().table_mut().select(line.into());
                }
                _ => (),
            }
            Ok(None)
        }
        AppAction::TableInferColumns(type_inferer) => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                let df = tab.table_mut().data_frame_mut();
                type_inferer.update(df);
            }
            Ok(None)
        }
        AppAction::TableGotoFirst => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                tab.table_mut().select_first()
            }
            Ok(None)
        }
        AppAction::TableGotoLast => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                tab.table_mut().select_last()
            }
            Ok(None)
        }
        AppAction::TableGotoRandom => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                let random_row = rand::rng().random_range(0..tab.table().data_frame().height());
                tab.table_mut().select(random_row);
            }
            Ok(None)
        }
        AppAction::TableGoUp(lines) => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                tab.table_mut().select_up(lines)
            }
            Ok(None)
        }
        AppAction::TableGoUpHalfPage => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                let len = tab.table().rendered_rows().div(2).into();
                tab.table_mut().select_up(len)
            }
            Ok(None)
        }
        AppAction::TableGoUpFullPage => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                let len = tab.table().rendered_rows().into();
                tab.table_mut().select_up(len)
            }
            Ok(None)
        }
        AppAction::TableGoDown(lines) => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                tab.table_mut().select_down(lines)
            }
            Ok(None)
        }
        AppAction::TableGoDownHalfPage => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                let len = tab.table().rendered_rows().div(2).into();
                tab.table_mut().select_down(len)
            }
            Ok(None)
        }
        AppAction::TableGoDownFullPage => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                let len = tab.table().rendered_rows().into();
                tab.table_mut().select_down(len)
            }
            Ok(None)
        }
        AppAction::TableSelect(select) => Ok(Some(AppAction::TableQuery(format!(
            "SELECT {select} FROM _"
        )))),
        AppAction::TableOrder(order) => Ok(Some(AppAction::TableQuery(format!(
            "SELECT * FROM _ ORDER BY {order}"
        )))),
        AppAction::TableFilter(filter) => Ok(Some(AppAction::TableQuery(format!(
            "SELECT * FROM _ where {filter}"
        )))),
        AppAction::TableQuery(query) => {
            let df = sql().execute(
                &query,
                app.tabs()
                    .selected()
                    .map(TabContentState::table)
                    .map(DataFrameTableState::data_frame)
                    .cloned(),
            )?;
            Ok(Some(AppAction::TableSetDataFrame(df)))
        }
        AppAction::TableSetDataFrame(df) => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                tab.table_mut().set_data_frame(df.clone());
            }
            Ok(None)
        }
        AppAction::TableReset => {
            let query = match app.tabs_mut().selected().map(|ts| ts.table_type()) {
                Some(TableType::Name(name)) => Some(format!("SELECT * FROM '{name}'")),
                Some(TableType::Query(query)) => Some(query.to_owned()),
                Some(_) => None,
                None => None,
            };
            Ok(query.map(AppAction::TableQuery))
        }
        AppAction::SheetScrollUp => {
            if let Some(sheet) = app
                .tabs_mut()
                .selected_mut()
                .map(TabContentState::modal_mut)
                .and_then(Modal::sheet_mut)
            {
                sheet.scroll_up();
            }
            Ok(None)
        }
        AppAction::SheetScrollDown => {
            if let Some(sheet) = app
                .tabs_mut()
                .selected_mut()
                .map(TabContentState::modal_mut)
                .and_then(Modal::sheet_mut)
            {
                sheet.scroll_down();
            }
            Ok(None)
        }
        AppAction::TabNewQuery(query) => {
            if sql().schema().iter().any(|(name, _)| name == &query) {
                let df = sql().execute(&format!("SELECT * FROM '{query}'"), None)?;
                app.tabs_mut()
                    .add(TabContentState::new(df, TableType::Name(query)));
            } else {
                let df = sql().execute(
                    &query,
                    app.tabs()
                        .selected()
                        .map(TabContentState::table)
                        .map(DataFrameTableState::data_frame)
                        .cloned(),
                )?;
                app.tabs_mut()
                    .add(TabContentState::new(df, TableType::Query(query)));
            }

            Ok(Some(AppAction::TabSelect(
                app.tabs_mut().len().saturating_sub(1),
            )))
        }
        AppAction::TabSelect(idx) => {
            let idx = idx.min(app.tabs_mut().len().saturating_sub(1));
            app.tabs_mut().select(idx);
            app.switch_tabular();
            Ok(None)
        }
        AppAction::TabRemove(idx) => {
            app.tabs_mut().remove(idx);
            Ok(Some(AppAction::TabSelect(idx)))
        }
        AppAction::TabRename(_idx, _new_name) => {
            todo!()
        }
        AppAction::TabPrev => Ok(Some(AppAction::TabSelect(
            app.tabs_mut().idx().saturating_sub(1),
        ))),
        AppAction::TabNext => Ok(Some(AppAction::TabSelect(
            app.tabs_mut().idx().saturating_add(1),
        ))),
        AppAction::TabRemoveOrQuit => {
            if app.tabs_mut().len() == 1 {
                app.quit();
                Ok(None)
            } else {
                let idx = app.tabs_mut().idx();
                app.tabs_mut().remove(idx);
                Ok(Some(AppAction::TabSelect(idx)))
            }
        }
        AppAction::ExportDsv {
            destination,
            separator,
            quote,
            header,
        } => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                WriteToCsv::default()
                    .with_separator_char(separator)
                    .with_quote_char(quote)
                    .with_header(header)
                    .write_to_file(destination, tab.table_mut().data_frame_mut())?;
                Ok(None)
            } else {
                Err(anyhow!("Unable to export the data frame"))
            }
        }
        AppAction::ExportParquet(path) => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                WriteToParquet.write_to_file(path, tab.table_mut().data_frame_mut())?;
                Ok(None)
            } else {
                Err(anyhow!("Unable to export the data frame"))
            }
        }
        AppAction::ExportJson(path, fmt) => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                WriteToJson::default()
                    .with_format(fmt)
                    .write_to_file(path, tab.table_mut().data_frame_mut())?;
                Ok(None)
            } else {
                Err(anyhow!("Unable to export the data frame"))
            }
        }
        AppAction::ExportArrow(path) => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                WriteToArrow.write_to_file(path, tab.table_mut().data_frame_mut())?;
                Ok(None)
            } else {
                Err(anyhow!("Unable to export the data frame"))
            }
        }
        AppAction::ImportDsv {
            source,
            separator,
            has_header,
            quote,
        } => {
            let frames = CsvToDataFrame::default()
                .with_separator(separator)
                .with_quote_char(quote)
                .with_no_header(!has_header)
                .named_frames(source.clone())?;
            for (name, df) in frames {
                let name = sql().register(&name, df.clone(), source.clone());
                app.tabs_mut()
                    .add(TabContentState::new(df, TableType::Name(name)));
            }
            Ok(None)
        }
        AppAction::ImportParquet(source) => {
            let frames = ParquetToDataFrame.named_frames(source.clone())?;
            for (name, df) in frames {
                let name = sql().register(&name, df.clone(), source.clone());
                app.tabs_mut()
                    .add(TabContentState::new(df, TableType::Name(name)));
            }
            Ok(Some(AppAction::TabSelect(
                app.tabs_mut().len().saturating_sub(1),
            )))
        }
        AppAction::ImportJson(source, json_format) => {
            let frames = match json_format {
                JsonFormat::Json => JsonToDataFrame::default().named_frames(source.clone())?,
                JsonFormat::JsonLine => {
                    JsonLineToDataFrame::default().named_frames(source.clone())?
                }
            };
            for (name, df) in frames {
                let name = sql().register(&name, df.clone(), source.clone());
                app.tabs_mut()
                    .add(TabContentState::new(df, TableType::Name(name)));
            }
            Ok(Some(AppAction::TabSelect(
                app.tabs_mut().len().saturating_sub(1),
            )))
        }
        AppAction::ImportArrow(source) => {
            let frames = ArrowIpcToDataFrame.named_frames(source.clone())?;
            for (name, df) in frames {
                let name = sql().register(&name, df.clone(), source.clone());
                app.tabs_mut()
                    .add(TabContentState::new(df, TableType::Name(name)));
            }
            Ok(Some(AppAction::TabSelect(
                app.tabs_mut().len().saturating_sub(1),
            )))
        }
        AppAction::ImportSqlite(source) => {
            let frames = SqliteToDataFrames::default().named_frames(source.clone())?;
            for (name, df) in frames {
                let name = sql().register(&name, df.clone(), source.clone());
                app.tabs_mut()
                    .add(TabContentState::new(df, TableType::Name(name)));
            }
            Ok(Some(AppAction::TabSelect(
                app.tabs_mut().len().saturating_sub(1),
            )))
        }
        AppAction::ImportFwf {
            source,
            widths,
            separator_length,
            flexible_width,
            has_header,
        } => {
            let frames = FwfToDataFrame::default()
                .with_widths(widths)
                .with_separator_length(separator_length)
                .with_flexible_width(flexible_width)
                .with_has_header(has_header)
                .named_frames(source.clone())?;
            for (name, df) in frames {
                let name = sql().register(&name, df.clone(), source.clone());
                app.tabs_mut()
                    .add(TabContentState::new(df, TableType::Name(name)));
            }
            Ok(Some(AppAction::TabSelect(
                app.tabs_mut().len().saturating_sub(1),
            )))
        }
        AppAction::SearchGotoNext => {
            if let Some(sb) = app
                .tabs_mut()
                .selected_mut()
                .map(TabContentState::modal_mut)
                .and_then(Modal::search_bar_mut)
            {
                sb.goto_next();
            }
            Ok(None)
        }
        AppAction::SearchGotoPrev => {
            if let Some(sb) = app
                .tabs_mut()
                .selected_mut()
                .map(TabContentState::modal_mut)
                .and_then(Modal::search_bar_mut)
            {
                sb.goto_prev();
            }
            Ok(None)
        }
        AppAction::SearchGotoNextWord => {
            if let Some(sb) = app
                .tabs_mut()
                .selected_mut()
                .map(TabContentState::modal_mut)
                .and_then(Modal::search_bar_mut)
            {
                sb.goto_next_word();
            }
            Ok(None)
        }
        AppAction::SearchGotoPrevWord => {
            if let Some(sb) = app
                .tabs_mut()
                .selected_mut()
                .map(TabContentState::modal_mut)
                .and_then(Modal::search_bar_mut)
            {
                sb.goto_prev_word();
            }
            Ok(None)
        }
        AppAction::SearchGotoStart => {
            if let Some(sb) = app
                .tabs_mut()
                .selected_mut()
                .map(TabContentState::modal_mut)
                .and_then(Modal::search_bar_mut)
            {
                sb.goto_start();
            }
            Ok(None)
        }
        AppAction::SearchGotoEnd => {
            if let Some(sb) = app
                .tabs_mut()
                .selected_mut()
                .map(TabContentState::modal_mut)
                .and_then(Modal::search_bar_mut)
            {
                sb.goto_end();
            }
            Ok(None)
        }
        AppAction::SearchDeleteNext => {
            if let Some(sb) = app
                .tabs_mut()
                .selected_mut()
                .map(TabContentState::modal_mut)
                .and_then(Modal::search_bar_mut)
            {
                sb.delete_next();
            }
            Ok(None)
        }
        AppAction::SearchDeletePrev => {
            if let Some(sb) = app
                .tabs_mut()
                .selected_mut()
                .map(TabContentState::modal_mut)
                .and_then(Modal::search_bar_mut)
            {
                sb.delete_prev();
            }
            Ok(None)
        }
        AppAction::SearchDeleteNextWord => {
            if let Some(sb) = app
                .tabs_mut()
                .selected_mut()
                .map(TabContentState::modal_mut)
                .and_then(Modal::search_bar_mut)
            {
                sb.delete_next_word();
            }
            Ok(None)
        }
        AppAction::SearchDeletePrevWord => {
            if let Some(sb) = app
                .tabs_mut()
                .selected_mut()
                .map(TabContentState::modal_mut)
                .and_then(Modal::search_bar_mut)
            {
                sb.delete_prev_word();
            }
            Ok(None)
        }
        AppAction::SearchInsert(c) => {
            if let Some(sb) = app
                .tabs_mut()
                .selected_mut()
                .map(TabContentState::modal_mut)
                .and_then(Modal::search_bar_mut)
            {
                sb.insert(c);
            }
            Ok(None)
        }
        AppAction::SearchCommit => {
            if let Some(tab) = app.tabs_mut().selected_mut()
                && let Some(df) = tab
                    .modal_take()
                    .into_search_bar()
                    .and_then(|sb| sb.search().latest())
            {
                tab.table_mut().set_data_frame(df);
            }
            Ok(None)
        }
        AppAction::SearchRollback => {
            if let Some(tab) = app.tabs_mut().selected_mut()
                && let Some(df) = tab
                    .modal_take()
                    .into_search_bar()
                    .map(SearchBarState::into_rollback_df)
            {
                tab.table_mut().set_data_frame(df);
            }
            Ok(None)
        }
        AppAction::Help => {
            let idx =
                app.tabs_mut()
                    .iter()
                    .enumerate()
                    .find_map(|(idx, tab)| match tab.table_type() {
                        TableType::Help => Some(idx),
                        _ => None,
                    });
            if let Some(idx) = idx {
                Ok(Some(AppAction::TabSelect(idx)))
            } else {
                app.tabs_mut().add(TabContentState::new(
                    commands_help_data_frame(),
                    TableType::Help,
                ));
                Ok(Some(AppAction::TabSelect(
                    app.tabs_mut().len().saturating_sub(1),
                )))
            }
        }
        AppAction::Quit => {
            app.quit();
            Ok(None)
        }
        AppAction::TableScrollRight => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                tab.table_mut().scroll_right();
            }
            Ok(None)
        }
        AppAction::TableScrollLeft => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                tab.table_mut().scroll_left();
            }
            Ok(None)
        }
        AppAction::TableScrollRightColumn => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                tab.table_mut().scroll_right_column();
            }
            Ok(None)
        }
        AppAction::TableScrollLeftColumn => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                tab.table_mut().scroll_left_column();
            }
            Ok(None)
        }
        AppAction::TableScrollStart => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                tab.table_mut().scroll_start();
            }
            Ok(None)
        }
        AppAction::TableScrollEnd => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                tab.table_mut().scroll_end();
            }
            Ok(None)
        }
        AppAction::TableToggleExpansion => {
            if let Some(tab) = app.tabs_mut().selected_mut() {
                tab.table_mut().toggle_expansion()?;
            }
            Ok(None)
        }
        AppAction::PaletteGotoNext => {
            if let Some(palette) = app.palette_mut() {
                palette.input().goto_next();
            }
            Ok(None)
        }
        AppAction::PaletteGotoPrev => {
            if let Some(palette) = app.palette_mut() {
                palette.input().goto_prev();
            }
            Ok(None)
        }
        AppAction::PaletteGotoStart => {
            if let Some(palette) = app.palette_mut() {
                palette.input().goto_start();
            }
            Ok(None)
        }
        AppAction::PaletteGotoNextWord => {
            if let Some(palette) = app.palette_mut() {
                palette.input().goto_next_word();
            }
            Ok(None)
        }
        AppAction::PaletteGotoPrevWord => {
            if let Some(palette) = app.palette_mut() {
                palette.input().goto_prev_word();
            }
            Ok(None)
        }
        AppAction::PaletteGotoEnd => {
            if let Some(palette) = app.palette_mut() {
                palette.input().goto_end();
            }
            Ok(None)
        }
        AppAction::PaletteDeleteNext => {
            if let Some(palette) = app.palette_mut() {
                palette.input().delete_next();
            }
            Ok(None)
        }
        AppAction::PaletteDeletePrev => {
            if let Some(palette) = app.palette_mut() {
                palette.input().delete_prev();
            }
            Ok(None)
        }
        AppAction::PaletteDeleteNextWord => {
            if let Some(palette) = app.palette_mut() {
                palette.input().delete_next_word();
            }
            Ok(None)
        }
        AppAction::PaletteDeletePrevWord => {
            if let Some(palette) = app.palette_mut() {
                palette.input().delete_prev_word();
            }
            Ok(None)
        }
        AppAction::PaletteInsert(c) => {
            if let Some(palette) = app.palette_mut() {
                palette.input().insert(c);
                palette.list().select(None);
            }
            Ok(None)
        }
        AppAction::PaletteInsertSelectedOrCommit => {
            if let Some(selected) = app
                .palette_mut()
                .and_then(|palette| palette.list().selected())
            {
                if let Some(cmd) = app.history_mut().get(selected).map(String::to_owned)
                    && let Some(palette) = app.palette_mut()
                {
                    palette.set_input(cmd);
                    palette.list().select(None);
                }
                Ok(None)
            } else if let Some(cmd) = app.hide_palette() {
                if cmd.is_empty() {
                    Ok(Some(AppAction::PaletteDeselectOrDismiss))
                } else {
                    app.history_mut().push(cmd.clone());
                    parse_into_action(cmd).map(Some)
                }
            } else {
                Ok(None)
            }
        }
        AppAction::PaletteShow(text) => {
            app.show_palette(text);
            Ok(None)
        }
        AppAction::PaletteDeselectOrDismiss => {
            if let Some(palette) = app.palette_mut() {
                if palette.list().selected().is_some() {
                    palette.list().select(None);
                } else {
                    app.hide_palette();
                }
            }
            Ok(None)
        }
        AppAction::PaletteSelectPrevious => {
            if let Some(palette) = app.palette_mut() {
                palette.list().select_previous();
            }
            Ok(None)
        }
        AppAction::PaletteSelectNext => {
            if let Some(palette) = app.palette_mut() {
                palette.list().select_next();
            }
            Ok(None)
        }
        AppAction::SchemaNamesSelectPrev => {
            if app.content() == &Content::Schema {
                app.schema_mut().names_mut().table_mut().select_previous();
            }
            Ok(None)
        }
        AppAction::SchemaNamesSelectNext => {
            if app.content() == &Content::Schema {
                app.schema_mut().names_mut().table_mut().select_next();
            }
            Ok(None)
        }
        AppAction::SchemaNamesSelectFirst => {
            if app.content() == &Content::Schema {
                app.schema_mut().names_mut().table_mut().select_first();
            }
            Ok(None)
        }
        AppAction::SchemaNamesSelectLast => {
            if app.content() == &Content::Schema {
                app.schema_mut().names_mut().table_mut().select_last();
            }
            Ok(None)
        }
        AppAction::SchemaFieldsScrollUp => {
            if app.content() == &Content::Schema {
                *app.schema_mut()
                    .data_frame_info_mut()
                    .field_info_mut()
                    .table_state_mut()
                    .offset_mut() = app
                    .schema()
                    .data_frame_info()
                    .field_info()
                    .table_state()
                    .offset()
                    .saturating_sub(1);
            }
            Ok(None)
        }
        AppAction::SchemaFieldsScrollDown => {
            if app.content() == &Content::Schema {
                *app.schema_mut()
                    .data_frame_info_mut()
                    .field_info_mut()
                    .table_state_mut()
                    .offset_mut() = app
                    .schema()
                    .data_frame_info()
                    .field_info()
                    .table_state()
                    .offset()
                    .saturating_add(1);
            }
            Ok(None)
        }
        AppAction::SchemaOpenTable => {
            let table_name = app
                .schema()
                .names()
                .table()
                .selected()
                .and_then(|idx| {
                    sql()
                        .schema()
                        .get_by_index(idx)
                        .map(|(name, _)| name.to_owned())
                })
                .ok_or(anyhow!("No table is selected"))?;

            let tab_idx = app
                .tabs_mut()
                .iter()
                .map(|tabular| tabular.table_type())
                .enumerate()
                .find_map(|(idx, tab_type)| match tab_type {
                    TableType::Name(name) if name.as_str() == table_name => Some(idx),
                    _ => None,
                });

            if let Some(tab_idx) = tab_idx {
                Ok(Some(AppAction::TabSelect(tab_idx)))
            } else {
                Ok(Some(AppAction::TabNewQuery(table_name)))
            }
        }
        AppAction::SchemaUnloadTable => {
            let table_name = app
                .schema()
                .names()
                .table()
                .selected()
                .and_then(|idx| {
                    sql()
                        .schema()
                        .get_by_index(idx)
                        .map(|(name, _)| name.to_owned())
                })
                .ok_or(anyhow!("No table is selected"))?;
            sql().unregister(&table_name);
            Ok(None)
        }
        AppAction::TabShowPanel => {
            app.tabs_mut().show_side_panel();
            Ok(None)
        }
        AppAction::TabHidePanel => {
            app.tabs_mut().take_side_panel();
            Ok(None)
        }
        AppAction::TabPanelPrev => {
            if let Some(side_panel) = app.tabs_mut().side_panel_mut() {
                side_panel.list_mut().select_previous();
            }
            Ok(None)
        }
        AppAction::TabPanelNext => {
            if let Some(side_panel) = app.tabs_mut().side_panel_mut() {
                side_panel.list_mut().select_next();
            }
            Ok(None)
        }
        AppAction::TabPanelSelect => {
            if let Some(idx) = app
                .tabs_mut()
                .take_side_panel()
                .and_then(|panel| panel.list().selected())
            {
                Ok(Some(AppAction::TabSelect(idx)))
            } else {
                Ok(None)
            }
        }
        AppAction::RegisterDataFrame(name) => {
            if sql().schema().iter().map(|(name, _)| name).contains(&name) {
                Err(anyhow!("Data frame with name '{}' already exists.", &name))
            } else {
                if let Some(data_frame) = app
                    .tabs()
                    .selected()
                    .map(TabContentState::table)
                    .map(DataFrameTableState::data_frame)
                    .cloned()
                {
                    sql().register(&name, data_frame, crate::misc::sql::Source::User);
                }

                Ok(None)
            }
        }
        AppAction::DataFrameInfoScrollUp => {
            if let Some(tabular) = app.tabs_mut().selected_mut()
                && let Modal::DataFrameInfo(data_frame_info_state) = tabular.modal_mut()
            {
                *data_frame_info_state
                    .field_info_mut()
                    .table_state_mut()
                    .offset_mut() = data_frame_info_state
                    .field_info()
                    .table_state()
                    .offset()
                    .saturating_sub(1);
            }
            Ok(None)
        }
        AppAction::DataFrameInfoScrollDown => {
            if let Some(tabular) = app.tabs_mut().selected_mut()
                && let Modal::DataFrameInfo(data_frame_info_state) = tabular.modal_mut()
            {
                *data_frame_info_state
                    .field_info_mut()
                    .table_state_mut()
                    .offset_mut() = data_frame_info_state
                    .field_info()
                    .table_state()
                    .offset()
                    .saturating_add(1);
            }
            Ok(None)
        }
        AppAction::DataFrameInfoShow => {
            if let Some(tabular) = app.tabs_mut().selected_mut() {
                *tabular.modal_mut() = Modal::DataFrameInfo(DataFrameInfoState::default());
            }
            Ok(None)
        }
        AppAction::ScatterPlot(x_lab, y_lab, group_by) => {
            if let Some(tab_content) = app.tabs_mut().selected_mut() {
                let df = tab_content.table().data_frame();
                if group_by.is_empty() {
                    let data = df.scatter_plot_data(&x_lab, &y_lab)?;
                    *tab_content.modal_mut() =
                        Modal::ScatterPlot(ScatterPlotState::new(x_lab, y_lab, vec![data])?)
                } else {
                    let mut groups = Vec::new();
                    let mut data = Vec::new();
                    for df in df.partition_by(&group_by, true)? {
                        let name = group_by
                            .iter()
                            .map(|col| {
                                df.column(col)
                                    .and_then(|column| column.get(0))
                                    .map(IntoString::into_single_line)
                                    .unwrap_or("null".to_owned())
                            })
                            .join(" - ");
                        groups.push(name);
                        data.push(df.scatter_plot_data(&x_lab, &y_lab)?);
                    }
                    *tab_content.modal_mut() = Modal::ScatterPlot(
                        ScatterPlotState::new(x_lab, y_lab, data)?.groups(groups),
                    )
                }
            }
            Ok(None)
        }
        AppAction::HistogramPlot(group_by, buckets) => {
            if let Some(tab_content) = app.tabs_mut().selected_mut() {
                let df = tab_content.table().data_frame();
                *tab_content.modal_mut() = Modal::HistogramPlot(HistogramPlotState::new(
                    df.histogram_plot_data(&group_by, buckets)?,
                ))
            }
            Ok(None)
        }
        AppAction::HistogramScrollUp => {
            if let Some(tab_content) = app.tabs_mut().selected_mut()
                && let Modal::HistogramPlot(hist) = tab_content.modal_mut()
            {
                hist.scroll_up();
            }
            Ok(None)
        }
        AppAction::HistogramScrollDown => {
            if let Some(tab_content) = app.tabs_mut().selected_mut()
                && let Modal::HistogramPlot(hist) = tab_content.modal_mut()
            {
                hist.scroll_down();
            }
            Ok(None)
        }
        AppAction::PreviewTheme(theme) => {
            *config().theme_mut() = theme;
            Ok(None)
        }
        AppAction::StoreConfig => {
            fs::write(
                config_path().ok_or(anyhow!("Home not found"))?,
                config().store()?,
            )?;
            Ok(None)
        }
        AppAction::ThemeSelectorSelectPrev => {
            if let Some(theme_selector) = app.theme_selector_mut() {
                theme_selector
                    .search_picker_mut()
                    .list_mut()
                    .select_previous();
                // if let Some(theme) = theme_selector
                //     .search_picker()
                //     .selected()
                //     .and_then(|i| Theme::all().get(i).cloned())
                // {
                //     Ok(Some(AppAction::PreviewTheme(theme)))
                // } else {
                //     Ok(None)
                // }
            }
            Ok(None)
        }
        AppAction::ThemeSelectorSelectNext => {
            if let Some(theme_selector) = app.theme_selector_mut() {
                theme_selector.search_picker_mut().list_mut().select_next();
                // if let Some(theme) = theme_selector
                //     .search_picker()
                //     .selected()
                //     .and_then(|i| Theme::all().get(i).cloned())
                // {
                //     Ok(Some(AppAction::PreviewTheme(theme)))
                // } else {
                //     Ok(None)
                // }
            }
            Ok(None)
        }
        AppAction::ThemeSelectorRollback => {
            if let Some(theme_selector) = app.take_theme_selector() {
                Ok(Some(AppAction::PreviewTheme(
                    theme_selector.into_rollback_theme(),
                )))
            } else {
                Ok(None)
            }
        }
        AppAction::ThemeSelectorCommit => {
            if let Some(_ts) = app.take_theme_selector() {
                Ok(Some(AppAction::StoreConfig))
            } else {
                Ok(None)
            }
        }
        AppAction::ThemeSelectorHandleEvent(event) => {
            if let Some(theme_selector) = app.theme_selector_mut() {
                theme_selector.search_picker_mut().input_mut().handle(event);
            }
            Ok(None)
        }
    }
}
