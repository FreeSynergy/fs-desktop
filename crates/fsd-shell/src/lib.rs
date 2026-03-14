pub mod desktop;
pub mod launcher;
pub mod notification;
pub mod taskbar;
pub mod wallpaper;
pub mod window;
pub mod window_frame;

pub use desktop::Desktop;
pub use launcher::{AppLauncher, LauncherState};
pub use notification::{Notification, NotificationKind, NotificationManager, NotificationStack};
pub use taskbar::Taskbar;
pub use window::{Window, WindowButton, WindowContent, WindowId, WindowManager, WindowSize};
pub use window_frame::WindowFrame;
