// front/mod.rs

// Declare submodules
pub mod app;
pub mod ui;

// Re-export key public items for easier access
// This way, users can do `use your_crate::front::{App, ui};` instead of `use your_crate::front::app::App;`
pub use app::App;
pub use ui::ui;  // Re-export the main UI function and helper; add more if needed (e.g., draw_* functions if you want them public)