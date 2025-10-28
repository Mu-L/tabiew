use anyhow::{Ok, anyhow};
use itertools::Itertools;
use polars::{df, frame::DataFrame};
use std::{collections::HashMap, sync::OnceLock};

use crate::{
    AppResult, handler::action::AppAction, misc::type_inferer::TypeInferer,
    tui::popups::inline_query::InlineQueryType,
};

pub fn parse_into_action(cmd: impl AsRef<str>) -> AppResult<AppAction> {
    let (s1, s2) = cmd.as_ref().split_once(' ').unwrap_or((cmd.as_ref(), ""));
    if let Some(parse_fn) = registary().get(s1) {
        parse_fn(s2)
    } else {
        Err(anyhow!("Invalid command '{}'", cmd.as_ref()))
    }
}

pub fn commands_help_data_frame() -> DataFrame {
    let len = ENTRIES.len();
    let (short, long, usage, description) = ENTRIES.iter().fold(
        (
            Vec::<&'static str>::with_capacity(len),
            Vec::<&'static str>::with_capacity(len),
            Vec::<&'static str>::with_capacity(len),
            Vec::<&'static str>::with_capacity(len),
        ),
        |(mut v1, mut v2, mut v3, mut v4), cmd| {
            v1.push(cmd.prefix.short().unwrap_or("-"));
            v2.push(cmd.prefix.long().unwrap_or("-"));
            v3.push(cmd.usage);
            v4.push(cmd.description);
            (v1, v2, v3, v4)
        },
    );
    df! {
        "Command" => long,
        "Short Form" => short,
        "Usage" => usage,
        "Description" => description,
    }
    .unwrap()
}

static REGISTRY: OnceLock<Registry> = OnceLock::new();

fn registary() -> &'static Registry {
    REGISTRY.get_or_init(|| {
        ENTRIES
            .iter()
            .flat_map(|cmd| {
                match cmd.prefix {
                    Prefix::Long(long) => vec![(long, cmd.parser)],
                    Prefix::ShortAndLong(short, long) => {
                        vec![(short, cmd.parser), (long, cmd.parser)]
                    }
                }
                .into_iter()
            })
            .collect()
    })
}

type ParseFn = fn(&str) -> AppResult<AppAction>;
type Registry = HashMap<&'static str, ParseFn>;

enum Prefix {
    Long(&'static str),
    ShortAndLong(&'static str, &'static str),
}

impl Prefix {
    fn short(&self) -> Option<&'static str> {
        match self {
            Prefix::ShortAndLong(short, _) => Some(short),
            _ => None,
        }
    }

    fn long(&self) -> Option<&'static str> {
        match self {
            Prefix::Long(long) => Some(long),
            Prefix::ShortAndLong(_, long) => Some(long),
        }
    }
}

struct Entry {
    prefix: Prefix,
    usage: &'static str,
    description: &'static str,
    parser: ParseFn,
}

