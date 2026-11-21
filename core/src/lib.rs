//! Pure logic for the virtual pet. No Xous dependencies.

pub mod clock;
pub mod pet;

pub use clock::Timekeeper;
pub use pet::{Cause, LoadError, Mood, Pet, Saved, Stage, AWAY_AFTER_S, LIFESPAN_S, MAX_CATCHUP_S, NAME_MAX, TICK_S};
// appkit doesn't re-export `text`; the shell gets the text field through this crate.
pub use precursor_appkit_core::text::TextField;
