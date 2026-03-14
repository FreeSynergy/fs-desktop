pub mod desktop;
pub mod taskbar;
pub mod window;
pub mod wallpaper;

pub use desktop::Desktop;
pub use taskbar::Taskbar;
pub use window::{Window, WindowId, WindowButton, WindowContent, WindowManager, WindowSize};