static ENTRIES: [Entry; 22] = [
    Entry {
        prefix: Prefix::ShortAndLong("Q", "query"),
        usage: "Q <query>",
        description: "Query the data in Structured Query Language(SQL).",
        parser: |query| Ok(AppAction::TableQuery(query.to_owned())),
    },
    Entry {
        prefix: Prefix::ShortAndLong("q", "quit"),
        usage: "q",
        description: "Close all tables and quit Tabiew",
        parser: |_| Ok(AppAction::Quit),
    },
    Entry {
        prefix: Prefix::Long("goto"),
        usage: "goto <line_index>",
        description: "Jumps to the specified line",
        parser: |line| {
            if line.is_empty() {
                Ok(AppAction::GoToLineShow)
            } else {
                Ok(AppAction::GotoLine(
                    line.parse::<usize>()?.saturating_sub(1),
                ))
            }
        },
    },
    Entry {
        prefix: Prefix::Long("goup"),
        usage: "goup <lines>",
        description: "Jump specified number of line(s) up",
        parser: |lines| {
            Ok(match lines {
                "page" => AppAction::TableGoUpFullPage,
                "half" => AppAction::TableGoUpHalfPage,
                _ => AppAction::TableGoUp(lines.parse()?),
            })
        },
    },
    Entry {
        prefix: Prefix::Long("godown"),
        usage: "godown <lines>",
        description: "Jump specified number of line(s) down",
        parser: |lines| {
            Ok(match lines {
                "page" => AppAction::TableGoDownFullPage,
                "half" => AppAction::TableGoDownHalfPage,
                _ => AppAction::TableGoDown(lines.parse()?),
            })
        },
    },
    Entry {
        prefix: Prefix::Long("reset"),
        usage: "reset",
        description: "Reset the data frame to its original state, removing all filters, orders, searches, selects, and aggregations effects.",
        parser: |_| Ok(AppAction::TableReset),
    },
    Entry {
        prefix: Prefix::Long("help"),
        usage: "help",
        description: "Show help",
        parser: |_| Ok(AppAction::Help),
    },
    Entry {
        prefix: Prefix::ShortAndLong("S", "select"),
        usage: "select <column_name(s)>",
        description: "Query the data frame for columns / functions",
        parser: |query| Ok(AppAction::TableSelect(query.to_owned())),
    },
    Entry {
        prefix: Prefix::ShortAndLong("F", "filter"),
        usage: "filter <condition(s)>",
        description: "Filter the data frame, keeping rows were the condition(s) match",
        parser: |query| {
            if query.is_empty() {
                Ok(AppAction::InlineQueryShow(InlineQueryType::Filter))
            } else {
                Ok(AppAction::TableFilter(query.to_owned()))
            }
        },
    },
    Entry {
        prefix: Prefix::ShortAndLong("O", "order"),
        usage: "order <column(s)_and_order(s)>",
        description: "Sort the data frame by column(s)",
        parser: |query| {
            if query.is_empty() {
                Ok(AppAction::InlineQueryShow(InlineQueryType::Order))
            } else {
                Ok(AppAction::TableOrder(query.to_owned()))
            }
        },
    },
    Entry {
        prefix: Prefix::Long("schema"),
        usage: "schema",
        description: "Show loaded data frame(s) and their schema(s)",
        parser: |_| Ok(AppAction::SwitchToSchema),
    },
    Entry {
        prefix: Prefix::Long("rand"),
        usage: "rand",
        description: "Select a random row from current data frame",
        parser: |_| Ok(AppAction::TableGotoRandom),
    },
    Entry {
        prefix: Prefix::Long("tabn"),
        usage: "tabn <query>",
        description: "Create a new tab using the query",
        parser: |query| Ok(AppAction::TabNewQuery(query.to_owned())),
    },
    Entry {
        prefix: Prefix::Long("tabr"),
        usage: "tabr <tab_index>",
        description: "Remove the tab at the index",
        parser: |query| Ok(AppAction::TabRemove(query.parse()?)),
    },
    Entry {
        prefix: Prefix::Long("tab"),
        usage: "tab <tab_index>",
        description: "Select the tab at the index",
        parser: |query| Ok(AppAction::TabSelect(query.parse()?)),
    },
    Entry {
        prefix: Prefix::Long("infer"),
        usage: "infer <types>",
        description: "Perform extra processing to infer column types",
        parser: |args| {
            let ti = args
                .split(' ')
                .try_fold(TypeInferer::default(), |ti, slice| match slice {
                    "int" => Ok(ti.int()),
                    "float" => Ok(ti.float()),
                    "boolean" => Ok(ti.boolean()),
                    "date" => Ok(ti.date()),
                    "datetime" => Ok(ti.datetime()),
                    "all" => Ok(ti.int().float().boolean().date().datetime()),
                    _ => Err(anyhow!("Invalid type {slice}")),
                })?;
            Ok(AppAction::TableInferColumns(ti))
        },
    },
    Entry {
        prefix: Prefix::Long("register"),
        usage: "register <data_frame_name>",
        description: "register current data frame to the SQL backend with the given name",
        parser: |name| Ok(AppAction::RegisterDataFrame(name.to_owned())),
    },
    Entry {
        prefix: Prefix::Long("scatter"),
        usage: "scatter <x-axis> <y-axis> <group-by>",
        description: "Draw a scatter plot given the axes",
        parser: |args| {
            let mut args = shell_words::split(args)?.into_iter();
            let x = args.next().ok_or(anyhow!("require two axes"))?;
            let y = args.next().ok_or(anyhow!("require two axes"))?;
            let gb = args.collect_vec();
            Ok(AppAction::ScatterPlot(x, y, gb))
        },
    },
    Entry {
        prefix: Prefix::Long("hist"),
        usage: "hist <axes> [buckets]",
        description: "Draw a histogram plot given the axes",
        parser: |col| {
            //
            let args = shell_words::split(col)?;
            if args.len() == 1 {
                Ok(AppAction::HistogramPlot(
                    args.into_iter().next().unwrap(),
                    38,
                ))
            } else if args.len() == 2 {
                let mut args = args.into_iter();
                Ok(AppAction::HistogramPlot(
                    args.next().unwrap(),
                    args.next().unwrap().parse()?,
                ))
            } else {
                Err(anyhow!("histogram should be supplied with only one column"))
            }
        },
    },
    Entry {
        prefix: Prefix::Long("theme"),
        usage: "theme",
        description: "Show theme selector",
        parser: |_| Ok(AppAction::ThemeSelectorShow),
    },
    export::entry(),
    import::entry(),
];

mod export {
    use anyhow::anyhow;

