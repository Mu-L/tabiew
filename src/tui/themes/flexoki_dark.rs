
use ratatui::style::Color;

use crate::tui::themes::styler::SixColorsTwoRowsStyler;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct FlexokiDark;

impl SixColorsTwoRowsStyler for FlexokiDark {
    const BACKGROUND: Color = Color::from_u32(0x00100f0f);
    const LIGHT_BACKGROUND: Color = Color::from_u32(0x00302F2F);
    const FOREGROUND: Color = Color::from_u32(0x00cecdc3);
    const DARK_FOREGROUND: Color = Color::from_u32(0x00100f0f);

    const COLORS: [Color; 6] = [
        Color::from_u32(0x00af3029),
        Color::from_u32(0x0066800b),
        Color::from_u32(0x00ad8301),
        Color::from_u32(0x00205ea6),
        Color::from_u32(0x00a02f6f),
        Color::from_u32(0x0024837b),
    ];
    const DARK_COLORS: [Color; 6] = [
        Color::from_u32(0x00d14d41),
        Color::from_u32(0x00879a39),
        Color::from_u32(0x00d0a215),
        Color::from_u32(0x004385be),
        Color::from_u32(0x00ce5d97),
        Color::from_u32(0x003aa99f),
    ];

    const ROW_BACKGROUNDS: [Color; 2] = [Color::from_u32(0x00161515), Color::from_u32(0x001C1B1B)];
    const HIGHLIGHT_BACKGROUND: Color = Color::from_u32(0x00AEADA3);
    const HIGHLIGHT_FOREGROUND: Color = Self::FOREGROUND;

    const STATUS_BAR_ERROR: Color = Color::from_u32(0x00A11D11);
}
