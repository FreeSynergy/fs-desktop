/// Theme Manager — main app component.
use dioxus::prelude::*;
use fsn_components::{FsnSidebar, FsnSidebarItem, FSN_SIDEBAR_CSS};
use fsn_i18n;

use crate::chrome_view::ChromeView;
use crate::colors_view::ColorsView;
use crate::cursor_view::CursorView;
use crate::themes_view::ThemesView;

/// Active section in the Theme Manager.
#[derive(Clone, PartialEq, Debug)]
pub enum ThemeSection {
    Themes,
    Colors,
    Cursor,
    Chrome,
}

impl ThemeSection {
    pub fn id(&self) -> &str {
        match self {
            Self::Themes => "themes",
            Self::Colors => "colors",
            Self::Cursor => "cursor",
            Self::Chrome => "chrome",
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Themes => fsn_i18n::t("theme.section.themes"),
            Self::Colors => fsn_i18n::t("theme.section.colors"),
            Self::Cursor => fsn_i18n::t("theme.section.cursor"),
            Self::Chrome => fsn_i18n::t("theme.section.chrome"),
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Themes => "🎨",
            Self::Colors => "🖌",
            Self::Cursor => "🖱",
            Self::Chrome => "🪟",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "themes" => Some(Self::Themes),
            "colors" => Some(Self::Colors),
            "cursor" => Some(Self::Cursor),
            "chrome" => Some(Self::Chrome),
            _        => None,
        }
    }
}

const ALL_SECTIONS: &[ThemeSection] = &[
    ThemeSection::Themes,
    ThemeSection::Colors,
    ThemeSection::Cursor,
    ThemeSection::Chrome,
];

/// Root component of the Theme Manager.
#[component]
pub fn ThemeManagerApp() -> Element {
    let mut active = use_signal(|| ThemeSection::Themes);

    let sidebar_items: Vec<FsnSidebarItem> = ALL_SECTIONS.iter()
        .map(|s| FsnSidebarItem::new(s.id(), s.icon(), s.label()))
        .collect();

    rsx! {
        style { "{FSN_SIDEBAR_CSS}" }
        div {
            class: "fsd-theme-manager",
            style: "display: flex; flex-direction: column; height: 100%; width: 100%; overflow: hidden; \
                    background: var(--fsn-color-bg-base);",

            // App title bar
            div {
                style: "padding: 10px 16px; border-bottom: 1px solid var(--fsn-border); \
                        flex-shrink: 0; background: var(--fsn-bg-surface);",
                h2 {
                    style: "margin: 0; font-size: 16px; font-weight: 600; color: var(--fsn-text-primary);",
                    {fsn_i18n::t("theme.title")}
                }
            }

            // Sidebar + Content row
            div {
                style: "display: flex; flex: 1; overflow: hidden;",

                FsnSidebar {
                    items:     sidebar_items,
                    active_id: active.read().id().to_string(),
                    on_select: move |id: String| {
                        if let Some(section) = ThemeSection::from_id(&id) {
                            active.set(section);
                        }
                    },
                }

                div {
                    style: "flex: 1; overflow: auto; padding: 20px;",
                    match *active.read() {
                        ThemeSection::Themes => rsx! { ThemesView {} },
                        ThemeSection::Colors => rsx! { ColorsView {} },
                        ThemeSection::Cursor => rsx! { CursorView {} },
                        ThemeSection::Chrome => rsx! { ChromeView {} },
                    }
                }
            }
        }
    }
}
