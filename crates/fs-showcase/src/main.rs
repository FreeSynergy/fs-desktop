#![deny(clippy::all, clippy::pedantic, warnings)]
//! fs-showcase — Component gallery for FreeSynergy.Desktop.
//!
//! Only meaningful in debug builds. In release mode it exits immediately.

fn main() {
    #[cfg(not(debug_assertions))]
    {
        eprintln!("fs-showcase is a debug-only tool. Run with `cargo run` (without --release).");
        return;
    }

    #[cfg(debug_assertions)]
    run();
}

#[cfg(debug_assertions)]
fn run() {
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting fs-showcase");

    #[cfg(feature = "iced")]
    {
        use fs_gui_engine_iced::IcedEngine;
        let _ = IcedEngine::run_app::<ShowcaseApp, ShowcaseMessage, _, _>(
            "FreeSynergy \u{2013} Component Showcase",
            ShowcaseApp::update,
            ShowcaseApp::view,
        );
    }
    #[cfg(not(feature = "iced"))]
    {
        eprintln!("fs-showcase: no iced feature enabled");
    }
}

// ── ShowcaseMessage ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ShowcaseMessage {
    SectionSelected(String),
    ButtonClicked(String),
    Noop,
}

// ── ShowcaseApp ───────────────────────────────────────────────────────────────

#[cfg(debug_assertions)]
pub struct ShowcaseApp {
    active_section: String,
    last_action: Option<String>,
}

#[cfg(debug_assertions)]
impl Default for ShowcaseApp {
    fn default() -> Self {
        Self {
            active_section: "buttons".to_string(),
            last_action: None,
        }
    }
}

#[cfg(debug_assertions)]
#[cfg(feature = "iced")]
impl ShowcaseApp {
    pub fn update(
        &mut self,
        msg: ShowcaseMessage,
    ) -> fs_gui_engine_iced::iced::Task<ShowcaseMessage> {
        match msg {
            ShowcaseMessage::SectionSelected(s) => self.active_section = s,
            ShowcaseMessage::ButtonClicked(label) => self.last_action = Some(label),
            ShowcaseMessage::Noop => {}
        }
        fs_gui_engine_iced::iced::Task::none()
    }

    #[must_use]
    pub fn view(&self) -> fs_gui_engine_iced::iced::Element<'_, ShowcaseMessage> {
        use fs_gui_engine_iced::iced::{
            widget::{button, column, container, row, scrollable, text, Space},
            Alignment, Element, Length,
        };

        let sections = ["buttons", "text", "containers", "layout"];

        let nav_buttons: Vec<Element<'_, ShowcaseMessage>> = sections
            .iter()
            .map(|s| {
                let is_active = *s == self.active_section;
                let label = capitalize(s);
                let btn = button(text(label).size(13))
                    .on_press(ShowcaseMessage::SectionSelected(s.to_string()))
                    .width(Length::Fill)
                    .padding([8, 16]);
                if is_active {
                    container(btn)
                        .style(
                            |_theme| fs_gui_engine_iced::iced::widget::container::Style {
                                background: Some(fs_gui_engine_iced::iced::Background::Color(
                                    fs_gui_engine_iced::iced::Color::from_rgba(
                                        0.02, 0.74, 0.84, 0.15,
                                    ),
                                )),
                                ..Default::default()
                            },
                        )
                        .into()
                } else {
                    btn.into()
                }
            })
            .collect();

        let nav = container(
            column![
                text("FreeSynergy")
                    .size(12)
                    .color(fs_gui_engine_iced::iced::Color::from_rgb(0.02, 0.74, 0.84)),
                text("Showcase")
                    .size(10)
                    .color(fs_gui_engine_iced::iced::Color::from_rgb(0.5, 0.5, 0.6)),
                Space::with_height(16),
            ]
            .extend(nav_buttons)
            .spacing(2)
            .padding([16, 12]),
        )
        .width(180)
        .height(Length::Fill)
        .style(
            |_theme| fs_gui_engine_iced::iced::widget::container::Style {
                background: Some(fs_gui_engine_iced::iced::Background::Color(
                    fs_gui_engine_iced::iced::Color::from_rgb(0.09, 0.11, 0.13),
                )),
                ..Default::default()
            },
        );

        let content = scrollable(self.view_section())
            .height(Length::Fill)
            .width(Length::Fill);

        let main_row = row![nav, content].spacing(0);

