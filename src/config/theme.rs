use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct Theme {
    pub accent: Color,
    pub accent_alt: Color,
    pub warning: Color,
    pub danger: Color,
    pub success: Color,
    pub subtle: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            accent: Color::Cyan,
            accent_alt: Color::Blue,
            warning: Color::Yellow,
            danger: Color::Red,
            success: Color::Green,
            subtle: Color::Gray,
        }
    }
}
