/// Loading spinner re-exports from fsn-components.
///
/// `LoadingSpinner`, `LoadingOverlay` and `SpinnerSize` live in `fsn-components`
/// so that every fsd-* crate can use them without depending on fsd-shell.
/// fsd-shell re-exports them here for convenience.
pub use fsn_components::{LoadingOverlay, LoadingSpinner, SpinnerSize};