    use crate::{handler::action::AppAction, writer::JsonFormat};

    use super::{Entry, Prefix};

    pub const fn entry() -> Entry {
        Entry {
            prefix: Prefix::Long("export"),
            usage: "export <format> <path>",
            description: "Export the data frame to a format specified file",
            parser: |query| {
                let (fmt, path_str) = query
                    .split_once(' ')
                    .ok_or(anyhow!("Export should provide format and path"))?;
                match fmt {
                    "csv" => Ok(AppAction::ExportDsv {
                        destination: path_str.into(),
                        separator: ',',
                        quote: '"',
                        header: true,
                    }),

                    "tsv" => Ok(AppAction::ExportDsv {
                        destination: path_str.into(),
                        separator: '\t',
                        quote: '"',
                        header: true,
                    }),

                    "parquet" => Ok(AppAction::ExportParquet(path_str.into())),

                    "json" => Ok(AppAction::ExportJson(path_str.into(), JsonFormat::Json)),

                    "jsonl" => Ok(AppAction::ExportJson(path_str.into(), JsonFormat::JsonLine)),

                    "arrow" => Ok(AppAction::ExportArrow(path_str.into())),

                    _ => Err(anyhow!(
                        "Unsupported format. Supported ones: csv, tsv, parquet, json, jsonl, and arrow"
                    )),
                }
            },
        }
    }
}

mod import {
    use std::sync::OnceLock;

    use anyhow::anyhow;
    use regex::{Captures, Regex};

    use crate::{AppResult, handler::action::AppAction, writer::JsonFormat};

    use super::{Entry, Prefix};

    type ParseFn = fn(Captures) -> AppResult<AppAction>;

    pub const fn entry() -> Entry {
        Entry {
            prefix: Prefix::Long("import"),
            usage: "import <format> <path>",
            description: "Import data frame from a file into the sql engine",
            parser: |query| {
                let lock: OnceLock<[(Regex, ParseFn); 9]> = OnceLock::new();
                lock.get_or_init(|| {
                    [
                        (Regex::new(r"csv\s+(?<path>.*)").unwrap(), csv_no_args),
                        (
                            Regex::new(r"csv\s*\[(?<args>.+)\]\s+(?<path>.*)").unwrap(),
                            csv_with_args,
                        ),
                        (
                            Regex::new(r"parquet\s+(?<path>.*)").unwrap(),
                            parquet_no_args,
                        ),
                        (Regex::new(r"json\s+(?<path>.*)").unwrap(), json_no_args),
                        (Regex::new(r"jsonl\s+(?<path>.*)").unwrap(), jsonl_no_args),
                        (Regex::new(r"arrow\s+(?<path>.*)").unwrap(), arrow_no_args),
                        (Regex::new(r"sqlite\s+(?<path>.*)").unwrap(), sqlite_no_args),
                        (Regex::new(r"fwf\s+(?<path>.*)").unwrap(), fwf_no_args),
                        (
                            Regex::new(r"fwf\s*\[(?<args>.*)\]\s+(?<path>.*)").unwrap(),
                            fwf_with_args,
                        ),
                    ]
                })
                .iter()
                .find_map(|(re, func)| re.captures(query).map(|cap| func(cap)))
                .unwrap_or(Err(anyhow!("Import should provide format and path")))
            },
        }
    }

    fn csv_no_args(caps: Captures) -> AppResult<AppAction> {
        let path = caps
            .name("path")
            .ok_or(anyhow!("Import path not found"))?
            .as_str()
            .to_owned();
        Ok(AppAction::ImportDsv {
            source: path.into(),
            separator: ',',
            quote: '"',
            has_header: true,
        })
    }

    fn csv_with_args(caps: Captures) -> AppResult<AppAction> {
        let path = caps
            .name("path")
            .ok_or(anyhow!("Import path not found"))?
            .as_str()
            .to_owned();

        let args = caps
            .name("args")
            .ok_or(anyhow!("Empty arguments"))?
            .as_str()
            .split(' ')
            .map(str::trim)
            .filter(|slice| !slice.is_empty())
            .try_fold(CsvImportArgs::default(), |args, slice| args.update(slice))?;

        Ok(AppAction::ImportDsv {
            source: path.into(),
            separator: args.separator.unwrap_or(','),
            has_header: args.has_header.unwrap_or(true),
            quote: args.quote.unwrap_or('"'),
        })
    }

    fn parquet_no_args(caps: Captures) -> AppResult<AppAction> {
        let path = caps
            .name("path")
            .ok_or(anyhow!("Import path not found"))?
            .as_str()
            .to_owned();
        Ok(AppAction::ImportParquet(path.into()))
    }

