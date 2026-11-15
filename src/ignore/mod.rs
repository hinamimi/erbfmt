mod directive;

pub use directive::{IgnoreSelector, parse_ignore_directive};

mod edits;
pub use edits::ignore_targets;
