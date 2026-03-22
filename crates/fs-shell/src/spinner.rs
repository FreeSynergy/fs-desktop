/// Loading spinner re-exports from fs-components.
///
/// `LoadingSpinner`, `LoadingOverlay` and `SpinnerSize` live in `fs-components`
/// so that every fs-* crate can use them without depending on fs-shell.
/// fs-shell re-exports them here for convenience.
pub use fs_components::{LoadingOverlay, LoadingSpinner, SpinnerSize};