    fn json_no_args(caps: Captures) -> AppResult<AppAction> {
        let path = caps
            .name("path")
            .ok_or(anyhow!("Import path not found"))?
            .as_str()
            .to_owned();
        Ok(AppAction::ImportJson(path.into(), JsonFormat::Json))
    }

    fn jsonl_no_args(caps: Captures) -> AppResult<AppAction> {
        let path = caps
            .name("path")
            .ok_or(anyhow!("Import path not found"))?
            .as_str()
            .to_owned();
        Ok(AppAction::ImportJson(path.into(), JsonFormat::JsonLine))
    }

    fn arrow_no_args(caps: Captures) -> AppResult<AppAction> {
        let path = caps
            .name("path")
            .ok_or(anyhow!("Import path not found"))?
            .as_str()
            .to_owned();
        Ok(AppAction::ImportArrow(path.into()))
    }

    fn sqlite_no_args(caps: Captures) -> AppResult<AppAction> {
        let path = caps
            .name("path")
            .ok_or(anyhow!("Import path not found"))?
            .as_str()
            .to_owned();
        Ok(AppAction::ImportSqlite(path.into()))
    }

    fn fwf_no_args(caps: Captures) -> AppResult<AppAction> {
        let path = caps
            .name("path")
            .ok_or(anyhow!("Import path not found"))?
            .as_str()
            .to_owned();
        Ok(AppAction::ImportFwf {
            source: path.into(),
            widths: Vec::default(),
            separator_length: 0,
            flexible_width: false,
            has_header: true,
        })
    }

    fn fwf_with_args(caps: Captures) -> AppResult<AppAction> {
        let path = caps
            .name("path")
            .ok_or(anyhow!("Import path not found"))?
            .as_str()
            .to_owned();
        let args = caps
            .name("args")
            .ok_or(anyhow!("Empty arguments"))?
            .as_str()
            .split(' ')
            .map(str::trim)
            .filter(|slice| !slice.is_empty())
            .try_fold(FwfImportArgs::default(), |args, slice| args.update(slice))?;
        Ok(AppAction::ImportFwf {
            source: path.into(),
            widths: args.widths,
            separator_length: args.separator_length.unwrap_or_default(),
            flexible_width: args.flexible_width.unwrap_or(false),
            has_header: args.has_header.unwrap_or(true),
        })
    }

    #[derive(Debug, Default)]
    struct CsvImportArgs {
        separator: Option<char>,
        quote: Option<char>,
        has_header: Option<bool>,
    }

    impl CsvImportArgs {
        fn update(mut self, arg: &str) -> AppResult<Self> {
            match arg {
                "no-header" | "nh" => {
                    if self.has_header.is_none() {
                        self.has_header = false.into();
                        Ok(self)
                    } else {
                        Err(anyhow!("no-header is allowed only once"))
                    }
                }
                "\\t" => {
                    if self.separator.is_none() {
                        self.separator = '\t'.into();
                        Ok(self)
                    } else if self.quote.is_none() {
                        self.quote = '\t'.into();
                        Ok(self)
                    } else {
                        Err(anyhow!(
                            "More than two character arguments provided: {}",
                            arg
                        ))
                    }
                }
                _ if arg.len() == 1 && arg.is_ascii() => {
                    if self.separator.is_none() {
                        self.separator = arg.chars().next().unwrap().into();
                        Ok(self)
                    } else if self.quote.is_none() {
                        self.quote = arg.chars().next().unwrap().into();
                        Ok(self)
                    } else {
                        Err(anyhow!(
                            "More than two character arguments provided: {}",
                            arg
                        ))
                    }
                }
                _ => Err(anyhow!("Invalid argument: '{}'", arg)),
            }
        }
    }

    #[derive(Debug, Default)]
    struct FwfImportArgs {
        widths: Vec<usize>,
        separator_length: Option<usize>,
        flexible_width: Option<bool>,
        has_header: Option<bool>,
    }

    impl FwfImportArgs {
        fn update(mut self, arg: &str) -> AppResult<Self> {
            match arg {
                "flexible-width" | "fw" => {
                    if self.flexible_width.is_none() {
                        self.flexible_width = true.into();
                        Ok(self)
                    } else {
                        Err(anyhow!("flexible-width is allowed only once"))
                    }
                }

                "no-header" | "nh" => {
                    if self.has_header.is_none() {
                        self.has_header = false.into();
                        Ok(self)
                    } else {
                        Err(anyhow!("no-header is allowed only once"))
                    }
                }

                _ => {
                    if let Ok(w) = arg.parse::<usize>() {
                        if self.separator_length.is_none() {
                            self.separator_length = w.into();
                        } else {
                            self.widths.push(w);
                        }
                        Ok(self)
                    } else {
                        Err(anyhow!("Invalid argument: '{}'", arg))
                    }
                }
            }
        }
    }
}