        let status: Element<'_, ShowcaseMessage> = if let Some(action) = &self.last_action {
            text(format!("Last action: {action}"))
                .size(11)
                .color(fs_gui_engine_iced::iced::Color::from_rgb(0.5, 0.5, 0.6))
                .into()
        } else {
            Space::with_height(0).into()
        };

        let root = column![
            container(
                row![
                    text("FreeSynergy — Component Showcase")
                        .size(16)
                        .color(fs_gui_engine_iced::iced::Color::from_rgb(0.02, 0.74, 0.84)),
                    Space::with_width(Length::Fill),
                    status,
                ]
                .align_y(Alignment::Center)
                .padding([0, 16]),
            )
            .height(48)
            .width(Length::Fill)
            .style(
                |_theme| fs_gui_engine_iced::iced::widget::container::Style {
                    background: Some(fs_gui_engine_iced::iced::Background::Color(
                        fs_gui_engine_iced::iced::Color::from_rgb(0.04, 0.06, 0.10),
                    )),
                    ..Default::default()
                }
            ),
            main_row.height(Length::Fill),
        ]
        .spacing(0);

        container(root)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_section(&self) -> fs_gui_engine_iced::iced::Element<'_, ShowcaseMessage> {
        #[allow(unused_imports)]
        use fs_gui_engine_iced::iced::Length;
        use fs_gui_engine_iced::iced::{
            widget::{button, column, container, row, text, Space},
            Alignment, Element,
        };

        let section_title = text(capitalize(&self.active_section))
            .size(20)
            .color(fs_gui_engine_iced::iced::Color::from_rgb(0.02, 0.74, 0.84));

        let body: Element<'_, ShowcaseMessage> = match self.active_section.as_str() {
            "buttons" => {
                let variants = [
                    ("Primary", "primary"),
                    ("Secondary", "secondary"),
                    ("Danger", "danger"),
                    ("Small", "small"),
                    ("Large", "large"),
                ];
                let btns: Vec<Element<'_, ShowcaseMessage>> = variants
                    .iter()
                    .map(|(label, id)| {
                        button(text(*label).size(14))
                            .on_press(ShowcaseMessage::ButtonClicked(id.to_string()))
                            .padding([8, 20])
                            .into()
                    })
                    .collect();
                row(btns)
                    .spacing(12)
                    .align_y(Alignment::Center)
                    .wrap()
                    .into()
            }
            "text" => column![
                text("Heading 1").size(28),
                text("Heading 2").size(22),
                text("Heading 3").size(18),
                text("Body text — 14px").size(14),
                text("Caption text — 11px")
                    .size(11)
                    .color(fs_gui_engine_iced::iced::Color::from_rgb(0.5, 0.5, 0.6)),
                text("Cyan accent text")
                    .size(14)
                    .color(fs_gui_engine_iced::iced::Color::from_rgb(0.02, 0.74, 0.84)),
            ]
            .spacing(12)
            .into(),
            "containers" => {
                let card_style = |_theme: &_| fs_gui_engine_iced::iced::widget::container::Style {
                    background: Some(fs_gui_engine_iced::iced::Background::Color(
                        fs_gui_engine_iced::iced::Color::from_rgb(0.09, 0.11, 0.13),
                    )),
                    border: fs_gui_engine_iced::iced::Border {
                        color: fs_gui_engine_iced::iced::Color::from_rgba(0.58, 0.67, 0.78, 0.18),
                        width: 1.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                };
                row![
                    container(
                        column![
                            text("Standard Card").size(14),
                            Space::with_height(4),
                            text("Background: bg-surface with border.")
                                .size(12)
                                .color(fs_gui_engine_iced::iced::Color::from_rgb(0.5, 0.5, 0.6)),
                        ]
                        .spacing(0)
                        .padding([16, 20]),
                    )
                    .style(card_style)
                    .width(260),
                    Space::with_width(16),
                    container(
                        column![
                            text("Info Card").size(14),
                            Space::with_height(4),
                            text("Highlighted with accent border.")
                                .size(12)
                                .color(fs_gui_engine_iced::iced::Color::from_rgb(0.5, 0.5, 0.6)),
                        ]
                        .spacing(0)
                        .padding([16, 20]),
                    )
                    .style(|_theme| {
                        fs_gui_engine_iced::iced::widget::container::Style {
                            background: Some(fs_gui_engine_iced::iced::Background::Color(
                                fs_gui_engine_iced::iced::Color::from_rgba(0.02, 0.74, 0.84, 0.08),
                            )),
                            border: fs_gui_engine_iced::iced::Border {
                                color: fs_gui_engine_iced::iced::Color::from_rgb(0.02, 0.74, 0.84),
                                width: 1.0,
                                radius: 6.0.into(),
                            },
                            ..Default::default()
                        }
                    })
                    .width(260),
                ]
                .spacing(0)
                .into()
            }
            _ => column![
                text(format!("Section: {}", self.active_section)).size(14),
                Space::with_height(8),
                text("Content coming soon…")
                    .size(13)
                    .color(fs_gui_engine_iced::iced::Color::from_rgb(0.5, 0.5, 0.6)),
            ]
            .spacing(0)
            .into(),
        };

        column![section_title, Space::with_height(16), body]
            .spacing(0)
            .padding([32, 32])
            .into()
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

// ── Non-debug stubs ───────────────────────────────────────────────────────────

#[cfg(not(debug_assertions))]
pub struct ShowcaseApp;
#[cfg(not(debug_assertions))]
impl Default for ShowcaseApp {
    fn default() -> Self {
        Self
    }
}
