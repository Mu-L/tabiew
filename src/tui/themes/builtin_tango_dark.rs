
use ratatui::style::Color;

use crate::tui::themes::styler::SixColorsTwoRowsStyler;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct BuiltinTangoDark;

impl SixColorsTwoRowsStyler for BuiltinTangoDark {
    const BACKGROUND: Color = Color::from_u32(0x00000000);
    const LIGHT_BACKGROUND: Color = Color::from_u32(0x00202020);
    const FOREGROUND: Color = Color::from_u32(0x00ffffff);
    const DARK_FOREGROUND: Color = Color::from_u32(0x00000000);

    const COLORS: [Color; 6] = [
        Color::from_u32(0x00ef2929),
        Color::from_u32(0x008ae234),
        Color::from_u32(0x00fce94f),
        Color::from_u32(0x00729fcf),
        Color::from_u32(0x00ad7fa8),
        Color::from_u32(0x0034e2e2),
    ];
    const DARK_COLORS: [Color; 6] = [
        Color::from_u32(0x00cc0000),
        Color::from_u32(0x004e9a06),
        Color::from_u32(0x00c4a000),
        Color::from_u32(0x003465a4),
        Color::from_u32(0x0075507b),
        Color::from_u32(0x0006989a),
    ];

    const ROW_BACKGROUNDS: [Color; 2] = [Color::from_u32(0x00060606), Color::from_u32(0x000C0C0C)];
    const HIGHLIGHT_BACKGROUND: Color = Color::from_u32(0x00DFDFDF);
    const HIGHLIGHT_FOREGROUND: Color = Self::FOREGROUND;

    const STATUS_BAR_ERROR: Color = Color::from_u32(0x009C0000);

    fn id(&self) -> &str {
        "builtin_tango_dark"
    }

    fn title(&self) -> &str {
        "BuiltinTangoDark"
    }
}
