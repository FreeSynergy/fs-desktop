/// Desktop — the root layout: wallpaper + window area + taskbar.
use dioxus::prelude::*;

use crate::taskbar::{default_apps, Taskbar};
use crate::wallpaper::Wallpaper;
use crate::window::WindowManager;

/// Root desktop component.
#[component]
pub fn Desktop() -> Element {
    let wallpaper = use_signal(Wallpaper::default);
    let window_manager = use_signal(WindowManager::default);
    let mut apps = use_signal(default_apps);

    let bg = wallpaper.read().to_css_background();

    rsx! {
        div {
            id: "fsd-desktop",
            style: "width: 100vw; height: 100vh; display: flex; flex-direction: column; overflow: hidden; {bg}",

            // Window area — fills all space above the taskbar
            div {
                id: "fsd-window-area",
                style: "flex: 1; position: relative; overflow: hidden;",

                // Render all open windows
                for window in window_manager.read().windows() {
                    // TODO: render WindowFrame component per window
                    div {
                        key: "{window.id.0}",
                        style: "position: absolute; z-index: {window.z_index};",
                    }
                }
            }

            // Taskbar at the bottom
            Taskbar {
                apps: apps.read().clone(),
                on_launch: move |app_id: String| {
                    tracing::info!("Launching app: {}", app_id);
                    // TODO: dispatch to app registry to open the window
                },
            }
        }
    }
}
