/// Wallpaper management — load from URL or local file.
use serde::{Deserialize, Serialize};

/// How a wallpaper is sourced.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[derive(Default)]
pub enum WallpaperSource {
    /// Solid color background.
    Color { hex: String },
    /// CSS gradient background (any valid CSS gradient string).
    Gradient { css: String },
    /// Loaded from a URL.
    Url { url: String },
    /// Local file path.
    File { path: String },
    /// Built-in default.
    #[default]
    Default,
}

/// How the wallpaper is rendered.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WallpaperFit {
    #[default]
    Cover,
    Contain,
    Stretch,
    Center,
    Tile,
}

/// Wallpaper configuration.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Wallpaper {
    pub source: WallpaperSource,
    pub fit: WallpaperFit,
    pub blur: bool,
    pub dim: u8, // 0–100 darkness overlay
}

impl Wallpaper {
    /// Returns the CSS background property value.
    #[must_use]
    pub fn to_css_background(&self) -> String {
        match &self.source {
            WallpaperSource::Color { hex } => format!("background-color: {hex};"),
            WallpaperSource::Gradient { css } => format!("background: {css};"),
            WallpaperSource::Url { url } => {
                format!(
                    "background-image: url('{}'); background-size: {};",
                    url,
                    self.fit_css()
                )
            }
            WallpaperSource::File { path } => {
                format!(
                    "background-image: url('file://{}'); background-size: {};",
                    path,
                    self.fit_css()
                )
            }
            WallpaperSource::Default => {
                "background: linear-gradient(135deg, #0f172a, #1e293b);".into()
            }
        }
    }

    fn fit_css(&self) -> &'static str {
        match self.fit {
            WallpaperFit::Cover => "cover",
            WallpaperFit::Contain => "contain",
            WallpaperFit::Stretch => "100% 100%",
            WallpaperFit::Center | WallpaperFit::Tile => "auto",
        }
    }
}
