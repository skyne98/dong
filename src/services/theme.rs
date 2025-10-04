use crate::vue::Val;

/// Represents the current theme (light or dark)
#[derive(Clone, Debug, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
    Auto, // Auto-detect based on terminal
}

/// Adaptive color that changes based on theme
#[derive(Clone, Debug)]
pub struct AdaptiveColor {
    pub light: ratatui::style::Color,
    pub dark: ratatui::style::Color,
}

impl AdaptiveColor {
    pub fn new(light: ratatui::style::Color, dark: ratatui::style::Color) -> Self {
        Self { light, dark }
    }

    pub fn get(&self, mode: &ThemeMode) -> ratatui::style::Color {
        match mode {
            ThemeMode::Light => self.light,
            ThemeMode::Dark => self.dark,
            ThemeMode::Auto => {
                // For now, default to dark. In a real implementation,
                // this would detect the terminal's background color
                self.dark
            }
        }
    }
}

/// Theme service - manages app-wide theming
pub struct ThemeService {
    pub mode: Val<ThemeMode>,

    // Core theme colors
    pub background: AdaptiveColor,
    pub foreground: AdaptiveColor,
    pub border: AdaptiveColor,
    pub accent: AdaptiveColor,
    pub secondary: AdaptiveColor,

    // Semantic colors
    pub success: AdaptiveColor,
    pub warning: AdaptiveColor,
    pub error: AdaptiveColor,
    pub info: AdaptiveColor,
}

impl ThemeService {
    pub fn new() -> Self {
        Self {
            mode: Val::new(ThemeMode::Auto),

            background: AdaptiveColor::new(ratatui::style::Color::White, ratatui::style::Color::Black),
            foreground: AdaptiveColor::new(ratatui::style::Color::Black, ratatui::style::Color::White),
            border: AdaptiveColor::new(ratatui::style::Color::Gray, ratatui::style::Color::Gray),
            accent: AdaptiveColor::new(ratatui::style::Color::Blue, ratatui::style::Color::Cyan),
            secondary: AdaptiveColor::new(ratatui::style::Color::DarkGray, ratatui::style::Color::Gray),

            success: AdaptiveColor::new(ratatui::style::Color::Green, ratatui::style::Color::Green),
            warning: AdaptiveColor::new(ratatui::style::Color::Yellow, ratatui::style::Color::Yellow),
            error: AdaptiveColor::new(ratatui::style::Color::Red, ratatui::style::Color::Red),
            info: AdaptiveColor::new(ratatui::style::Color::Blue, ratatui::style::Color::Cyan),
        }
    }

    /// Get the current background color
    pub fn background_color(&self) -> ratatui::style::Color {
        self.background.get(&*self.mode.value())
    }

    /// Get the current foreground color
    pub fn foreground_color(&self) -> ratatui::style::Color {
        self.foreground.get(&*self.mode.value())
    }

    /// Get the current border color
    pub fn border_color(&self) -> ratatui::style::Color {
        self.border.get(&*self.mode.value())
    }

    /// Get the current accent color
    pub fn accent_color(&self) -> ratatui::style::Color {
        self.accent.get(&*self.mode.value())
    }

    /// Get the current secondary color
    pub fn secondary_color(&self) -> ratatui::style::Color {
        self.secondary.get(&*self.mode.value())
    }

    /// Set the theme mode
    pub fn set_mode(&self, mode: ThemeMode) {
        self.mode.set(mode);
    }

    /// Get semantic color for success
    pub fn success_color(&self) -> ratatui::style::Color {
        self.success.get(&*self.mode.value())
    }

    /// Get semantic color for warning
    pub fn warning_color(&self) -> ratatui::style::Color {
        self.warning.get(&*self.mode.value())
    }

    /// Get semantic color for error
    pub fn error_color(&self) -> ratatui::style::Color {
        self.error.get(&*self.mode.value())
    }

    /// Get semantic color for info
    pub fn info_color(&self) -> ratatui::style::Color {
        self.info.get(&*self.mode.value())
    }
}

thread_local! {
    /// Global theme service singleton
    pub static THEME_SERVICE: std::cell::RefCell<ThemeService> = std::cell::RefCell::new(ThemeService::new());
}
